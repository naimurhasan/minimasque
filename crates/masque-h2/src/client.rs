use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use h2::client::SendRequest;
use h2::{RecvStream, SendStream};
use http::{Method, Request};
use masque_core::{Target, connect_udp};
use masque_h3::stats::{Stats, Tracked};
use masque_h3::{Error, event, request};
use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use crate::capsules::{Reader, encode_udp};

pub struct Client {
    send: SendRequest<Bytes>,
    stats: Arc<Stats>,
}

pub struct Tunnel {
    send: SendStream<Bytes>,
    body: RecvStream,
    reader: Reader,
    pending: VecDeque<Bytes>,
    tracked: Tracked,
}

pub fn connect_request(template: &str, target: &Target) -> Result<Request<()>, Error> {
    let mut req = Request::builder()
        .method(Method::CONNECT)
        .uri(connect_udp::expand(template, target)?)
        .header(request::CAPSULE_PROTOCOL, "?1")
        .body(())?;
    req.extensions_mut()
        .insert(h2::ext::Protocol::from(connect_udp::PROTOCOL));
    Ok(req)
}

impl Client {
    pub async fn connect(
        addr: SocketAddr,
        name: &str,
        connector: &TlsConnector,
    ) -> Result<Self, Error> {
        let tcp = TcpStream::connect(addr).await?;
        let tls = connector
            .connect(ServerName::try_from(name.to_owned())?, tcp)
            .await?;
        let (send, connection) = h2::client::handshake(tls).await?;
        tokio::spawn(connection);
        Ok(Self {
            send: send.ready().await?,
            stats: Arc::default(),
        })
    }

    pub fn stats(&self) -> Arc<Stats> {
        self.stats.clone()
    }

    pub async fn open(&self, template: &str, target: &Target) -> Result<Tunnel, Error> {
        let head = connect_request(template, target)?;
        let mut sender = self.send.clone().ready().await?;
        let (response, send) = sender.send_request(connect_request(template, target)?, false)?;
        let stream_id = u64::from(u32::from(response.stream_id()));
        self.stats.emit(event::request(
            stream_id,
            &head,
            Some(connect_udp::PROTOCOL),
        ));
        let resp = response.await?;
        let (parts, body) = resp.into_parts();
        let head = http::Response::from_parts(parts, ());
        self.stats.emit(event::response(stream_id, &head));
        if !head.status().is_success() {
            return Err(format!("proxy refused tunnel: {}", head.status()).into());
        }
        Ok(Tunnel {
            send,
            body,
            reader: Reader::default(),
            pending: VecDeque::new(),
            tracked: self.stats.track(stream_id, target.clone()),
        })
    }
}

impl Tunnel {
    pub fn send(&mut self, payload: &[u8]) -> Result<(), Error> {
        let sent = self.send.send_data(encode_udp(payload)?, false);
        match &sent {
            Ok(()) => self.tracked.up(payload.len()),
            Err(_) => self.tracked.drop_one(),
        }
        Ok(sent?)
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        loop {
            if let Some(udp) = self.pending.pop_front() {
                self.tracked.down(udp.len());
                return Some(udp);
            }
            let chunk = self.body.data().await?.ok()?;
            let _ = self.body.flow_control().release_capacity(chunk.len());
            let decoded = self.reader.push(&chunk);
            (0..decoded.skipped).for_each(|_| self.tracked.drop_one());
            self.pending.extend(decoded.udp);
        }
    }
}
