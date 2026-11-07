use std::collections::VecDeque;
use std::time::Duration;

use masque_h3::event::Event;
use masque_h3::stats::Snapshot;

use super::history::{History, rate};

const LOG_CAPACITY: usize = 200;
const HISTORY_CAPACITY: usize = 240;

pub struct App {
    pub mode: &'static str,
    pub address: String,
    pub snapshot: Snapshot,
    pub log: VecDeque<Event>,
    pub up: History,
    pub down: History,
}

impl App {
    pub fn new(mode: &'static str, address: String) -> Self {
        Self {
            mode,
            address,
            snapshot: Snapshot::default(),
            log: VecDeque::new(),
            up: History::new(HISTORY_CAPACITY),
            down: History::new(HISTORY_CAPACITY),
        }
    }

    pub fn update(
        &mut self,
        snapshot: Snapshot,
        events: impl IntoIterator<Item = Event>,
        elapsed: Duration,
    ) {
        let (before, after) = (&self.snapshot.totals, &snapshot.totals);
        self.up.push(rate(before.bytes_up, after.bytes_up, elapsed));
        self.down
            .push(rate(before.bytes_down, after.bytes_down, elapsed));
        self.snapshot = snapshot;
        for event in events {
            if self.log.len() == LOG_CAPACITY {
                self.log.pop_front();
            }
            self.log.push_back(event);
        }
    }
}
