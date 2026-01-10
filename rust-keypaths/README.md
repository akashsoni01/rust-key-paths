# 🔑 Rust KeyPaths Library

**A faster alternative to `key-paths-core`** - A lightweight, zero-cost abstraction library for safe, composable access to nested data structures in Rust. Inspired by Functional lenses and Swift's KeyPath system, this library provides type-safe keypaths for struct fields and enum variants for superior performance. 



## 🚀 Why This Library?

This is a **faster alternative** to the `rust-key-paths` library :
- ✅ **Better Performance**: Write operations can be **faster than manual unwrapping** at deeper nesting levels
- ✅ **Zero Runtime Overhead**: Eliminates dynamic dispatch costs
- ✅ **Compiler Optimizations**: Better inlining and optimization opportunities
- ✅ **Type Safety**: Full compile-time type checking with zero runtime cost

## ✨ Features

### Core Types

- **`KeyPath<Root, Value, F>`** - Readable keypath for direct field access
- **`OptionalKeyPath<Root, Value, F>`** - Failable keypath for `Option<T>` chains
- **`WritableKeyPath<Root, Value, F>`** - Writable keypath for mutable field access
- **`WritableOptionalKeyPath<Root, Value, F>`** - Failable writable keypath for mutable `Option<T>` chains
- **`EnumKeyPaths`** - Static factory for enum variant extraction and container unwrapping

### Key Features

- ✅ **Zero-cost abstractions** - Compiles to direct field access
- ✅ **Type-safe** - Full compile-time type checking
- ✅ **Composable** - Chain keypaths with `.then()` for nested access
- ✅ **Automatic type inference** - No need to specify types explicitly
- ✅ **Container support** - Built-in support for `Box<T>`, `Arc<T>`, `Rc<T>`, `Option<T>`
- ✅ **Writable keypaths** - Full support for mutable access to nested data
- ✅ **Enum variant extraction** - Extract values from enum variants safely
- ✅ **Cloneable** - Keypaths can be cloned without cloning underlying data
- ✅ **Memory efficient** - No unnecessary allocations or cloning

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-keypaths = { path = "../rust-keypaths" }

# Optional features
[features]
tagged = ["rust-keypaths/tagged"]  # Enable tagged-core support
parking_lot = ["rust-keypaths/parking_lot"]  # Enable parking_lot support
```

## 🚀 Quick Start

### Basic Usage

```rust
use rust_keypaths::{KeyPath, OptionalKeyPath};

#[derive(Debug)]
struct User {
    name: String,
    email: Option<String>,
}

let user = User {
    name: "Alice".to_string(),
    email: Some("akash@example.com".to_string()),
};

// Create readable keypath
let name_kp = KeyPath::new(|u: &User| &u.name);
println!("Name: {}", name_kp.get(&user));

// Create failable keypath for Option
let email_kp = OptionalKeyPath::new(|u: &User| u.email.as_ref());
if let Some(email) = email_kp.get(&user) {
    println!("Email: {}", email);
}
```

### Chaining Keypaths

```rust
#[derive(Debug)]
struct Address {
    street: String,
}

#[derive(Debug)]
struct Profile {
    address: Option<Address>,
}

#[derive(Debug)]
struct User {
    profile: Option<Profile>,
}

let user = User {
    profile: Some(Profile {
        address: Some(Address {
            street: "123 Main St".to_string(),
        }),
    }),
};

// Chain keypaths to access nested field
let street_kp = OptionalKeyPath::new(|u: &User| u.profile.as_ref())
    .then(OptionalKeyPath::new(|p: &Profile| p.address.as_ref()))
    .then(OptionalKeyPath::new(|a: &Address| Some(&a.street)));

if let Some(street) = street_kp.get(&user) {
    println!("Street: {}", street);
}
```

### Container Unwrapping

```rust
use rust_keypaths::{OptionalKeyPath, KeyPath};

struct Container {
    boxed: Option<Box<String>>,
}

let container = Container {
    boxed: Some(Box::new("Hello".to_string())),
};

// Unwrap Option<Box<String>> to Option<&String> automatically
let kp = OptionalKeyPath::new(|c: &Container| c.boxed.as_ref())
    .for_box(); // Type automatically inferred!

