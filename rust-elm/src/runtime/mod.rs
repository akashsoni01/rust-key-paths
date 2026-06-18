//! Tokio-backed live runtime: bus, store, effect interpreter, subscriptions.
//!
//! Store flavors:
//! - [`store::Store`] — [`Mutex`](parking_lot::Mutex) (default [`Runtime`])
//! - [`rw_store::RwStore`] — [`RwLock`](parking_lot::RwLock) ([`RwRuntime`])
//! - [`tea_store::TeaStore`] — channel-pushed model snapshots ([`TeaRuntime`], true TEA)
//! - [`swap_store::SwapStore`] — lock-free snapshot reads (`arc-swap` feature, [`SwapRuntime`])

mod config;
pub mod error;
pub(crate) mod dispatch;
mod engine;
mod lifecycle;
mod rw_engine;
mod tea_engine;
#[cfg(feature = "arc-swap")]
mod swap_engine;
pub mod binding;
pub(crate) mod interpreter;
pub(crate) mod state_access;
pub mod store;
pub mod rw_store;
pub mod tea_store;
#[cfg(feature = "arc-swap")]
pub mod swap_store;
pub mod subscription;

pub use config::RuntimeConfig;
pub use error::RuntimeError;
pub use engine::Runtime;
pub use rw_engine::RwRuntime;
pub use tea_engine::TeaRuntime;
#[cfg(feature = "arc-swap")]
pub use swap_engine::SwapRuntime;
