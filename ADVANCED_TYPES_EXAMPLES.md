# Advanced Keypath Examples: Pin, MaybeUninit, and Smart Pointers

## Overview

This document demonstrates how to use `Kp` keypaths with advanced Rust types including `Pin`, `MaybeUninit`, and various smart pointer patterns (`Arc`, `Rc`, `Weak`).

## Pin<T> Examples

### What is Pin?

`Pin<P>` ensures a value won't be moved in memory. This is crucial for:
- Self-referential structs
- Async/await (futures that reference themselves)
- FFI with position-dependent data structures

### Example 1: Basic Pin with Self-Referential Struct

```rust
use std::pin::Pin;

#[derive(Debug)]
struct SelfReferential {
    value: String,
    ptr_to_value: *const String, // Points to value field
}

impl SelfReferential {
    fn new(s: String) -> Self {
        let mut sr = Self {
            value: s,
            ptr_to_value: std::ptr::null(),
        };
        sr.ptr_to_value = &sr.value as *const String;
        sr
    }
}

// Create pinned value
let pinned: Pin<Box<SelfReferential>> = Box::into_pin(
    Box::new(SelfReferential::new("pinned_data".to_string()))
);

// Keypath to access value field through Pin
let kp: KpType<Pin<Box<SelfReferential>>, String> = Kp::new(
    |p: &Pin<Box<SelfReferential>>| {
        Some(&p.as_ref().get_ref().value)
    },
    |p: &mut Pin<Box<SelfReferential>>| {
        unsafe {
            let sr = Pin::get_unchecked_mut(p.as_mut());
            Some(&mut sr.value)
        }
    },
);

let result = kp.get(&pinned);
assert_eq!(result, Some(&"pinned_data".to_string()));
```

**Key Points:**
- Use `Pin::as_ref().get_ref()` for immutable access
- Use `Pin::get_unchecked_mut()` for mutable access (requires `unsafe`)
- For `T: Unpin`, you can use `Pin::get_mut()` instead

### Example 2: Pin<Arc<T>> Pattern (Common in Async)

```rust
use std::pin::Pin;
use std::sync::Arc;

struct AsyncState {
    status: String,
    data: Vec<i32>,
}

let pinned_arc: Pin<Arc<AsyncState>> = Arc::pin(AsyncState {
    status: "ready".to_string(),
    data: vec![1, 2, 3, 4, 5],
});

// Keypath to status through Pin<Arc<T>>
let status_kp: KpType<Pin<Arc<AsyncState>>, String> = Kp::new(
    |p: &Pin<Arc<AsyncState>>| Some(&p.as_ref().get_ref().status),
    |_: &mut Pin<Arc<AsyncState>>| None::<&mut String>, // Arc is immutable
);

let status = status_kp.get(&pinned_arc);
assert_eq!(status, Some(&"ready".to_string()));
```

**Key Points:**
- `Pin<Arc<T>>` is common in async contexts
- Arc is immutable, so mutable keypaths typically return `None`
- Use `as_ref().get_ref()` to access the inner value

## MaybeUninit<T> Examples

### What is MaybeUninit?

`MaybeUninit<T>` represents potentially uninitialized memory. Useful for:
- Optimizing initialization sequences
- Working with FFI/C code
- Building complex data structures incrementally
- Avoiding unnecessary zeroing

### Example: Safe MaybeUninit with Keypaths