if let Some(value) = kp.get(&container) {
    println!("Value: {}", value);
}
```
### Functional Chains for Arc<Mutex<T>> and Arc<RwLock<T>>

Compose keypaths through synchronization primitives with a functional, compose-first approach:

```rust
use std::sync::{Arc, Mutex, RwLock};
use keypaths_proc::{Keypaths, WritableKeypaths};

#[derive(Debug, Keypaths, WritableKeypaths)]
struct Container {
    mutex_data: Arc<Mutex<DataStruct>>,
    rwlock_data: Arc<RwLock<DataStruct>>,
}

#[derive(Debug, Keypaths, WritableKeypaths)]
struct DataStruct {
    name: String,
    optional_value: Option<String>,
}

fn main() {
    let container = Container::new();
    
    // Read through Arc<Mutex<T>> - compose the chain, then apply
    Container::mutex_data_r()
        .chain_arc_mutex_at_kp(DataStruct::name_r())
        .get(&container, |value| {
            println!("Name: {}", value);
        });
    
    // Write through Arc<Mutex<T>>
    Container::mutex_data_r()
        .chain_arc_mutex_writable_at_kp(DataStruct::name_w())
        .get_mut(&container, |value| {
            *value = "New name".to_string();
        });
    
    // Read through Arc<RwLock<T>> (read lock)
    Container::rwlock_data_r()
        .chain_arc_rwlock_at_kp(DataStruct::name_r())
        .get(&container, |value| {
            println!("Name: {}", value);
        });
    
    // Write through Arc<RwLock<T>> (write lock)
    Container::rwlock_data_r()
        .chain_arc_rwlock_writable_at_kp(DataStruct::name_w())
        .get_mut(&container, |value| {
            *value = "New name".to_string();
        });
}
```

**Running the example:**
```bash
cargo run --example readable_keypaths_new_containers_test
```

### Collection Access

The library provides utilities for accessing elements in various collection types:

```rust
use rust_keypaths::{OptionalKeyPath, containers};
use std::collections::{HashMap, VecDeque, HashSet, BinaryHeap};

struct Data {
    vec: Vec<String>,
    map: HashMap<String, i32>,
    deque: VecDeque<String>,
    set: HashSet<String>,
    heap: BinaryHeap<String>,
}

let data = Data { /* ... */ };

// Access Vec element at index
let vec_kp = OptionalKeyPath::new(|d: &Data| Some(&d.vec))
    .then(containers::for_vec_index::<String>(1));

// Access HashMap value by key
let map_kp = OptionalKeyPath::new(|d: &Data| Some(&d.map))
    .then(containers::for_hashmap_key("key1".to_string()));

// Access VecDeque element at index
let deque_kp = OptionalKeyPath::new(|d: &Data| Some(&d.deque))
    .then(containers::for_vecdeque_index::<String>(0));

// Access HashSet element
let set_kp = OptionalKeyPath::new(|d: &Data| Some(&d.set))
    .then(containers::for_hashset_get("value".to_string()));

// Peek at BinaryHeap top element
let heap_kp = OptionalKeyPath::new(|d: &Data| Some(&d.heap))
    .then(containers::for_binaryheap_peek::<String>());
```

**Supported Collections:**
- `Vec<T>` - Indexed access via `for_vec_index(index)`
- `VecDeque<T>` - Indexed access via `for_vecdeque_index(index)`
- `LinkedList<T>` - Indexed access via `for_linkedlist_index(index)`
- `HashMap<K, V>` - Key-based access via `for_hashmap_key(key)`
- `BTreeMap<K, V>` - Key-based access via `for_btreemap_key(key)`
- `HashSet<T>` - Element access via `for_hashset_get(value)`
- `BTreeSet<T>` - Element access via `for_btreeset_get(value)`
- `BinaryHeap<T>` - Peek access via `for_binaryheap_peek()`

### Enum Variant Extraction

```rust
use rust_keypaths::{OptionalKeyPath, EnumKeyPaths};

enum Result {
    Ok(String),
    Err(String),
}

let result = Result::Ok("success".to_string());

// Extract from enum variant
let ok_kp = EnumKeyPaths::for_variant(|r: &Result| {
    if let Result::Ok(value) = r {
        Some(value)
    } else {
        None
    }
});

