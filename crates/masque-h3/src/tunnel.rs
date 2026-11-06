use std::sync::Arc;

use bytes::{Buf, Bytes};
use h3::server::RequestStream;
use masque_core::{Datagram, datagram, h3_datagram};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Receiver;

use crate::Error;
use crate::router::Router;

const MAX_UDP: usize = 65_535;

pub struct Tunnel {
    pub quic: quinn::Connection,
    pub router: Arc<Router>,
    pub stream_id: u64,
    pub socket: UdpSocket,
}

pub fn to_udp(http_payload: &[u8]) -> Option<&[u8]> {
    datagram::decode(http_payload)
        .ok()
        .filter(|d| d.context_id == datagram::UDP_PAYLOAD)
        .map(|d| d.payload)
}

pub fn to_quic(stream_id: u64, udp_payload: &[u8]) -> Result<Bytes, Error> {
    let mut http_payload = Vec::with_capacity(udp_payload.len() + 1);
    datagram::encode(
        Datagram {
            context_id: datagram::UDP_PAYLOAD,
            payload: udp_payload,
        },
        &mut http_payload,
    )?;
    let mut out = Vec::with_capacity(http_payload.len() + 8);
    h3_datagram::encode(stream_id, &http_payload, &mut out)?;
    Ok(out.into())
}

impl Tunnel {
    pub async fn relay<S, B>(
        self,
        mut inbound: Receiver<Bytes>,
        stream: &mut RequestStream<S, B>,
    ) -> Result<(), Error>
    where
        S: h3::quic::BidiStream<B>,
        B: Buf,
    {
        let result = self.pump(&mut inbound, stream).await;
        self.router.remove(self.stream_id);
        result
    }

    async fn pump<S, B>(
        &self,
        inbound: &mut Receiver<Bytes>,
        stream: &mut RequestStream<S, B>,
    ) -> Result<(), Error>
    where
        S: h3::quic::BidiStream<B>,
        B: Buf,
    {
        let mut buf = vec![0; MAX_UDP];
        loop {
            tokio::select! {
                Some(payload) = inbound.recv() => {
                    if let Some(udp) = to_udp(&payload) {
                        let _ = self.socket.send(udp).await;
                    }
                }
                received = self.socket.recv(&mut buf) => {
                    let _ = self.quic.send_datagram(to_quic(self.stream_id, &buf[..received?])?);
                }
                data = stream.recv_data() => {
                    if data?.is_none() {
                        return Ok(());
                    }
                }
            }
        }
    }
}
