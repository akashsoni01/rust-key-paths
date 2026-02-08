//! # Async Lock Keypath Module
//!
//! This module provides `AsyncLockKp` for safely navigating through async locked/synchronized data structures.
//!
//! # Naming convention (aligned with [crate::lock::LockKp] and [crate::Kp])
//!
//! - **`then`** – chain with a plain [crate::Kp]
//! - **`then_lock`** – chain with a sync [crate::lock::LockKp]
//! - **`then_async`** – chain with another async keypath (e.g. tokio RwLock)
//!
//! Example: `root_lock.then_lock(parking_kp).then_async(async_kp).then_lock(std_lock_kp)`
//!
//! # SHALLOW CLONING GUARANTEE
//!
//! **IMPORTANT**: All cloning operations in this module are SHALLOW (reference-counted) clones:
//!
//! 1. **`AsyncLockKp` derives `Clone`**: Clones function pointers and PhantomData only
//!    - `prev` and `next` fields contain function pointers (cheap to copy)
//!    - `mid` field is typically just `PhantomData<T>` (zero-sized, zero-cost)
//!    - No heap allocations or deep data copies
//!
//! 2. **`Lock: Clone` bound** (e.g., `Arc<tokio::sync::Mutex<T>>`):
//!    - For `Arc<T>`: Only increments the atomic reference count (one atomic operation)
//!    - The actual data `T` inside is **NEVER** cloned
//!    - This is the whole point of Arc - shared ownership without copying data
//!
//! 3. **`L: Clone` bound** (e.g., `TokioMutexAccess<T>`):
//!    - Only clones `PhantomData<T>` which is zero-sized
//!    - Compiled away completely - zero runtime cost

use crate::Kp;
use async_trait::async_trait;
use std::sync::Arc;

// Re-export tokio sync types for convenience
#[cfg(feature = "tokio")]
pub use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};

// =============================================================================
// Why two traits: AsyncLockLike and AsyncKeyPathLike
// =============================================================================
//
// - AsyncLockLike<Lock, Inner>: One "step" through a lock. Given a Lock (e.g.
//   Arc<Mutex<T>>), it async yields Inner (e.g. &T). Used by the `mid` field of
//   AsyncLockKp to go from "container" to "value inside the lock". Implemented
//   by TokioMutexAccess, TokioRwLockAccess, etc.
//
// - AsyncKeyPathLike<Root, MutRoot>: A full keypath from Root to a value. Used
//   so we can chain at any depth: both AsyncLockKp and ComposedAsyncLockKp
//   implement it, so we can write `kp1.then_async(kp2).then_async(kp3).get(&root)`.
//   Without this trait we could not express "first and second can be either a
//   single AsyncLockKp or another ComposedAsyncLockKp" in the type system.

/// Async trait for types that can provide async lock/unlock behavior
/// Converts from a Lock type to Inner or InnerMut value asynchronously
#[async_trait]
pub trait AsyncLockLike<Lock, Inner>: Send + Sync {
    /// Get immutable access to the inner value asynchronously
    async fn lock_read(&self, lock: &Lock) -> Option<Inner>;

    /// Get mutable access to the inner value asynchronously
    async fn lock_write(&self, lock: &mut Lock) -> Option<Inner>;
}

/// Sync keypath that can be used as the "second" in [AsyncLockKpThenLockKp] for blanket impls.
pub trait SyncKeyPathLike<Root, Value, MutRoot, MutValue> {
    fn sync_get(&self, root: Root) -> Option<Value>;
    fn sync_get_mut(&self, root: MutRoot) -> Option<MutValue>;
}

impl<
        R,
        Lock,
        Mid,
        V,
        Root,
        LockValue,
        MidValue,
        Value,
        MutRoot,
        MutLock,
        MutMid,
        MutValue,
        G1,
        S1,
        L,
        G2,
        S2,
    >
    SyncKeyPathLike<Root, Value, MutRoot, MutValue> for crate::lock::LockKp<
        R,
        Lock,
        Mid,
        V,
        Root,
        LockValue,
        MidValue,
        Value,
        MutRoot,
        MutLock,
        MutMid,
        MutValue,
        G1,
        S1,
        L,
        G2,
        S2,
    >
where
    Root: std::borrow::Borrow<R>,
    LockValue: std::borrow::Borrow<Lock>,
    MidValue: std::borrow::Borrow<Mid>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutLock: std::borrow::BorrowMut<Lock>,
    MutMid: std::borrow::BorrowMut<Mid>,
    MutValue: std::borrow::BorrowMut<V>,
    G1: Fn(Root) -> Option<LockValue>,
    S1: Fn(MutRoot) -> Option<MutLock>,
    L: crate::lock::LockAccess<Lock, MidValue> + crate::lock::LockAccess<Lock, MutMid>,
    G2: Fn(MidValue) -> Option<Value>,
    S2: Fn(MutMid) -> Option<MutValue>,
    V: Clone,
{
    fn sync_get(&self, root: Root) -> Option<Value> {
        self.get(root)
    }
    fn sync_get_mut(&self, root: MutRoot) -> Option<MutValue> {
        self.get_mut(root)
    }
}

/// Trait for async keypaths (both [AsyncLockKp] and [ComposedAsyncLockKp]) so composition can be any depth.
#[async_trait(?Send)]
pub trait AsyncKeyPathLike<Root, MutRoot> {
    /// Value type at the end of the keypath.
    type Value;
    /// Mutable value type at the end of the keypath.
    type MutValue;
    /// Get the value at the end of the keypath.
    async fn get(&self, root: Root) -> Option<Self::Value>;
    /// Get mutable access to the value at the end of the keypath.
    async fn get_mut(&self, root: MutRoot) -> Option<Self::MutValue>;
}

/// An async keypath that handles async locked values (e.g., Arc<tokio::sync::Mutex<T>>)
///
/// Structure:
/// - `prev`: Keypath from Root to Lock container (e.g., Arc<tokio::sync::Mutex<Mid>>)
/// - `mid`: Async lock access handler that goes from Lock to Inner value
/// - `next`: Keypath from Inner value to final Value
///
/// # Type Parameters
/// - `R`: Root type (base)
/// - `Lock`: Lock container type (e.g., Arc<tokio::sync::Mutex<Mid>>)
/// - `Mid`: The type inside the lock
/// - `V`: Final value type
/// - Rest are the same generic parameters as Kp
///
/// # Cloning Behavior
///
/// **IMPORTANT**: All `Clone` operations in this struct are SHALLOW clones:
///
/// - `AsyncLockKp` itself derives `Clone` - this clones the three field references/closures
/// - `prev` and `next` fields are `Kp` structs containing function pointers (cheap to clone)
/// - `mid` field implements `AsyncLockLike` trait - typically just `PhantomData` (zero-cost clone)
/// - When `Lock: Clone` (e.g., `Arc<tokio::sync::Mutex<T>>`), cloning is just incrementing reference count
/// - NO deep data cloning occurs - all clones are pointer/reference increments
#[derive(Clone)] // SHALLOW: Clones function pointers and PhantomData only
pub struct AsyncLockKp<
    R,
    Lock,
    Mid,
    V,
    Root,
    LockValue,
    MidValue,
    Value,
    MutRoot,
    MutLock,
    MutMid,
    MutValue,
    G1,
    S1,
    L,
    G2,
    S2,
> where
    Root: std::borrow::Borrow<R>,
    LockValue: std::borrow::Borrow<Lock>,
    MidValue: std::borrow::Borrow<Mid>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutLock: std::borrow::BorrowMut<Lock>,
    MutMid: std::borrow::BorrowMut<Mid>,
    MutValue: std::borrow::BorrowMut<V>,
    G1: Fn(Root) -> Option<LockValue> + Clone,
    S1: Fn(MutRoot) -> Option<MutLock> + Clone,
    L: AsyncLockLike<Lock, MidValue> + AsyncLockLike<Lock, MutMid> + Clone,
    G2: Fn(MidValue) -> Option<Value> + Clone,
    S2: Fn(MutMid) -> Option<MutValue> + Clone,
{
    /// Keypath from Root to Lock container
    pub prev: Kp<R, Lock, Root, LockValue, MutRoot, MutLock, G1, S1>,

    /// Async lock access handler (converts Lock -> Inner)
    pub mid: L,

    /// Keypath from Inner to final Value
    pub next: Kp<Mid, V, MidValue, Value, MutMid, MutValue, G2, S2>,
}