if let Some(value) = ok_kp.get(&result) {
    println!("Success: {}", value);
}
```

## 📚 API Reference

### KeyPath

#### Methods

- **`new(getter: F) -> Self`** - Create a new keypath from a getter function
- **`get(&self, root: &Root) -> &Value`** - Get a reference to the value
- **`for_box<Target>(self) -> KeyPath<Root, Target, ...>`** - Unwrap `Box<T>` to `T` (type inferred)
- **`for_arc<Target>(self) -> KeyPath<Root, Target, ...>`** - Unwrap `Arc<T>` to `T` (type inferred)
- **`for_rc<Target>(self) -> KeyPath<Root, Target, ...>`** - Unwrap `Rc<T>` to `T` (type inferred)

#### Example

```rust
let kp = KeyPath::new(|b: &Box<String>| b.as_ref());
let unwrapped = kp.for_box(); // Automatically infers String
```

### OptionalKeyPath

#### Methods

- **`new(getter: F) -> Self`** - Create a new optional keypath
- **`get(&self, root: &Root) -> Option<&Value>`** - Get an optional reference
- **`then<SubValue, G>(self, next: OptionalKeyPath<Value, SubValue, G>) -> OptionalKeyPath<Root, SubValue, ...>`** - Chain keypaths
- **`for_box<Target>(self) -> OptionalKeyPath<Root, Target, ...>`** - Unwrap `Option<Box<T>>` to `Option<&T>`
- **`for_arc<Target>(self) -> OptionalKeyPath<Root, Target, ...>`** - Unwrap `Option<Arc<T>>` to `Option<&T>`
- **`for_rc<Target>(self) -> OptionalKeyPath<Root, Target, ...>`** - Unwrap `Option<Rc<T>>` to `Option<&T>`
- **`for_option<T>() -> OptionalKeyPath<Option<T>, T, ...>`** - Static method to create keypath for `Option<T>`

#### Example

```rust
let kp1 = OptionalKeyPath::new(|s: &Struct| s.field.as_ref());
let kp2 = OptionalKeyPath::new(|o: &Other| o.value.as_ref());
let chained = kp1.then(kp2); // Chain them together
```

### WritableKeyPath

#### Methods

- **`new(getter: F) -> Self`** - Create a new writable keypath from a getter function
- **`get_mut(&self, root: &mut Root) -> &mut Value`** - Get a mutable reference to the value
- **`for_box<Target>(self) -> WritableKeyPath<Root, Target, ...>`** - Unwrap `Box<T>` to `T` (mutable, type inferred)
- **`for_arc<Target>(self) -> WritableKeyPath<Root, Target, ...>`** - Unwrap `Arc<T>` to `T` (mutable, type inferred)
- **`for_rc<Target>(self) -> WritableKeyPath<Root, Target, ...>`** - Unwrap `Rc<T>` to `T` (mutable, type inferred)

#### Example

```rust
let mut data = MyStruct { field: "value".to_string() };
let kp = WritableKeyPath::new(|s: &mut MyStruct| &mut s.field);
*kp.get_mut(&mut data) = "new_value".to_string();
```

### WritableOptionalKeyPath

#### Methods

- **`new(getter: F) -> Self`** - Create a new writable optional keypath
- **`get_mut(&self, root: &mut Root) -> Option<&mut Value>`** - Get an optional mutable reference
- **`then<SubValue, G>(self, next: WritableOptionalKeyPath<Value, SubValue, G>) -> WritableOptionalKeyPath<Root, SubValue, ...>`** - Chain writable keypaths
- **`for_box<Target>(self) -> WritableOptionalKeyPath<Root, Target, ...>`** - Unwrap `Option<Box<T>>` to `Option<&mut T>`
- **`for_arc<Target>(self) -> WritableOptionalKeyPath<Root, Target, ...>`** - Unwrap `Option<Arc<T>>` to `Option<&mut T>`
- **`for_rc<Target>(self) -> WritableOptionalKeyPath<Root, Target, ...>`** - Unwrap `Option<Rc<T>>` to `Option<&mut T>`
- **`for_option<T>() -> WritableOptionalKeyPath<Option<T>, T, ...>`** - Static method to create writable keypath for `Option<T>`

#### Example

```rust
let mut data = MyStruct { field: Some("value".to_string()) };
let kp = WritableOptionalKeyPath::new(|s: &mut MyStruct| s.field.as_mut());
if let Some(value) = kp.get_mut(&mut data) {
    *value = "new_value".to_string();
}
```

### EnumKeyPaths

#### Static Methods

- **`for_variant<Enum, Variant, ExtractFn>(extractor: ExtractFn) -> OptionalKeyPath<Enum, Variant, ...>`** - Extract from enum variant
- **`for_match<Enum, Output, MatchFn>(matcher: MatchFn) -> KeyPath<Enum, Output, ...>`** - Match against multiple variants
- **`for_ok<T, E>() -> OptionalKeyPath<Result<T, E>, T, ...>`** - Extract `Ok` from `Result`
- **`for_err<T, E>() -> OptionalKeyPath<Result<T, E>, E, ...>`** - Extract `Err` from `Result`
- **`for_some<T>() -> OptionalKeyPath<Option<T>, T, ...>`** - Extract from `Option<T>`
- **`for_option<T>() -> OptionalKeyPath<Option<T>, T, ...>`** - Alias for `for_some`
- **`for_box<T>() -> KeyPath<Box<T>, T, ...>`** - Create keypath for `Box<T>`
- **`for_arc<T>() -> KeyPath<Arc<T>, T, ...>`** - Create keypath for `Arc<T>`
- **`for_rc<T>() -> KeyPath<Rc<T>, T, ...>`** - Create keypath for `Rc<T>`
- **`for_box_mut<T>() -> WritableKeyPath<Box<T>, T, ...>`** - Create writable keypath for `Box<T>`

#### Example

```rust
// Extract from Result
let ok_kp = EnumKeyPaths::for_ok::<String, String>();
let result: Result<String, String> = Ok("value".to_string());
if let Some(value) = ok_kp.get(&result) {
    println!("{}", value);
}
```

### containers Module

Utility functions for accessing elements in standard library collections.

#### Functions (Readable)

- **`for_vec_index<T>(index: usize) -> OptionalKeyPath<Vec<T>, T, ...>`** - Access element at index in `Vec<T>`
- **`for_vecdeque_index<T>(index: usize) -> OptionalKeyPath<VecDeque<T>, T, ...>`** - Access element at index in `VecDeque<T>`
- **`for_linkedlist_index<T>(index: usize) -> OptionalKeyPath<LinkedList<T>, T, ...>`** - Access element at index in `LinkedList<T>`
- **`for_hashmap_key<K, V>(key: K) -> OptionalKeyPath<HashMap<K, V>, V, ...>`** - Access value by key in `HashMap<K, V>`
- **`for_btreemap_key<K, V>(key: K) -> OptionalKeyPath<BTreeMap<K, V>, V, ...>`** - Access value by key in `BTreeMap<K, V>`
- **`for_hashset_get<T>(value: T) -> OptionalKeyPath<HashSet<T>, T, ...>`** - Get element from `HashSet<T>`
- **`for_btreeset_get<T>(value: T) -> OptionalKeyPath<BTreeSet<T>, T, ...>`** - Get element from `BTreeSet<T>`
- **`for_binaryheap_peek<T>() -> OptionalKeyPath<BinaryHeap<T>, T, ...>`** - Peek at top element in `BinaryHeap<T>`

#### Functions (Writable)

- **`for_vec_index_mut<T>(index: usize) -> WritableOptionalKeyPath<Vec<T>, T, ...>`** - Mutate element at index in `Vec<T>`
- **`for_vecdeque_index_mut<T>(index: usize) -> WritableOptionalKeyPath<VecDeque<T>, T, ...>`** - Mutate element at index in `VecDeque<T>`
- **`for_linkedlist_index_mut<T>(index: usize) -> WritableOptionalKeyPath<LinkedList<T>, T, ...>`** - Mutate element at index in `LinkedList<T>`
- **`for_hashmap_key_mut<K, V>(key: K) -> WritableOptionalKeyPath<HashMap<K, V>, V, ...>`** - Mutate value by key in `HashMap<K, V>`
- **`for_btreemap_key_mut<K, V>(key: K) -> WritableOptionalKeyPath<BTreeMap<K, V>, V, ...>`** - Mutate value by key in `BTreeMap<K, V>`
- **`for_hashset_get_mut<T>(value: T) -> WritableOptionalKeyPath<HashSet<T>, T, ...>`** - Limited mutable access (see limitations)
- **`for_btreeset_get_mut<T>(value: T) -> WritableOptionalKeyPath<BTreeSet<T>, T, ...>`** - Limited mutable access (see limitations)
- **`for_binaryheap_peek_mut<T>() -> WritableOptionalKeyPath<BinaryHeap<T>, T, ...>`** - Limited mutable access (see limitations)

#### Synchronization Primitives (Helper Functions)

**Note**: Mutex and RwLock return guards that own the lock, not references. These helper functions are provided for convenience, but direct `lock()`, `read()`, and `write()` calls are recommended for better control.

- **`lock_mutex<T>(mutex: &Mutex<T>) -> Option<MutexGuard<T>>`** - Lock a `Mutex<T>` and return guard
- **`read_rwlock<T>(rwlock: &RwLock<T>) -> Option<RwLockReadGuard<T>>`** - Read-lock an `RwLock<T>` and return guard
- **`write_rwlock<T>(rwlock: &RwLock<T>) -> Option<RwLockWriteGuard<T>>`** - Write-lock an `RwLock<T>` and return guard
- **`lock_arc_mutex<T>(arc_mutex: &Arc<Mutex<T>>) -> Option<MutexGuard<T>>`** - Lock an `Arc<Mutex<T>>` and return guard
- **`read_arc_rwlock<T>(arc_rwlock: &Arc<RwLock<T>>) -> Option<RwLockReadGuard<T>>`** - Read-lock an `Arc<RwLock<T>>` and return guard
- **`write_arc_rwlock<T>(arc_rwlock: &Arc<RwLock<T>>) -> Option<RwLockWriteGuard<T>>`** - Write-lock an `Arc<RwLock<T>>` and return guard
- **`upgrade_weak<T>(weak: &Weak<T>) -> Option<Arc<T>>`** - Upgrade a `Weak<T>` to `Arc<T>`
- **`upgrade_rc_weak<T>(weak: &Rc::Weak<T>) -> Option<Rc<T>>`** - Upgrade an `Rc::Weak<T>` to `Rc<T>`

#### Parking Lot Support (Optional Feature)

When the `parking_lot` feature is enabled:

- **`lock_parking_mutex<T>(mutex: &parking_lot::Mutex<T>) -> MutexGuard<T>`** - Lock a parking_lot `Mutex<T>`
- **`read_parking_rwlock<T>(rwlock: &parking_lot::RwLock<T>) -> RwLockReadGuard<T>`** - Read-lock a parking_lot `RwLock<T>`
- **`write_parking_rwlock<T>(rwlock: &parking_lot::RwLock<T>) -> RwLockWriteGuard<T>`** - Write-lock a parking_lot `RwLock<T>`

#### Tagged Types Support (Optional Feature)

When the `tagged` feature is enabled:

- **`for_tagged<Tag, T>() -> KeyPath<Tagged<Tag, T>, T, ...>`** - Access inner value of `Tagged<Tag, T>` (requires `Deref`)
- **`for_tagged_mut<Tag, T>() -> WritableKeyPath<Tagged<Tag, T>, T, ...>`** - Mutably access inner value of `Tagged<Tag, T>` (requires `DerefMut`)

#### Example

```rust
use rust_keypaths::containers;

