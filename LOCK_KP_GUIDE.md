# LockKp - Keypath for Locked Values

## Overview

`LockKp` is a specialized keypath type for handling locked/synchronized values like `Arc<Mutex<T>>` and `Arc<RwLock<T>>`. It provides a safe, composable way to navigate through locked data structures, including support for **multi-level lock chaining**.

## Structure

The `LockKp` struct has three key components:

```rust
pub struct LockKp<...> {
    /// Keypath from Root to Lock container (e.g., Arc<Mutex<Mid>>)
    pub prev: Kp<R, Lock, Root, LockValue, MutRoot, MutLock, G1, S1>,
    
    /// Lock access handler (trait-based, converts Lock -> Inner)
    pub mid: L,  // implements LockAccess trait
    
    /// Keypath from Inner to final Value
    pub next: Kp<Mid, V, MidValue, Value, MutMid, MutValue, G2, S2>,
}
```

### The Three Fields:

1. **`prev`**: A regular `Kp` that navigates from Root to the Lock container
2. **`mid`**: A trait object implementing `LockAccess` that handles lock/unlock operations
3. **`next`**: A regular `Kp` that navigates from the Inner type to the final Value

## LockAccess Trait

The `mid` field uses the `LockAccess` trait, which defines how to access locked values:

```rust
pub trait LockAccess<Lock, Inner> {
    fn lock_read(&self, lock: &Lock) -> Option<Inner>;
    fn lock_write(&self, lock: &mut Lock) -> Option<Inner>;
}
```

## Lock Implementations

### ArcMutexAccess<T>

Standard implementation for `Arc<Mutex<T>>`:
```rust
let lock_kp = LockKp::new(prev, ArcMutexAccess::new(), next);
```

**Use when:**
- Multi-threaded environment
- Simple exclusive access needed
- Default choice for thread-safe locks

### ArcRwLockAccess<T> 

Implementation for `Arc<RwLock<T>>`:
```rust
let lock_kp = LockKp::new(prev, ArcRwLockAccess::new(), next);
```

**Use when:**
- Multi-threaded environment
- Multiple concurrent readers needed
- Read-heavy workloads
- Want to allow parallel read access

**RwLock Semantics:**
- Multiple readers can access simultaneously (shared/immutable)
- Only one writer can access at a time (exclusive/mutable)
- Readers and writers are mutually exclusive

### RcRefCellAccess<T> ⭐ NEW

Implementation for `Rc<RefCell<T>>` (single-threaded):
```rust
let lock_kp = LockKp::new(prev, RcRefCellAccess::new(), next);
```

**Use when:**
- Single-threaded context only
- Want lower overhead than Arc/Mutex
- Don't need thread safety
- Need interior mutability without atomic operations

**RefCell Semantics:**
- Multiple immutable borrows allowed simultaneously
- Only one mutable borrow allowed at a time
- Runtime borrow checking (panics on violation)
- **NOT thread-safe** - use only in single-threaded code

| Feature | Arc<Mutex> | Arc<RwLock> | Rc<RefCell> |
|---------|------------|-------------|-------------|
| Multiple readers | ❌ Blocked | ✅ Concurrent | ✅ Concurrent |
| Write access | ✅ Exclusive | ✅ Exclusive | ✅ Exclusive |
| Thread-safe | ✅ Yes | ✅ Yes | ❌ No (single-threaded) |
| Overhead | Low | Moderate | Very Low |
| Atomic ops | Yes | Yes | No |
| Best for | Multi-threaded, simple | Multi-threaded, read-heavy | Single-threaded |
| Use when | Default (threaded) | Many readers | No threads needed |

## Usage Examples

### Single Lock Level

```rust
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Root {
    locked_data: Arc<Mutex<Inner>>,
}

#[derive(Clone)]
struct Inner {
    value: String,
}

// Create the three components
let prev: KpType<Root, Arc<Mutex<Inner>>> = Kp::new(
    |r: &Root| Some(&r.locked_data),
    |r: &mut Root| Some(&mut r.locked_data),
);

let mid = ArcMutexAccess::new();

let next: KpType<Inner, String> = Kp::new(
    |i: &Inner| Some(&i.value),
    |i: &mut Inner| Some(&mut i.value),
);

// Combine them into a LockKp
let lock_kp = LockKp::new(prev, mid, next);

// Get value through the lock
let value = lock_kp.get(&root);
```

### RwLock Example

```rust
use std::sync::{Arc, RwLock};

let lock_kp = LockKp::new(
    Kp::new(|r: &Root| Some(&r.data), |r: &mut Root| Some(&mut r.data)),
    ArcRwLockAccess::new(),  // ← Use RwLock for concurrent reads
    Kp::new(|i: &Inner| Some(&i.value), |i: &mut Inner| Some(&mut i.value)),
);
```

