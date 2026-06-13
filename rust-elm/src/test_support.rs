//! Test helpers for asserting store/reducer paths do not clone feature state.
//!
//! Wrap reducer state types with [`panic_on_state_clone!`] and allow only explicit
//! clones via [`allow_state_clones`].

use std::cell::Cell;

use crate::optics::Casepath;
use crate::store::{ScopedStateSubscriber, StateSubscriber};

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

/// [`Store::state`] clones feature state once — use in tests that intentionally read a snapshot.
pub fn store_state<S, M>(store: &crate::Store<S, M>) -> S
where
    S: Send + Sync + Clone + 'static,
    M: Send + 'static,
{
    allow_state_clones(1, || store.state())
}

/// [`Store::subscribe_state`] clones once when the subscriber is created.
pub fn subscribe_state<S, M>(
    store: &crate::Store<S, M>,
) -> crate::StateSubscriber<S>
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
    SK: crate::StateLens<S, CS> + Clone,
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
    SK: crate::StateLens<S, CS> + Clone,
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
    SK: crate::StateLens<S, CS> + Clone,
{
    allow_state_clones(1, || sub.next())
}

/// [`ReplayHarness::snapshot`] clones feature state once.
pub fn replay_snapshot<S, M>(harness: &crate::ReplayHarness<S, M>) -> S
where
    S: Clone,
    M: Clone,
{
    allow_state_clones(1, || harness.snapshot())
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

pub use panic_on_state_clone;
