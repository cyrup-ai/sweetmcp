use anyhow::{Context, Result};
use log::info;
use nix::sys::stat::{umask, Mode};
use nix::unistd::{chdir, close, dup2, fork, setsid, ForkResult};
use std::fs;
use std::os::unix::io::RawFd;
use std::path::Path;

#[cfg(feature = "systemd-notify")]
use systemd::daemon;

/// Detect whether we are launched *by* systemd.  If so, we should **not**
/// daemonise; systemd is already the babysitter.
fn running_under_systemd() -> bool {
    std::env::var_os("INVOCATION_ID").is_some()
}

/// Tell systemd the daemon is ready (no‑op when feature is off).
pub fn systemd_ready() {
    #[cfg(feature = "systemd-notify")]
    {
        if let Err(e) = daemon::notify(false, &[daemon::NotifyState::Ready]) {
            warn!("sd_notify failed: {e}");
        }
    }
}

/// Detect whether we should stay in foreground (systemd or macOS)
pub fn need_foreground() -> bool {
    running_under_systemd() || cfg!(target_os = "macos")
}

/// Perform the traditional Unix "double‑fork" daemonisation in *one small
/// allocation‑free function*.
///
/// Steps:
/// 1. `fork`; parent exits.
/// 2. Child calls `setsid` to drop the controlling TTY.
/// 3. `fork` again so we are **not** a session leader (protects from reacquiring a TTY).
/// 4. `chdir /`, reset umask.
/// 5. Close every FD ≥ 3.
/// 6. Re‑open `/dev/null` on stdin/stdout/stderr.
pub fn daemonise(pid_file: &Path) -> Result<()> {
    if running_under_systemd() {
        info!("systemd detected – skipping classic daemonise");
        return Ok(());
    }

    match unsafe { fork().context("first fork")? } {
        ForkResult::Parent { .. } => std::process::exit(0),
        ForkResult::Child => {}
    }

    setsid().context("setsid")?;

    match unsafe { fork().context("second fork")? } {
        ForkResult::Parent { .. } => std::process::exit(0),
        ForkResult::Child => {}
    }

    chdir("/").context("chdir")?;
    umask(Mode::from_bits_truncate(0o022));

    // Close everything except stdin/out/err.
    // `/proc/self/fd` is cheapest on Linux; fall back to a brute range.
    let max_fd = if let Ok(entries) = fs::read_dir("/proc/self/fd") {
        entries.count() as RawFd + 8
    } else {
        256
    };
    for fd in 3..max_fd {
        let _ = close(fd);
    }

    // stdin, stdout, stderr → /dev/null
    use std::os::unix::io::{FromRawFd, OwnedFd};
    let devnull = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/null")
        .context("open /dev/null")?;

    // Redirect stdin, stdout, stderr to /dev/null
    for target in 0..=2 {
        // Create an OwnedFd from the target fd
        let mut target_fd = unsafe { OwnedFd::from_raw_fd(target) };
        // Duplicate devnull to the target fd
        dup2(&devnull, &mut target_fd).ok();
        // Forget the OwnedFd to prevent it from closing the standard descriptors
        std::mem::forget(target_fd);
    }

    // Write PID file *after* we are fully detached.
    fs::write(pid_file, std::process::id().to_string()).context("pidfile")?;

    Ok(())
}
