use std::sync::Arc;

use masque_h3::event::Event;
use masque_h3::stats::{Snapshot, Stats};
use tokio::sync::broadcast::Receiver;

pub struct Feed {
    stats: Arc<Stats>,
    events: Receiver<Event>,
}

impl Feed {
    pub fn new(stats: Arc<Stats>) -> Self {
        let events = stats.subscribe();
        Self { stats, events }
    }

    pub fn poll(&mut self) -> (Snapshot, Vec<Event>) {
        let events = std::iter::from_fn(|| self.events.try_recv().ok()).collect();
        (self.stats.snapshot(), events)
    }
}
