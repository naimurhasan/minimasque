use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use masque_core::Target;
use tokio::sync::broadcast;

use crate::counters::{CounterSnapshot, Counters};
use crate::event::Event;

const EVENT_BACKLOG: usize = 1024;

struct Entry {
    target: Target,
    opened: Instant,
    counters: Counters,
}

pub struct Stats {
    next_id: AtomicU64,
    open: Mutex<BTreeMap<u64, Arc<Entry>>>,
    totals: Counters,
    events: broadcast::Sender<Event>,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            next_id: AtomicU64::default(),
            open: Mutex::default(),
            totals: Counters::default(),
            events: broadcast::channel(EVENT_BACKLOG).0,
        }
    }
}

pub struct Tracked {
    id: u64,
    stats: Arc<Stats>,
    entry: Arc<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelSnapshot {
    pub id: u64,
    pub target: Target,
    pub age: Duration,
    pub counters: CounterSnapshot,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub opened: u64,
    pub totals: CounterSnapshot,
    pub tunnels: Vec<TunnelSnapshot>,
}

impl Stats {
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    pub fn emit(&self, event: Event) {
        let _ = self.events.send(event);
    }

    pub fn track(self: &Arc<Self>, stream_id: u64, target: Target) -> Tracked {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.emit(Event::Opened {
            tunnel_id: id,
            stream_id,
            target: target.clone(),
        });
        let entry = Arc::new(Entry {
            target,
            opened: Instant::now(),
            counters: Counters::default(),
        });
        self.open().insert(id, entry.clone());
        Tracked {
            id,
            stats: self.clone(),
            entry,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let now = Instant::now();
        Snapshot {
            opened: self.next_id.load(Ordering::Relaxed),
            totals: self.totals.snapshot(),
            tunnels: self
                .open()
                .iter()
                .map(|(id, entry)| TunnelSnapshot {
                    id: *id,
                    target: entry.target.clone(),
                    age: now - entry.opened,
                    counters: entry.counters.snapshot(),
                })
                .collect(),
        }
    }

    fn open(&self) -> MutexGuard<'_, BTreeMap<u64, Arc<Entry>>> {
        self.open.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Tracked {
    pub fn up(&self, bytes: usize) {
        self.entry.counters.up(bytes);
        self.stats.totals.up(bytes);
    }

    pub fn down(&self, bytes: usize) {
        self.entry.counters.down(bytes);
        self.stats.totals.down(bytes);
    }

    pub fn drop_one(&self) {
        self.entry.counters.drop_one();
        self.stats.totals.drop_one();
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        self.stats.open().remove(&self.id);
        self.stats.emit(Event::Closed { tunnel_id: self.id });
    }
}
