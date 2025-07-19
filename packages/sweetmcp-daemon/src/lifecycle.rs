use crate::state_machine::{Action, Event, State, Transition};

/// Thin, inlineable helper that owns the state enum and
/// returns the side‑effect requested by the transition table.
#[derive(Copy, Clone)]
pub struct Lifecycle {
    state: State,
}
impl Default for Lifecycle {
    fn default() -> Self {
        Self {
            state: State::Stopped,
        }
    }
}
impl Lifecycle {
    /// Feed an `Event`, get back an `Action`.
    #[inline(always)]
    pub fn step(&mut self, e: Event) -> Action {
        let (next, act) = Transition::next(self.state, e);
        self.state = next;
        act
    }

    #[inline(always)]
    pub fn is_running(&self) -> bool {
        self.state == State::Running
    }
}