impl<
    R,
    Lock,
    Mid,
    V,
    Root,
    LockValue,
    MidValue,
    Value,
    MutRoot,
    MutLock,
    MutMid,
    MutValue,
    G1,
    S1,
    L,
    G2,
    S2,
>
    AsyncLockKp<
        R,
        Lock,
        Mid,
        V,
        Root,
        LockValue,
        MidValue,
        Value,
        MutRoot,
        MutLock,
        MutMid,
        MutValue,
        G1,
        S1,
        L,
        G2,
        S2,
    >
where
    Root: std::borrow::Borrow<R>,
    LockValue: std::borrow::Borrow<Lock>,
    MidValue: std::borrow::Borrow<Mid>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutLock: std::borrow::BorrowMut<Lock>,
    MutMid: std::borrow::BorrowMut<Mid>,
    MutValue: std::borrow::BorrowMut<V>,
    G1: Fn(Root) -> Option<LockValue> + Clone,
    S1: Fn(MutRoot) -> Option<MutLock> + Clone,
    L: AsyncLockLike<Lock, MidValue> + AsyncLockLike<Lock, MutMid> + Clone,
    G2: Fn(MidValue) -> Option<Value> + Clone,
    S2: Fn(MutMid) -> Option<MutValue> + Clone,
{
    /// Create a new AsyncLockKp with prev, mid, and next components
    pub fn new(
        prev: Kp<R, Lock, Root, LockValue, MutRoot, MutLock, G1, S1>,
        mid: L,
        next: Kp<Mid, V, MidValue, Value, MutMid, MutValue, G2, S2>,
    ) -> Self {
        Self { prev, mid, next }
    }

    /// Get the value through the lock
    ///
    /// This will:
    /// 1. Use `prev` to get to the Lock
    /// 2. Use `mid` to asynchronously lock and get the Inner value
    /// 3. Use `next` to get to the final Value
    ///
    /// # SHALLOW CLONING NOTE
    ///
    /// When `lock` is cloned (e.g., `Arc<tokio::sync::Mutex<T>>`):
    /// - Only the Arc reference count is incremented (one atomic operation)
    /// - The actual data `T` inside the Mutex is **NEVER** cloned
    /// - This is safe and efficient - the whole point of Arc
    pub async fn get(&self, root: Root) -> Option<Value>
    where
        Lock: Clone,
    {
        // SHALLOW CLONE: For Arc<Mutex<T>>, only increments Arc refcount
        // The actual data T is NOT cloned
        let lock_value = (self.prev.get)(root)?;
        let lock: &Lock = lock_value.borrow();
        let lock_clone = lock.clone(); // SHALLOW: Arc refcount++

        // Async lock and get the mid value
        let mid_value = self.mid.lock_read(&lock_clone).await?;

        // Navigate from mid to final value
        (self.next.get)(mid_value)
    }

    /// Get mutable access to the value through the lock
    pub async fn get_mut(&self, root: MutRoot) -> Option<MutValue>
    where
        Lock: Clone,
    {
        // SHALLOW CLONE: For Arc<Mutex<T>>, only increments Arc refcount
        let mut lock_value = (self.prev.set)(root)?;
        let lock: &mut Lock = lock_value.borrow_mut();
        let mut lock_clone = lock.clone(); // SHALLOW: Arc refcount++

        // Async lock and get the mid value
        let mid_value = self.mid.lock_write(&mut lock_clone).await?;

        // Navigate from mid to final value
        (self.next.set)(mid_value)
    }

    /// Set the value through the lock using an updater function
    pub async fn set<F>(&self, root: Root, updater: F) -> Result<(), String>
    where
        Lock: Clone,
        F: FnOnce(&mut V),
    {
        // SHALLOW CLONE: For Arc<Mutex<T>>, only increments Arc refcount
        let lock_value = (self.prev.get)(root).ok_or("Failed to get lock from root")?;
        let lock: &Lock = lock_value.borrow();
        let lock_clone = lock.clone(); // SHALLOW: Arc refcount++

        // Async lock and get the mid value
        let mut mid_value = self
            .mid
            .lock_read(&lock_clone)
            .await
            .ok_or("Failed to lock")?;

        // Get the final value
        let mut mut_value = (self.next.set)(mid_value).ok_or("Failed to navigate to value")?;
        let v: &mut V = mut_value.borrow_mut();

        // Apply the updater
        updater(v);

        Ok(())
    }

    // ========================================================================
    // Interoperability: then (Kp), then_lock (sync LockKp), then_async (async keypath)
    // ========================================================================

    /// Chain this AsyncLockKp with a regular [crate::Kp] (no root at call site).
    /// Returns an AsyncLockKp that goes one step further; use [AsyncLockKp::get] or [AsyncLockKp::get_mut] with root later.
    ///
    /// # Example
    /// ```
    /// // Root -> Arc<tokio::Mutex<Inner>> -> Inner -> field
    /// let async_kp = AsyncLockKp::new(root_to_lock, TokioMutexAccess::new(), lock_to_inner);
    /// let field_kp = Kp::new(|inner: &Inner| Some(&inner.field), |inner: &mut Inner| Some(&mut inner.field));
    /// let chained = async_kp.then(field_kp);
    /// let result = chained.get(&root).await;
    /// ```
    pub fn then<V2, Value2, MutValue2, G3, S3>(
        self,
        next_kp: crate::Kp<V, V2, Value, Value2, MutValue, MutValue2, G3, S3>,
    ) -> AsyncLockKp<
        R,
        Lock,
        Mid,
        V2,
        Root,
        LockValue,
        MidValue,
        Value2,
        MutRoot,
        MutLock,
        MutMid,
        MutValue2,
        G1,
        S1,
        L,
        impl Fn(MidValue) -> Option<Value2> + Clone,
        impl Fn(MutMid) -> Option<MutValue2> + Clone,
    >
    where
        V: 'static,
        V2: 'static,
        Value: std::borrow::Borrow<V>,
        Value2: std::borrow::Borrow<V2>,
        MutValue: std::borrow::BorrowMut<V>,
        MutValue2: std::borrow::BorrowMut<V2>,
        G3: Fn(Value) -> Option<Value2> + Clone,
        S3: Fn(MutValue) -> Option<MutValue2> + Clone,
    {
        let next_get = self.next.get;
        let next_set = self.next.set;
        let chained_kp = crate::Kp::new(
            move |mid_value: MidValue| next_get(mid_value).and_then(|v| (next_kp.get)(v)),
            move |mid_value: MutMid| next_set(mid_value).and_then(|v| (next_kp.set)(v)),
        );
        AsyncLockKp::new(self.prev, self.mid, chained_kp)
    }

    /// Chain this AsyncLockKp with a sync [crate::lock::LockKp] (no root at call site).
    /// Returns a keypath that first goes through the async lock, then through the sync lock; use `.get(&root).await` later.
    pub fn then_lock<
        Lock2,
        Mid2,
        V2,
        LockValue2,
        MidValue2,
        Value2,
        MutLock2,
        MutMid2,
        MutValue2,
        G2_1,
        S2_1,
        L2,
        G2_2,
        S2_2,
    >(
        self,
        lock_kp: crate::lock::LockKp<
            V,
            Lock2,
            Mid2,
            V2,
            Value,
            LockValue2,
            MidValue2,
            Value2,
            MutValue,
            MutLock2,
            MutMid2,
            MutValue2,
            G2_1,
            S2_1,
            L2,
            G2_2,
            S2_2,
        >,
    ) -> AsyncLockKpThenLockKp<
        R,
        V2,
        Root,
        Value2,
        MutRoot,
        MutValue2,
        Self,
        crate::lock::LockKp<
            V,
            Lock2,
            Mid2,
            V2,
            Value,
            LockValue2,
            MidValue2,
            Value2,
            MutValue,
            MutLock2,
            MutMid2,
            MutValue2,
            G2_1,
            S2_1,
            L2,
            G2_2,
            S2_2,
        >,
    >
    where
        V: 'static,
        V2: 'static,
        Value: std::borrow::Borrow<V>,
        Value2: std::borrow::Borrow<V2>,
        MutValue: std::borrow::BorrowMut<V>,
        MutValue2: std::borrow::BorrowMut<V2>,
        LockValue2: std::borrow::Borrow<Lock2>,
        MidValue2: std::borrow::Borrow<Mid2>,
        MutLock2: std::borrow::BorrowMut<Lock2>,
        MutMid2: std::borrow::BorrowMut<Mid2>,
        G2_1: Fn(Value) -> Option<LockValue2>,
        S2_1: Fn(MutValue) -> Option<MutLock2>,
        L2: crate::lock::LockAccess<Lock2, MidValue2> + crate::lock::LockAccess<Lock2, MutMid2>,
        G2_2: Fn(MidValue2) -> Option<Value2>,
        S2_2: Fn(MutMid2) -> Option<MutValue2>,
    {
        AsyncLockKpThenLockKp {
            first: self,
            second: lock_kp,
            _p: std::marker::PhantomData,
        }
    }

    /// Chain with another async keypath (like [crate::lock::LockKp::then_lock] for sync locks).
    ///
    /// Chain with another async keypath (e.g. tokio RwLock). Use [ComposedAsyncLockKp::get] or
    /// [ComposedAsyncLockKp::get_mut] with root later.
    ///
    /// Root -> AsyncLock1 -> Container -> AsyncLock2 -> Value
    ///
    /// # Example
    /// ```
    /// // Root -> Arc<tokio::Mutex<Container>> -> Container -> Arc<tokio::Mutex<Value>> -> Value
    /// let async_kp1 = AsyncLockKp::new(...); // Root -> Container
    /// let async_kp2 = AsyncLockKp::new(...); // Container -> Value
    /// let chained = async_kp1.then_async(async_kp2);
    /// let result = chained.get(&root).await;
    /// ```
    pub fn then_async<
        Lock2,
        Mid2,
        V2,
        LockValue2,
        MidValue2,
        Value2,
        MutLock2,
        MutMid2,
        MutValue2,
        G2_1,
        S2_1,
        L2,
        G2_2,
        S2_2,
    >(
        self,
        other: AsyncLockKp<
            V,
            Lock2,
            Mid2,
            V2,
            Value,
            LockValue2,
            MidValue2,
            Value2,
            MutValue,
            MutLock2,
            MutMid2,
            MutValue2,
            G2_1,
            S2_1,
            L2,
            G2_2,
            S2_2,
        >,
    ) -> ComposedAsyncLockKp<
        R,
        V2,
        Root,
        Value2,
        MutRoot,
        MutValue2,
        Self,
        AsyncLockKp<
            V,
            Lock2,
            Mid2,
            V2,
            Value,
            LockValue2,
            MidValue2,
            Value2,
            MutValue,
            MutLock2,
            MutMid2,
            MutValue2,
            G2_1,
            S2_1,
            L2,
            G2_2,
            S2_2,
        >,
    >
    where
        Lock: Clone,
        Lock2: Clone,
        V: 'static,
        V2: 'static,
        Value: std::borrow::Borrow<V>,
        LockValue2: std::borrow::Borrow<Lock2>,
        MidValue2: std::borrow::Borrow<Mid2>,
        Value2: std::borrow::Borrow<V2>,
        MutValue: std::borrow::BorrowMut<V>,
        MutLock2: std::borrow::BorrowMut<Lock2>,
        MutMid2: std::borrow::BorrowMut<Mid2>,
        MutValue2: std::borrow::BorrowMut<V2>,
        G2_1: Fn(Value) -> Option<LockValue2> + Clone,
        S2_1: Fn(MutValue) -> Option<MutLock2> + Clone,
        L2: AsyncLockLike<Lock2, MidValue2> + AsyncLockLike<Lock2, MutMid2> + Clone,
        G2_2: Fn(MidValue2) -> Option<Value2> + Clone,
        S2_2: Fn(MutMid2) -> Option<MutValue2> + Clone,
    {
        ComposedAsyncLockKp {
            first: self,
            second: other,
            _p: std::marker::PhantomData,
        }
    }
}

