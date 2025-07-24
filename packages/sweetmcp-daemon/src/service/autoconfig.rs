use crate::config::ServiceDefinition;
use crate::ipc::{Cmd, Evt};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use sweetmcp_client_autoconfig::{clients::all_clients, watcher::AutoConfigWatcher};
use tokio_util::sync::CancellationToken;

/// Auto-configuration service that watches for MCP client installations
pub struct AutoConfigService {
    name: String,
    bus: Sender<Evt>,
}

impl AutoConfigService {
    pub fn new(def: ServiceDefinition, bus: Sender<Evt>) -> Self {
        Self {
            name: def.name,
            bus,
        }
    }

    pub fn run(self, cmd_rx: Receiver<Cmd>) -> Result<()> {
        info!("🍯 Starting MCP client auto-configuration service");

        // Create tokio runtime for the watcher
        let rt = tokio::runtime::Runtime::new()?;

        // Create cancellation token for graceful shutdown
        let cancel_token = CancellationToken::new();
        let shutdown_complete = Arc::new(AtomicBool::new(false));

        // Create the watcher with all client plugins
        let clients = all_clients();
        let watcher = AutoConfigWatcher::new(clients)?;

        // Spawn the watcher task with graceful cancellation
        let watcher_handle = rt.spawn({
            let bus = self.bus.clone();
            let service_name = self.name.clone();
            let cancel_token = cancel_token.clone();
            let shutdown_complete = shutdown_complete.clone();

            async move {
                // Notify daemon we're starting
                let _ = bus.send(Evt::State {
                    service: service_name.clone(),
                    kind: "running",
                    ts: chrono::Utc::now(),
                    pid: Some(std::process::id()),
                });

                // Run watcher with cancellation support
                tokio::select! {
                    result = watcher.run() => {
                        if let Err(e) = result {
                            error!("Auto-config watcher failed: {}", e);
                            let _ = bus.send(Evt::Fatal {
                                service: service_name.clone(),
                                msg: "Watcher error occurred",
                                ts: chrono::Utc::now(),
                            });
                        }
                    }
                    _ = cancel_token.cancelled() => {
                        info!("Auto-config watcher cancelled gracefully");
                        let _ = bus.send(Evt::State {
                            service: service_name.clone(),
                            kind: "stopped",
                            ts: chrono::Utc::now(),
                            pid: Some(std::process::id()),
                        });
                    }
                }

                // Signal shutdown completion atomically
                shutdown_complete.store(true, Ordering::Release);
            }
        });

        // Handle control commands with lock-free coordination
        loop {
            match cmd_rx.recv()? {
                Cmd::Start => {
                    info!("Auto-config service already started");
                }
                Cmd::Stop => {
                    info!("Stopping auto-config service");

                    // Trigger graceful cancellation
                    cancel_token.cancel();

                    // Wait for shutdown completion with timeout
                    let shutdown_timeout = std::time::Duration::from_secs(5);
                    let start_time = std::time::Instant::now();

                    // Spin-wait with exponential backoff for shutdown completion
                    let mut backoff_ms = 1;
                    while !shutdown_complete.load(Ordering::Acquire) {
                        if start_time.elapsed() > shutdown_timeout {
                            info!("Graceful shutdown timeout, aborting task");
                            watcher_handle.abort();
                            break;
                        }

                        // Lock-free backoff using thread sleep
                        std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                        backoff_ms = (backoff_ms * 2).min(100); // Cap at 100ms
                    }

                    break;
                }
                Cmd::Shutdown => {
                    info!("Shutting down auto-config service");

                    // Trigger graceful cancellation
                    cancel_token.cancel();

                    // Wait for shutdown completion with timeout
                    let shutdown_timeout = std::time::Duration::from_secs(5);
                    let start_time = std::time::Instant::now();

                    // Spin-wait with exponential backoff for shutdown completion
                    let mut backoff_ms = 1;
                    while !shutdown_complete.load(Ordering::Acquire) {
                        if start_time.elapsed() > shutdown_timeout {
                            info!("Graceful shutdown timeout, aborting task");
                            watcher_handle.abort();
                            break;
                        }

                        // Lock-free backoff using thread sleep
                        std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                        backoff_ms = (backoff_ms * 2).min(100); // Cap at 100ms
                    }

                    break;
                }
                _ => {}
            }
        }

        // Ensure task is fully cleaned up
        if !shutdown_complete.load(Ordering::Acquire) {
            watcher_handle.abort();
        }

        // Wait for final task completion
        let _ = rt.block_on(watcher_handle);

        Ok(())
    }
}

/// Spawn the auto-configuration service thread
pub fn spawn_autoconfig(def: ServiceDefinition, bus: Sender<Evt>) -> Sender<Cmd> {
    let (cmd_tx, cmd_rx) = crossbeam_channel::bounded(16);

    thread::spawn(move || {
        let service = AutoConfigService::new(def, bus);
        if let Err(e) = service.run(cmd_rx) {
            error!("Auto-config service error: {}", e);
        }
    });

    cmd_tx
}
