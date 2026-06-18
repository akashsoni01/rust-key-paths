//! Test helpers for asserting store/reducer paths do not clone feature state.
//!
//! Wrap reducer state types with [`panic_on_state_clone!`] and allow only explicit
//! clones via [`allow_state_clones`].

use std::cell::Cell;

thread_local! {
    static CLONE_BUDGET: Cell<usize> = const { Cell::new(0) };
}

/// Panics unless the current thread has remaining clone budget from [`allow_state_clones`].
pub fn on_state_clone(type_name: &str) {
    CLONE_BUDGET.with(|budget| {
        let remaining = budget.get();
        assert!(
            remaining > 0,
            "unexpected clone of {type_name} (clone budget exhausted)"
        );
        budget.set(remaining - 1);
    });
}

/// Temporarily allow `count` feature-state clones on this thread.
pub fn allow_state_clones<T>(count: usize, f: impl FnOnce() -> T) -> T {
    CLONE_BUDGET.with(|budget| {
        let prev = budget.get();
        budget.set(prev + count);
        let out = f();
        budget.set(prev);
        out
    })
}

#[cfg(feature = "runtime")]
mod store_helpers {
    use super::allow_state_clones;
    use crate::optics::Casepath;
    use crate::store::{ScopedStateSubscriber, StateSubscriber};

    /// [`Store::state`] clones feature state once — use in tests that intentionally read a snapshot.
    pub fn store_state<S, M>(store: &crate::Store<S, M>) -> S
    where
        S: Send + Sync + Clone + 'static,
        M: Send + 'static,
    {
        allow_state_clones(1, || store.state())
    }

    /// [`Store::subscribe_state`] clones once when the subscriber is created.
    pub fn subscribe_state<S, M>(store: &crate::Store<S, M>) -> crate::StateSubscriber<S>
    where
        S: Clone + PartialEq + Send + Sync + 'static,
        M: Send + 'static,
    {
        allow_state_clones(1, || store.subscribe_state())
    }

    /// [`ScopedStore::child_state`] clones parent state once via [`Store::state`].
    pub fn scoped_child_state<S, M, CS, CM, AK, SK>(
        scoped: &crate::ScopedStore<S, M, CS, CM, AK, SK>,
    ) -> Option<CS>
    where
        S: Send + Sync + Clone,
        M: Send,
        CS: Clone + PartialEq + Send + Sync,
        CM: Clone + Send,
        AK: Casepath<M, CM> + Clone + Send + Sync + 'static,
        SK: crate::keypath::RefKpTrait<S, CS> + Clone,
    {
        allow_state_clones(1, || scoped.child_state())
    }

    /// [`ScopedStore::subscribe_state`] clones parent state once when the subscriber is created.
    pub fn scoped_subscribe_state<S, M, CS, CM, AK, SK>(
        scoped: &crate::ScopedStore<S, M, CS, CM, AK, SK>,
    ) -> ScopedStateSubscriber<S, CS, SK>
    where
        S: Send + Sync + Clone + PartialEq,
        M: Send,
        CS: Clone + PartialEq + Send + Sync,
        CM: Clone + Send,
        AK: Casepath<M, CM> + Clone + Send + Sync + 'static,
        SK: crate::keypath::RefKpTrait<S, CS> + Clone,
    {
        allow_state_clones(1, || scoped.subscribe_state())
    }

    /// One [`StateSubscriber::next`] / [`StateSubscriber::wait_next`] may clone feature state once.
    pub fn subscriber_wait_next<S>(
        sub: &mut StateSubscriber<S>,
        timeout: std::time::Duration,
    ) -> Option<std::sync::Arc<S>>
    where
        S: PartialEq + Clone,
    {
        allow_state_clones(1, || sub.wait_next(timeout))
    }

    /// One [`ScopedStateSubscriber::next`] may clone parent feature state once.
    pub fn scoped_subscriber_next<S, CS, SK>(
        sub: &mut ScopedStateSubscriber<S, CS, SK>,
    ) -> Option<CS>
    where
        S: PartialEq + Clone,
        CS: Clone + PartialEq,
        SK: crate::keypath::RefKpTrait<S, CS> + Clone,
    {
        allow_state_clones(1, || sub.next())
    }
}

#[cfg(feature = "runtime")]
pub use store_helpers::{
    scoped_child_state, scoped_subscribe_state, scoped_subscriber_next, store_state,
    subscribe_state, subscriber_wait_next,
};