// Implement AsyncKeyPathLike for AsyncLockKp so it can be used in composition at any depth.
#[async_trait(?Send)]
impl<
    R,
    Lock,
    Mid,
    V,
    Root,
    LockValue,
    MidValue,
    Value,
    MutRoot,
    MutLock,
    MutMid,
    MutValue,
    G1,
    S1,
    L,
    G2,
    S2,
> AsyncKeyPathLike<Root, MutRoot>
    for AsyncLockKp<R, Lock, Mid, V, Root, LockValue, MidValue, Value, MutRoot, MutLock, MutMid, MutValue, G1, S1, L, G2, S2>
where
    Root: std::borrow::Borrow<R>,
    LockValue: std::borrow::Borrow<Lock>,
    MidValue: std::borrow::Borrow<Mid>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutLock: std::borrow::BorrowMut<Lock>,
    MutMid: std::borrow::BorrowMut<Mid>,
    MutValue: std::borrow::BorrowMut<V>,
    G1: Fn(Root) -> Option<LockValue> + Clone,
    S1: Fn(MutRoot) -> Option<MutLock> + Clone,
    L: AsyncLockLike<Lock, MidValue> + AsyncLockLike<Lock, MutMid> + Clone,
    G2: Fn(MidValue) -> Option<Value> + Clone,
    S2: Fn(MutMid) -> Option<MutValue> + Clone,
    Lock: Clone,
{
    type Value = Value;
    type MutValue = MutValue;
    async fn get(&self, root: Root) -> Option<Value> {
        AsyncLockKp::get(self, root).await
    }
    async fn get_mut(&self, root: MutRoot) -> Option<MutValue> {
        AsyncLockKp::get_mut(self, root).await
    }
}

/// Chained async lock keypath: two or more async keypaths (Root -> V -> V2 -> ...). Root is passed at get/get_mut time.
///
/// Use [AsyncLockKp::then_async] to create (or [ComposedAsyncLockKp::then_async] for more levels). Then call [ComposedAsyncLockKp::get] or
/// [ComposedAsyncLockKp::get_mut] with root when you need the value.
///
/// Chain any depth: `kp1.then_async(kp2).then_async(kp3).then_async(kp4)...` then `.get(&root).await`.
#[derive(Clone)]
pub struct ComposedAsyncLockKp<R, V2, Root, Value2, MutRoot, MutValue2, First, Second> {
    pub(crate) first: First,
    pub(crate) second: Second,
    _p: std::marker::PhantomData<(R, V2, Root, Value2, MutRoot, MutValue2)>,
}

impl<R, V2, Root, Value2, MutRoot, MutValue2, First, Second>
    ComposedAsyncLockKp<R, V2, Root, Value2, MutRoot, MutValue2, First, Second>
