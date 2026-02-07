# LockKp - Keypath for Locked Values

## Overview

`LockKp` is a specialized keypath type for handling locked/synchronized values like `Arc<Mutex<T>>`. It provides a safe, composable way to navigate through locked data structures.

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
3. **Flexible**: Trait-based `mid` allows custom lock implementations
4. **Ergonomic**: Handles the complexity of locking internally

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

All 59 tests pass (56 from core library + 3 from lock module).
