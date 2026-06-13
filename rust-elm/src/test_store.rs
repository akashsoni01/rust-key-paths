use std::collections::VecDeque;
use std::fmt;
use std::time::{Duration, Instant};

use crate::cmd::Cmd;
use crate::effect::{run_leaf, Effect};
use crate::env::Environment;

/// Exhaustive test store — every effect-produced action must be explicitly consumed (TCA parity).
pub struct ExhaustiveTestStore<S, M> {
    pub state: S,
    update: fn(&mut S, M) -> Cmd<M>,
    env: Environment,
    pending_actions: VecDeque<M>,
    pending_effects: Vec<Effect<M>>,
    exhaustive: bool,
}

impl<S, M> ExhaustiveTestStore<S, M>
where
    S: fmt::Debug + PartialEq + Clone,
    M: PartialEq + fmt::Debug + Send + 'static,
{
    pub fn new(state: S, update: fn(&mut S, M) -> Cmd<M>) -> Self {
        Self {
            state,
            update,
            env: Environment::test(),
            pending_actions: VecDeque::new(),
            pending_effects: Vec::new(),
            exhaustive: true,
        }
    }

    pub fn with_exhaustivity(mut self, on: bool) -> Self {
        self.exhaustive = on;
        self
    }

    pub fn send(&mut self, action: M) {
        self.send_expect(action, None);
    }

    pub fn send_with(&mut self, action: M, expect: impl FnOnce(&mut S)) {
        let mut expected = self.state.clone();
        expect(&mut expected);
        self.send_expect(action, Some(expected));
    }

    fn send_expect(&mut self, action: M, expected: Option<S>) {
        if self.exhaustive {
            self.assert_idle();
        }
        let cmd = (self.update)(&mut self.state, action);
        if let Some(expected) = expected {
            assert_state_eq(&expected, &self.state);
        }
        self.enqueue_effects(cmd.into_effects());
    }

    pub fn receive(&mut self, expected: M) {
        let actual = self
            .pending_actions
            .pop_front()
            .unwrap_or_else(|| panic!("expected action {expected:?}, but effect queue was empty"));
        assert_eq!(actual, expected, "unexpected effect-produced action");
        let cmd = (self.update)(&mut self.state, actual);
        self.enqueue_effects(cmd.into_effects());
    }

    pub fn finish(&self) {
        if self.exhaustive {
            self.assert_idle();
        }
    }

    fn assert_idle(&self) {
        assert!(
            self.pending_actions.is_empty(),
            "unconsumed effect actions: {:?}",
            self.pending_actions
        );
        assert!(
            self.pending_effects.is_empty(),
            "uninterpreted effects: {:?}",
            self.pending_effects
        );
    }

    fn enqueue_effects(&mut self, effects: Vec<Effect<M>>) {
        for effect in effects {
            self.pending_effects.push(effect);
        }
        while let Some(effect) = self.pending_effects.pop() {
            self.run_effect_leaf(effect);
        }
    }

    fn run_effect_leaf(&mut self, effect: Effect<M>) {
        match effect {
            Effect::None => {}
            Effect::Batch(items) | Effect::Race(items) => {
                for item in items {
                    self.run_effect_leaf(item);
                }
            }
            Effect::Sequence(items) => {
                for item in items {
                    self.run_effect_leaf(item);
                }
            }
            leaf => {
                let fut = run_leaf(leaf, &self.env);
                let msg = tokio_test_block_on(fut);
                match msg {
                    Ok(action) => self.pending_actions.push_back(action),
                    Err(err) => panic!("effect failed in test store: {err}"),
                }
            }
        }
    }

    pub fn receive_timeout(&mut self, expected: M, timeout: Duration) -> Result<(), TestStoreError> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if !self.pending_actions.is_empty() {
                self.receive(expected);
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Err(TestStoreError::ReceiveTimeout)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStoreError {
    ReceiveTimeout,
}

fn assert_state_eq<S: fmt::Debug + PartialEq>(expected: &S, actual: &S) {
    assert_eq!(
        expected, actual,
        "state mismatch after send\n  expected: {expected:?}\n  actual:   {actual:?}"
    );
}

fn tokio_test_block_on<M: Send + 'static>(
    fut: std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<M, crate::EffectError>> + Send>,
    >,
) -> Result<M, crate::EffectError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test tokio runtime")
        .block_on(fut)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    struct Counter {
        n: i32,
    }

    fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
        match msg {
            0 => Cmd::single(Effect::task(1, || Box::pin(async { Ok(10) }))),
            n => {
                s.n = n;
                Cmd::none()
            }
        }
    }

    #[test]
    fn exhaustive_requires_effect_actions_be_received() {
        let mut store = ExhaustiveTestStore::new(Counter::default(), update);
        store.send(0);
        assert_eq!(store.pending_actions.len(), 1);
        store.receive(10);
        store.finish();
        assert_eq!(store.state.n, 10);
    }

    #[test]
    fn send_with_validates_expected_state() {
        let mut store = ExhaustiveTestStore::new(Counter::default(), update);
        store.send_with(5, |s| {
            s.n = 5;
        });
        store.finish();
    }

    #[test]
    #[should_panic(expected = "unconsumed effect actions")]
    fn finish_fails_when_actions_remain() {
        let mut store = ExhaustiveTestStore::new(Counter::default(), update);
        store.send(0);
        store.finish();
    }

    #[test]
    fn non_exhaustive_allows_leftover_actions() {
        let mut store = ExhaustiveTestStore::new(Counter::default(), update).with_exhaustivity(false);
        store.send(0);
        store.finish();
    }
}