```rust
use std::mem::MaybeUninit;

struct Config {
    name: MaybeUninit<String>,
    value: MaybeUninit<i32>,
    initialized: bool,
}

impl Config {
    fn new_uninit() -> Self {
        Self {
            name: MaybeUninit::uninit(),
            value: MaybeUninit::uninit(),
            initialized: false,
        }
    }

    fn init(&mut self, name: String, value: i32) {
        self.name.write(name);
        self.value.write(value);
        self.initialized = true;
    }

    fn get_name(&self) -> Option<&String> {
        if self.initialized {
            unsafe { Some(self.name.assume_init_ref()) }
        } else {
            None
        }
    }
}

// Create keypath that safely checks initialization
let name_kp: KpType<Config, String> = Kp::new(
    |c: &Config| c.get_name(),
    |c: &mut Config| {
        if c.initialized {
            unsafe { Some(c.name.assume_init_mut()) }
        } else {
            None
        }
    },
);

// Test with uninitialized config
let uninit_config = Config::new_uninit();
assert_eq!(name_kp.get(&uninit_config), None);

// Test with initialized config
let mut init_config = Config::new_uninit();
init_config.init("test_config".to_string(), 42);
assert_eq!(name_kp.get(&init_config), Some(&"test_config".to_string()));
```

**Key Points:**
- Always check initialization state before using `assume_init_ref()`/`assume_init_mut()`
- Return `None` from keypaths when accessing uninitialized data
- Wrap unsafe operations in safe helper methods

### Example: MaybeUninit Array Buffer

```rust
use std::mem::MaybeUninit;

struct Buffer {
    data: [MaybeUninit<u8>; 10],
    len: usize,  // Track initialized elements
}

impl Buffer {
    fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    fn push(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.len >= self.data.len() {
            return Err("Buffer full");
        }
        self.data[self.len].write(byte);
        self.len += 1;
        Ok(())
    }

    fn get(&self, idx: usize) -> Option<&u8> {
        if idx < self.len {
            unsafe { Some(self.data[idx].assume_init_ref()) }
        } else {
            None
        }
    }
}

// Keypath to buffer length
let len_kp: KpType<Buffer, usize> = Kp::new(
    |b: &Buffer| Some(&b.len),
    |b: &mut Buffer| Some(&mut b.len),
);

let mut buffer = Buffer::new();
buffer.push(1).unwrap();
buffer.push(2).unwrap();

assert_eq!(len_kp.get(&buffer), Some(&2));
assert_eq!(buffer.get(0), Some(&1));
```

**Key Points:**
- Track initialization state separately (e.g., `len` field)
- Only access initialized elements
- Provide safe wrapper methods around unsafe operations

## Smart Pointer Examples

### Arc and Weak Pattern

`Weak<T>` provides non-owning references that don't prevent deallocation. Useful for:
- Breaking reference cycles
- Caching without preventing cleanup
- Observer patterns

### Example: Simple Arc/Option Pattern

```rust
use std::sync::Arc;

struct NodeWithParent {
    value: i32,
    parent: Option<Arc<Node>>, // Strong reference
}

struct Node {
    value: i32,
}

let parent = Arc::new(Node { value: 100 });

let child = NodeWithParent {
    value: 42,
    parent: Some(parent.clone()),
};

// Keypath to access parent value
let parent_value_kp: KpType<NodeWithParent, i32> = Kp::new(
    |n: &NodeWithParent| n.parent.as_ref().map(|arc| &arc.value),
    |_: &mut NodeWithParent| None::<&mut i32>,
);

let parent_val = parent_value_kp.get(&child);
assert_eq!(parent_val, Some(&100));
```

### Example: Rc Pattern (Single-Threaded)

```rust
use std::rc::Rc;

struct TreeNode {
    value: String,
    parent: Option<Rc<TreeNode>>,
}

let root = Rc::new(TreeNode {
    value: "root".to_string(),
    parent: None,
});

let child = TreeNode {
    value: "child1".to_string(),
    parent: Some(root.clone()),
};

// Keypath to access parent's value
let parent_name_kp: KpType<TreeNode, String> = Kp::new(
    |node: &TreeNode| node.parent.as_ref().map(|rc| &rc.value),
    |_: &mut TreeNode| None::<&mut String>,
);

assert_eq!(parent_name_kp.get(&child), Some(&"root".to_string()));
assert_eq!(parent_name_kp.get(&root), None); // Root has no parent
```

