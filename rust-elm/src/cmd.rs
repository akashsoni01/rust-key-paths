use crate::effect::Effect;

/// Commands returned from `update` — zero or more effects to run.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Cmd<M> {
    #[default]
    None,
    Single(Effect<M>),
    Batch(Vec<Cmd<M>>),
}

impl<M> Cmd<M> {
    pub fn none() -> Self {
        Self::None
    }

    pub fn single(effect: Effect<M>) -> Self {
        Self::Single(effect)
    }

    pub fn batch(cmds: impl IntoIterator<Item = Cmd<M>>) -> Self {
        let cmds: Vec<_> = cmds.into_iter().collect();
        if cmds.is_empty() {
            Self::None
        } else if cmds.len() == 1 {
            cmds.into_iter().next().unwrap()
        } else {
            Self::Batch(cmds)
        }
    }

    pub fn map<N>(self, f: fn(M) -> N) -> Cmd<N>
    where
        M: Send + 'static,
        N: Send + 'static,
    {
        match self {
            Self::None => Cmd::None,
            Self::Single(e) => {
                let mapped = e.map(f);
                if matches!(mapped, Effect::None) {
                    Cmd::None
                } else {
                    Cmd::Single(mapped)
                }
            }
            Self::Batch(cmds) => Cmd::Batch(cmds.into_iter().map(|c| c.map(f)).collect()),
        }
    }

    pub fn into_effects(self) -> Vec<Effect<M>> {
        match self {
            Self::None => vec![],
            Self::Single(e) => vec![e],
            Self::Batch(cmds) => cmds.into_iter().flat_map(Cmd::into_effects).collect(),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn effect_count(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Single(_) => 1,
            Self::Batch(cmds) => cmds.iter().map(|c| c.effect_count()).sum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;

    #[test]
    fn batch_flattens_and_map_preserves_structure() {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum Msg {
            A,
            B,
        }

        let cmd = Cmd::batch([
            Cmd::single(Effect::<Msg>::none()),
            Cmd::none(),
            Cmd::single(Effect::<Msg>::none()),
        ]);
        assert_eq!(cmd.into_effects().len(), 2);

        let mapped = Cmd::<Msg>::single(Effect::<Msg>::none()).map(|m| match m {
            Msg::A => 1,
            Msg::B => 2,
        });
        assert!(mapped.is_none());
    }
}