where
    First: AsyncKeyPathLike<Root, MutRoot>,
    Second: AsyncKeyPathLike<First::Value, First::MutValue, Value = Value2, MutValue = MutValue2>,
{
    /// Get through all chained async locks (root is passed here).
    pub async fn get(&self, root: Root) -> Option<Value2> {
        let value = self.first.get(root).await?;
        self.second.get(value).await
    }

    /// Get mutable through all composed locks (root is passed here).
    pub async fn get_mut(&self, root: MutRoot) -> Option<MutValue2> {
        let mut_value = self.first.get_mut(root).await?;
        self.second.get_mut(mut_value).await
    }

    /// Chain with another async keypath: `a.then_async(b).then_async(c).get(&root).await`.
    pub fn then_async<
        Lock3,
        Mid3,
        V3,
        LockValue3,
        MidValue3,
        Value3,
        MutLock3,
        MutMid3,
        MutValue3,
        G3_1,
        S3_1,
        L3,
        G3_2,
        S3_2,
    >(
        self,
        other: AsyncLockKp<
            V2,
            Lock3,
            Mid3,
            V3,
            Value2,
            LockValue3,
            MidValue3,
            Value3,
            MutValue2,
            MutLock3,
            MutMid3,
            MutValue3,
            G3_1,
            S3_1,
            L3,
            G3_2,
            S3_2,
        >,
    ) -> ComposedAsyncLockKp<
        R,
        V3,
        Root,
        Value3,
        MutRoot,
        MutValue3,
        Self,
        AsyncLockKp<
            V2,
            Lock3,
            Mid3,
            V3,
            Value2,
            LockValue3,
            MidValue3,
            Value3,
            MutValue2,
            MutLock3,
            MutMid3,
            MutValue3,
            G3_1,
            S3_1,
            L3,
            G3_2,
            S3_2,
        >,
    >
    where
        V2: 'static,
        V3: 'static,
        Value2: std::borrow::Borrow<V2>,
        Value3: std::borrow::Borrow<V3>,
        MutValue2: std::borrow::BorrowMut<V2>,
        MutValue3: std::borrow::BorrowMut<V3>,
        LockValue3: std::borrow::Borrow<Lock3>,
        MidValue3: std::borrow::Borrow<Mid3>,
        MutLock3: std::borrow::BorrowMut<Lock3>,
        MutMid3: std::borrow::BorrowMut<Mid3>,
        G3_1: Fn(Value2) -> Option<LockValue3> + Clone,
        S3_1: Fn(MutValue2) -> Option<MutLock3> + Clone,
        L3: AsyncLockLike<Lock3, MidValue3> + AsyncLockLike<Lock3, MutMid3> + Clone,
        G3_2: Fn(MidValue3) -> Option<Value3> + Clone,
        S3_2: Fn(MutMid3) -> Option<MutValue3> + Clone,
        Lock3: Clone,
    {
        ComposedAsyncLockKp {
            first: self,
            second: other,
            _p: std::marker::PhantomData,
        }
    }

    /// Chain with a regular [crate::Kp] (no root at call site). Use `.get(&root).await` later.
    pub fn then<V3, Value3, MutValue3, G3, S3>(
        self,
        next_kp: crate::Kp<V2, V3, Value2, Value3, MutValue2, MutValue3, G3, S3>,
    ) -> AsyncKeyPathThenKp<R, V3, Root, Value3, MutRoot, MutValue3, Self, crate::Kp<V2, V3, Value2, Value3, MutValue2, MutValue3, G3, S3>>
    where
        V2: 'static,
        V3: 'static,
        Value2: std::borrow::Borrow<V2>,
        Value3: std::borrow::Borrow<V3>,
        MutValue2: std::borrow::BorrowMut<V2>,
        MutValue3: std::borrow::BorrowMut<V3>,
        G3: Fn(Value2) -> Option<Value3> + Clone,
        S3: Fn(MutValue2) -> Option<MutValue3> + Clone,
    {
        AsyncKeyPathThenKp {
            first: self,
            second: next_kp,
            _p: std::marker::PhantomData,
        }
    }

    /// Chain with a sync [crate::lock::LockKp] (no root at call site). Use `.get(&root).await` later.
    pub fn then_lock<
        Lock3,
        Mid3,
        V3,
        LockValue3,
        MidValue3,
        Value3,
        MutLock3,
        MutMid3,
        MutValue3,
        G3_1,
        S3_1,
        L3,
        G3_2,
        S3_2,
    >(
        self,
        lock_kp: crate::lock::LockKp<V2, Lock3, Mid3, V3, Value2, LockValue3, MidValue3, Value3, MutValue2, MutLock3, MutMid3, MutValue3, G3_1, S3_1, L3, G3_2, S3_2>,
    ) -> AsyncLockKpThenLockKp<R, V3, Root, Value3, MutRoot, MutValue3, Self, crate::lock::LockKp<V2, Lock3, Mid3, V3, Value2, LockValue3, MidValue3, Value3, MutValue2, MutLock3, MutMid3, MutValue3, G3_1, S3_1, L3, G3_2, S3_2>>
    where
        V2: 'static,
        V3: 'static,
        Value2: std::borrow::Borrow<V2>,
        Value3: std::borrow::Borrow<V3>,
        MutValue2: std::borrow::BorrowMut<V2>,
        MutValue3: std::borrow::BorrowMut<V3>,
        LockValue3: std::borrow::Borrow<Lock3>,
        MidValue3: std::borrow::Borrow<Mid3>,
        MutLock3: std::borrow::BorrowMut<Lock3>,
        MutMid3: std::borrow::BorrowMut<Mid3>,
        G3_1: Fn(Value2) -> Option<LockValue3>,
        S3_1: Fn(MutValue2) -> Option<MutLock3>,
        L3: crate::lock::LockAccess<Lock3, MidValue3> + crate::lock::LockAccess<Lock3, MutMid3>,
        G3_2: Fn(MidValue3) -> Option<Value3>,
        S3_2: Fn(MutMid3) -> Option<MutValue3>,
    {
        AsyncLockKpThenLockKp {
            first: self,
            second: lock_kp,
            _p: std::marker::PhantomData,
        }
    }
}

/// Keypath that chains an [AsyncKeyPathLike] (async get) with a [crate::Kp] (sync step). Use `.get(&root).await` to run.
#[derive(Clone)]
pub struct AsyncKeyPathThenKp<R, V2, Root, Value2, MutRoot, MutValue2, First, Second> {
    pub(crate) first: First,
    pub(crate) second: Second,
    _p: std::marker::PhantomData<(R, V2, Root, Value2, MutRoot, MutValue2)>,
}

impl<R, V2, Root, Value2, MutRoot, MutValue2, Value, MutValue, First, G, S>
    AsyncKeyPathThenKp<R, V2, Root, Value2, MutRoot, MutValue2, First, crate::Kp<Value, V2, Value, Value2, MutValue, MutValue2, G, S>>
where
    First: AsyncKeyPathLike<Root, MutRoot, Value = Value, MutValue = MutValue>,
    Value: std::borrow::Borrow<Value>,
    Value2: std::borrow::Borrow<V2>,
    MutValue: std::borrow::BorrowMut<Value>,
    MutValue2: std::borrow::BorrowMut<V2>,
    G: Fn(Value) -> Option<Value2>,
    S: Fn(MutValue) -> Option<MutValue2>,
{
    /// Get through async keypath then Kp (root is passed here).
    pub async fn get(&self, root: Root) -> Option<Value2> {
        let value = self.first.get(root).await?;
        (self.second.get)(value)
    }
    /// Get mutable through async keypath then Kp (root is passed here).
    pub async fn get_mut(&self, root: MutRoot) -> Option<MutValue2> {
        let mut_value = self.first.get_mut(root).await?;
        (self.second.set)(mut_value)
    }
}

#[async_trait(?Send)]
impl<R, V2, Root, Value2, MutRoot, MutValue2, First, Second>
    AsyncKeyPathLike<Root, MutRoot> for ComposedAsyncLockKp<R, V2, Root, Value2, MutRoot, MutValue2, First, Second>
