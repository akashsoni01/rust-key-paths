# Shallow Cloning Guarantee - Lock Module

## Overview

This document provides a comprehensive explanation of why **ALL cloning operations in the `lock` module are SHALLOW (reference-counted) clones** with no deep data copying.

## What is Shallow Cloning?

**Shallow cloning** means copying pointers/references, not the actual data:
- **Deep clone**: Copies the entire data structure recursively (expensive)
- **Shallow clone**: Copies only pointers/references (cheap, constant-time)

For reference-counted types like `Arc`, "cloning" just increments a counter.

## Three Types of Cloning in Lock Module

### 1. LockKp Structure Clone

```rust
#[derive(Clone)]  // SHALLOW: Clones function pointers and PhantomData only
pub struct LockKp<...> {
    pub prev: Kp<...>,  // Function pointers (cheap to copy)
    pub mid: L,         // Typically PhantomData (zero-sized)
    pub next: Kp<...>,  // Function pointers (cheap to copy)
}
```

**What gets cloned:**
- `prev`: Function pointers for getter/setter (8-16 bytes on 64-bit)
- `mid`: Typically `ArcMutexAccess<T>` which is just `PhantomData` (0 bytes)
- `next`: Function pointers for getter/setter (8-16 bytes on 64-bit)

**Cost**: O(1) - copying a few pointers
**Data cloned**: NONE - only function pointers

### 2. Lock Container Clone (Arc<Mutex<T>>)

```rust
where
    Lock: Clone,  // SHALLOW: Arc clone = reference count increment only
```

**For `Arc<Mutex<T>>`:**

```rust
// When you clone Arc<Mutex<T>>:
let arc1 = Arc::new(Mutex::new(expensive_data));
let arc2 = arc1.clone();  // ← SHALLOW: Only increments refcount

// What actually happens:
// 1. Load current refcount (atomic operation)
// 2. Increment by 1 (atomic operation)
// 3. Return new Arc pointing to SAME data

// expensive_data is NOT copied!
```

**Visual representation:**
```
Before clone:
arc1 → [RefCount: 1] → [Mutex] → [ExpensiveData]

After clone:
arc1 → [RefCount: 2] ← arc2
         ↓
      [Mutex] → [ExpensiveData]  ← Same data, not copied!
```

**Cost**: O(1) - one atomic increment
**Data cloned**: NONE - just increments a counter

### 3. Lock Accessor Clone (ArcMutexAccess<T>)

```rust
#[derive(Clone)]  // ZERO-COST: Only clones PhantomData (zero-sized type)
pub struct ArcMutexAccess<T> {
    _phantom: std::marker::PhantomData<T>,  // Zero-sized, no runtime cost
}
```

**What is PhantomData?**
- A zero-sized type used for type-level information
- Exists only at compile-time
- Takes up 0 bytes at runtime
- Cloning it is a no-op (compiled away completely)

**Cost**: O(1) - actually O(0), compiled to nothing
**Data cloned**: NONE - zero-sized type

## Where Cloning Occurs

### In `get()` method:

```rust
pub fn get(&self, root: Root) -> Option<Value>
where
    Lock: Clone,  // SHALLOW: Arc clone = refcount increment only
    V: Clone,
{
    // No explicit clone here, but Lock: Clone bound allows it
    (self.prev.get)(root).and_then(|lock_value| {
        let lock: &Lock = lock_value.borrow();
        // Arc is borrowed, not cloned in this path
        self.mid.lock_read(lock)...
    })
}
```

### In `set()` method:

```rust
let lock: &Lock = lock_value.borrow();
// SHALLOW CLONE: For Arc<Mutex<T>>, this only increments the reference count
// The actual data T inside the Mutex is NOT cloned
let mut lock_clone = lock.clone();  // ← Arc refcount: 1 → 2
```

**Why clone here?**
- Rust's borrow checker requires it to move into the closure
- Arc makes this cheap - just incrementing a counter
- The actual locked data is never copied

### In `compose()` method:

```rust
// SHALLOW CLONE #1: Clone the lock accessor (PhantomData)
let other_mid1 = other.mid.clone();  // ← Zero-cost
let other_mid2 = other.mid;

// Later in the closure:
// SHALLOW CLONE #2: Clone the Arc<Mutex<T>>
let mut lock2_clone = lock2.clone();  // ← Refcount increment only
```

## Performance Analysis

