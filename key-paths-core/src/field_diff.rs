//! Per-field change detection without cloning whole state trees.

use core::hash::{Hash, Hasher};

/// FNV-1a 64-bit hasher (no `std` required).
struct FnvHasher(u64);

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 ^= *b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

/// Hash any [`Hash`] value without cloning it.
pub fn hash_value<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut h = FnvHasher(0xcbf29ce484222325);
    value.hash(&mut h);
    h.finish()
}

/// State types implement this so subscribers can detect which fields changed
/// by comparing per-field hashes — never by cloning the whole value.
pub trait FieldDiff: Hash {
    /// Stable identifier for a top-level (or nested) field path.
    type Path: Copy + Eq + Hash + Send + Sync + 'static;

    /// Push `(path, field_hash)` for each tracked field.
    fn field_hashes(&self, out: &mut alloc::vec::Vec<(Self::Path, u64)>);
}
