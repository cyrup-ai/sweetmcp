use crate::config::ServiceDefinition;
use crate::ipc::{Cmd, Evt};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use std::thread;
use sweetmcp_client_autoconfig::{clients::all_clients, watcher::AutoConfigWatcher};

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

        // Create the watcher with all client plugins
        let clients = all_clients();
        let watcher = AutoConfigWatcher::new(clients)?;

        // Spawn the watcher task
        let watcher_handle = rt.spawn({
            let bus = self.bus.clone();

            async move {
                // Notify daemon we're starting
                let _ = bus.send(Evt::State {
                    service: "autoconfig",
                    kind: "running",
                    ts: chrono::Utc::now(),
                    pid: Some(std::process::id()),
                });

                if let Err(e) = watcher.run().await {
                    error!("Auto-config watcher failed: {}", e);
                    let _ = bus.send(Evt::Fatal {
                        service: "autoconfig",
                        msg: "Watcher error occurred",
                        ts: chrono::Utc::now(),
                    });
                }
            }
        });

        // Handle control commands
        loop {
            match cmd_rx.recv()? {
                Cmd::Start => {
                    info!("Auto-config service already started");
                }
                Cmd::Stop => {
                    info!("Stopping auto-config service");
                    rt.block_on(async {
                        watcher_handle.abort();
                    });
                    break;
                }
                Cmd::Shutdown => {
                    info!("Shutting down auto-config service");
                    rt.block_on(async {
                        watcher_handle.abort();
                    });
                    break;
                }
                _ => {}
            }
        }

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
