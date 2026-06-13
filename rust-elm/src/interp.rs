use std::collections::VecDeque;

use crate::effect::Effect;
use crate::env::Environment;

/// Context passed to the effect interpreter inside `runtime.rs`.
#[derive(Debug, Clone, Default)]
pub struct InterpretCtx {
    pub env: Environment,
    pub sequence: VecDeque<Effect<()>>,
}

impl InterpretCtx {
    pub fn new(env: Environment) -> Self {
        Self {
            env,
            sequence: VecDeque::new(),
        }
    }
}

pub fn flatten_effects<M>(effect: Effect<M>) -> Vec<Effect<M>> {
    match effect {
        Effect::None => vec![],
        Effect::Batch(items) => items.into_iter().flat_map(flatten_effects).collect(),
        other => vec![other],
    }
}

/// Identity helper kept for roadmap compatibility.
pub fn normalize<M>(effect: Effect<M>) -> Effect<M> {
    effect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_expands_batch() {
        let effect = Effect::<i32>::batch([
            Effect::none(),
            Effect::batch([Effect::none(), Effect::none()]),
        ]);
        assert_eq!(flatten_effects(effect).len(), 0);
    }
}
