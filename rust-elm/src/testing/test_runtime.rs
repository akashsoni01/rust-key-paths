use crate::cmd::Cmd;
use crate::effect::Effect;

/// Synchronous test runtime for asserting effect descriptions without Tokio.
pub struct TestRuntime<S, M> {
    pub state: S,
    pub update: fn(&mut S, M) -> Cmd<M>,
    pub pending_effects: Vec<Effect<M>>,
}

impl<S, M> TestRuntime<S, M>
where
    M: Clone + Send + 'static,
{
    pub fn new(state: S, update: fn(&mut S, M) -> Cmd<M>) -> Self {
        Self {
            state,
            update,
            pending_effects: Vec::new(),
        }
    }

    pub fn send(&mut self, action: M) -> Cmd<M> {
        let cmd = (self.update)(&mut self.state, action);
        self.pending_effects.extend(cmd.clone().into_effects());
        cmd
    }

    pub fn drain_effects(&mut self) -> Vec<Effect<M>> {
        std::mem::take(&mut self.pending_effects)
    }

    pub fn assert_no_pending_effects(&self) {
        assert!(
            self.pending_effects.is_empty(),
            "expected no pending effects, found {}",
            self.pending_effects.len()
        );
    }
}

/// Assert that the last command contains an effect matching a predicate.
#[macro_export]
macro_rules! assert_effect {
    ($runtime:expr, $variant:pat) => {{
        let effects = $runtime.drain_effects();
        assert!(
            effects.iter().any(|e| matches!(e, $variant)),
            "expected effect matching {}, got {:?}",
            stringify!($variant),
            effects
        );
    }};
}

pub use assert_effect;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EffectError;

    #[derive(Default)]
    struct S {
        n: i32,
    }

    fn update(s: &mut S, msg: i32) -> Cmd<i32> {
        s.n = msg;
        Cmd::single(Effect::task(1, effect_echo))
    }

    fn effect_echo() -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<i32, EffectError>> + Send>,
    > {
        Box::pin(async { Ok(7) })
    }

    #[test]
    fn test_runtime_collects_effects() {
        let mut rt = TestRuntime::new(S::default(), update);
        rt.send(5);
        assert_eq!(rt.pending_effects.len(), 1);
        rt.drain_effects();
        rt.assert_no_pending_effects();
    }
}
