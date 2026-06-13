//! # rust-elm
//!
//! A Rust port of The Elm Architecture evolving toward
//! [TCA](https://github.com/pointfreeco/swift-composable-architecture) parity.
//!
//! - **Pure descriptions**: `Cmd`, `Effect`, and `Sub` are data — interpretation lives in [`Runtime`].
//! - **Zero-cost updates**: `update` uses fn pointers; no `Box<dyn Fn>` on hot paths.
//! - **Composition**: [`Slot`](component::Slot), [`optics`](optics) (via `rust-key-paths`), and (soon) `Reducer` / `Scope`.

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
pub mod replay;
pub mod runtime;
pub mod sub;
pub mod test_runtime;

/// Re-export keypath types for state/action focusing (see `optics`).
pub mod keypath {
    pub use key_paths_core::{KeyPath, KpTrait, Readable, Writable};
    pub use rust_key_paths::{Kp, KpType};
}

pub use batch::batch;
pub use bus::{Bus, BusSender};
pub use cmd::Cmd;
pub use component::{lift, Slot};
pub use effect::{Effect, EffectId, EnvTaskFn, TaskFn};
pub use env::{defer_batch, Environment, FakeClock, MockHttp};
pub use error::EffectError;
pub use interp::{flatten_effects, normalize, InterpretCtx};
pub use optics::{extract, extract_mut, wrap_action, ActionCase, KpType, StateKey};
pub use program::Program;
pub use replay::{ReplayHarness, ReplayLog};
pub use runtime::Runtime;
pub use sub::Sub;
pub use test_runtime::TestRuntime;

pub use rust_key_paths;
pub use key_paths_core;
