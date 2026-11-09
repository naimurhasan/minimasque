use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use h3::client::{RequestStream, SendRequest};
use h3::ext::Protocol;
use http::{Method, Request};
use masque_core::connect_ip::{self, Scope};
use masque_core::{Target, connect_udp};
use quinn::Endpoint;
use tokio::sync::mpsc::Receiver;

use crate::ip_session::{self, IpSession};
use crate::migrate::Migrator;
use crate::remote::Remote;
use crate::router::{self, Router};
use crate::stats::{Stats, Tracked};
use crate::{Error, event, request, tunnel};

type Stream = RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>;

pub struct Client {
    endpoint: Endpoint,
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

fn extended_connect(uri: String, protocol: Protocol) -> Result<Request<()>, Error> {
    let mut req = Request::builder()
        .method(Method::CONNECT)
        .uri(uri)
        .header(request::CAPSULE_PROTOCOL, "?1")
        .body(())?;
    req.extensions_mut().insert(protocol);
    Ok(req)
}

pub fn connect_request(template: &str, target: &Target) -> Result<Request<()>, Error> {
    extended_connect(
        connect_udp::expand(template, target)?,
        Protocol::CONNECT_UDP,
    )
}

pub fn connect_ip_request(template: &str, scope: &Scope) -> Result<Request<()>, Error> {
    extended_connect(connect_ip::expand(template, scope)?, Protocol::CONNECT_IP)
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
            endpoint: endpoint.clone(),
            quic,
            send,
            router,
            stats: Arc::default(),
        })
    }

    pub fn stats(&self) -> Arc<Stats> {
        self.stats.clone()
    }

    pub fn migrator(&self) -> Migrator {
        Migrator::new(self.endpoint.clone(), self.stats.clone())
    }

    async fn request(&self, req: Request<()>) -> Result<(Stream, u64), Error> {
        let mut stream = self.send.clone().send_request(req.clone()).await?;
        let stream_id = stream.id().into_inner();
        self.stats
            .emit(event::request(stream_id, &req, request::protocol(&req)));
        let resp = stream.recv_response().await?;
        self.stats.emit(event::response(stream_id, &resp));
        let status = resp.status();
        if !status.is_success() {
            return Err(format!("proxy refused tunnel: {status}").into());
        }
        Ok((stream, stream_id))
    }

    pub async fn open_ip(&self, template: &str, scope: &Scope) -> Result<IpSession, Error> {
        let (mut stream, stream_id) = self.request(connect_ip_request(template, scope)?).await?;
        let (addresses, routes) =
            ip_session::configuration(&mut stream, &self.stats, stream_id).await?;
        let prefix = addresses.first().ok_or("proxy assigned no address")?.prefix;
        Ok(IpSession {
            addresses,
            routes,
            quic: self.quic.clone(),
            router: self.router.clone(),
            stream_id,
            inbound: self.router.register(stream_id),
            tracked: self.stats.track(stream_id, Remote::Ip(prefix)),
            _stream: stream,
        })
    }

    pub async fn open(&self, template: &str, target: &Target) -> Result<UdpTunnel, Error> {
        let req = connect_request(template, target)?;
        let mut stream = self.send.clone().send_request(req.clone()).await?;
        let stream_id = stream.id().into_inner();
        self.stats
            .emit(event::request(stream_id, &req, request::protocol(&req)));
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
