use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use h3::client::{RequestStream, SendRequest};
use h3::ext::Protocol;
use http::{Method, Request};
use masque_core::{Target, connect_udp};
use quinn::Endpoint;
use tokio::sync::mpsc::Receiver;

use crate::router::{self, Router};
use crate::stats::{Stats, Tracked};
use crate::{Error, event, request, tunnel};

type Stream = RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>;

pub struct Client {
    quic: quinn::Connection,
    send: SendRequest<h3_quinn::OpenStreams, Bytes>,
    router: Arc<Router>,
    stats: Arc<Stats>,
}

pub struct UdpTunnel {
    quic: quinn::Connection,
    router: Arc<Router>,
    stream_id: u64,
    inbound: Receiver<Bytes>,
    tracked: Tracked,
    _stream: Stream,
}

pub fn connect_request(template: &str, target: &Target) -> Result<Request<()>, Error> {
    let mut req = Request::builder()
        .method(Method::CONNECT)
        .uri(connect_udp::expand(template, target)?)
        .header(request::CAPSULE_PROTOCOL, "?1")
        .body(())?;
    req.extensions_mut().insert(Protocol::CONNECT_UDP);
    Ok(req)
}

impl Client {
    pub async fn connect(endpoint: &Endpoint, addr: SocketAddr, name: &str) -> Result<Self, Error> {
        let quic = endpoint.connect(addr, name)?.await?;
        let router = Arc::new(Router::default());
        tokio::spawn(router::demux(quic.clone(), router.clone()));
        let (mut driver, send) = h3::client::builder()
            .enable_extended_connect(true)
            .enable_datagram(true)
            .build(h3_quinn::Connection::new(quic.clone()))
            .await?;
        tokio::spawn(async move { driver.wait_idle().await });
        Ok(Self {
            quic,
            send,
            router,
            stats: Arc::default(),
        })
    }

    pub fn stats(&self) -> Arc<Stats> {
        self.stats.clone()
    }

    pub async fn open(&self, template: &str, target: &Target) -> Result<UdpTunnel, Error> {
        let req = connect_request(template, target)?;
        let mut stream = self.send.clone().send_request(req.clone()).await?;
        let stream_id = stream.id().into_inner();
        self.stats.emit(event::request(stream_id, &req));
        let inbound = self.router.register(stream_id);
        let resp = stream.recv_response().await?;
        self.stats.emit(event::response(stream_id, &resp));
        let status = resp.status();
        if !status.is_success() {
            self.router.remove(stream_id);
            return Err(format!("proxy refused tunnel: {status}").into());
        }
        Ok(UdpTunnel {
            quic: self.quic.clone(),
            router: self.router.clone(),
            stream_id,
            inbound,
            tracked: self.stats.track(stream_id, target.clone()),
            _stream: stream,
        })
    }
}

impl UdpTunnel {
    pub fn send(&self, payload: &[u8]) -> Result<(), Error> {
        let sent = self
            .quic
            .send_datagram(tunnel::to_quic(self.stream_id, payload)?);
        match &sent {
            Ok(()) => self.tracked.up(payload.len()),
            Err(_) => self.tracked.drop_one(),
        }
        Ok(sent?)
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        loop {
            let http_payload = self.inbound.recv().await?;
            match tunnel::to_udp(&http_payload) {
                Some(udp) => {
                    self.tracked.down(udp.len());
                    return Some(http_payload.slice_ref(udp));
                }
                None => self.tracked.drop_one(),
            }
        }
    }
}

impl Drop for UdpTunnel {
    fn drop(&mut self) {
        self.router.remove(self.stream_id);
    }
}