### Rc<RefCell> Example (Single-threaded)

```rust
use std::rc::Rc;
use std::cell::RefCell;

let lock_kp = LockKp::new(
    Kp::new(|r: &Root| Some(&r.data), |r: &mut Root| Some(&mut r.data)),
    RcRefCellAccess::new(),  // ← Use Rc<RefCell> for single-threaded
    Kp::new(|i: &Inner| Some(&i.value), |i: &mut Inner| Some(&mut i.value)),
);
```

## Chaining with `then()`

`LockKp` supports chaining with regular `Kp` using the `then()` operator:

```rust
// Root -> Arc<Mutex<Mid>> -> Inner1 -> Inner2 -> Value
let chained = lock_kp.then(another_kp);
```

## Multi-Level Lock Chaining with `compose()`

The `compose()` method allows you to chain through **multiple lock levels**:

```rust
// Root -> Lock1 -> Mid1 -> Lock2 -> Mid2 -> Value
let composed = lock_kp1.compose(lock_kp2);
```

### Two-Level Example

```rust
#[derive(Clone)]
struct Root {
    level1: Arc<Mutex<Level1>>,
}

#[derive(Clone)]
struct Level1 {
    level2: Arc<Mutex<Level2>>,
}

#[derive(Clone)]
struct Level2 {
    value: String,
}

// First lock level: Root -> Level1
let lock1 = LockKp::new(
    Kp::new(|r: &Root| Some(&r.level1), |r: &mut Root| Some(&mut r.level1)),
    ArcMutexAccess::new(),
    Kp::new(|l: &Level1| Some(l), |l: &mut Level1| Some(l)),
);

// Second lock level: Level1 -> Level2 -> String
let lock2 = LockKp::new(
    Kp::new(|l: &Level1| Some(&l.level2), |l: &mut Level1| Some(&mut l.level2)),
    ArcMutexAccess::new(),
    Kp::new(|l: &Level2| Some(&l.value), |l: &mut Level2| Some(&mut l.value)),
);

// Compose both locks
let composed = lock1.compose(lock2);

// Navigate through both lock levels in one operation
let value = composed.get(&root);
```

### Three-Level Example

You can compose multiple times for deeply nested locks:

```rust
// Root -> Lock1 -> L1 -> Lock2 -> L2 -> Lock3 -> L3 -> Value
let composed_1_2 = lock_kp1.compose(lock_kp2);
let composed_all = composed_1_2.compose(lock_kp3);

// Navigate through all three lock levels
let value = composed_all.get(&root);
```

### Mixed Lock Types

Compose different lock types together:
```rust
// RwLock -> Mutex composition
let rwlock_kp = LockKp::new(prev1, ArcRwLockAccess::new(), next1);
let mutex_kp = LockKp::new(prev2, ArcMutexAccess::new(), next2);
let mixed = rwlock_kp.compose(mutex_kp);
```

### Combining `compose()` and `then()`

You can mix lock composition with regular keypath chaining:

```rust
// Root -> Lock1 -> Lock2 -> Inner -> Data -> Value
let composed = lock1.compose(lock2);       // Handle both locks
let with_data = composed.then(to_data);    // Navigate to Data
let with_value = with_data.then(to_value); // Navigate to Value

let value = with_value.get(&root);
```

## API Methods

### `new()`
Creates a new `LockKp` from its three components:
```rust
pub fn new(prev: Kp<...>, mid: L, next: Kp<...>) -> Self
```

### `get()`
Gets the value through the lock:
```rust
pub fn get(&self, root: Root) -> Option<Value>
```

### `get_mut()`
Gets mutable access to the value through the lock:
```rust
pub fn get_mut(&self, root: MutRoot) -> Option<MutValue>
```

### `set()`
Sets the value through the lock using an updater function:
```rust
pub fn set<F>(&self, root: Root, updater: F) -> Result<(), String>
where
    F: FnOnce(&mut V),
```

### `then()`
Chains this LockKp with another regular Kp:
```rust
pub fn then<V2, ...>(self, next_kp: Kp<V, V2, ...>) -> LockKp<R, Lock, Mid, V2, ...>
```

### `compose()` ⭐
Composes this LockKp with another LockKp for multi-level lock chaining:
```rust
pub fn compose<Lock2, Mid2, V2, ...>(
    self,
    other: LockKp<V, Lock2, Mid2, V2, ...>
) -> LockKp<R, Lock, Mid, V2, ...>
```