where
    First: AsyncKeyPathLike<Root, MutRoot>,
    Second: AsyncKeyPathLike<First::Value, First::MutValue, Value = Value2, MutValue = MutValue2>,
{
    type Value = Value2;
    type MutValue = MutValue2;
    async fn get(&self, root: Root) -> Option<Value2> {
        ComposedAsyncLockKp::get(self, root).await
    }
    async fn get_mut(&self, root: MutRoot) -> Option<MutValue2> {
        ComposedAsyncLockKp::get_mut(self, root).await
    }
}

// =============================================================================
// AsyncLockKpThenLockKp: AsyncLockKp .then_lock(LockKp) — async then sync lock
// =============================================================================

/// Keypath that goes through an async lock then a sync [crate::lock::LockKp].
/// Use [AsyncLockKp::then_lock] to create; then call [AsyncLockKpThenLockKp::get] or [AsyncLockKpThenLockKp::get_mut] with root.
#[derive(Clone)]
pub struct AsyncLockKpThenLockKp<R, V2, Root, Value2, MutRoot, MutValue2, First, Second> {
    pub(crate) first: First,
    pub(crate) second: Second,
    _p: std::marker::PhantomData<(R, V2, Root, Value2, MutRoot, MutValue2)>,
}

impl<
        R,
        V2,
        Root,
        Value2,
        MutRoot,
        MutValue2,
        Lock,
        Mid,
        V,
        LockValue,
        MidValue,
        Value,
        MutLock,
        MutMid,
        MutValue,
        G1,
        S1,
        L,
        G2,
        S2,
        Lock2,
        Mid2,
        LockValue2,
        MidValue2,
        MutLock2,
        MutMid2,
        G2_1,
        S2_1,
        L2,
        G2_2,
        S2_2,
    >
    AsyncLockKpThenLockKp<
        R,
        V2,
        Root,
        Value2,
        MutRoot,
        MutValue2,
        AsyncLockKp<R, Lock, Mid, V, Root, LockValue, MidValue, Value, MutRoot, MutLock, MutMid, MutValue, G1, S1, L, G2, S2>,
        crate::lock::LockKp<V, Lock2, Mid2, V2, Value, LockValue2, MidValue2, Value2, MutValue, MutLock2, MutMid2, MutValue2, G2_1, S2_1, L2, G2_2, S2_2>,
    >
where
    Root: std::borrow::Borrow<R>,
    LockValue: std::borrow::Borrow<Lock>,
    MidValue: std::borrow::Borrow<Mid>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutLock: std::borrow::BorrowMut<Lock>,
    MutMid: std::borrow::BorrowMut<Mid>,
    MutValue: std::borrow::BorrowMut<V>,
    Value2: std::borrow::Borrow<V2>,
    MutValue2: std::borrow::BorrowMut<V2>,
    G1: Fn(Root) -> Option<LockValue> + Clone,
    S1: Fn(MutRoot) -> Option<MutLock> + Clone,
    L: AsyncLockLike<Lock, MidValue> + AsyncLockLike<Lock, MutMid> + Clone,
    G2: Fn(MidValue) -> Option<Value> + Clone,
    S2: Fn(MutMid) -> Option<MutValue> + Clone,
    LockValue2: std::borrow::Borrow<Lock2>,
    MidValue2: std::borrow::Borrow<Mid2>,
    MutLock2: std::borrow::BorrowMut<Lock2>,
    MutMid2: std::borrow::BorrowMut<Mid2>,
    G2_1: Fn(Value) -> Option<LockValue2>,
    S2_1: Fn(MutValue) -> Option<MutLock2>,
    L2: crate::lock::LockAccess<Lock2, MidValue2> + crate::lock::LockAccess<Lock2, MutMid2>,
    G2_2: Fn(MidValue2) -> Option<Value2>,
    S2_2: Fn(MutMid2) -> Option<MutValue2>,
    Lock: Clone,
    V: Clone,
    V2: Clone,
{
    /// Get through async lock then sync lock (root is passed here).
    pub async fn get(&self, root: Root) -> Option<Value2> {
        let value = self.first.get(root).await?;
        self.second.get(value)
    }

    /// Get mutable through async lock then sync lock (root is passed here).
    pub async fn get_mut(&self, root: MutRoot) -> Option<MutValue2> {
        let mut_value = self.first.get_mut(root).await?;
        self.second.get_mut(mut_value)
    }
}

// AsyncLockKpThenLockKp when First is ComposedAsyncLockKp (so ComposedAsyncLockKp::then_lock works).
impl<
        R,
        V2,
        Root,
        Value2,
        MutRoot,
        MutValue2,
        Lock3,
        Mid3,
        LockValue3,
        MidValue3,
        MutLock3,
        MutMid3,
        G3_1,
        S3_1,
        L3,
        G3_2,
        S3_2,
        First,
        Second,
    >
    AsyncLockKpThenLockKp<
        R,
        V2,
        Root,
        Value2,
        MutRoot,
        MutValue2,
        ComposedAsyncLockKp<R, V2, Root, Value2, MutRoot, MutValue2, First, Second>,
        crate::lock::LockKp<Value2, Lock3, Mid3, V2, Value2, LockValue3, MidValue3, Value2, MutValue2, MutLock3, MutMid3, MutValue2, G3_1, S3_1, L3, G3_2, S3_2>,
    >
where
    First: AsyncKeyPathLike<Root, MutRoot>,
    Second: AsyncKeyPathLike<First::Value, First::MutValue, Value = Value2, MutValue = MutValue2>,
    Value2: std::borrow::Borrow<V2>,
    MutValue2: std::borrow::BorrowMut<Value2> + std::borrow::BorrowMut<V2>,
    LockValue3: std::borrow::Borrow<Lock3>,
    MidValue3: std::borrow::Borrow<Mid3>,
    MutLock3: std::borrow::BorrowMut<Lock3>,
    MutMid3: std::borrow::BorrowMut<Mid3>,
    G3_1: Fn(Value2) -> Option<LockValue3>,
    S3_1: Fn(MutValue2) -> Option<MutLock3>,
    L3: crate::lock::LockAccess<Lock3, MidValue3> + crate::lock::LockAccess<Lock3, MutMid3>,
    G3_2: Fn(MidValue3) -> Option<Value2>,
    S3_2: Fn(MutMid3) -> Option<MutValue2>,
    Value2: Clone,
    V2: Clone,
{
    /// Get through composed async then sync lock (root is passed here).
    pub async fn get(&self, root: Root) -> Option<Value2> {
        let value = self.first.get(root).await?;
        self.second.get(value)
    }
    /// Get mutable through composed async then sync lock (root is passed here).
    pub async fn get_mut(&self, root: MutRoot) -> Option<MutValue2> {
        let mut_value = self.first.get_mut(root).await?;
        self.second.get_mut(mut_value)
    }
}

// AsyncLockKpThenLockKp when First is AsyncLockKpThenLockKp (nested; enables .then_lock().then_lock() chains).
impl<
        R,
        V2,
        Root,
        Value2,
        MutRoot,
        MutValue2,
        Lock3,
        Mid3,
        LockValue3,
        MidValue3,
        MutLock3,
        MutMid3,
        G3_1,
        S3_1,
        L3,
        G3_2,
        S3_2,
        F,
        S,
    >
    AsyncLockKpThenLockKp<
        R,
        V2,
        Root,
        Value2,
        MutRoot,
        MutValue2,
        AsyncLockKpThenLockKp<R, V2, Root, Value2, MutRoot, MutValue2, F, S>,
        crate::lock::LockKp<Value2, Lock3, Mid3, V2, Value2, LockValue3, MidValue3, Value2, MutValue2, MutLock3, MutMid3, MutValue2, G3_1, S3_1, L3, G3_2, S3_2>,
    >
