//! # rust-key-paths
//!
//! Type-safe, composable keypaths for Rust with zero-cost abstractions.
//!
//! This crate re-exports `rust-keypaths` and `keypaths-proc` for convenience:
//! - `rust_keypaths` - Direct re-export (required for proc macros)
//! - `kp` - Short alias for `rust-keypaths`
//! - `kpm` - Short alias for `keypaths-proc`
//!
//! ## Usage
//!
//! ```rust
//! use rust_key_paths::kpm::Keypaths;
//! use rust_key_paths::rust_keypaths; // Required for derive macros
//!
//! #[derive(Keypaths)]
//! #[Writable]
//! struct MyStruct {
//!     field: String,
//! }
//! ```

// Direct re-export required for proc macros to resolve `rust_keypaths::`
pub use rust_keypaths;

/// Re-export of `rust-keypaths` - the core keypaths library (short alias)
pub use rust_keypaths as kp;

/// Re-export of `keypaths-proc` - the proc-macro derive library
pub use keypaths_proc as kpm;

// Re-export commonly used items at the root for convenience
pub use rust_keypaths::{
    KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath,
};

// Re-export derive macros at the root for convenience
pub use keypaths_proc::{Keypaths, Casepaths};
