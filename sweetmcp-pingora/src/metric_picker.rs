//! Sugora peer picker that selects the lowest `node_load1`.

use pingora::protocols::l4::socket::SocketAddr;
use pingora_load_balancing::Backend;
// prometheus_parser crate
use reqwest::Client;
use std::{
    collections::BTreeSet,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{task, time};

pub struct MetricPicker {
    backends: Vec<Backend>,
    #[allow(dead_code)]
    urls: Vec<String>,
    load1: Vec<Arc<AtomicU64>>, // f64 bits
}

impl MetricPicker {
    pub fn from_backends(backends: &BTreeSet<Backend>) -> Self {
        let backends_vec: Vec<Backend> = backends.iter().cloned().collect();
        let urls: Vec<String> = backends_vec
            .iter()
            .filter_map(|b| {
                match &b.addr {
                    SocketAddr::Inet(addr) => Some(format!("http://{}", addr)),
                    SocketAddr::Unix(_) => {
                        // Skip Unix sockets for prometheus scraping
                        None
                    }
                }
            })
            .collect();

        let load1: Vec<Arc<AtomicU64>> = (0..backends_vec.len())
            .map(|_| Arc::new(AtomicU64::new(0)))
            .collect();

        let load_clone = load1.clone();
        let url_clone = urls.clone();
        task::spawn(async move {
            let client = Client::new();

            loop {
                for (i, url) in url_clone.iter().enumerate() {
                    let text_result = client
                        .get(format!("{url}/metrics"))
                        .timeout(Duration::from_secs(2))
                        .send()
                        .await;
                    if let Ok(response) = text_result {
                        if let Ok(text) = response.text().await {
                            // Production prometheus text format parsing
                            for line in text.lines() {
                                if line.starts_with("node_load1 ") {
                                    if let Some(value_str) = line.split_whitespace().nth(1) {
                                        if let Ok(value) = value_str.parse::<f64>() {
                                            load_clone[i].store(value.to_bits(), Ordering::Relaxed);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                time::sleep(Duration::from_secs(5)).await;
            }
        });

        Self {
            backends: backends_vec,
            urls,
            load1,
        }
    }

    pub fn pick(&self) -> Option<&Backend> {
        if self.backends.is_empty() {
            return None;
        }

        let idx = self
            .load1
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let a_val = f64::from_bits(a.load(Ordering::Relaxed));
                let b_val = f64::from_bits(b.load(Ordering::Relaxed));

                // Handle NaN values safely - treat NaN as higher load (less preferred)
                match (a_val.is_nan(), b_val.is_nan()) {
                    (true, true) => std::cmp::Ordering::Equal, // Both NaN, equal
                    (true, false) => std::cmp::Ordering::Greater, // a is NaN, b preferred
                    (false, true) => std::cmp::Ordering::Less, // b is NaN, a preferred
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