where
    F: AsyncKeyPathLike<Root, MutRoot, Value = Value2, MutValue = MutValue2>,
    S: SyncKeyPathLike<Value2, Value2, MutValue2, MutValue2>,
    Value2: std::borrow::Borrow<V2>,
    MutValue2: std::borrow::BorrowMut<Value2> + std::borrow::BorrowMut<V2>,
    LockValue3: std::borrow::Borrow<Lock3>,
    MidValue3: std::borrow::Borrow<Mid3>,
    MutLock3: std::borrow::BorrowMut<Lock3>,
    MutMid3: std::borrow::BorrowMut<Mid3>,
    G3_1: Fn(Value2) -> Option<LockValue3>,
    S3_1: Fn(MutValue2) -> Option<MutLock3>,
    L3: crate::lock::LockAccess<Lock3, MidValue3> + crate::lock::LockAccess<Lock3, MutMid3>,
    G3_2: Fn(MidValue3) -> Option<Value2>,
    S3_2: Fn(MutMid3) -> Option<MutValue2>,
    Value2: Clone,
    V2: Clone,
{
    /// Get through async then sync then sync lock (root is passed here).
    pub async fn get(&self, root: Root) -> Option<Value2> {
        let value = AsyncKeyPathLike::get(&self.first, root).await?;
        self.second.get(value)
    }
    /// Get mutable through async then sync then sync lock (root is passed here).
    pub async fn get_mut(&self, root: MutRoot) -> Option<MutValue2> {
        let mut_value = AsyncKeyPathLike::get_mut(&self.first, root).await?;
        self.second.get_mut(mut_value)
    }
}

// Blanket AsyncKeyPathLike for AsyncLockKpThenLockKp so nested chains (then_lock().then_lock()) work.
#[async_trait(?Send)]
impl<R, V2, Root, Value2, MutRoot, MutValue2, First, Second> AsyncKeyPathLike<Root, MutRoot>
    for AsyncLockKpThenLockKp<R, V2, Root, Value2, MutRoot, MutValue2, First, Second>
where
    First: AsyncKeyPathLike<Root, MutRoot>,
    Second: SyncKeyPathLike<First::Value, Value2, First::MutValue, MutValue2>,
{
    type Value = Value2;
    type MutValue = MutValue2;
    async fn get(&self, root: Root) -> Option<Value2> {
        let value = AsyncKeyPathLike::get(&self.first, root).await?;
        SyncKeyPathLike::sync_get(&self.second, value)
    }
    async fn get_mut(&self, root: MutRoot) -> Option<MutValue2> {
        let mut_value = AsyncKeyPathLike::get_mut(&self.first, root).await?;
        SyncKeyPathLike::sync_get_mut(&self.second, mut_value)
    }
}

// then_lock and then on AsyncLockKpThenLockKp so chains can continue.
impl<R, V2, Root, Value2, MutRoot, MutValue2, First, Second> AsyncLockKpThenLockKp<R, V2, Root, Value2, MutRoot, MutValue2, First, Second>
where
    First: AsyncKeyPathLike<Root, MutRoot>,
{
    /// Chain with an async keypath (e.g. tokio RwLock): ... -> Value2 -> async lock -> Value3.
    /// Use `.get(&root).await` or `.get_mut(...).await` on the returned [ComposedAsyncLockKp].
    pub fn then_async<
        Lock3,
        Mid3,
        V3,
        LockValue3,
        MidValue3,
        Value3,
        MutLock3,
        MutMid3,
        MutValue3,
        G3_1,
        S3_1,
        L3,
        G3_2,
        S3_2,
    >(
        self,
        other: AsyncLockKp<
            Value2,
            Lock3,
            Mid3,
            V3,
            Value2,
            LockValue3,
            MidValue3,
            Value3,
            MutValue2,
            MutLock3,
            MutMid3,
            MutValue3,
            G3_1,
            S3_1,
            L3,
            G3_2,
            S3_2,
        >,
    ) -> ComposedAsyncLockKp<
        Root,
        V3,
        Root,
        Value3,
        MutRoot,
        MutValue3,
        Self,
        AsyncLockKp<
            Value2,
            Lock3,
            Mid3,
            V3,
            Value2,
            LockValue3,
            MidValue3,
            Value3,
            MutValue2,
            MutLock3,
            MutMid3,
            MutValue3,
            G3_1,
            S3_1,
            L3,
            G3_2,
            S3_2,
        >,
    >
    where
        V2: 'static,
        V3: 'static,
        Value2: std::borrow::Borrow<V2>,
        Value3: std::borrow::Borrow<V3>,
        MutValue2: std::borrow::BorrowMut<V2> + std::borrow::BorrowMut<Value2>,
        MutValue3: std::borrow::BorrowMut<V3>,
        LockValue3: std::borrow::Borrow<Lock3>,
        MidValue3: std::borrow::Borrow<Mid3>,
        MutLock3: std::borrow::BorrowMut<Lock3>,
        MutMid3: std::borrow::BorrowMut<Mid3>,
        G3_1: Fn(Value2) -> Option<LockValue3> + Clone,
        S3_1: Fn(MutValue2) -> Option<MutLock3> + Clone,
        L3: AsyncLockLike<Lock3, MidValue3> + AsyncLockLike<Lock3, MutMid3> + Clone,
        G3_2: Fn(MidValue3) -> Option<Value3> + Clone,
        S3_2: Fn(MutMid3) -> Option<MutValue3> + Clone,
        Lock3: Clone,
    {
        ComposedAsyncLockKp {
            first: self,
            second: other,
            _p: std::marker::PhantomData,
        }
    }

    /// Chain with another sync [crate::lock::LockKp]. Use `.get(&root).await` later.
    pub fn then_lock<
        Lock3,
        Mid3,
        V3,
        LockValue3,
        MidValue3,
        Value3,
        MutLock3,
        MutMid3,
        MutValue3,
        G3_1,
        S3_1,
        L3,
        G3_2,
        S3_2,
    >(
        self,
        lock_kp: crate::lock::LockKp<Value2, Lock3, Mid3, V3, Value2, LockValue3, MidValue3, Value3, MutValue2, MutLock3, MutMid3, MutValue3, G3_1, S3_1, L3, G3_2, S3_2>,
    ) -> AsyncLockKpThenLockKp<R, V3, Root, Value3, MutRoot, MutValue3, Self, crate::lock::LockKp<Value2, Lock3, Mid3, V3, Value2, LockValue3, MidValue3, Value3, MutValue2, MutLock3, MutMid3, MutValue3, G3_1, S3_1, L3, G3_2, S3_2>>
    where
        V3: 'static,
        Value2: std::borrow::Borrow<V2>,
        Value3: std::borrow::Borrow<V3>,
        MutValue2: std::borrow::BorrowMut<Value2>,
        MutValue3: std::borrow::BorrowMut<V3>,
        LockValue3: std::borrow::Borrow<Lock3>,
        MidValue3: std::borrow::Borrow<Mid3>,
        MutLock3: std::borrow::BorrowMut<Lock3>,
        MutMid3: std::borrow::BorrowMut<Mid3>,
        G3_1: Fn(Value2) -> Option<LockValue3>,
        S3_1: Fn(MutValue2) -> Option<MutLock3>,
        L3: crate::lock::LockAccess<Lock3, MidValue3> + crate::lock::LockAccess<Lock3, MutMid3>,
        G3_2: Fn(MidValue3) -> Option<Value3>,
        S3_2: Fn(MutMid3) -> Option<MutValue3>,
    {
        AsyncLockKpThenLockKp {
            first: self,
            second: lock_kp,
            _p: std::marker::PhantomData,
        }
    }

    /// Chain with a regular [crate::Kp]. Use `.get(&root).await` later.
    pub fn then<V3, Value3, MutValue3, G3, S3>(
        self,
        next_kp: crate::Kp<Value2, V3, Value2, Value3, MutValue2, MutValue3, G3, S3>,
    ) -> AsyncKeyPathThenKp<R, V3, Root, Value3, MutRoot, MutValue3, Self, crate::Kp<Value2, V3, Value2, Value3, MutValue2, MutValue3, G3, S3>>
    where
        V3: 'static,
        Value2: std::borrow::Borrow<V2>,
        Value3: std::borrow::Borrow<V3>,
        MutValue2: std::borrow::BorrowMut<Value2>,
        MutValue3: std::borrow::BorrowMut<V3>,
        G3: Fn(Value2) -> Option<Value3> + Clone,
        S3: Fn(MutValue2) -> Option<MutValue3> + Clone,
    {
        AsyncKeyPathThenKp {
            first: self,
            second: next_kp,
            _p: std::marker::PhantomData,
        }
    }
}

