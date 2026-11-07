use std::collections::VecDeque;
use std::time::Duration;

pub struct History {
    samples: VecDeque<u64>,
    capacity: usize,
}

pub fn rate(previous: u64, current: u64, elapsed: Duration) -> u64 {
    if elapsed.is_zero() {
        return 0;
    }
    (current.saturating_sub(previous) as f64 / elapsed.as_secs_f64()) as u64
}

impl History {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, sample: u64) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn latest(&self) -> u64 {
        self.samples.back().copied().unwrap_or_default()
    }

    pub fn samples(&self) -> impl Iterator<Item = u64> + '_ {
        self.samples.iter().copied()
    }
}