let vec = vec!["a", "b", "c"];
let vec_kp = containers::for_vec_index::<&str>(1);
if let Some(value) = vec_kp.get(&vec) {
    println!("{}", value); // prints "b"
}

// Using locks
use std::sync::Mutex;
let mutex = Mutex::new(42);
if let Some(guard) = containers::lock_mutex(&mutex) {
    println!("Mutex value: {}", *guard);
}
```

## 🎯 Advanced Usage

### Deeply Nested Structures

```rust
use rust_keypaths::{OptionalKeyPath, EnumKeyPaths};

// 7 levels deep: Root -> Option -> Option -> Option -> Enum -> Option -> Option -> Box<String>
let chained_kp = OptionalKeyPath::new(|r: &Root| r.level1.as_ref())
    .then(OptionalKeyPath::new(|l1: &Level1| l1.level2.as_ref()))
    .then(OptionalKeyPath::new(|l2: &Level2| l2.level3.as_ref()))
    .then(EnumKeyPaths::for_variant(|e: &Enum| {
        if let Enum::Variant(v) = e { Some(v) } else { None }
    }))
    .then(OptionalKeyPath::new(|v: &Variant| v.level4.as_ref()))
    .then(OptionalKeyPath::new(|l4: &Level4| l4.level5.as_ref()))
    .for_box(); // Automatically unwraps Box<String> to &String
