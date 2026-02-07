# Keypath Interoperability: AsyncLockKp ⇒ LockKp ⇒ Kp

## Overview

The keypath library now supports seamless interoperability between three keypath types, allowing you to chain through different lock levels and regular field access in a single navigation path:

- **`Kp`**: Regular field access (no locks)
- **`LockKp`**: Synchronous locks (`Arc<Mutex>`, `Arc<RwLock>`, `Rc<RefCell>`)
- **`AsyncLockKp`**: Asynchronous locks (`Arc<tokio::sync::Mutex>`, `Arc<tokio::sync::RwLock>`)

## Hierarchy

```
AsyncLockKp  (top level - async locks)
    ↓
LockKp       (middle level - sync locks)
    ↓
Kp           (bottom level - regular fields)
```

## Interoperability Methods

### 1. AsyncLockKp Methods

#### `then()` - Chain with `Kp`

Navigate through an async lock, then continue with regular field access.

```rust
pub async fn then<'a, V2, Value2, MutValue2, G3, S3>(
    &'a self,
    next_kp: &'a crate::Kp<V, V2, Value, Value2, MutValue, MutValue2, G3, S3>,
    root: Root,
) -> Option<Value2>
```

**Example:**
```rust
use tokio::sync::Mutex;

struct Root {
    data: Arc<Mutex<Inner>>,
}

struct Inner {
    value: i32,
}

// Create AsyncLockKp to navigate to Inner
let async_kp = AsyncLockKp::new(
    Kp::new(|r: &Root| Some(&r.data), ...),  // prev: Root -> Arc<Mutex<Inner>>
    TokioMutexAccess::new(),                  // mid: unlock Inner
    Kp::new(|i: &Inner| Some(i), ...),       // next: Inner -> Inner
);

// Create Kp to navigate to value field
let value_kp = Kp::new(
    |i: &Inner| Some(&i.value),
    |i: &mut Inner| Some(&mut i.value),
);

// Chain: async lock -> field access
let result = async_kp.then(&value_kp, &root).await;
// result == Some(&42)
```

#### `later_then()` - Compose with another `AsyncLockKp`

Navigate through multiple nested async locks.

**Note**: Due to Rust's closure `Clone` constraints, this method takes the components of the second `AsyncLockKp` (prev, mid, next) separately rather than the struct itself.

```rust
pub async fn later_then<'a, Lock2, Mid2, V2, ...>(
    &'a self,
    other_prev: crate::Kp<V, Lock2, ...>,
    other_mid: L2,
    other_next: crate::Kp<Mid2, V2, ...>,
    root: Root,
) -> Option<Value2>
```

**Example:**
```rust
use tokio::sync::Mutex;

struct Root {
    lock1: Arc<Mutex<Container>>,
}

struct Container {
    lock2: Arc<Mutex<i32>>,
}

// First AsyncLockKp: Root -> Container
let async_kp1 = AsyncLockKp::new(
    Kp::new(|r: &Root| Some(&r.lock1), ...),
    TokioMutexAccess::new(),
    Kp::new(|c: &Container| Some(c), ...),
);

// Second AsyncLockKp: Container -> i32
let async_kp2 = AsyncLockKp::new(
    Kp::new(|c: &Container| Some(&c.lock2), ...),
    TokioMutexAccess::new(),
    Kp::new(|n: &i32| Some(n), ...),
);

// Compose: async lock1 -> async lock2
// Pass components separately to avoid Clone bounds on closures
let result = async_kp1.later_then(
    async_kp2.prev,
    async_kp2.mid,
    async_kp2.next,
    &root
).await;
// result == Some(&999)
```

#### `then_lock_kp_get()` - Chain with `LockKp`

Navigate through an async lock, then through a sync lock.

```rust
pub async fn then_lock_kp_get<'a, Lock2, ...>(
    &'a self,
    lock_kp: &'a crate::lock::LockKp<...>,
    root: Root,
) -> Option<Value2>
```

**Example:**
```rust
use tokio::sync::Mutex as TokioMutex;
use std::sync::{Arc as StdArc, Mutex as StdMutex};

struct Root {
    async_lock: Arc<TokioMutex<Container>>,
}

struct Container {
    sync_lock: StdArc<StdMutex<i32>>,
}

let async_kp = AsyncLockKp::new(...); // Root -> Container
let lock_kp = LockKp::new(...);       // Container -> i32

// Chain: async lock -> sync lock
let result = async_kp.then_lock_kp_get(&lock_kp, &root).await;
```