### Example: Nested Arc Structure

```rust
use std::sync::Arc;

struct Cache {
    data: String,
    backup: Option<Arc<Cache>>,
}

let primary = Arc::new(Cache {
    data: "primary_data".to_string(),
    backup: None,
});

let backup = Arc::new(Cache {
    data: "backup_data".to_string(),
    backup: Some(primary.clone()),
});

// Keypath to access backup's data
let backup_data_kp: KpType<Arc<Cache>, String> = Kp::new(
    |cache_arc: &Arc<Cache>| {
        cache_arc.backup.as_ref().map(|arc| &arc.data)
    },
    |_: &mut Arc<Cache>| None::<&mut String>,
);

let data = backup_data_kp.get(&backup);
assert_eq!(data, Some(&"primary_data".to_string()));
```

## Combining Pin with Smart Pointers

### Pin<Arc<T>> Chaining Example

```rust
use std::pin::Pin;
use std::sync::Arc;

struct Outer {
    inner: Arc<Inner>,
}

struct Inner {
    value: String,
}

let pinned_outer = Box::pin(Outer {
    inner: Arc::new(Inner {
        value: "nested_value".to_string(),
    }),
});

// First keypath: Pin<Box<Outer>> -> Arc<Inner>
let to_inner: KpType<Pin<Box<Outer>>, Arc<Inner>> = Kp::new(
    |p: &Pin<Box<Outer>>| Some(&p.as_ref().get_ref().inner),
    |_: &mut Pin<Box<Outer>>| None::<&mut Arc<Inner>>,
);

// Second keypath: Arc<Inner> -> String
let to_value: KpType<Arc<Inner>, String> = Kp::new(
    |a: &Arc<Inner>| Some(&a.value),
    |_: &mut Arc<Inner>| None::<&mut String>,
);

// Chain them together
let chained = to_inner.then(to_value);
let result = chained.get(&pinned_outer);
assert_eq!(result, Some(&"nested_value".to_string()));
```

## Safety Considerations

### Pin Safety

- **Immutable access**: Safe with `as_ref().get_ref()`
- **Mutable access**: Requires `unsafe` unless `T: Unpin`
- **Moving pinned data**: Never move data out of Pin
- **Purpose**: Guarantees memory location stability

### MaybeUninit Safety

- **Always check**: Track initialization state separately
- **Never assume**: Don't call `assume_init_*()` on uninitialized data
- **Undefined behavior**: Accessing uninitialized data is UB
- **Safe wrappers**: Provide methods that check before accessing

### Weak Safety

- **Upgrade can fail**: Always handle `None` from `upgrade()`
- **Temporary references**: Upgraded Arc lives only for the closure scope
- **No cycles**: Use Weak to break reference cycles
- **Thread safety**: `Weak<T>` vs `rc::Weak<T>` for threading needs

## Best Practices

### 1. Pin Usage
```rust
// ✅ Good: Safe immutable access
|p: &Pin<Box<T>>| Some(&p.as_ref().get_ref().field)

// ⚠️ Requires unsafe: Mutable access to non-Unpin
|p: &mut Pin<Box<T>>| unsafe { Some(&mut Pin::get_unchecked_mut(p.as_mut()).field) }

// ✅ Safe if T: Unpin
|p: &mut Pin<Box<T>>| Some(&mut Pin::get_mut(p).field)
```

### 2. MaybeUninit Usage
```rust
// ✅ Good: Check initialization state
|config: &Config| {
    if config.initialized {
        unsafe { Some(config.value.assume_init_ref()) }
    } else {
        None
    }
}

// ❌ Bad: Assuming initialization (UB if false!)
|config: &Config| unsafe { Some(config.value.assume_init_ref()) }
```

