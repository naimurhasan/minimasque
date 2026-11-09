use std::sync::Arc;

use bytes::Bytes;
use h3::client::RequestStream;
use masque_core::ip_capsule::{self, ADDRESS_ASSIGN, Address, ROUTE_ADVERTISEMENT, Range};
use tokio::sync::mpsc::Receiver;

use crate::Error;
use crate::capsules::Reader;
use crate::event::Event;
use crate::router::Router;
use crate::stats::{Stats, Tracked};
use crate::tunnel::{to_quic, to_udp};

pub type Stream = RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>;

pub struct IpSession {
    pub addresses: Vec<Address>,
    pub routes: Vec<Range>,
    pub(crate) quic: quinn::Connection,
    pub(crate) router: Arc<Router>,
    pub(crate) stream_id: u64,
    pub(crate) inbound: Receiver<Bytes>,
    pub(crate) tracked: Tracked,
    pub(crate) _stream: Stream,
}

pub async fn configuration(
    stream: &mut Stream,
    stats: &Stats,
    stream_id: u64,
) -> Result<(Vec<Address>, Vec<Range>), Error> {
    let mut reader = Reader::default();
    let (mut addresses, mut routes) = (None, None);
    while addresses.is_none() || routes.is_none() {
        let chunk = stream
            .recv_data()
            .await?
            .ok_or("proxy closed the stream before assigning an address")?;
        for (kind, payload) in reader.push(chunk) {
            match kind {
                ADDRESS_ASSIGN => {
                    let assigned = ip_capsule::decode_addresses(&payload)?;
                    stats.emit(Event::AddressAssign {
                        stream_id,
                        addresses: assigned.clone(),
                    });
                    addresses = Some(assigned);
                }
                ROUTE_ADVERTISEMENT => {
                    let advertised = ip_capsule::decode_routes(&payload)?;
                    stats.emit(Event::RouteAdvertisement {
                        stream_id,
                        routes: advertised.clone(),
                    });
                    routes = Some(advertised);
                }
                _ => {}
            }
        }
    }
    Ok((addresses.unwrap_or_default(), routes.unwrap_or_default()))
}

impl IpSession {
    pub fn send(&self, packet: &[u8]) -> Result<(), Error> {
        let sent = self.quic.send_datagram(to_quic(self.stream_id, packet)?);
        match &sent {
            Ok(()) => self.tracked.up(packet.len()),
            Err(_) => self.tracked.drop_one(),
        }
        Ok(sent?)
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        loop {
            let http_payload = self.inbound.recv().await?;
            match to_udp(&http_payload) {
                Some(packet) => {
                    self.tracked.down(packet.len());
                    return Some(http_payload.slice_ref(packet));
                }
                None => self.tracked.drop_one(),
            }
        }
    }
}

impl Drop for IpSession {
    fn drop(&mut self) {
        self.router.remove(self.stream_id);
    }
}
