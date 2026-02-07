# Async Lock Keypath Implementation

## Overview

The `async_lock` module provides `AsyncLockKp` for type-safe, composable navigation through async locked data structures like `Arc<tokio::sync::Mutex<T>>` and `Arc<tokio::sync::RwLock<T>>`.

## Key Components

### 1. `AsyncLockAccess` Trait

```rust
#[async_trait]
pub trait AsyncLockAccess<Lock, Inner>: Send + Sync {
    async fn lock_read(&self, lock: &Lock) -> Option<Inner>;
    async fn lock_write(&self, lock: &mut Lock) -> Option<Inner>;
}
```

Unlike the synchronous `LockAccess` trait, this trait uses `async fn` methods, allowing for asynchronous lock acquisition.

### 2. `AsyncLockKp` Struct

```rust
pub struct AsyncLockKp<R, Lock, Mid, V, Root, LockValue, MidValue, Value, MutRoot, MutLock, MutMid, MutValue, G1, S1, L, G2, S2>
```

Structure:
- `prev`: Keypath from Root to Lock container (e.g., `Arc<tokio::sync::Mutex<Mid>>`)
- `mid`: Async lock access handler (e.g., `TokioMutexAccess<T>`)
- `next`: Keypath from Inner value to final Value

### 3. Lock Implementations

#### `TokioMutexAccess<T>`

Provides async access to `Arc<tokio::sync::Mutex<T>>`:
- Uses `tokio::sync::Mutex::lock().await` for both read and write access
- Mutex provides exclusive access for all operations
- Good for simple async synchronization scenarios

#### `TokioRwLockAccess<T>`

Provides async access to `Arc<tokio::sync::RwLock<T>>`:
- Uses `tokio::sync::RwLock::read().await` for immutable access
- Uses `tokio::sync::RwLock::write().await` for mutable access
- Allows multiple concurrent readers
- Good for read-heavy async workloads

## Comparison: Async vs Sync Locks

| Feature | `LockKp` (Sync) | `AsyncLockKp` (Async) |
|---------|-----------------|----------------------|
| **Lock Acquisition** | Blocking | Asynchronous (`.await`) |
| **Trait Methods** | `fn lock_read/write` | `async fn lock_read/write` |
| **Supported Locks** | `Arc<Mutex>`, `Arc<RwLock>`, `Rc<RefCell>` | `Arc<tokio::sync::Mutex>`, `Arc<tokio::sync::RwLock>` |
| **Runtime** | Any (sync) | Requires async runtime (tokio) |
| **Use Case** | Thread-safe sync code | Async/await concurrent code |
| **Performance** | Lower overhead for sync code | Better for I/O-bound async workloads |

## API Methods

### `new(prev, mid, next)`

Create a new `AsyncLockKp` from three components.

### `async fn get_async(&self, root: Root) -> Option<Value>`

Asynchronously get the value through the lock:
1. Navigate to the Lock using `prev`
2. Asynchronously lock and get the Inner value using `mid`
3. Navigate from Inner to final Value using `next`

### `async fn get_mut_async(&self, root: MutRoot) -> Option<MutValue>`

Asynchronously get mutable access to the value through the lock.

### `async fn set_async<F>(&self, root: Root, updater: F) -> Result<(), String>`

Asynchronously set the value through the lock using an updater function.

## Shallow Cloning Guarantee

**IMPORTANT**: All cloning operations in `async_lock` are SHALLOW:

1. **`AsyncLockKp` derives `Clone`**: Only clones function pointers and `PhantomData`
2. **`Lock: Clone` bound** (e.g., `Arc<tokio::sync::Mutex<T>>`):
   - For `Arc<T>`: Only increments the atomic reference count
   - The actual data `T` inside is **NEVER** cloned
3. **`L: Clone` bound** (e.g., `TokioMutexAccess<T>`):
   - Only clones `PhantomData<T>` (zero-sized, zero-cost)

This is proven by the `test_async_lock_kp_panic_on_clone_proof` test which uses a struct that panics if deeply cloned.

## Example Usage

### Basic Tokio Mutex

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use rust_key_paths::{Kp, AsyncLockKp, AsyncTokioMutexAccess, KpType};

#[derive(Clone)]
struct Root {
    data: Arc<Mutex<String>>,
}

#[tokio::main]
async fn main() {
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
        AsyncLockKp::new(prev, AsyncTokioMutexAccess::new(), next)
    };

    // Async get
    let value = lock_kp.get_async(&root).await;
    assert_eq!(value, Some(&"hello".to_string()));
}
```

### Concurrent Reads with RwLock

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use rust_key_paths::{Kp, AsyncLockKp, AsyncTokioRwLockAccess, KpType};

#[derive(Clone)]
struct Root {
    data: Arc<RwLock<i32>>,
}

#[tokio::main]
async fn main() {
    let root = Root {
        data: Arc::new(RwLock::new(42)),
    };

    // Create AsyncLockKp
    let make_lock_kp = || {
        let prev: KpType<Root, Arc<RwLock<i32>>> = Kp::new(
            |r: &Root| Some(&r.data),
            |r: &mut Root| Some(&mut r.data),
        );
        let next: KpType<i32, i32> = Kp::new(
            |n: &i32| Some(n),
            |n: &mut i32| Some(n),
        );
        AsyncLockKp::new(prev, AsyncTokioRwLockAccess::new(), next)
    };

    // Spawn multiple concurrent reads
    let mut handles = vec![];
    for _ in 0..10 {
        let root_clone = root.clone();
        let lock_kp = make_lock_kp();
        
        let handle = tokio::spawn(async move {
            lock_kp.get_async(&root_clone).await
        });
        handles.push(handle);
    }

    // All reads can happen concurrently with RwLock
    for handle in handles {
        let result = handle.await.unwrap();
        assert_eq!(result, Some(&42));
    }
}
```

