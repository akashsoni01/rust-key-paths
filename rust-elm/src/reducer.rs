use parking_lot::Mutex;

use crate::cmd::Cmd;
use crate::store::{catch_reduce, ReducePanic};

/// Composable update logic (TCA `Reducer` parity).
pub trait Reducer {
    type State;
    type Action;

    fn reduce(&self, state: &mut Self::State, action: Self::Action) -> Cmd<Self::Action>;
}

/// Blanket impl — existing `update` fn pointers are reducers (zero-cost hot path).
impl<S, A> Reducer for fn(&mut S, A) -> Cmd<A> {
    type State = S;
    type Action = A;

    fn reduce(&self, state: &mut S, action: A) -> Cmd<A> {
        self(state, action)
    }
}

/// Coerce a function item to an fn pointer for use in [`CombineReducers`].
#[inline]
pub const fn coerce_fn<S, A>(f: fn(&mut S, A) -> Cmd<A>) -> fn(&mut S, A) -> Cmd<A> {
    f
}

/// Named inline reducer wrapping a fn pointer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Reduce<S, A> {
    reduce: fn(&mut S, A) -> Cmd<A>,
}

impl<S, A> Reduce<S, A> {
    pub const fn new(reduce: fn(&mut S, A) -> Cmd<A>) -> Self {
        Self { reduce }
    }

    pub const fn none() -> Self
    where
        S: Default,
    {
        Self::new(|_, _| Cmd::none())
    }
}

impl<S, A> Reducer for Reduce<S, A> {
    type State = S;
    type Action = A;

    fn reduce(&self, state: &mut S, action: A) -> Cmd<A> {
        (self.reduce)(state, action)
    }
}

/// Runs multiple reducers in order and merges their commands.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CombineReducers<R>(pub R);

macro_rules! impl_combine_reducers {
    ($($R:ident),+) => {
        impl<S, A, $($R),+> Reducer for CombineReducers<($($R,)+)>
        where
            $($R: Reducer<State = S, Action = A>),+,
            A: Clone,
        {
            type State = S;
            type Action = A;

            fn reduce(&self, state: &mut S, action: A) -> Cmd<A> {
                let ($($R,)+) = &self.0;
                Cmd::batch([$( $R.reduce(state, action.clone()) ),+])
            }
        }
    };
}

impl_combine_reducers!(R1);
impl_combine_reducers!(R1, R2);
impl_combine_reducers!(R1, R2, R3);
impl_combine_reducers!(R1, R2, R3, R4);
impl_combine_reducers!(R1, R2, R3, R4, R5);

/// Wraps a reducer with panic recovery — state is rolled back on panic (TCA `catch` parity).
///
/// Keeps a committed checkpoint updated after each successful reduce; panics swap back without
/// cloning on the failure path.
pub struct CatchReducer<R, F, S> {
    inner: R,
    recover: F,
    checkpoint: Mutex<S>,
}

impl<R, F, S> CatchReducer<R, F, S>
where
    S: Clone,
{
    pub fn new(inner: R, recover: F, committed: &S) -> Self {
        Self {
            inner,
            recover,
            checkpoint: Mutex::new(committed.clone()),
        }
    }
}

impl<R, F, S> Clone for CatchReducer<R, F, S>
where
    R: Clone,
    F: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            recover: self.recover.clone(),
            checkpoint: Mutex::new(self.checkpoint.lock().clone()),
        }
    }
}

impl<R, F, S> std::fmt::Debug for CatchReducer<R, F, S>
where
    R: std::fmt::Debug,
    F: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CatchReducer")
            .field("inner", &self.inner)
            .field("recover", &self.recover)
            .finish_non_exhaustive()
    }
}

impl<R, F, S, A> Reducer for CatchReducer<R, F, S>
where
    R: Reducer<State = S, Action = A>,
    S: Clone,
    F: Fn(ReducePanic) -> Cmd<A>,
{
    type State = S;
    type Action = A;

    fn reduce(&self, state: &mut S, action: A) -> Cmd<A> {
        let mut checkpoint = self.checkpoint.lock();
        match catch_reduce(state, &mut *checkpoint, |s, a| self.inner.reduce(s, a), action) {
            Ok(cmd) => cmd,
            Err(panic) => (self.recover)(panic),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;
    use crate::panic_on_state_clone;
    use crate::test_support::allow_state_clones;

    panic_on_state_clone! {
        #[derive(Default, Debug, PartialEq, Eq)]
        struct App {
            a: i32,
            b: i32,
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Action {
        Tick,
    }

    fn reduce_a(state: &mut App, _: Action) -> Cmd<Action> {
        state.a += 1;
        Cmd::none()
    }

    fn reduce_b(state: &mut App, _: Action) -> Cmd<Action> {
        state.b += 10;
        Cmd::none()
    }

    fn effect_a(_: &mut App, _: Action) -> Cmd<Action> {
        Cmd::single(Effect::task(1, || Box::pin(async { Ok(Action::Tick) })))
    }

    #[test]
    fn fn_pointer_is_reducer() {
        let mut app = App::default();
        let cmd = coerce_fn(reduce_a).reduce(&mut app, Action::Tick);
        assert_eq!(app.a, 1);
        assert!(cmd.is_none());
    }

    #[test]
    fn reduce_wrapper_delegates() {
        let r = Reduce::new(reduce_b);
        let mut app = App::default();
        r.reduce(&mut app, Action::Tick);
        assert_eq!(app.b, 10);
    }

    #[test]
    fn combine_runs_in_order_and_merges_cmds() {
        let combined = CombineReducers((coerce_fn(reduce_a), coerce_fn(effect_a)));
        let mut app = App::default();
        let cmd = combined.reduce(&mut app, Action::Tick);
        assert_eq!(app.a, 1);
        assert_eq!(cmd.effect_count(), 1);
    }

    #[test]
    fn combine_three_reducers() {
        fn inc_a(s: &mut App, _: Action) -> Cmd<Action> {
            s.a += 1;
            Cmd::none()
        }
        fn inc_b(s: &mut App, _: Action) -> Cmd<Action> {
            s.b += 1;
            Cmd::none()
        }
        fn noop(_: &mut App, _: Action) -> Cmd<Action> {
            Cmd::none()
        }

        let combined = CombineReducers((coerce_fn(inc_a), coerce_fn(inc_b), coerce_fn(noop)));
        let mut app = App::default();
        combined.reduce(&mut app, Action::Tick);
        assert_eq!(app.a, 1);
        assert_eq!(app.b, 1);
    }

    #[test]
    fn reducers_macro_builds_combine() {
        let combined = crate::reducers![reduce_a, reduce_b];
        let mut app = App::default();
        combined.reduce(&mut app, Action::Tick);
        assert_eq!(app.a, 1);
        assert_eq!(app.b, 10);
    }

    #[test]
    fn catch_reducer_recovers_from_panic_and_unwinds_state() {
        fn panicking(s: &mut App, _: Action) -> Cmd<Action> {
            s.a = 99;
            panic!("boom");
        }

        fn recover(_: ReducePanic) -> Cmd<Action> {
            Cmd::none()
        }

        let mut app = App::default();
        let caught = allow_state_clones(1, || CatchReducer::new(coerce_fn(panicking), recover, &app));
        let cmd = caught.reduce(&mut app, Action::Tick);
        assert_eq!(app.a, 0);
        assert!(cmd.is_none());
    }
}
