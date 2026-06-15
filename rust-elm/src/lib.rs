//! # rust-elm
//!
//! A Rust port of The Elm Architecture
//!
//! - **Pure descriptions**: `Cmd`, `Effect`, and `Sub` are data — interpretation lives in [`Runtime`] (requires `runtime` feature).
//! - **Zero-cost updates**: `update` uses fn pointers; no `Box<dyn Fn>` on hot paths.
//! - **Composition**: [`Slot`](component::Slot), [`optics`](optics), [`Reducer`](reducer::Reducer) / `CombineReducers`.

#![allow(
    non_snake_case,
    unpredictable_function_pointer_comparisons,
)]

pub mod batch;
pub mod bus;
pub mod cmd;
pub mod component;
pub mod effect;
pub mod env;
pub mod error;
pub mod interp;
pub mod macros;
pub mod optics;
pub mod program;
pub mod reduce_panic;
pub mod reducer;
pub mod replay;
pub mod scope;
pub mod shared;
pub mod sub;
pub mod test_runtime;
#[cfg(feature = "runtime")]
pub mod test_store;
pub mod test_support;

#[cfg(feature = "runtime")]
pub mod runtime;
#[cfg(feature = "runtime")]
pub mod store;
#[cfg(feature = "runtime")]
pub mod subscription;

/// Re-export keypath types for state/action focusing (see `optics`).
pub mod keypath {
    pub use key_paths_core::{KeyPath, KpTrait, Readable, RefKpTrait, Writable};
    pub use rust_key_paths::{EnumKp, EnumKpType, EnumValueKpType, Kp};
}

pub use batch::batch;
pub use bus::{Bus, BusSender};
pub use cmd::Cmd;
pub use component::{lift, Slot};
pub use effect::{Effect, EffectId, EnvTaskFn, RunSender, TaskFn};
pub use dependencies::{
    Clock, ClockDep, ClockKey, DepRng, DependencyError, DependencyKey, DependencyValues,
    LiveRng, LiveUuidGen, Now, NowDep, NowKey, RealClock, RngDep, RngKey, SeededRng,
    SeededUuidGen, TestClock, TestNow, UuidDep, UuidGen, UuidKey,
};
pub use rust_dependencies as dependencies;
pub use env::{defer_batch, Environment, FakeClock, MockHttp};
pub use error::EffectError;
pub use interp::{flatten_effects, normalize, InterpretCtx};
pub use rust_identified_vec::{Identifiable, IdentifiedVec};
pub use optics::{
    enum_err, enum_ok, enum_some, enum_variant, variant_of, CasePath, Casepath, EnumKp,
    EnumKpType, EnumValueKpType, Kp,
};
pub use program::{Program, ReducerProgram};
pub use reduce_panic::{catch_reduce, catch_reduce_panic, ReducePanic};
pub use reducer::{coerce_fn, CatchReducer, CombineReducers, Reduce, Reducer, RollbackCatchReducer};
pub use replay::{ReplayHarness, ReplayLog};
#[cfg(feature = "serde")]
pub use replay::StateSnapshot;
#[cfg(feature = "runtime")]
pub use runtime::{Runtime, RuntimeConfig, RuntimeHost};
pub use scope::{
    lift_cmd, lift_cmd_with_id, ForEachReducer, IfCaseLetReducer, IfLetReducer, OptionalReducer,
    ScopeReducer,
};
pub use shared::{InMemoryStorage, Shared, SharedSubscriber, Storage, StorageError};
#[cfg(feature = "serde")]
pub use shared::FileStorage;
#[cfg(feature = "runtime")]
pub use store::{
    ScopedStore, StateSubscriber, Store, StoreTask, StoreTaskError,
};
pub use sub::Sub;
pub use test_runtime::TestRuntime;
#[cfg(feature = "runtime")]
pub use test_store::{ExhaustiveTestStore, TestStoreError};
pub use test_support::{
    allow_state_clones, on_state_clone, replay_snapshot, shared_get, shared_with_mut,
};
#[cfg(feature = "runtime")]
pub use test_support::{
    scoped_child_state, scoped_subscribe_state, scoped_subscriber_next, store_state,
    subscribe_state, subscriber_wait_next,
};

pub use rust_key_paths;
pub use key_paths_core;
