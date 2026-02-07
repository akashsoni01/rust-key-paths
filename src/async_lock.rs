//! # Async Lock Keypath Module
//!
//! This module provides `AsyncLockKp` for safely navigating through async locked/synchronized data structures.
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

use std::sync::Arc;
use crate::Kp;
use async_trait::async_trait;

// Re-export tokio sync types for convenience
#[cfg(feature = "tokio")]
pub use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};

/// Async trait for types that can provide async lock/unlock behavior
/// Converts from a Lock type to Inner or InnerMut value asynchronously
#[async_trait]
pub trait AsyncLockAccess<Lock, Inner>: Send + Sync {
    /// Get immutable access to the inner value asynchronously
    async fn lock_read(&self, lock: &Lock) -> Option<Inner>;
    
    /// Get mutable access to the inner value asynchronously
    async fn lock_write(&self, lock: &mut Lock) -> Option<Inner>;
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
/// - `mid` field implements `AsyncLockAccess` trait - typically just `PhantomData` (zero-cost clone)
/// - When `Lock: Clone` (e.g., `Arc<tokio::sync::Mutex<T>>`), cloning is just incrementing reference count
/// - NO deep data cloning occurs - all clones are pointer/reference increments
#[derive(Clone)] // SHALLOW: Clones function pointers and PhantomData only
pub struct AsyncLockKp<R, Lock, Mid, V, Root, LockValue, MidValue, Value, MutRoot, MutLock, MutMid, MutValue, G1, S1, L, G2, S2>
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
    L: AsyncLockAccess<Lock, MidValue> + AsyncLockAccess<Lock, MutMid> + Clone,
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

impl<R, Lock, Mid, V, Root, LockValue, MidValue, Value, MutRoot, MutLock, MutMid, MutValue, G1, S1, L, G2, S2>
    AsyncLockKp<R, Lock, Mid, V, Root, LockValue, MidValue, Value, MutRoot, MutLock, MutMid, MutValue, G1, S1, L, G2, S2>
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
    L: AsyncLockAccess<Lock, MidValue> + AsyncLockAccess<Lock, MutMid> + Clone,
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

    /// Async get the value through the lock
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
    pub async fn get_async(&self, root: Root) -> Option<Value>
    where
        Lock: Clone,
    {
        // SHALLOW CLONE: For Arc<Mutex<T>>, only increments Arc refcount
        // The actual data T is NOT cloned
        let lock_value = (self.prev.get)(root)?;
        let lock: &Lock = lock_value.borrow();
        let lock_clone = lock.clone();  // SHALLOW: Arc refcount++
        
        // Async lock and get the mid value
        let mid_value = self.mid.lock_read(&lock_clone).await?;
        
        // Navigate from mid to final value
        (self.next.get)(mid_value)
    }

    /// Async get mutable access to the value through the lock
    pub async fn get_mut_async(&self, root: MutRoot) -> Option<MutValue>
    where
        Lock: Clone,
    {
        // SHALLOW CLONE: For Arc<Mutex<T>>, only increments Arc refcount
        let mut lock_value = (self.prev.set)(root)?;
        let lock: &mut Lock = lock_value.borrow_mut();
        let mut lock_clone = lock.clone();  // SHALLOW: Arc refcount++
        
        // Async lock and get the mid value
        let mid_value = self.mid.lock_write(&mut lock_clone).await?;
        
        // Navigate from mid to final value
        (self.next.set)(mid_value)
    }

    /// Async set the value through the lock using an updater function
    pub async fn set_async<F>(&self, root: Root, updater: F) -> Result<(), String>
    where
        Lock: Clone,
        F: FnOnce(&mut V),
    {
        // SHALLOW CLONE: For Arc<Mutex<T>>, only increments Arc refcount
        let lock_value = (self.prev.get)(root)
            .ok_or("Failed to get lock from root")?;
        let lock: &Lock = lock_value.borrow();
        let lock_clone = lock.clone();  // SHALLOW: Arc refcount++
        
        // Async lock and get the mid value
        let mut mid_value = self.mid.lock_read(&lock_clone).await
            .ok_or("Failed to lock")?;
        
        // Get the final value
        let mut mut_value = (self.next.set)(mid_value)
            .ok_or("Failed to navigate to value")?;
        let v: &mut V = mut_value.borrow_mut();
        
        // Apply the updater
        updater(v);
        
        Ok(())
    }

    // ========================================================================
    // Interoperability Methods: AsyncLockKp => Kp
    // ========================================================================

