# elevated_daemon_installer

Cross-platform privileged daemon installer with GUI authorization prompts.

## Features

- **Cross-platform support**: Linux (systemd), macOS (launchd), Windows (Service Control Manager)
- **GUI privilege escalation**: No manual `sudo` required
- **Proper service integration**: Native service management on each platform
- **User-friendly errors**: Clear distinction between cancellation and permission denial
- **Async support**: Optional tokio integration
- **Security hardening**: Proper permissions and sandboxing on each platform

## Installation

```toml
[dependencies]
elevated_daemon_installer = "0.1"
```

For async support:
```toml
[dependencies]
elevated_daemon_installer = { version = "0.1", features = ["runtime"] }
```

## Usage

### Basic Installation

```rust
use elevated_daemon_installer::{InstallerBuilder, install_daemon};

let installer = InstallerBuilder::new("my-daemon", "/usr/local/bin/my-daemon")
    .description("My Awesome Daemon Service")
    .auto_restart(true);

// This will prompt the user for admin privileges
install_daemon(installer)?;
```

### Advanced Configuration

```rust
let installer = InstallerBuilder::new("cyrupd", "/usr/local/bin/cyrupd")
    .arg("--config")
    .arg("/etc/cyrupd/config.toml")
    .env("RUST_LOG", "info")
    .env("CYRUPD_HOME", "/var/lib/cyrupd")
    .user("cyrupd")
    .group("cyops")
    .description("Cyrup Service Manager")
    .auto_restart(true)
    .network(true);

match install_daemon(installer) {
    Ok(()) => println!("Daemon installed successfully"),
    Err(InstallerError::Cancelled) => println!("User cancelled installation"),
    Err(InstallerError::PermissionDenied) => println!("Permission denied"),
    Err(e) => eprintln!("Installation failed: {}", e),
}
```

### Uninstallation

```rust
use elevated_daemon_installer::uninstall_daemon;

uninstall_daemon("my-daemon")?;
```

### Async API

```rust
use elevated_daemon_installer::{InstallerBuilder, install_daemon_async};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let installer = InstallerBuilder::new("my-daemon", "/usr/local/bin/my-daemon")
        .description("My Async Daemon");
    
    install_daemon_async(installer).await?;
    Ok(())
}
```

## Platform Behavior

### Linux
- Uses PolicyKit (`pkexec`) for GUI authorization
- Creates systemd service units in `/etc/systemd/system/`
- Installs binaries to `/usr/local/bin/`
- Supports user/group creation
- Applies security hardening (PrivateTmp, ProtectSystem, etc.)

### macOS
- Uses `osascript` for GUI authorization dialog
- Creates launchd plists in `/Library/LaunchDaemons/`
- Installs binaries to `/usr/local/bin/`
- Configures logging to `/var/log/{service}/`
- Supports KeepAlive for auto-restart

### Windows
- Uses UAC elevation for administrator privileges
- Registers services with Service Control Manager
- Configures automatic startup
- Sets restart actions on failure
- Manages service dependencies

## Error Handling

The library provides specific error types for common scenarios:

```rust
match install_daemon(installer) {
    Ok(()) => { /* success */ },
    Err(InstallerError::Cancelled) => {
        // User clicked "Cancel" on authorization dialog
    },
    Err(InstallerError::PermissionDenied) => {
        // Wrong password or policy restriction
    },
    Err(InstallerError::MissingExecutable(exe)) => {
        // Required system tool not found (e.g., pkexec)
    },
    Err(InstallerError::Io(e)) => {
        // File system error
    },
    Err(InstallerError::System(msg)) => {
        // Platform-specific error
    },
    Err(InstallerError::Other(e)) => {
        // Other errors
    },
}
```

## Security Considerations

- The installer runs with elevated privileges only for system modifications
- Service binaries are installed with appropriate permissions (755)
- Configuration files are created with restricted permissions (644)
- Services run as specified user/group, not as root
- Platform-specific security features are applied (systemd hardening, etc.)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.