// ============================================================================
// Tokio Mutex Access Implementation
// ============================================================================

#[cfg(feature = "tokio")]
/// Async lock access implementation for Arc<tokio::sync::Mutex<T>>
///
/// # Cloning Behavior
///
/// This struct only contains `PhantomData<T>`.
/// Cloning is a **zero-cost operation** - no data is copied.
#[derive(Clone)] // ZERO-COST: Only clones PhantomData (zero-sized type)
pub struct TokioMutexAccess<T> {
    _phantom: std::marker::PhantomData<T>,
}

#[cfg(feature = "tokio")]
impl<T> TokioMutexAccess<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "tokio")]
impl<T> Default for TokioMutexAccess<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Implementation for immutable access
#[cfg(feature = "tokio")]
#[async_trait]
impl<'a, T: 'static + Send + Sync> AsyncLockLike<Arc<tokio::sync::Mutex<T>>, &'a T>
    for TokioMutexAccess<T>
{
    async fn lock_read(&self, lock: &Arc<tokio::sync::Mutex<T>>) -> Option<&'a T> {
        // SHALLOW CLONE: Only Arc refcount is incremented
        let guard = lock.lock().await;
        let ptr = &*guard as *const T;
        unsafe { Some(&*ptr) }
    }

    async fn lock_write(&self, lock: &mut Arc<tokio::sync::Mutex<T>>) -> Option<&'a T> {
        let guard = lock.lock().await;
        let ptr = &*guard as *const T;
        unsafe { Some(&*ptr) }
    }
}

// Implementation for mutable access
#[cfg(feature = "tokio")]
#[async_trait]
impl<'a, T: 'static + Send + Sync> AsyncLockLike<Arc<tokio::sync::Mutex<T>>, &'a mut T>
    for TokioMutexAccess<T>
{
    async fn lock_read(&self, lock: &Arc<tokio::sync::Mutex<T>>) -> Option<&'a mut T> {
        // SHALLOW CLONE: Only Arc refcount is incremented
        let mut guard = lock.lock().await;
        let ptr = &mut *guard as *mut T;
        unsafe { Some(&mut *ptr) }
    }

    async fn lock_write(&self, lock: &mut Arc<tokio::sync::Mutex<T>>) -> Option<&'a mut T> {
        let mut guard = lock.lock().await;
        let ptr = &mut *guard as *mut T;
        unsafe { Some(&mut *ptr) }
    }
}

// ============================================================================
// Tokio RwLock Access Implementation
// ============================================================================

#[cfg(feature = "tokio")]
/// Async lock access implementation for Arc<tokio::sync::RwLock<T>>
///
/// # Cloning Behavior
///
/// This struct only contains `PhantomData<T>`.
/// Cloning is a **zero-cost operation** - no data is copied.
/// Manual Clone impl so `T: Clone` is not required (e.g. for `Level3` with `RwLock<i32>`).
pub struct TokioRwLockAccess<T> {
    _phantom: std::marker::PhantomData<T>,
}

#[cfg(feature = "tokio")]
impl<T> TokioRwLockAccess<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "tokio")]
impl<T> Default for TokioRwLockAccess<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "tokio")]
impl<T> Clone for TokioRwLockAccess<T> {
    fn clone(&self) -> Self {
        Self {
            _phantom: self._phantom,
        }
    }
}

// Implementation for immutable access (read lock)
#[cfg(feature = "tokio")]
#[async_trait]
impl<'a, T: 'static + Send + Sync> AsyncLockLike<Arc<tokio::sync::RwLock<T>>, &'a T>
    for TokioRwLockAccess<T>
{
    async fn lock_read(&self, lock: &Arc<tokio::sync::RwLock<T>>) -> Option<&'a T> {
        // SHALLOW CLONE: Only Arc refcount is incremented
        let guard = lock.read().await;
        let ptr = &*guard as *const T;
        unsafe { Some(&*ptr) }
    }

    async fn lock_write(&self, lock: &mut Arc<tokio::sync::RwLock<T>>) -> Option<&'a T> {
        // For immutable access, use read lock
        let guard = lock.read().await;
        let ptr = &*guard as *const T;
        unsafe { Some(&*ptr) }
    }
}

// Implementation for mutable access (write lock)
#[cfg(feature = "tokio")]
#[async_trait]
impl<'a, T: 'static + Send + Sync> AsyncLockLike<Arc<tokio::sync::RwLock<T>>, &'a mut T>
    for TokioRwLockAccess<T>
{
    async fn lock_read(&self, lock: &Arc<tokio::sync::RwLock<T>>) -> Option<&'a mut T> {
        // For mutable access, use write lock
        let mut guard = lock.write().await;
        let ptr = &mut *guard as *mut T;
        unsafe { Some(&mut *ptr) }
    }

    async fn lock_write(&self, lock: &mut Arc<tokio::sync::RwLock<T>>) -> Option<&'a mut T> {
        // SHALLOW CLONE: Only Arc refcount is incremented
        let mut guard = lock.write().await;
        let ptr = &mut *guard as *mut T;
        unsafe { Some(&mut *ptr) }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(all(test, feature = "tokio"))]
mod tests {
    use super::*;
    use crate::KpType;

