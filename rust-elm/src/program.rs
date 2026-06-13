use crate::cmd::Cmd;
use crate::reducer::Reducer;
use crate::sub::Sub;

/// Elm `Program` — init, update, subscriptions (fn-pointer path).
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

/// Program backed by any [`Reducer`] value.
pub struct ReducerProgram<R: Reducer> {
    pub reducer: R,
    pub init: fn() -> (R::State, Cmd<R::Action>),
    pub subscriptions: fn(&R::State) -> Sub<R::Action>,
}

impl<R: Reducer> ReducerProgram<R> {
    pub fn new(
        reducer: R,
        init: fn() -> (R::State, Cmd<R::Action>),
        subscriptions: fn(&R::State) -> Sub<R::Action>,
    ) -> Self {
        Self {
            reducer,
            init,
            subscriptions,
        }
    }

    pub fn from_parts(
        reducer: R,
        init: fn() -> (R::State, Cmd<R::Action>),
        subscriptions: fn(&R::State) -> Sub<R::Action>,
    ) -> Self {
        Self::new(reducer, init, subscriptions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reducer::{coerce_fn, CombineReducers, Reduce};

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

    #[test]
    fn reducer_program_delegates_to_reducer() {
        fn bump(state: &mut Counter, msg: i32) -> Cmd<i32> {
            state.n += msg * 2;
            Cmd::none()
        }
        let program = ReducerProgram::new(Reduce::new(bump), init, subs);
        let (mut state, _) = (program.init)();
        let _ = program.reducer.reduce(&mut state, 3);
        assert_eq!(state.n, 6);
    }

    #[test]
    fn reducer_program_accepts_combine() {
        fn add_one(s: &mut Counter, m: i32) -> Cmd<i32> {
            s.n += m;
            Cmd::none()
        }
        fn add_ten(s: &mut Counter, m: i32) -> Cmd<i32> {
            s.n += m * 10;
            Cmd::none()
        }
        let program = ReducerProgram::new(
            CombineReducers((coerce_fn(add_one), coerce_fn(add_ten))),
            init,
            subs,
        );
        let (mut state, _) = (program.init)();
        program.reducer.reduce(&mut state, 2);
        assert_eq!(state.n, 22);
    }
}
