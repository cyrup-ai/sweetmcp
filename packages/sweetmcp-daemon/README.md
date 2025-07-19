# Cyrup Daemon

A high-performance Unix daemon library with crossbeam-based service management.

## Features

- **Zero-allocation hot paths**: Uses crossbeam channels and fixed-size enums
- **True Unix daemon**: Double-fork daemonization with systemd auto-detection  
- **Service management**: Supervise processes with automatic restart and health monitoring
- **System integration**: Self-installation with systemd/launchd support
- **Async compatible**: Can manage async applications while using sync internals

## Quick Start

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
cyrup-daemon = "0.1"
```

Use in your application:

```rust
use cyrup_daemon::{ServiceConfig, ServiceDefinition, ServiceManager, daemonise};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Daemonize the process
    let pid_file = Path::new("/var/run/mydaemon.pid");
    daemonise(&pid_file)?;

    // Create service configuration
    let mut config = ServiceConfig::default();
    config.services.push(ServiceDefinition {
        name: "my-service".to_string(),
        command: "/usr/bin/my-app --daemon".to_string(),
        auto_restart: true,
        ..Default::default()
    });

    // Start daemon manager
    let manager = ServiceManager::new(&config)?;
    manager.run()?; // Runs until SIGTERM/SIGINT
    Ok(())
}
```

### As a Standalone Daemon

Build and install:

```bash
cargo build --release
sudo ./target/release/cyrupd install
```

Configure services in `/etc/cyrupd/services/`:

```toml
# /etc/cyrupd/services/my-app.toml
name = "my-app"
command = "/usr/local/bin/my-app --serve"
auto_restart = true
user = "www-data"
restart_delay_s = 5

[health_check]
check_type = "http"
target = "http://localhost:8080/health"
interval_secs = 30
```

Start the daemon:

```bash
sudo systemctl start cyrupd
```

## Architecture

- **ServiceManager**: Central event loop using crossbeam channels
- **ServiceWorker**: Individual service supervisor threads  
- **IPC**: Wait-free message passing with `Cmd`/`Evt` enums
- **Daemon**: Unix daemonization with systemd detection
- **Installer**: Self-installation with system integration

## System Requirements

- Unix-like OS (Linux, macOS)
- systemd (Linux) or launchd (macOS) for system integration
- Root privileges for installation

## License

Apache-2.0