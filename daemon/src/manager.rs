use crate::config::ServiceConfig;
use crate::ipc::{Cmd, Evt};
use anyhow::Result;
use crossbeam_channel::{bounded, select, tick, Receiver, Sender};
use log::{error, info};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::time::Duration;

/// Global event bus size – small fixed size → zero heap growth.
const BUS_BOUND: usize = 128;

/// Top‑level in‑process manager supervising *all* workers.
pub struct ServiceManager {
    bus_tx: Sender<Evt>,
    bus_rx: Receiver<Evt>,
    workers: HashMap<&'static str, Sender<Cmd>>,
}

impl ServiceManager {
    /// Load config, spawn workers, and return the fully‑primed manager.
    pub fn new(cfg: &ServiceConfig) -> Result<Self> {
        let (bus_tx, bus_rx) = bounded::<Evt>(BUS_BOUND);
        let mut workers = HashMap::new();
        for def in cfg.services.clone() {
            let tx = crate::service::spawn(def.clone(), bus_tx.clone());
            workers.insert(Box::leak(def.name.clone().into_boxed_str()) as &'static str, tx);
        }
        Ok(Self { bus_tx, bus_rx, workers })
    }

    /// Central event‑loop.  Runs until SIGINT / SIGTERM.
    pub fn run(mut self) -> Result<()> {
        // Initial start‑up pass.
        for tx in self.workers.values() {
            tx.send(Cmd::Start)?;
        }

        let sig_tick = tick(Duration::from_millis(200));
        loop {
            select! {
                recv(self.bus_rx) -> evt => self.handle_event(evt?)?,
                recv(sig_tick)    -> _   => {
                    if let Some(sig) = check_signals() { // coarse polling ≈200 ms
                        info!("signal {:?} – orderly shutdown", sig);
                        for tx in self.workers.values() { tx.send(Cmd::Shutdown).ok(); }
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, evt: Evt) -> Result<()> {
        match &evt {
            Evt::State { service, kind, .. } => info!("{} → {}", service, kind),
            Evt::Fatal { service, msg, .. } => error!("{} FATAL {}", service, msg),
            _ => {}
        }
        Ok(())
    }
}

// Cheap, polling‑based Unix signal handling (lock‑free).
static RECEIVED_SIGNAL: Lazy<std::sync::atomic::AtomicUsize> =
    Lazy::new(|| std::sync::atomic::AtomicUsize::new(0));

pub fn install_signal_handlers() {
    use nix::sys::signal::{self, Signal};
    extern "C" fn handler(sig: i32) {
        RECEIVED_SIGNAL.store(sig as usize, std::sync::atomic::Ordering::SeqCst);
    }
    unsafe {
        signal::sigaction(
            Signal::SIGINT,
            &signal::SigAction::new(
                signal::SigHandler::Handler(handler),
                signal::SaFlags::empty(),
                signal::SigSet::empty(),
            ),
        ).unwrap();
        signal::sigaction(
            Signal::SIGTERM,
            &signal::SigAction::new(
                signal::SigHandler::Handler(handler),
                signal::SaFlags::empty(),
                signal::SigSet::empty(),
            ),
        ).unwrap();
    }
}

/// Non‑blocking check – returns Some(signal) once.
fn check_signals() -> Option<nix::sys::signal::Signal> {
    use std::sync::atomic::Ordering::*;
    use nix::sys::signal::Signal;
    let val = RECEIVED_SIGNAL.swap(0, AcqRel);
    if val == 0 { None } else { Some(Signal::try_from(val as i32).unwrap()) }
}