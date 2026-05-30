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

/// Navigate by **how the root is passed** — like `let x = y` / `&y` / `&mut y`.
///
/// | Call | `root` type | Path | Typical return |
/// |------|-------------|------|----------------|
/// | `kp.get(root)` | link `Root` (e.g. `Arc<R>`) | read | `Option<Value>` |
/// | `kp.get(&y)` | `&R` | read ([`Readable`]) | `Option<&V>` |
/// | `kp.get(&mut y)` | `&mut R` | write ([`Writable`]) | `Option<&mut V>` |
///
/// Reference roots dispatch through [`NavigateVia`]; link roots (e.g. `Arc<R>`) need a
/// [`NavigateVia`] impl in your concrete crate. For shared link roots, use [`Writable::set`]
/// / `get_mut` when `get(&mut root)` does not apply.
pub trait KeyPathGet<Root> {
    /// Navigated value (usually `Option<…>`).
    type Value;

    /// Navigate from `root`. The `Root` type parameter is inferred from `root`.
    fn get(self, root: Root) -> Self::Value;
}

/// Root-side dispatch: the type of `root` at the call site picks read vs write.
pub trait NavigateVia<KP> {
    /// Result of navigating `self` through `kp`.
    type Output;

    /// Navigate through `kp`.
    fn navigate_via(self, kp: &KP) -> Self::Output;
}

impl<'a, T, KP, V> NavigateVia<KP> for &'a T
where
    KP: Readable<&'a T, V>,
{
    type Output = Option<V>;

    #[inline]
    fn navigate_via(self, kp: &KP) -> Option<V> {
        Readable::get(kp, self)
    }
}

impl<'a, T, KP, MV> NavigateVia<KP> for &'a mut T
where
    KP: Writable<&'a mut T, MV>,
{
    type Output = Option<MV>;

    #[inline]
    fn navigate_via(self, kp: &KP) -> Option<MV> {
        Writable::set(kp, self)
    }
}

impl<KP, Root> KeyPathGet<Root> for &KP
where
    Root: NavigateVia<KP>,
{
    type Value = <Root as NavigateVia<KP>>::Output;

    #[inline]
    fn get(self, root: Root) -> Self::Value {
        root.navigate_via(self)
    }
}

mod private {
    /// Root passed as `&mut T` — used by [`writable_get`].
    pub trait MutRootRef {}
    impl<'a, T: ?Sized> MutRootRef for &'a mut T {}
}

pub use private::MutRootRef;

/// Method-syntax helper: `(&kp).get(root)` dispatches through [`KeyPathGet`].
///
/// Concrete keypath types (e.g. `rust-key-paths` [`Kp`]) should expose a generic inherent
/// `get` that forwards here so `kp.get(&mut root)` does not coerce to `&root`.
pub trait KeyPathAccess: Sized {
    /// Dispatch `get` by the type of `root` (owned / shared / exclusive).
    fn get<CallRoot>(self, root: CallRoot) -> <Self as KeyPathGet<CallRoot>>::Value
    where
        Self: KeyPathGet<CallRoot>,
    {
        KeyPathGet::get(self, root)
    }
}

impl<T: ?Sized> KeyPathAccess for &T {}

/// Navigate with any [`Readable`] keypath.
#[inline]
pub fn readable_get<KP, Root, Value>(kp: &KP, root: Root) -> Option<Value>
where
    KP: Readable<Root, Value>,
{
    Readable::get(kp, root)
}

/// Navigate with any [`Writable`] keypath when `root` is `&mut T`.
#[inline]
pub fn writable_get<KP, MutRoot, MutValue>(kp: &KP, root: MutRoot) -> Option<MutValue>
where
    KP: Writable<MutRoot, MutValue>,
    MutRoot: private::MutRootRef,
{
    Writable::set(kp, root)
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
