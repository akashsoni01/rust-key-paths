//! Tokio-backed live runtime: bus, store, effect interpreter, subscriptions.
//!
//! Store flavors:
//! - [`store::Store`] — [`Mutex`](parking_lot::Mutex) (default [`Runtime`])
//! - [`rw_store::RwStore`] — [`RwLock`](parking_lot::RwLock) ([`RwRuntime`])
//! - [`swap_store::SwapStore`] — lock-free snapshot reads (`arc-swap` feature, [`SwapRuntime`])

mod config;
pub(crate) mod dispatch;
mod engine;
mod rw_engine;
#[cfg(feature = "arc-swap")]
mod swap_engine;
pub mod binding;
pub(crate) mod interpreter;
pub(crate) mod state_access;
pub mod store;
pub mod rw_store;
#[cfg(feature = "arc-swap")]
pub mod swap_store;
pub mod subscription;

pub use config::RuntimeConfig;
pub use engine::Runtime;
pub use rw_engine::RwRuntime;
#[cfg(feature = "arc-swap")]
pub use swap_engine::SwapRuntime;