### 3. Weak References
```rust
// ✅ Good: Handle upgrade failure
|node: &Node| {
    node.parent.as_ref()
        .and_then(|weak| weak.upgrade())
        .map(|arc| &arc.value)
}

// ❌ Bad: Unwrapping could panic
|node: &Node| {
    Some(&node.parent.as_ref().unwrap().upgrade().unwrap().value)
}
```

## Performance Characteristics

### Pin<T>
- **Zero-cost abstraction**: No runtime overhead
- **Compile-time guarantee**: Memory location stability
- **Access cost**: Same as accessing T directly

### MaybeUninit<T>
- **No initialization**: Avoids default/zero initialization
- **Memory efficient**: Only initialize what you need
- **Access cost**: Negligible (just a pointer cast with safety check)

### Weak<T>
- **Upgrade cost**: Atomic operation to check reference count
- **Memory**: Weak pointer is same size as Arc/Rc
- **Failure handling**: Must handle `None` from upgrade

## Common Patterns

### Pattern 1: Pin + Arc for Async State

```rust
let state: Pin<Arc<AsyncState>> = Arc::pin(AsyncState::new());
let status_kp = Kp::new(
    |p: &Pin<Arc<AsyncState>>| Some(&p.as_ref().get_ref().status),
    |_| None,
);
```

**Use when:** Working with async tasks that need stable memory locations

### Pattern 2: MaybeUninit for Lazy Initialization

```rust
struct LazyData {
    value: MaybeUninit<ExpensiveType>,
    initialized: bool,
}

let kp = Kp::new(
    |d: &LazyData| {
        if d.initialized {
            unsafe { Some(d.value.assume_init_ref()) }
        } else {
            None
        }
    },
    |_| None,
);
```

**Use when:** Expensive initialization that should be deferred

### Pattern 3: Arc/Option for Optional Parents

```rust
struct Node {
    value: T,
    parent: Option<Arc<Node>>,
}

let parent_kp = Kp::new(
    |n: &Node| n.parent.as_ref().map(|arc| &arc.value),
    |_| None,
);
```

**Use when:** Tree structures with optional parent references

## Tests

All examples are validated with comprehensive tests:

1. **`test_kp_with_pin`**: Basic Pin with self-referential struct
2. **`test_kp_with_pin_arc`**: Pin<Arc<T>> pattern for async state
3. **`test_kp_with_maybe_uninit`**: Safe MaybeUninit with initialization tracking
4. **`test_kp_with_maybe_uninit_array`**: MaybeUninit array buffer
5. **`test_kp_with_weak`**: Arc with optional parent pattern
6. **`test_kp_with_rc_weak`**: Rc tree structure
7. **`test_kp_with_complex_weak_structure`**: Nested Arc references
8. **`test_kp_chain_with_pin_and_arc`**: Chaining through Pin and Arc

**Total: 96 tests passing** (including all advanced type examples)

## Limitations and Gotchas

### Pin Limitations
- **Mutable access requires unsafe**: Unless `T: Unpin`
- **Can't move**: Once pinned, can't extract the value
- **Projection**: Pinning nested fields requires careful unsafe code

### MaybeUninit Limitations
- **Must track state**: Need separate flag for initialization
- **All unsafe**: No safe way to access without assume_init
- **Lifetime complexity**: References to MaybeUninit data have complex lifetimes

### Weak Limitations  
- **Upgrade can fail**: Must always handle None case
- **Temporary lifetime**: Upgraded Arc/Rc lives only in closure scope
- **Can't return references**: Can't return `&T` from upgraded Weak in keypath closure
  - **Workaround**: Use strong references (Arc/Rc) or copy/clone the value

## Conclusion

Keypaths work seamlessly with Rust's advanced type system features:
- **Pin**: Safe access to pinned data with clear unsafe boundaries
- **MaybeUninit**: Safe patterns for uninitialized memory with proper checking
- **Smart pointers**: Natural integration with Arc, Rc, and optional references

All patterns maintain type safety while providing ergonomic access to complex data structures.
