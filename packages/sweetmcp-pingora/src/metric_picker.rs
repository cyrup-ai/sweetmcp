//! Sugora peer picker that selects the lowest `node_load1`.

use pingora::protocols::l4::socket::SocketAddr;
use pingora_load_balancing::Backend;
use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

/// Metrics-based backend picker that selects the backend with lowest load
pub struct MetricPicker {
    backends: Vec<Backend>,
    load_values: Vec<Arc<AtomicU64>>, // f64 bits representation
}

impl MetricPicker {
    /// Create a new metric picker from a set of backends
    #[inline]
    pub fn from_backends(backends: &BTreeSet<Backend>) -> Self {
        let backends_vec: Vec<Backend> = backends.iter().cloned().collect();
        let load_values: Vec<Arc<AtomicU64>> = (0..backends_vec.len())
            .map(|_| Arc::new(AtomicU64::new(0)))
            .collect();

        Self {
            backends: backends_vec,
            load_values,
        }
    }

    /// Get the backends and their URLs for metrics collection
    #[inline]
    pub fn get_metrics_targets(&self) -> Vec<(usize, String)> {
        self.backends
            .iter()
            .enumerate()
            .filter_map(|(idx, backend)| {
                match &backend.addr {
                    SocketAddr::Inet(addr) => Some((idx, format!("http://{}/metrics", addr))),
                    SocketAddr::Unix(_) => None, // Skip Unix sockets
                }
            })
            .collect()
    }

    /// Update load value for a specific backend
    #[inline]
    pub fn update_load(&self, backend_idx: usize, load_value: f64) {
        if let Some(load_atomic) = self.load_values.get(backend_idx) {
            load_atomic.store(load_value.to_bits(), Ordering::Release);
        }
    }

    /// Pick the backend with the lowest load
    #[inline]
    pub fn pick(&self) -> Option<&Backend> {
        if self.backends.is_empty() {
            return None;
        }

        // Fast path for single backend
        if self.backends.len() == 1 {
            return Some(&self.backends[0]);
        }

        let idx = self
            .load_values
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let a_val = f64::from_bits(a.load(Ordering::Acquire));
                let b_val = f64::from_bits(b.load(Ordering::Acquire));

                // Handle NaN values safely - treat NaN as higher load (less preferred)
                match (a_val.is_nan(), b_val.is_nan()) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    (false, false) => a_val
                        .partial_cmp(&b_val)
                        .unwrap_or(std::cmp::Ordering::Equal),
                }
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        Some(&self.backends[idx])
    }
}
