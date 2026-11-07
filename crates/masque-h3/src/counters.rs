use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct Counters {
    packets_up: AtomicU64,
    bytes_up: AtomicU64,
    packets_down: AtomicU64,
    bytes_down: AtomicU64,
    dropped: AtomicU64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CounterSnapshot {
    pub packets_up: u64,
    pub bytes_up: u64,
    pub packets_down: u64,
    pub bytes_down: u64,
    pub dropped: u64,
}

impl Counters {
    pub fn up(&self, bytes: usize) {
        self.packets_up.fetch_add(1, Ordering::Relaxed);
        self.bytes_up.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn down(&self, bytes: usize) {
        self.packets_down.fetch_add(1, Ordering::Relaxed);
        self.bytes_down.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn drop_one(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            packets_up: self.packets_up.load(Ordering::Relaxed),
            bytes_up: self.bytes_up.load(Ordering::Relaxed),
            packets_down: self.packets_down.load(Ordering::Relaxed),
            bytes_down: self.bytes_down.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
        }
    }
}
