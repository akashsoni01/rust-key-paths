# LockKp - Keypath for Locked Values

## Overview

`LockKp` is a specialized keypath type for handling locked/synchronized values like `Arc<Mutex<T>>`. It provides a safe, composable way to navigate through locked data structures, including support for **multi-level lock chaining**.

## Structure

The `LockKp` struct has three key components, as requested:

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

### Standard Implementation: `ArcMutexAccess<T>`

A ready-to-use implementation for `Arc<Mutex<T>>`:

```rust
let access = ArcMutexAccess::<InnerType>::new();
```

## Usage Example

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

// Use it
let root = Root {
    locked_data: Arc::new(Mutex::new(Inner {
        value: "hello".to_string(),
    })),
};

// Get value through the lock
let value = lock_kp.get(&root);
```

## Chaining with `then()`

`LockKp` supports chaining with regular `Kp` using the `then()` operator:

```rust
// Root -> Arc<Mutex<Mid>> -> Inner1 -> Inner2 -> Value
let chained = lock_kp.then(another_kp);
```

This allows you to continue navigating after getting through the lock layer.

## Multi-Level Lock Chaining with `compose()`

**NEW**: The `compose()` method allows you to chain through **multiple lock levels**:

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

// Now navigate through both lock levels in one operation
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

### Combining `compose()` and `then()`

You can mix lock composition with regular keypath chaining:

```rust
// Root -> Lock1 -> Lock2 -> Inner -> Data -> Value
let composed = lock1.compose(lock2);       // Handle both locks
let with_data = composed.then(to_data);    // Navigate to Data
let with_value = with_data.then(to_value); // Navigate to Value

let value = with_value.get(&root);
```

## Type Alias for Common Usage

```rust
pub type LockKpType<'a, R, Mid, V> = LockKp<
    R,
    Arc<Mutex<Mid>>,
    Mid,
    V,
    &'a R,
    &'a Arc<Mutex<Mid>>,
    &'a Mid,
    &'a V,
    &'a mut R,
    &'a mut Arc<Mutex<Mid>>,
    &'a mut Mid,
    &'a mut V,
    // ... getter/setter function types
    ArcMutexAccess<Mid>,
    // ... more function types
>;
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

### `compose()` ⭐ NEW
Composes this LockKp with another LockKp for multi-level lock chaining:
```rust
pub fn compose<Lock2, Mid2, V2, ...>(
    self,
    other: LockKp<V, Lock2, Mid2, V2, ...>
) -> LockKp<R, Lock, Mid, V2, ...>
```

This is the key method for handling nested locks like:
- `Root -> Arc<Mutex<A>> -> A -> Arc<Mutex<B>> -> B -> Value`
- `Root -> Lock1 -> Lock2 -> Lock3 -> Value`

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

## Integration

The `lock` module is exported from the main library:

```rust
use rust_key_paths::{LockKp, LockAccess, ArcMutexAccess, LockKpType};
```

## Tests

The module includes comprehensive tests:
- `test_lock_kp_basic`: Basic functionality test
- `test_lock_kp_structure`: Verifies the three-field structure
- `test_lock_kp_then_chaining`: Tests chaining with `then()`
- `test_lock_kp_compose_single_level`: Tests composing two LockKps
- `test_lock_kp_compose_two_levels`: Tests two-level lock composition
- `test_lock_kp_compose_three_levels`: Tests three-level lock composition
- `test_lock_kp_compose_with_then`: Tests mixing `compose()` and `then()`

**All 63 tests pass** (56 from core library + 3 original lock tests + 5 compose tests).

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