#### `compose_async_get()` - Chain with another `AsyncLockKp`

Navigate through multiple async locks.

```rust
pub async fn compose_async_get<'a, Lock2, ...>(
    &'a self,
    other: &'a AsyncLockKp<...>,
    root: Root,
) -> Option<Value2>
```

**Example:**
```rust
struct Root {
    lock1: Arc<tokio::sync::Mutex<Container>>,
}

struct Container {
    lock2: Arc<tokio::sync::Mutex<i32>>,
}

let async_kp1 = AsyncLockKp::new(...); // Root -> Container
let async_kp2 = AsyncLockKp::new(...); // Container -> i32

// Chain: async lock1 -> async lock2
let result = async_kp1.compose_async_get(&async_kp2, &root).await;
```

### 2. LockKp Methods

#### `then_async_kp_get()` - Chain with `AsyncLockKp`

Navigate through a sync lock, then through an async lock.

```rust
#[cfg(feature = "tokio")]
pub async fn then_async_kp_get<'a, Lock2, ...>(
    &'a self,
    async_kp: &'a crate::async_lock::AsyncLockKp<...>,
    root: Root,
) -> Option<Value2>
```

**Example:**
```rust
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

struct Root {
    sync_lock: Arc<Mutex<Container>>,
}

struct Container {
    async_lock: Arc<TokioMutex<String>>,
}

let lock_kp = LockKp::new(...);   // Root -> Container
let async_kp = AsyncLockKp::new(...); // Container -> String

// Chain: sync lock -> async lock
let result = lock_kp.then_async_kp_get(&async_kp, &root).await;
```

### 3. Kp Methods

#### `then_lock_kp_get()` - Chain with `LockKp`

Navigate to a field, then through a sync lock.

```rust
pub fn then_lock_kp_get<Lock, Mid, V2, ...>(
    &self,
    lock_kp: &crate::lock::LockKp<...>,
    root: Root,
) -> Option<Value2>
```

**Example:**
```rust
struct Root {
    container: Container,
}

struct Container {
    sync_lock: Arc<Mutex<i32>>,
}

let regular_kp: KpType<Root, Container> = Kp::new(
    |r: &Root| Some(&r.container),
    |r: &mut Root| Some(&mut r.container),
);

let lock_kp = LockKp::new(...); // Container -> i32

// Chain: regular field -> sync lock
let result = regular_kp.then_lock_kp_get(&lock_kp, &root);
```

#### `then_async_kp_get()` - Chain with `AsyncLockKp`

Navigate to a field, then through an async lock.

```rust
#[cfg(feature = "tokio")]
pub async fn then_async_kp_get<Lock, Mid, V2, ...>(
    &self,
    async_kp: &crate::async_lock::AsyncLockKp<...>,
    root: Root,
) -> Option<Value2>
```

**Example:**
```rust
struct Root {
    container: Container,
}

struct Container {
    async_lock: Arc<tokio::sync::Mutex<String>>,
}

let regular_kp: KpType<Root, Container> = Kp::new(
    |r: &Root| Some(&r.container),
    |r: &mut Root| Some(&mut r.container),
);

let async_kp = AsyncLockKp::new(...); // Container -> String

// Chain: regular field -> async lock
let result = regular_kp.then_async_kp_get(&async_kp, &root).await;
```

## Complete Example: Chaining All Three Levels

```rust
use std::sync::{Arc as StdArc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;

struct Root {
    level1: Level1,  // Regular field
}

struct Level1 {
    sync_lock: StdArc<StdMutex<Level2>>,  // Sync lock
}

struct Level2 {
    async_lock: Arc<TokioMutex<i32>>,  // Async lock
}

#[tokio::main]
async fn main() {
    let root = Root {
        level1: Level1 {
            sync_lock: StdArc::new(StdMutex::new(Level2 {
                async_lock: Arc::new(TokioMutex::new(888)),
            })),
        },
    };

    // Step 1: Regular Kp to navigate to Level1
    let kp: KpType<Root, Level1> = Kp::new(
        |r: &Root| Some(&r.level1),
        |r: &mut Root| Some(&mut r.level1),
    );

    // Step 2: LockKp through sync lock to Level2
    let lock_kp = LockKp::new(
        Kp::new(|l1: &Level1| Some(&l1.sync_lock), ...),
        ArcMutexAccess::new(),
        Kp::new(|l2: &Level2| Some(l2), ...),
    );

    // Step 3: AsyncLockKp through async lock to i32
    let async_kp = AsyncLockKp::new(
        Kp::new(|l2: &Level2| Some(&l2.async_lock), ...),
        TokioMutexAccess::new(),
        Kp::new(|n: &i32| Some(n), ...),
    );

    // Chain all three: Kp -> LockKp -> AsyncLockKp
    let level1 = kp.get(&root).unwrap();
    let level2 = lock_kp.get(level1).unwrap();
    let result = async_kp.get_async(level2).await;
    
    assert_eq!(result, Some(&888));
}
```