## Tests

The module includes 5 comprehensive tests:

1. **`test_async_lock_kp_tokio_mutex_basic`**: Basic Tokio Mutex functionality
2. **`test_async_lock_kp_tokio_rwlock_basic`**: Basic Tokio RwLock functionality
3. **`test_async_lock_kp_concurrent_reads`**: Multiple concurrent async reads with RwLock
4. **`test_async_lock_kp_panic_on_clone_proof`**: Proves shallow cloning with panic-on-clone struct
5. **`test_async_lock_kp_structure`**: Verifies the three-field structure (prev, mid, next)

## Feature Flag

The async lock functionality requires the `tokio` feature flag:

```toml
[dependencies]
rust-key-paths = { version = "1.27.0", features = ["tokio"] }
```

## Thread Safety

- `AsyncLockAccess` requires `Send + Sync` bounds
- All implementations work across async task boundaries
- `Arc<tokio::sync::Mutex<T>>` and `Arc<tokio::sync::RwLock<T>>` are both `Send + Sync` when `T: Send + Sync`

## Performance Characteristics

### Shallow Cloning
- Cloning `AsyncLockKp`: O(1) - copies function pointers only
- Cloning `Arc<tokio::sync::Mutex<T>>`: O(1) - atomic increment
- Cloning `TokioMutexAccess<T>`: O(1) - zero-cost (PhantomData)

### Lock Acquisition
- `tokio::sync::Mutex`: Uses OS-level parking for fairness
- `tokio::sync::RwLock`: Allows multiple concurrent readers
- Both are async-aware and won't block the executor

## Unsafe Code

The `AsyncLockAccess` implementations use `unsafe` to extend the lifetime of references obtained from async lock guards:

```rust
async fn lock_read(&self, lock: &Arc<tokio::sync::Mutex<T>>) -> Option<&'a T> {
    let guard = lock.lock().await;
    let ptr = &*guard as *const T;
    unsafe { Some(&*ptr) }
}
```

**Safety Rationale**: 
- The `Arc` ensures the data outlives the reference
- The lock guard is held during the critical section
- This pattern is necessary to work around lifetime constraints with async guards

## Limitations

### No Clone on AsyncLockKp with Closures

Unlike function pointers, closures in Rust don't automatically implement `Clone`. Therefore, `AsyncLockKp` can only be cloned if all its function types are `Clone` (which is true for function pointers but not arbitrary closures).

**Workaround**: Re-create the `AsyncLockKp` instead of cloning it:

```rust
let make_lock_kp = || {
    let prev = Kp::new(/* ... */);
    let next = Kp::new(/* ... */);
    AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
};

// Use in multiple tasks
for _ in 0..10 {
    let lock_kp = make_lock_kp();
    tokio::spawn(async move {
        lock_kp.get_async(&root).await
    });
}
```

### No Chaining Methods (Yet)

The `then()` and `compose()` methods present in `LockKp` are not yet implemented for `AsyncLockKp` due to complexity with async closures and the `impl Trait` return type with `Clone` bounds.

**Current Status**: Basic functionality (new, get_async, get_mut_async, set_async) is fully working and tested.

## Future Work

- Implement `then()` for chaining with regular `Kp`
- Implement `compose()` for multi-level async lock chaining
- Support for other async runtimes (async-std, smol)
- Generic async lock trait that works across runtimes

## Migration from Sync to Async

If you have existing code using `LockKp` with `Arc<Mutex<T>>` or `Arc<RwLock<T>>` and want to migrate to async:

1. Replace `Arc<std::sync::Mutex<T>>` with `Arc<tokio::sync::Mutex<T>>`
2. Replace `Arc<std::sync::RwLock<T>>` with `Arc<tokio::sync::RwLock<T>>`
3. Replace `LockKp` with `AsyncLockKp`
4. Replace `ArcMutexAccess` with `TokioMutexAccess` (re-exported as `AsyncTokioMutexAccess`)
5. Replace `ArcRwLockAccess` with `TokioRwLockAccess` (re-exported as `AsyncTokioRwLockAccess`)
6. Replace `.get()` with `.get_async().await`
7. Replace `.get_mut()` with `.get_mut_async().await`
8. Ensure your function is `async` and you're running on a Tokio runtime

## Conclusion

The `async_lock` module provides a type-safe, zero-cost abstraction for navigating through async locked data structures, maintaining the same shallow cloning guarantees and composability principles as the sync `lock` module, but adapted for the async/await paradigm.
