//! 1-minute load-average + inflight counter overload check.

use std::sync::atomic::{AtomicU64, Ordering};
use sysinfo::System;

pub struct Load {
    inflight: AtomicU64,
    #[allow(dead_code)]
    sys: System,
    cpus: usize,
}

impl Load {
    pub fn new() -> Self {
        let mut s = System::new();
        s.refresh_cpu_all();
        let cpus = s.cpus().len();
        Self {
            inflight: AtomicU64::new(0),
            sys: s,
            cpus,
        }
    }
    pub fn inc(&self) {
        self.inflight.fetch_add(1, Ordering::Relaxed);
    }
    pub fn dec(&self) {
        self.inflight.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn overload(&mut self, max_inflight: u64) -> bool {
        let load_avg = System::load_average();
        let load1 = load_avg.one;
        load1 > self.cpus as f64 || self.inflight.load(Ordering::Relaxed) > max_inflight
    }
}