## Method Naming Convention

| From | To | Method Name | Description |
|------|-----|-------------|-------------|
| AsyncLockKp | Kp | `then()` | Chain to regular field |
| AsyncLockKp | AsyncLockKp | `later_then()` | Compose with another async lock |
| AsyncLockKp | LockKp | `then_lock_kp_get()` | Chain to sync lock |
| AsyncLockKp | AsyncLockKp | `compose_async_get()` | Chain to another async lock (alternative) |
| LockKp | AsyncLockKp | `then_async_kp_get()` | Chain to async lock |
| Kp | LockKp | `then_lock_kp_get()` | Chain to sync lock |
| Kp | AsyncLockKp | `then_async_kp_get()` | Chain to async lock |

## Design Principles

### 1. **Explicit Chaining**
Each chain operation requires explicitly calling a method with the next keypath. This makes the navigation path clear and the lock acquisitions visible.

### 2. **Async Awareness**
- Methods that cross into async territory return `async fn` and require `.await`
- Methods staying in sync world remain synchronous

### 3. **Lifetime Management**
All chaining methods use lifetime parameters (`'a`) to ensure the keypaths and their closures live long enough for the operation.

### 4. **No Automatic Composition**
Unlike the `then()` and `compose()` methods on individual keypath types that return new composed keypaths, the interoperability methods directly execute the navigation and return the value. This avoids complex lifetime and `Clone` bound issues with closures.

## Performance Characteristics

### Shallow Cloning Guarantee
All interoperability operations maintain the shallow cloning guarantee:
- `Arc` clones only increment reference counts
- Lock accessor structs (`TokioMutexAccess`, etc.) contain only `PhantomData` (zero-cost)
- Function pointers are cheap to copy
- **No deep data cloning occurs**

### Lock Acquisition
- Sync locks block the current thread
- Async locks yield to the executor (non-blocking)
- Chaining multiple locks acquires them sequentially (as needed by the navigation path)

## Tests

The interoperability is tested with:

1. **`test_async_kp_then`** (async_lock.rs): AsyncLockKp -> Kp
2. **`test_kp_then_lock_kp`** (lib.rs): Kp -> LockKp
3. **`test_kp_then_async_kp`** (lib.rs): Kp -> AsyncLockKp
4. **`test_full_chain_kp_to_lock_to_async`** (lib.rs): Kp -> LockKp -> AsyncLockKp (full chain)
5. **`test_lock_kp_then_async_kp`** (lock.rs): LockKp -> AsyncLockKp

Total: **82 tests passing** (including all interoperability tests)

## Limitations

### No Automatic Type Inference
Due to Rust's type system, you often need to provide type annotations when creating the intermediate keypaths:

```rust
let value_kp: KpType<Inner, i32> = Kp::new(
    |i: &Inner| Some(&i.value),
    |i: &mut Inner| Some(&mut i.value),
);
```

### Async Propagation
Once you enter async territory (via `AsyncLockKp` or `then_async_kp_get()`), all subsequent operations must be `async` and require `.await`.

### Clone Bounds
The `AsyncLockKp` struct requires `Clone` bounds on its function types (`G1`, `S1`, etc.), which means closures that capture non-`Clone` data cannot be used. Use function pointers or ensure captured data is `Clone`.

## Future Enhancements

Potential improvements for interoperability:

1. **Builder Pattern**: Fluent API for chaining
   ```rust
   ChainBuilder::new()
       .with_kp(regular_kp)
       .with_lock_kp(lock_kp)
       .with_async_kp(async_kp)
       .execute(&root)
       .await
   ```

2. **Macro Support**: Generate interoperability chains from a declarative syntax
   ```rust
   chain!(root => field => sync_lock => async_lock => value)
   ```

3. **Composed Return Types**: Return new composed keypaths instead of directly executing
   (Currently challenging due to closure lifetime and `Clone` constraints)

## Conclusion

The interoperability feature enables seamless navigation through complex data structures with mixed synchronization strategies, maintaining type safety, shallow cloning guarantees, and explicit lock acquisition points throughout the navigation path.
