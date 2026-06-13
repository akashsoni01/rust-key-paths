use crate::cmd::Cmd;
use crate::sub::Sub;

/// Elm `Program` — init, update, subscriptions.
pub struct Program<S, M> {
    pub init: fn() -> (S, Cmd<M>),
    pub update: fn(&mut S, M) -> Cmd<M>,
    pub subscriptions: fn(&S) -> Sub<M>,
}

impl<S, M> Program<S, M> {
    pub fn new(
        init: fn() -> (S, Cmd<M>),
        update: fn(&mut S, M) -> Cmd<M>,
        subscriptions: fn(&S) -> Sub<M>,
    ) -> Self {
        Self {
            init,
            update,
            subscriptions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;

    #[derive(Default)]
    struct Counter {
        n: i32,
    }

    fn init() -> (Counter, Cmd<i32>) {
        (Counter::default(), Cmd::none())
    }

    fn update(state: &mut Counter, msg: i32) -> Cmd<i32> {
        state.n += msg;
        Cmd::none()
    }

    fn subs(_: &Counter) -> Sub<i32> {
        Sub::none()
    }

    #[test]
    fn program_triple_wires() {
        let program = Program::new(init, update, subs);
        let (mut state, _) = (program.init)();
        let _ = (program.update)(&mut state, 1);
        assert_eq!(state.n, 1);
        assert!(matches!((program.subscriptions)(&state), Sub::None));
    }
}