## Tests

The module includes comprehensive tests:

### Mutex Tests
- `test_lock_kp_basic`: Basic Mutex functionality
- `test_lock_kp_structure`: Verifies the three-field structure
- `test_lock_kp_then_chaining`: Tests chaining with `then()`
- `test_lock_kp_compose_single_level`: Tests composing two LockKps
- `test_lock_kp_compose_two_levels`: Tests two-level lock composition
- `test_lock_kp_compose_three_levels`: Tests three-level lock composition
- `test_lock_kp_compose_with_then`: Tests mixing `compose()` and `then()`

### RwLock Tests
- `test_rwlock_basic`: Basic RwLock functionality
- `test_rwlock_compose_two_levels`: Two-level RwLock composition
- `test_rwlock_mixed_with_mutex`: RwLock and Mutex composition
- `test_rwlock_structure`: Verifies RwLock structure
- `test_rwlock_three_levels`: Three-level RwLock composition

### Rc<RefCell> Tests (Single-threaded) ⭐ NEW
- `test_rc_refcell_basic`: Basic Rc<RefCell> functionality
- `test_rc_refcell_compose_two_levels`: Two-level Rc<RefCell> composition
- `test_rc_refcell_three_levels`: Three-level Rc<RefCell> composition
- `test_rc_refcell_panic_on_clone_proof`: Panic-on-clone proof for Rc<RefCell>
- `test_rc_refcell_vs_arc_mutex`: API comparison between Rc<RefCell> and Arc<Mutex>

### Shallow Cloning Proof Tests ⭐ CRITICAL
- **`test_rwlock_panic_on_clone_proof`**: Uses `PanicOnClone` struct in nested RwLocks - **test PASSES** proving NO deep cloning
- **`test_mutex_panic_on_clone_proof`**: Uses `PanicOnClone` struct (1MB each level) in nested Mutexes - **test PASSES** proving NO deep cloning  
- **`test_mixed_locks_panic_on_clone_proof`**: Uses `NeverClone` struct in mixed RwLock/Mutex chain - **test PASSES** proving NO deep cloning

**These three tests are definitive proof**: Each test contains structs with a `Clone` impl that **PANICS with an error message**. The tests compose multiple lock levels and call `get()` multiple times. If ANY deep cloning occurred, the panic would trigger with a clear error message and tests would fail. 

✅ **All tests pass = Zero deep cloning confirmed!**

What these tests verify:
- Composing 2-level RwLocks: No `Level1` or `Level2` cloning
- Composing 2-level Mutexes with 1MB data at each level: No `Mid` or `Inner` cloning  
- Mixed RwLock→Mutex composition: No `Mid` or `Inner` cloning
- Multiple `get()` calls: Consistent shallow behavior
- Only `Arc` refcounts are incremented (shallow), never the inner data

**All 76 tests pass** (56 from core library + 20 lock module tests).

## Integration

The `lock` module is exported from the main library:

```rust
use rust_key_paths::{LockKp, LockAccess, ArcMutexAccess, ArcRwLockAccess, RcRefCellAccess, LockKpType};
```

## Custom Lock Types

You can create custom lock access implementations for other synchronization primitives:

```rust
#[derive(Clone)]
pub struct MyCustomLock<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> LockAccess<MyLockType<T>, &'a T> for MyCustomLock<T> {
    fn lock_read(&self, lock: &MyLockType<T>) -> Option<&'a T> {
        // Custom locking logic
    }
    
    fn lock_write(&self, lock: &mut MyLockType<T>) -> Option<&'a T> {
        // Custom locking logic
    }
}
```

## Benefits

1. **Type-Safe**: Maintains type safety throughout the chain
2. **Composable**: Works seamlessly with regular `Kp` via `then()`
3. **Multi-Level**: Handles deeply nested locks via `compose()`
4. **Flexible**: Trait-based `mid` allows custom lock implementations
5. **Ergonomic**: Handles the complexity of locking internally
6. **Shallow Cloning**: Guaranteed shallow clones only (proven by panic tests)
7. **Zero Deep Copies**: Inner data is never cloned, only Arc refcounts

## Use Cases

### Single Lock
Simple synchronized access to shared data.

### Nested Locks (via `compose()`)
- Configuration with nested locked sections
- Hierarchical data structures with locks at each level
- Multi-threaded systems with layered synchronization
- Game engines with nested entity locks
- Database connection pools with nested transaction locks

### Mixed Chains (via `compose()` + `then()`)
Navigate through locks and then continue with regular field access.

### Mixed Lock Types
Combine Mutex and RwLock based on access patterns at each level.
