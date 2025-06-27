use chrono::{DateTime, Utc};

/// Commands sent *to* a worker thread.
#[derive(Debug)]
pub enum Cmd {
    Start,
    Stop,
    Restart,
    Shutdown,      // worker should exit
    TickHealth,    // periodic health probe
    TickLogRotate, // periodic rotation
}

/// Events emitted *from* workers back to the manager.
#[derive(Debug, Clone)]
pub enum Evt {
    State {
        service: &'static str,
        kind: &'static str, // "running"|"stopped"|etc.
        ts: DateTime<Utc>,
        pid: Option<u32>,
    },
    Health {
        service: &'static str,
        healthy: bool,
        ts: DateTime<Utc>,
    },
    LogRotate {
        service: &'static str,
        ts: DateTime<Utc>,
    },
    Fatal {
        service: &'static str,
        msg: &'static str,
        ts: DateTime<Utc>,
    },
}
