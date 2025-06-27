/// Compile‑time service lifecycle state‑machine.
///
/// *   `State` is the **current** condition of a service supervisor.
/// *   `Event` is an instantaneous input (command, outcome, or external fact).
/// *   `Transition::next(state, event)` returns the **new** state plus an optional
///     `Action`, which the caller can map to real side‑effects (spawn process,
///     kill process, send notification, etc.).
///
/// The table is written entirely in a big `match` – the compiler turns that into
/// a jump table; **no allocation, no hashing, O(1)**.

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum State {
    Stopped,
    Starting,
    Running,
    Stopping,
    Restarting,
    Failed,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Event {
    /// External commands
    CmdStart,
    CmdStop,
    CmdRestart,

    /// Results emitted by the worker
    StartedOk,
    StartErr,
    ProcExit,            // unexpected process exit
    HealthOk,
    HealthBad,
    StopDone,
}

/// What the caller should *do* in response to a transition.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Action {
    SpawnProcess,
    KillProcess,
    NotifyHealthy,
    NotifyUnhealthy,
    Noop,                // nothing to do
}

/// The deterministic transition function.
pub struct Transition;

impl Transition {
    /// Decide the next `(State, Action)` pair.
    #[inline]
    pub const fn next(s: State, e: Event) -> (State, Action) {
        use Action::*;
        use Event::*;
        use State::*;

        match (s, e) {
            // ── Stopped ────────────────────────────────────────────────────────
            (Stopped, CmdStart)          => (Starting, SpawnProcess),
            (Stopped, CmdRestart)        => (Starting, SpawnProcess),
            (Stopped, _)                 => (Stopped, Noop),

            // ── Starting ───────────────────────────────────────────────────────
            (Starting, StartedOk)        => (Running, NotifyHealthy),
            (Starting, StartErr)         => (Failed,  NotifyUnhealthy),
            (Starting, CmdStop)          => (Stopping, KillProcess),
            (Starting, CmdRestart)       => (Restarting, KillProcess),
            (Starting, _)                => (Starting, Noop),

            // ── Running ────────────────────────────────────────────────────────
            (Running, HealthBad)         => (Failed,  NotifyUnhealthy),
            (Running, HealthOk)          => (Running, Noop),
            (Running, CmdStop)           => (Stopping, KillProcess),
            (Running, CmdRestart)        => (Restarting, KillProcess),
            (Running, ProcExit)          => (Failed,  NotifyUnhealthy),
            (Running, _)                 => (Running, Noop),

            // ── Stopping ───────────────────────────────────────────────────────
            (Stopping, StopDone)         => (Stopped, Noop),
            (Stopping, ProcExit)         => (Stopped, Noop),
            (Stopping, CmdStart)         => (Stopping, Noop), // ignore while stopping
            (Stopping, _)                => (Stopping, Noop),

            // ── Restarting ─────────────────────────────────────────────────────
            (Restarting, StopDone)       => (Starting, SpawnProcess),
            (Restarting, ProcExit)       => (Starting, SpawnProcess),
            (Restarting, StartedOk)      => (Running, NotifyHealthy),
            (Restarting, StartErr)       => (Failed,  NotifyUnhealthy),
            (Restarting, _)              => (Restarting, Noop),

            // ── Failed ─────────────────────────────────────────────────────────
            (Failed, CmdStart)           => (Starting, SpawnProcess),
            (Failed, CmdRestart)         => (Starting, SpawnProcess),
            (Failed, CmdStop)            => (Stopped, Noop),
            (Failed, HealthOk)           => (Running, NotifyHealthy),
            (Failed, _)                  => (Failed, Noop),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_start_stop() {
        let (s1, a1) = Transition::next(State::Stopped, Event::CmdStart);
        assert_eq!((s1, a1), (State::Starting, Action::SpawnProcess));

        let (s2, a2) = Transition::next(s1, Event::StartedOk);
        assert_eq!((s2, a2), (State::Running, Action::NotifyHealthy));

        let (s3, a3) = Transition::next(s2, Event::CmdStop);
        assert_eq!((s3, a3), (State::Stopping, Action::KillProcess));

        let (s4, a4) = Transition::next(s3, Event::StopDone);
        assert_eq!((s4, a4), (State::Stopped, Action::Noop));
    }
}