// src/prelude.rs
pub use crate::CoercionTrait;
pub use crate::constrain_get;
pub use crate::constrain_set;
pub use crate::Kp;
pub use crate::Readable;
pub use crate::KpTrait;
pub use crate::KpType;
pub use crate::Writable;
#[cfg(feature = "nightly")]
pub use std::ops::Shr;