    #[tokio::test]
    async fn test_async_lock_kp_tokio_mutex_basic() {
        use tokio::sync::Mutex;

        #[derive(Clone)]
        struct Root {
            data: Arc<Mutex<String>>,
        }

        let root = Root {
            data: Arc::new(Mutex::new("hello".to_string())),
        };

        // Create AsyncLockKp
        let lock_kp = {
            let prev: KpType<Root, Arc<Mutex<String>>> =
                Kp::new(|r: &Root| Some(&r.data), |r: &mut Root| Some(&mut r.data));
            let next: KpType<String, String> =
                Kp::new(|s: &String| Some(s), |s: &mut String| Some(s));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        // Test async get
        let value = lock_kp.get(&root).await;
        assert!(value.is_some());
        assert_eq!(value.unwrap(), &"hello".to_string());
    }

    #[tokio::test]
    async fn test_async_lock_kp_tokio_rwlock_basic() {
        use tokio::sync::RwLock;

        #[derive(Clone)]
        struct Root {
            data: Arc<RwLock<Vec<i32>>>,
        }

        let root = Root {
            data: Arc::new(RwLock::new(vec![1, 2, 3, 4, 5])),
        };

        // Create AsyncLockKp with RwLock
        let lock_kp = {
            let prev: KpType<Root, Arc<RwLock<Vec<i32>>>> =
                Kp::new(|r: &Root| Some(&r.data), |r: &mut Root| Some(&mut r.data));
            let next: KpType<Vec<i32>, Vec<i32>> =
                Kp::new(|v: &Vec<i32>| Some(v), |v: &mut Vec<i32>| Some(v));
            AsyncLockKp::new(prev, TokioRwLockAccess::new(), next)
        };

        // Test async get with RwLock (read lock)
        let value = lock_kp.get(&root).await;
        assert!(value.is_some());
        assert_eq!(value.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn test_async_lock_kp_concurrent_reads() {
        use tokio::sync::RwLock;

        #[derive(Clone)]
        struct Root {
            data: Arc<RwLock<i32>>,
        }

        let root = Root {
            data: Arc::new(RwLock::new(42)),
        };

        // Create AsyncLockKp
        let lock_kp = {
            let prev: KpType<Root, Arc<RwLock<i32>>> =
                Kp::new(|r: &Root| Some(&r.data), |r: &mut Root| Some(&mut r.data));
            let next: KpType<i32, i32> = Kp::new(|n: &i32| Some(n), |n: &mut i32| Some(n));
            AsyncLockKp::new(prev, TokioRwLockAccess::new(), next)
        };

        // Spawn multiple concurrent async reads
        let mut handles = vec![];
        for _ in 0..10 {
            let root_clone = root.clone();

            // Re-create lock_kp for each task since we can't clone it easily
            let lock_kp_for_task = {
                let prev: KpType<Root, Arc<RwLock<i32>>> =
                    Kp::new(|r: &Root| Some(&r.data), |r: &mut Root| Some(&mut r.data));
                let next: KpType<i32, i32> = Kp::new(|n: &i32| Some(n), |n: &mut i32| Some(n));
                AsyncLockKp::new(prev, TokioRwLockAccess::new(), next)
            };

            let handle = tokio::spawn(async move { lock_kp_for_task.get(&root_clone).await });
            handles.push(handle);
        }

        // All reads should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert_eq!(result, Some(&42));
        }

        // Test the original lock_kp as well
        let value = lock_kp.get(&root).await;
        assert_eq!(value, Some(&42));
    }

    #[tokio::test]
    async fn test_async_lock_kp_panic_on_clone_proof() {
        use tokio::sync::Mutex;

        /// This struct PANICS if cloned - proving no deep cloning occurs
        struct PanicOnClone {
            data: String,
        }

        impl Clone for PanicOnClone {
            fn clone(&self) -> Self {
                panic!("❌ ASYNC DEEP CLONE DETECTED! PanicOnClone was cloned!");
            }
        }

        #[derive(Clone)]
        struct Root {
            level1: Arc<Mutex<Level1>>,
        }

        struct Level1 {
            panic_data: PanicOnClone,
            value: i32,
        }

        impl Clone for Level1 {
            fn clone(&self) -> Self {
                panic!("❌ Level1 was deeply cloned in async context!");
            }
        }

        // Create structure with PanicOnClone
        let root = Root {
            level1: Arc::new(Mutex::new(Level1 {
                panic_data: PanicOnClone {
                    data: "test".to_string(),
                },
                value: 123,
            })),
        };

        // Create AsyncLockKp
        let lock_kp = {
            let prev: KpType<Root, Arc<Mutex<Level1>>> = Kp::new(
                |r: &Root| Some(&r.level1),
                |r: &mut Root| Some(&mut r.level1),
            );
            let next: KpType<Level1, i32> = Kp::new(
                |l: &Level1| Some(&l.value),
                |l: &mut Level1| Some(&mut l.value),
            );
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        // CRITICAL TEST: If any deep cloning occurs, PanicOnClone will trigger
        let value = lock_kp.get(&root).await;

        // ✅ SUCCESS: No panic means no deep cloning!
        assert_eq!(value, Some(&123));
    }

    #[tokio::test]
    async fn test_async_lock_kp_structure() {
        use tokio::sync::Mutex;

        #[derive(Clone)]
        struct Root {
            data: Arc<Mutex<String>>,
        }

        let lock_kp = {
            let prev: KpType<Root, Arc<Mutex<String>>> =
                Kp::new(|r: &Root| Some(&r.data), |r: &mut Root| Some(&mut r.data));
            let next: KpType<String, String> =
                Kp::new(|s: &String| Some(s), |s: &mut String| Some(s));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        // Verify structure has three fields (prev, mid, next)
        let _ = &lock_kp.prev;
        let _ = &lock_kp.mid;
        let _ = &lock_kp.next;
    }

    #[tokio::test]
    async fn test_async_kp_then() {
        use tokio::sync::Mutex;

        #[derive(Clone)]
        struct Root {
            data: Arc<Mutex<Inner>>,
        }

        #[derive(Clone)]
        struct Inner {
            value: i32,
        }

        let root = Root {
            data: Arc::new(Mutex::new(Inner { value: 42 })),
        };

        // Create AsyncLockKp to Inner
        let async_kp = {
            let prev: KpType<Root, Arc<Mutex<Inner>>> =
                Kp::new(|r: &Root| Some(&r.data), |r: &mut Root| Some(&mut r.data));
            let next: KpType<Inner, Inner> = Kp::new(|i: &Inner| Some(i), |i: &mut Inner| Some(i));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        // Chain with regular Kp to get value field
        let value_kp: KpType<Inner, i32> = Kp::new(
            |i: &Inner| Some(&i.value),
            |i: &mut Inner| Some(&mut i.value),
        );

        let chained = async_kp.then(value_kp);
        let result = chained.get(&root).await;
        assert_eq!(result, Some(&42));
    }

    #[tokio::test]
    async fn test_async_kp_later_then() {
        use tokio::sync::Mutex;

        #[derive(Clone)]
        struct Root {
            lock1: Arc<Mutex<Container>>,
        }

        #[derive(Clone)]
        struct Container {
            lock2: Arc<Mutex<i32>>,
        }

        let root = Root {
            lock1: Arc::new(Mutex::new(Container {
                lock2: Arc::new(Mutex::new(999)),
            })),
        };

        // First AsyncLockKp: Root -> Container
        let async_kp1 = {
            let prev: KpType<Root, Arc<Mutex<Container>>> =
                Kp::new(|r: &Root| Some(&r.lock1), |r: &mut Root| Some(&mut r.lock1));
            let next: KpType<Container, Container> =
                Kp::new(|c: &Container| Some(c), |c: &mut Container| Some(c));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        // Second AsyncLockKp: Container -> i32
        let async_kp2 = {
            let prev: KpType<Container, Arc<Mutex<i32>>> = Kp::new(
                |c: &Container| Some(&c.lock2),
                |c: &mut Container| Some(&mut c.lock2),
            );
            let next: KpType<i32, i32> = Kp::new(|n: &i32| Some(n), |n: &mut i32| Some(n));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        // Chain with then_async; get with root
        let chained = async_kp1.then_async(async_kp2);
        let result = chained.get(&root).await;
        assert_eq!(result, Some(&999));
    }

    #[tokio::test]
    async fn test_async_kp_then_async_three_levels() {
        use tokio::sync::Mutex;

        #[derive(Clone)]
        struct Root {
            a: Arc<Mutex<Level1>>,
        }
        #[derive(Clone)]
        struct Level1 {
            b: Arc<Mutex<Level2>>,
        }
        #[derive(Clone)]
        struct Level2 {
            c: Arc<Mutex<i32>>,
        }

        let root = Root {
            a: Arc::new(Mutex::new(Level1 {
                b: Arc::new(Mutex::new(Level2 {
                    c: Arc::new(Mutex::new(42)),
                })),
            })),
        };

        let kp1 = {
            let prev: KpType<Root, Arc<Mutex<Level1>>> =
                Kp::new(|r: &Root| Some(&r.a), |r: &mut Root| Some(&mut r.a));
            let next: KpType<Level1, Level1> =
                Kp::new(|l: &Level1| Some(l), |l: &mut Level1| Some(l));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };
        let kp2 = {
            let prev: KpType<Level1, Arc<Mutex<Level2>>> =
                Kp::new(|l: &Level1| Some(&l.b), |l: &mut Level1| Some(&mut l.b));
            let next: KpType<Level2, Level2> =
                Kp::new(|l: &Level2| Some(l), |l: &mut Level2| Some(l));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };
        let kp3 = {
            let prev: KpType<Level2, Arc<Mutex<i32>>> =
                Kp::new(|l: &Level2| Some(&l.c), |l: &mut Level2| Some(&mut l.c));
            let next: KpType<i32, i32> = Kp::new(|n: &i32| Some(n), |n: &mut i32| Some(n));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        let chained = kp1.then_async(kp2).then_async(kp3);
        let result = chained.get(&root).await;
        assert_eq!(result, Some(&42));
    }
}
