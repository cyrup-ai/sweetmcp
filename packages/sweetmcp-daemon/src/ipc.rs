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
        service: String,
        kind: &'static str, // "running"|"stopped"|etc.
        ts: DateTime<Utc>,
        pid: Option<u32>,
    },
    Health {
        service: String,
        healthy: bool,
        ts: DateTime<Utc>,
    },
    LogRotate {
        service: String,
        ts: DateTime<Utc>,
    },
    Fatal {
        service: String,
        msg: &'static str,
        ts: DateTime<Utc>,
    },
}