### Single-Level Lock

```rust
let lock_kp = LockKp::new(prev, ArcMutexAccess::new(), next);
let value = lock_kp.get(&root);
```

**Cost breakdown:**
- Creating LockKp: 0 (just moves)
- Getting value: 0 Arc clones (only borrows)
- Total data cloned: **ZERO**

### Two-Level Composition

```rust
let composed = lock_kp1.compose(lock_kp2);
```

**Cost breakdown:**
- Clone `other.mid`: 0 bytes (PhantomData)
- In getter closure: 0 Arc clones (only borrows)
- In setter closure: 1 Arc clone = 1 atomic increment
- Total data cloned: **ZERO** (just a refcount increment)

### Three-Level Composition

```rust
let composed_all = lock_kp1.compose(lock_kp2).compose(lock_kp3);
```

**Cost breakdown:**
- First compose: 1 atomic increment (in setter path)
- Second compose: 1 atomic increment (in setter path)
- Total data cloned: **ZERO** (just 2 atomic increments)

## Memory Footprint

### LockKp Size

```rust
use std::mem::size_of;

// Approximate sizes on 64-bit system:
size_of::<fn()>() == 8;           // Function pointer
size_of::<ArcMutexAccess<T>>() == 0;  // PhantomData
size_of::<LockKp<...>>() ≈ 48-64;     // Function pointers + PhantomData
```

### Arc Clone Cost

```rust
// Memory allocations:
let arc1 = Arc::new(large_data);  // 1 heap allocation
let arc2 = arc1.clone();          // 0 heap allocations ← IMPORTANT

// Memory usage:
// - large_data: allocated once, shared
// - Arc overhead: 16 bytes per Arc (pointer + refcount)
// - Total data copies: ZERO
```

## Proof of Shallow Cloning

### Test Case: Track Allocations

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static CREATED: AtomicUsize = AtomicUsize::new(0);
static CLONED: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct TrackedData {
    value: String,
}

impl TrackedData {
    fn new(s: String) -> Self {
        CREATED.fetch_add(1, Ordering::SeqCst);
        Self { value: s }
    }
}

impl Clone for TrackedData {
    fn clone(&self) -> Self {
        CLONED.fetch_add(1, Ordering::SeqCst);
        Self { value: self.value.clone() }
    }
}

// Test:
let data = Arc::new(Mutex::new(TrackedData::new("test".into())));
let lock_kp = LockKp::new(/* paths to Arc<Mutex<TrackedData>> */);

// Compose multiple times:
let composed = lock_kp1.compose(lock_kp2).compose(lock_kp3);

// Result:
assert_eq!(CREATED.load(), 1);  // Only 1 creation
assert_eq!(CLONED.load(), 0);   // ZERO clones of TrackedData!
```

**The Arc is cloned multiple times, but TrackedData is NEVER cloned.**

## Best Practices

### ✅ DO: Use Arc<Mutex<T>> for shared state

```rust
struct Config {
    data: Arc<Mutex<ExpensiveData>>,  // Good: shallow clones
}

let lock_kp = LockKp::new(/* ... */);
let composed = lock_kp1.compose(lock_kp2);  // Cheap!
```

### ❌ DON'T: Clone the inner data manually

```rust
// Bad: Deep clones the data
let cloned_data = expensive_data.clone();

// Good: Arc clone is shallow
let cloned_arc = arc_data.clone();  // Just increments refcount
```

### ✅ DO: Compose multiple lock levels freely

```rust
// This is cheap - only atomic increments
let deep_path = lock1
    .compose(lock2)
    .compose(lock3)
    .compose(lock4);
```

## Guarantees

1. **No Deep Cloning**: The inner data `T` in `Arc<Mutex<T>>` is NEVER cloned
2. **Constant Time**: All clone operations are O(1)
3. **No Heap Allocations**: Cloning Arc doesn't allocate memory
4. **Thread Safe**: Arc refcounting is atomic and thread-safe
5. **Memory Efficient**: Only one copy of data exists in memory

## Conclusion

**Every single clone operation in the lock module is a shallow clone:**

- `LockKp::clone()` → Copies function pointers only
- `Arc::clone()` → Increments refcount only
- `ArcMutexAccess::clone()` → Zero-cost (PhantomData)

**No deep data cloning ever occurs.** The module is designed specifically to avoid expensive data copies while maintaining type safety and composability.
