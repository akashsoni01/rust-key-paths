//! # rust-key-paths
//!
//! Type-safe, composable keypaths for Rust with zero-cost abstractions.
//!
//! This crate re-exports `rust-keypaths` and `keypaths-proc` for convenience:
//! - `kp` - The core keypaths library (`rust-keypaths`)
//! - `kpm` - The proc-macro derive library (`keypaths-proc`)
//!
//! ## Usage
//!
//! ```rust
//! use rust_key_paths::{kp, kpm};
//!
//! // Use derive macros from kpm
//! #[derive(kpm::Keypaths)]
//! #[Writable]
//! struct MyStruct {
//!     field: String,
//! }
//!
//! // Use keypath types from kp
//! fn example(kp: kp::KeyPath<MyStruct, String, impl Fn(&MyStruct) -> &String>) {
//!     // ...
//! }
//! ```

/// Re-export of `rust-keypaths` - the core keypaths library
pub use rust_keypaths as kp;

/// Re-export of `keypaths-proc` - the proc-macro derive library
pub use keypaths_proc as kpm;
