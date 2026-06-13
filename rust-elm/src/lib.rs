//! # rust-elm
//!
//! A Rust port of The Elm Architecture
//!
//! - **Pure descriptions**: `Cmd`, `Effect`, and `Sub` are data — interpretation lives in [`Runtime`].
//! - **Zero-cost updates**: `update` uses fn pointers; no `Box<dyn Fn>` on hot paths.
//! - **Composition**: [`Slot`](component::Slot), [`optics`](optics), [`Reducer`](reducer::Reducer) / `CombineReducers`.

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
pub mod reducer;
pub mod replay;
pub mod runtime;
pub mod scope;
pub mod shared;
pub mod store;
pub mod sub;
pub mod subscription;
pub mod test_runtime;
pub mod test_store;
pub mod test_support;

/// Re-export keypath types for state/action focusing (see `optics`).
pub mod keypath {
    pub use key_paths_core::{KeyPath, KpTrait, Readable, Writable};
    pub use rust_key_paths::{EnumKp, EnumKpType, EnumValueKpType, Kp, KpType};
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
    enum_err, enum_ok, enum_some, enum_variant, extract, extract_action, extract_mut, variant_of,
    wrap_action, ActionCase, ActionEnum, CasePath, Casepath, EnumKp, EnumKpType, EnumValueKpType,
    Kp, KpType, StateKey, StateKp, StateLens,
};
pub use program::{Program, ReducerProgram};
pub use reducer::{coerce_fn, CatchReducer, CombineReducers, Reduce, Reducer, RollbackCatchReducer};
pub use replay::{ReplayHarness, ReplayLog};
#[cfg(feature = "serde")]
pub use replay::StateSnapshot;
pub use runtime::Runtime;
pub use scope::{
    lift_cmd, lift_cmd_with_id, ForEachReducer, IfCaseLetReducer, IfLetReducer, OptionalReducer,
    ScopeReducer,
};
pub use shared::{InMemoryStorage, Shared, SharedSubscriber, Storage, StorageError};
#[cfg(feature = "serde")]
pub use shared::FileStorage;
pub use store::{
    catch_reduce, catch_reduce_panic, ReducePanic, ScopedStore, StateSubscriber, Store, StoreTask,
    StoreTaskError,
};
pub use sub::Sub;
pub use test_runtime::TestRuntime;
pub use test_store::{ExhaustiveTestStore, TestStoreError};
pub use test_support::{
    allow_state_clones, on_state_clone, replay_snapshot, scoped_child_state, scoped_subscribe_state,
    scoped_subscriber_next, shared_get, shared_with_mut, store_state, subscribe_state,
    subscriber_wait_next,
};

pub use rust_key_paths;
pub use key_paths_core;
