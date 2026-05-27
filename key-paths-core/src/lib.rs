//! Lightweight, dependency-free keypath traits for Rust.
//!
//! Implement [`Readable`] and [`Writable`] on your keypath types so callers can navigate
//! roots uniformly. Use [`KpTrait`] for `TypeId` helpers and [`KpTrait::then`] composition.
//!
//! Higher-level crates (for example `rust-key-paths`) add concrete keypath structs,
//! chaining, and lock/async adapters on top of these traits.

#![no_std]

use core::any::TypeId;

/// Used so async chaining can infer the referent of a reference-valued step
/// (e.g. `&T` and `&mut T` both map to `T`).
pub trait KeyPathValueTarget {
    /// The type pointed to when `Self` is a reference.
    type Target: Sized;
}

impl<T: Sized> KeyPathValueTarget for &T {
    type Target = T;
}

impl<T: Sized> KeyPathValueTarget for &mut T {
    type Target = T;
}

/// Read-only keypath: navigate from `Root` to `Value`.
pub trait Readable<Root, Value> {
    /// Getter path. Returns `None` when navigation fails.
    fn get(&self, root: Root) -> Option<Value>;
}

/// Mutable keypath: setter path (same semantics as a `get_mut` closure on many keypath APIs).
pub trait Writable<MutRoot, MutValue> {
    /// Setter path. Returns `None` when navigation fails.
    fn set(&self, root: MutRoot) -> Option<MutValue>;
}

/// A keypath that supports both read and write navigation.
pub trait KeyPath<Root, Value, MutRoot, MutValue>:
    Readable<Root, Value> + Writable<MutRoot, MutValue>
{
}

impl<T, Root, Value, MutRoot, MutValue> KeyPath<Root, Value, MutRoot, MutValue> for T where
    T: Readable<Root, Value> + Writable<MutRoot, MutValue>
{
}

/// Logical root/value type identity and composition for a keypath.
pub trait KpTrait<R, V, Root, Value, MutRoot, MutValue>:
    Readable<Root, Value> + Writable<MutRoot, MutValue>
{
    /// `TypeId` of the logical root type `R`.
    fn type_id_of_root() -> TypeId
    where
        R: 'static,
    {
        TypeId::of::<R>()
    }

    /// `TypeId` of the logical value type `V`.
    fn type_id_of_value() -> TypeId
    where
        V: 'static,
    {
        TypeId::of::<V>()
    }

    /// Chain with a keypath over this segment's value (`Value` / `MutValue` are the link types).
    ///
    /// `Next` must read/write from the current value. The returned type is opaque at the trait
    /// level; concrete crates (for example `rust-key-paths` `Kp`) choose their own struct.
    fn then<SV, SubValue, MutSubValue, Next>(
        self,
        next: Next,
    ) -> impl KeyPath<Root, SubValue, MutRoot, MutSubValue>
    where
        Self: Sized,
        SubValue: core::borrow::Borrow<SV>,
        MutSubValue: core::borrow::BorrowMut<SV>,
        Next: Readable<Value, SubValue> + Writable<MutValue, MutSubValue> + Clone;
}

/// Optional-root and fallback helpers built on [`Readable`] / [`Writable`].
pub trait AccessorTrait<Root, Value, MutRoot, MutValue>:
    Readable<Root, Value> + Writable<MutRoot, MutValue>
{
    /// Like [`Readable::get`], but takes an optional root.
    fn get_optional(&self, root: Option<Root>) -> Option<Value> {
        root.and_then(|r| Readable::get(self, r))
    }

    /// Like [`Writable::set`], but takes an optional root.
    fn get_mut_optional(&self, root: Option<MutRoot>) -> Option<MutValue> {
        root.and_then(|r| Writable::set(self, r))
    }

    /// Returns the value if the keypath succeeds, otherwise calls `f`.
    fn get_or_else<F>(&self, root: Root, f: F) -> Value
    where
        F: FnOnce() -> Value,
    {
        Readable::get(self, root).unwrap_or_else(f)
    }

    /// Returns the mutable value if the keypath succeeds, otherwise calls `f`.
    fn get_mut_or_else<F>(&self, root: MutRoot, f: F) -> MutValue
    where
        F: FnOnce() -> MutValue,
    {
        Writable::set(self, root).unwrap_or_else(f)
    }
}