/// [`ReplayHarness::snapshot`] clones feature state once.
pub fn replay_snapshot<S, M>(harness: &crate::ReplayHarness<S, M>) -> S
where
    S: Clone,
    M: Clone,
{
    allow_state_clones(1, || harness.snapshot())
}

/// [`Shared::with_mut`] clones feature state once to detect changes.
pub fn shared_with_mut<T, R>(shared: &crate::Shared<T>, f: impl FnOnce(&mut T) -> R) -> R
where
    T: Clone + PartialEq,
{
    allow_state_clones(1, || shared.with_mut(f))
}

/// [`Shared::get`] clones feature state once.
pub fn shared_get<S>(shared: &crate::Shared<S>) -> S
where
    S: Clone,
{
    allow_state_clones(1, || shared.get())
}

/// Define a reducer/store state struct whose [`Clone`] panics unless budget is available.
#[macro_export]
macro_rules! panic_on_state_clone {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident: $ty:ty),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $($field: $ty),*
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                $crate::test_support::on_state_clone(concat!(module_path!(), "::", stringify!($name)));
                Self {
                    $($field: self.$field.clone()),*
                }
            }
        }
    };
}

/// Start a [`Runtime`](crate::Runtime) in tests/examples; panics only if bootstrap fails.
#[cfg(feature = "runtime")]
pub fn start_runtime<S, M>(
    program: crate::Program<S, M>,
    env: crate::Environment,
    config: crate::RuntimeConfig,
) -> crate::Runtime<S, M>
where
    S: Send + Sync + 'static,
    M: Send + 'static,
{
    match crate::Runtime::from_program(program, env, config) {
        Ok(runtime) => runtime,
        Err(err) => panic!("runtime bootstrap failed: {err}"),
    }
}

/// Start a [`Runtime`](crate::Runtime) from a [`ReducerProgram`](crate::ReducerProgram).
#[cfg(feature = "runtime")]
pub fn start_reducer_runtime<R>(program: crate::ReducerProgram<R>, env: crate::Environment, config: crate::RuntimeConfig) -> crate::Runtime<R::State, R::Action>
where
    R: crate::Reducer + Send + Sync + 'static,
    R::State: Send + Sync + Clone + 'static,
    R::Action: Send + 'static,
{
    match crate::Runtime::from_reducer_program(program, env, config) {
        Ok(runtime) => runtime,
        Err(err) => panic!("runtime bootstrap failed: {err}"),
    }
}

#[cfg(feature = "runtime")]
pub fn start_rw_reducer_runtime<R>(program: crate::ReducerProgram<R>, env: crate::Environment, config: crate::RuntimeConfig) -> crate::RwRuntime<R::State, R::Action>
where
    R: crate::Reducer + Send + Sync + 'static,
    R::State: Send + Sync + 'static,
    R::Action: Send + 'static,
{
    match crate::RwRuntime::from_reducer_program(program, env, config) {
        Ok(runtime) => runtime,
        Err(err) => panic!("runtime bootstrap failed: {err}"),
    }
}

#[cfg(all(feature = "runtime", feature = "arc-swap"))]
pub fn start_swap_reducer_runtime<R>(program: crate::ReducerProgram<R>, env: crate::Environment, config: crate::RuntimeConfig) -> crate::SwapRuntime<R::State, R::Action>
where
    R: crate::Reducer + Send + Sync + 'static,
    R::State: Send + Sync + Clone + 'static,
    R::Action: Send + 'static,
{
    match crate::SwapRuntime::from_reducer_program(program, env, config) {
        Ok(runtime) => runtime,
        Err(err) => panic!("runtime bootstrap failed: {err}"),
    }
}

#[cfg(feature = "runtime")]
pub fn start_tea_reducer_runtime<R>(program: crate::ReducerProgram<R>, env: crate::Environment, config: crate::RuntimeConfig) -> crate::TeaRuntime<R::State, R::Action>
where
    R: crate::Reducer + Send + Sync + 'static,
    R::State: Send + Sync + Clone + 'static,
    R::Action: Send + 'static,
{
    match crate::TeaRuntime::from_reducer_program(program, env, config) {
        Ok(runtime) => runtime,
        Err(err) => panic!("runtime bootstrap failed: {err}"),
    }
}
