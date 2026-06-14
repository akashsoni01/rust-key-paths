#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::cmd::Cmd;

/// Serializable state snapshot for replay harnesses.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateSnapshot<S> {
    pub state: S,
}

/// Serializable log entry for replay harnesses.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEntry<M> {
    pub action: M,
    pub cmd_count: usize,
}

/// In-memory action log for deterministic replay.
#[derive(Debug, Clone)]
pub struct ReplayLog<M> {
    entries: Vec<(M, usize)>,
}

impl<M: Clone> Default for ReplayLog<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Clone> ReplayLog<M> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, action: M, cmd: &Cmd<M>) {
        self.entries.push((action, cmd.effect_count()));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[(M, usize)] {
        &self.entries
    }

    #[cfg(feature = "serde")]
    pub fn to_replay_entries(&self) -> Vec<ReplayEntry<M>> {
        self.entries
            .iter()
            .map(|(action, count)| ReplayEntry {
                action: action.clone(),
                cmd_count: *count,
            })
            .collect()
    }
}

/// Replays recorded actions through an update function.
pub struct ReplayHarness<S, M> {
    pub state: S,
    pub update: fn(&mut S, M) -> Cmd<M>,
    pub log: ReplayLog<M>,
}

impl<S, M: Clone> ReplayHarness<S, M> {
    pub fn new(state: S, update: fn(&mut S, M) -> Cmd<M>) -> Self {
        Self {
            state,
            update,
            log: ReplayLog::new(),
        }
    }

    pub fn send(&mut self, action: M) -> Cmd<M> {
        let cmd = (self.update)(&mut self.state, action.clone());
        self.log.record(action, &cmd);
        cmd
    }

    pub fn replay(&mut self, actions: impl IntoIterator<Item = M>) {
        for action in actions {
            self.send(action);
        }
    }

    /// Capture a clone of the current state (for snapshot/restore testing).
    pub fn snapshot(&self) -> S
    where
        S: Clone,
    {
        self.state.clone()
    }

    /// Replace state wholesale — e.g. after loading from [`Shared`] storage.
    pub fn restore(&mut self, state: S) {
        self.state = state;
    }

    #[cfg(feature = "serde")]
    pub fn snapshot_record(&self) -> StateSnapshot<S>
    where
        S: Clone,
    {
        StateSnapshot {
            state: self.snapshot(),
        }
    }

    #[cfg(feature = "serde")]
    pub fn restore_record(&mut self, record: StateSnapshot<S>) {
        self.restore(record.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panic_on_state_clone;
    use crate::test_support::allow_state_clones;

    panic_on_state_clone! {
        #[derive(Default)]
        struct S {
            n: i32,
        }
    }

    fn update(s: &mut S, msg: i32) -> Cmd<i32> {
        s.n += msg;
        Cmd::none()
    }

    #[test]
    fn replay_harness_records_actions() {
        let mut harness = ReplayHarness::new(S::default(), update);
        harness.send(1);
        harness.send(2);
        assert_eq!(harness.state.n, 3);
        assert_eq!(harness.log.len(), 2);
    }

    #[test]
    fn snapshot_restore_rewinds_state() {
        let mut harness = ReplayHarness::new(S::default(), update);
        harness.send(5);
        let snap = allow_state_clones(1, || harness.snapshot());
        harness.send(7);
        assert_eq!(harness.state.n, 12);
        harness.restore(snap);
        assert_eq!(harness.state.n, 5);
    }
}
