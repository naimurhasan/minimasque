use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use bytes::Bytes;
use masque_core::ip_capsule::{Prefix, Range};
use masque_core::ip_packet;
use tokio::sync::mpsc::{self, Receiver, Sender};

const QUEUE: usize = 512;
const FIRST_CLIENT: u32 = 2;

pub struct Gateway {
    pub template: Arc<str>,
    pool: (Ipv4Addr, u8),
    routes: Vec<Range>,
    leases: Mutex<HashMap<IpAddr, Sender<Bytes>>>,
    outbound: Sender<Bytes>,
}

pub struct Lease {
    pub addr: IpAddr,
    gateway: Arc<Gateway>,
}

pub fn host(network: Ipv4Addr, len: u8, index: u32) -> Option<Ipv4Addr> {
    let size = 1u64 << (32 - u32::from(len.min(32)));
    (u64::from(index) < size.saturating_sub(1))
        .then(|| Ipv4Addr::from(u32::from(network) & !((size - 1) as u32) | index))
}

impl Gateway {
    pub fn new(
        template: Arc<str>,
        pool: (Ipv4Addr, u8),
        routes: Vec<Range>,
    ) -> (Arc<Self>, Receiver<Bytes>) {
        let (outbound, packets) = mpsc::channel(QUEUE);
        let gateway = Arc::new(Self {
            template,
            pool,
            routes,
            leases: Mutex::default(),
            outbound,
        });
        (gateway, packets)
    }

    pub fn routes(&self) -> &[Range] {
        &self.routes
    }

    pub fn lease(self: &Arc<Self>) -> Option<(Lease, Receiver<Bytes>)> {
        let mut leases = self.leases();
        let addr = (FIRST_CLIENT..)
            .map_while(|index| host(self.pool.0, self.pool.1, index))
            .map(IpAddr::V4)
            .find(|addr| !leases.contains_key(addr))?;
        let (tx, rx) = mpsc::channel(QUEUE);
        leases.insert(addr, tx);
        let lease = Lease {
            addr,
            gateway: self.clone(),
        };
        Some((lease, rx))
    }

    pub fn send(&self, packet: Bytes) -> bool {
        self.outbound.try_send(packet).is_ok()
    }

    pub fn deliver(&self, packet: Bytes) -> bool {
        let Some(destination) = ip_packet::destination(&packet) else {
            return false;
        };
        self.leases()
            .get(&destination)
            .is_some_and(|tx| tx.try_send(packet).is_ok())
    }

    fn leases(&self) -> MutexGuard<'_, HashMap<IpAddr, Sender<Bytes>>> {
        self.leases.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Lease {
    pub fn prefix(&self) -> Prefix {
        Prefix {
            addr: self.addr,
            len: 32,
        }
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.gateway.leases().remove(&self.addr);
    }
}
