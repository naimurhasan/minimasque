use std::sync::Arc;

use bytes::Bytes;
use h3::server::RequestStream;
use masque_core::ip_capsule::{
    self, ADDRESS_ASSIGN, ADDRESS_REQUEST, Address, ROUTE_ADVERTISEMENT,
};
use masque_core::ip_packet;
use tokio::sync::mpsc::Receiver;

use crate::Error;
use crate::capsules::{self, Reader};
use crate::event::Event;
use crate::gateway::{Gateway, Lease};
use crate::router::Router;
use crate::stats::{Stats, Tracked};
use crate::tunnel::{to_quic, to_udp};

pub struct IpTunnel {
    pub quic: quinn::Connection,
    pub router: Arc<Router>,
    pub stream_id: u64,
    pub gateway: Arc<Gateway>,
    pub lease: Lease,
    pub tracked: Tracked,
    pub stats: Arc<Stats>,
}

pub fn assign(request_id: u64, lease: &Lease) -> Address {
    Address {
        request_id,
        prefix: lease.prefix(),
    }
}

impl IpTunnel {
    pub async fn relay<S>(
        self,
        mut quic_in: Receiver<Bytes>,
        mut net_in: Receiver<Bytes>,
        stream: &mut RequestStream<S, Bytes>,
    ) -> Result<(), Error>
    where
        S: h3::quic::BidiStream<Bytes>,
    {
        let result = match self.announce(stream, 0).await {
            Ok(()) => self.pump(&mut quic_in, &mut net_in, stream).await,
            Err(err) => Err(err),
        };
        self.router.remove(self.stream_id);
        result
    }

    pub async fn announce<S>(
        &self,
        stream: &mut RequestStream<S, Bytes>,
        request_id: u64,
    ) -> Result<(), Error>
    where
        S: h3::quic::BidiStream<Bytes>,
    {
        let addresses = vec![assign(request_id, &self.lease)];
        let routes = self.gateway.routes().to_vec();
        let mut assign_payload = Vec::new();
        ip_capsule::encode_addresses(&addresses, &mut assign_payload)?;
        let mut route_payload = Vec::new();
        ip_capsule::encode_routes(&routes, &mut route_payload)?;
        self.stats.emit(Event::AddressAssign {
            stream_id: self.stream_id,
            addresses,
        });
        stream
            .send_data(capsules::encode(ADDRESS_ASSIGN, &assign_payload)?)
            .await?;
        self.stats.emit(Event::RouteAdvertisement {
            stream_id: self.stream_id,
            routes,
        });
        stream
            .send_data(capsules::encode(ROUTE_ADVERTISEMENT, &route_payload)?)
            .await?;
        Ok(())
    }

    fn uplink(&self, payload: &Bytes) {
        let accepted = to_udp(payload)
            .filter(|packet| ip_packet::source(packet) == Some(self.lease.addr))
            .is_some_and(|packet| {
                let len = packet.len();
                self.gateway
                    .send(payload.slice_ref(packet))
                    .then(|| self.tracked.up(len))
                    .is_some()
            });
        if !accepted {
            self.tracked.drop_one();
        }
    }

    async fn pump<S>(
        &self,
        quic_in: &mut Receiver<Bytes>,
        net_in: &mut Receiver<Bytes>,
        stream: &mut RequestStream<S, Bytes>,
    ) -> Result<(), Error>
    where
        S: h3::quic::BidiStream<Bytes>,
    {
        let mut reader = Reader::default();
        loop {
            tokio::select! {
                Some(payload) = quic_in.recv() => self.uplink(&payload),
                Some(packet) = net_in.recv() => {
                    match self.quic.send_datagram(to_quic(self.stream_id, &packet)?) {
                        Ok(()) => self.tracked.down(packet.len()),
                        Err(_) => self.tracked.drop_one(),
                    }
                }
                data = stream.recv_data() => {
                    let Some(chunk) = data? else { return Ok(()) };
                    for (kind, payload) in reader.push(chunk) {
                        if kind == ADDRESS_REQUEST {
                            for request in ip_capsule::decode_addresses(&payload)? {
                                self.announce(stream, request.request_id).await?;
                            }
                        }
                    }
                }
            }
        }
    }
}