    /// Chain this AsyncLockKp with a regular Kp
    /// 
    /// After navigating through the async lock to get Mid value,
    /// continue navigating with a regular keypath.
    /// 
    /// # Example
    /// ```
    /// // Root -> Arc<tokio::Mutex<Inner>> -> Inner.field
    /// let async_kp = AsyncLockKp::new(root_to_lock, TokioMutexAccess::new(), lock_to_inner);
    /// let field_kp = Kp::new(|inner: &Inner| Some(&inner.field), ...);
    /// let result = async_kp.then(&field_kp, &root).await;
    /// ```
    pub async fn then<'a, V2, Value2, MutValue2, G3, S3>(
        &'a self,
        next_kp: &'a crate::Kp<V, V2, Value, Value2, MutValue, MutValue2, G3, S3>,
        root: Root,
    ) -> Option<Value2>
    where
        Lock: Clone,
        V: 'static,
        V2: 'static,
        Value: std::borrow::Borrow<V>,
        Value2: std::borrow::Borrow<V2>,
        MutValue: std::borrow::BorrowMut<V>,
        MutValue2: std::borrow::BorrowMut<V2>,
        G3: Fn(Value) -> Option<Value2> + 'a,
        S3: Fn(MutValue) -> Option<MutValue2> + 'a,
        G1: 'a,
        S1: 'a,
        L: 'a,
        G2: 'a,
        S2: 'a,
    {
        // First, navigate through async lock to get mid value
        let mid_value = {
            let lock_value = (self.prev.get)(root)?;
            let lock: &Lock = lock_value.borrow();
            let lock_clone = lock.clone();
            self.mid.lock_read(&lock_clone).await?
        };
        
        // Navigate from mid to V
        let value = (self.next.get)(mid_value)?;
        
        // Finally, use the next Kp to navigate further
        (next_kp.get)(value)
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
#[derive(Clone)]  // ZERO-COST: Only clones PhantomData (zero-sized type)
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
impl<'a, T: 'static + Send + Sync> AsyncLockAccess<Arc<tokio::sync::Mutex<T>>, &'a T> for TokioMutexAccess<T> {
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
impl<'a, T: 'static + Send + Sync> AsyncLockAccess<Arc<tokio::sync::Mutex<T>>, &'a mut T> for TokioMutexAccess<T> {
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
#[derive(Clone)]  // ZERO-COST: Only clones PhantomData (zero-sized type)
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

// Implementation for immutable access (read lock)
#[cfg(feature = "tokio")]
#[async_trait]
impl<'a, T: 'static + Send + Sync> AsyncLockAccess<Arc<tokio::sync::RwLock<T>>, &'a T> for TokioRwLockAccess<T> {
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
impl<'a, T: 'static + Send + Sync> AsyncLockAccess<Arc<tokio::sync::RwLock<T>>, &'a mut T> for TokioRwLockAccess<T> {
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
            let prev: KpType<Root, Arc<Mutex<String>>> = Kp::new(
                |r: &Root| Some(&r.data),
                |r: &mut Root| Some(&mut r.data),
            );
            let next: KpType<String, String> = Kp::new(
                |s: &String| Some(s),
                |s: &mut String| Some(s),
            );
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        // Test async get
        let value = lock_kp.get_async(&root).await;
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
            let prev: KpType<Root, Arc<RwLock<Vec<i32>>>> = Kp::new(
                |r: &Root| Some(&r.data),
                |r: &mut Root| Some(&mut r.data),
            );
            let next: KpType<Vec<i32>, Vec<i32>> = Kp::new(
                |v: &Vec<i32>| Some(v),
                |v: &mut Vec<i32>| Some(v),
            );
            AsyncLockKp::new(prev, TokioRwLockAccess::new(), next)
        };

        // Test async get with RwLock (read lock)
        let value = lock_kp.get_async(&root).await;
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
            let prev: KpType<Root, Arc<RwLock<i32>>> = Kp::new(
                |r: &Root| Some(&r.data),
                |r: &mut Root| Some(&mut r.data),
            );
            let next: KpType<i32, i32> = Kp::new(
                |n: &i32| Some(n),
                |n: &mut i32| Some(n),
            );
            AsyncLockKp::new(prev, TokioRwLockAccess::new(), next)
        };

        // Spawn multiple concurrent async reads
        let mut handles = vec![];
        for _ in 0..10 {
            let root_clone = root.clone();
            
            // Re-create lock_kp for each task since we can't clone it easily
            let lock_kp_for_task = {
                let prev: KpType<Root, Arc<RwLock<i32>>> = Kp::new(
                    |r: &Root| Some(&r.data),
                    |r: &mut Root| Some(&mut r.data),
                );
                let next: KpType<i32, i32> = Kp::new(
                    |n: &i32| Some(n),
                    |n: &mut i32| Some(n),
                );
                AsyncLockKp::new(prev, TokioRwLockAccess::new(), next)
            };
            
            let handle = tokio::spawn(async move {
                lock_kp_for_task.get_async(&root_clone).await
            });
            handles.push(handle);
        }

        // All reads should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert_eq!(result, Some(&42));
        }

        // Test the original lock_kp as well
        let value = lock_kp.get_async(&root).await;
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
        let value = lock_kp.get_async(&root).await;
        
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
            let prev: KpType<Root, Arc<Mutex<String>>> = Kp::new(
                |r: &Root| Some(&r.data),
                |r: &mut Root| Some(&mut r.data),
            );
            let next: KpType<String, String> = Kp::new(
                |s: &String| Some(s),
                |s: &mut String| Some(s),
            );
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
            let prev: KpType<Root, Arc<Mutex<Inner>>> = Kp::new(
                |r: &Root| Some(&r.data),
                |r: &mut Root| Some(&mut r.data),
            );
            let next: KpType<Inner, Inner> = Kp::new(
                |i: &Inner| Some(i),
                |i: &mut Inner| Some(i),
            );
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        // Chain with regular Kp to get value field
        let value_kp: KpType<Inner, i32> = Kp::new(
            |i: &Inner| Some(&i.value),
            |i: &mut Inner| Some(&mut i.value),
        );

        let result = async_kp.then(&value_kp, &root).await;
        assert_eq!(result, Some(&42));
    }
}
