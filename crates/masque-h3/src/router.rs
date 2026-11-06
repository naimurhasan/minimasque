use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use masque_core::h3_datagram;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::Error;

const QUEUE: usize = 256;

#[derive(Default)]
pub struct Router(Mutex<HashMap<u64, Sender<Bytes>>>);

impl Router {
    pub fn register(&self, stream_id: u64) -> Receiver<Bytes> {
        let (tx, rx) = mpsc::channel(QUEUE);
        self.routes().insert(stream_id, tx);
        rx
    }

    pub fn remove(&self, stream_id: u64) {
        self.routes().remove(&stream_id);
    }

    pub fn route(&self, stream_id: u64, payload: Bytes) {
        if let Some(tx) = self.routes().get(&stream_id) {
            let _ = tx.try_send(payload);
        }
    }

    fn routes(&self) -> std::sync::MutexGuard<'_, HashMap<u64, Sender<Bytes>>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

pub async fn demux(quic: quinn::Connection, router: Arc<Router>) -> Result<(), Error> {
    loop {
        let datagram = quic.read_datagram().await?;
        if let Ok((stream_id, payload)) = h3_datagram::decode(&datagram) {
            router.route(stream_id, datagram.slice_ref(payload));
        }
    }
}