```

### Reusing Keypaths

Keypaths are `Clone`, so you can reuse them efficiently:

```rust
let kp = OptionalKeyPath::new(|s: &Struct| s.field.as_ref());
let kp_clone = kp.clone(); // Clones the keypath, not the data!

// Use both
let value1 = kp.get(&instance1);
let value2 = kp_clone.get(&instance2);
```

## 📊 Performance Benchmarks

### Benchmark Setup

All benchmarks compare keypath access vs manual unwrapping on deeply nested structures.

### Results

#### 3-Level Deep Access (omsf field)

| Method | Time | Overhead |
|--------|------|----------|
| **Keypath** | ~855 ps | 2.24x |
| **Manual Unwrap** | ~382 ps | 1.0x (baseline) |

**Overhead**: ~473 ps (2.24x slower than manual unwrapping)

#### 7-Level Deep Access with Enum (desf field)

| Method | Time | Overhead |
|--------|------|----------|
| **Keypath** | ~1.064 ns | 2.77x |
| **Manual Unwrap** | ~384 ps | 1.0x (baseline) |

**Overhead**: ~680 ps (2.77x slower than manual unwrapping)

#### Keypath Creation

| Operation | Time |
|-----------|------|
| **Create 7-level chained keypath** | ~317 ps |

#### Keypath Reuse

| Method | Time | Overhead |
|--------|------|----------|
| **Pre-created keypath** | ~820 ps | Baseline |
| **Created on-the-fly** | ~833 ps | +1.6% |

### Performance Analysis

#### Overhead Breakdown

1. **3-Level Deep**: ~2.24x overhead (~473 ps)
   - Acceptable for most use cases
   - Provides significant ergonomic benefits
   - Still in sub-nanosecond range

2. **7-Level Deep**: ~2.77x overhead (~680 ps)
   - Still in sub-nanosecond range
   - Overhead is constant regardless of depth
   - Only ~207 ps additional overhead for 4 more levels

3. **Keypath Creation**: ~317 ps
   - One-time cost
   - Negligible compared to access overhead
   - Can be created once and reused

### Memory Efficiency

- ✅ **No data cloning**: Keypaths never clone underlying data
- ✅ **Zero allocations**: Keypath operations don't allocate
- ✅ **Proper cleanup**: Memory is properly released when values are dropped

## 🧪 Testing

The library includes comprehensive tests to verify:

- No unwanted cloning of data
- Proper memory management
- Correct behavior of all keypath operations
- Chaining and composition correctness

Run tests with:

```bash
cargo test --lib
```

## 📈 Benchmarking

Run benchmarks with:

```bash
cargo bench --bench deeply_nested
```

## 🎓 Design Principles

1. **Zero-Cost Abstractions**: Keypaths compile to efficient code
2. **Type Safety**: Full compile-time type checking
3. **Composability**: Keypaths can be chained seamlessly
4. **Ergonomics**: Clean, readable API inspired by Swift
5. **Performance**: Minimal overhead for maximum convenience

## 📝 Examples

See the `examples/` directory for more comprehensive examples:

- `deeply_nested.rs` - Deeply nested structures with enum variants
- `containers.rs` - Readable access to all container types
- `writable_containers.rs` - Writable access to containers with mutation examples
- `locks_and_tagged.rs` - Synchronization primitives and tagged types (requires `tagged` and `parking_lot` features)

## 🔧 Implementation Details

### Type Inference

All container unwrapping methods (`for_box()`, `for_arc()`, `for_rc()`) automatically infer the target type from the `Deref` trait, eliminating the need for explicit type parameters.

### Memory Safety

- Keypaths only hold references, never owned data
- All operations are safe and checked at compile time
- No risk of dangling references

## 📄 License

This project is part of the rust-key-paths workspace.

## 🤝 Contributing

Contributions are welcome! Please ensure all tests pass and benchmarks are updated.

---

**Note**: All performance measurements are on a modern CPU. Actual results may vary based on hardware and compiler optimizations.

