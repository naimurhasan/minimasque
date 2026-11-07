use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

use quinn::{Connection, Endpoint};

use crate::Error;
use crate::event::Event;
use crate::stats::Stats;

const PATH_POLL: Duration = Duration::from_millis(250);

#[derive(Clone)]
pub struct Migrator {
    endpoint: Endpoint,
    stats: Arc<Stats>,
}

impl Migrator {
    pub fn new(endpoint: Endpoint, stats: Arc<Stats>) -> Self {
        Self { endpoint, stats }
    }

    pub fn migrate(&self) -> Result<SocketAddr, Error> {
        let from = self.endpoint.local_addr()?;
        let socket = UdpSocket::bind(SocketAddr::new(from.ip(), 0))?;
        let to = socket.local_addr()?;
        self.endpoint.rebind(socket)?;
        self.stats.emit(Event::Migrated { from, to });
        Ok(to)
    }
}

pub async fn watch(quic: Connection, stats: Arc<Stats>) {
    let mut current = quic.remote_address();
    loop {
        tokio::select! {
            _ = quic.closed() => return,
            _ = tokio::time::sleep(PATH_POLL) => {}
        }
        let next = quic.remote_address();
        if next != current {
            stats.emit(Event::Migrated {
                from: current,
                to: next,
            });
            current = next;
        }
    }
}
