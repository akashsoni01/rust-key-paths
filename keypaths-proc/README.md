# 🔑 KeyPaths in Rust

Key paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **KeyPath and Functional Lenses** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

---

### `rust-keypaths` + `keypaths-proc` (Recommended)
- ✅ Faster performance, better compiler optimizations
- ✅ **Write operations can be faster than manual unwrapping** at deeper nesting levels
- ✅ **Zero runtime overhead**
- ✅ **Better inlining** - Compiler can optimize more aggressively
- ✅ **Functional chains for `Arc<Mutex<T>>`/`Arc<RwLock<T>>`** - Compose keypaths through sync primitives
- ✅ **parking_lot support** - Optional feature for faster locks
- ✅ **Tokio support** - Async keypath chains through `Arc<tokio::sync::Mutex<T>>` and `Arc<tokio::sync::RwLock<T>>`

```toml
[dependencies]
rust-keypaths = "1.7.0"
keypaths-proc = "1.7.0"
```
---

## ✨ Features

- ✅ **Readable/Writable keypaths** for struct fields
- ✅ **Failable keypaths** for `Option<T>` chains (`_fr`/`_fw`)
- ✅ **Enum CasePaths** (readable and writable prisms)
- ✅ **Composition** across structs, options and enum cases
- ✅ **Iteration helpers** over collections via keypaths
- ✅ **Proc-macros**: `#[derive(Kp)]` for structs/tuple-structs and enums, `#[derive(Casepaths)]` for enums
- ✅ **Functional chains for `Arc<Mutex<T>>` and `Arc<RwLock<T>>`** - Compose-first, apply-later pattern
- ✅ **parking_lot support** - Feature-gated support for faster synchronization primitives
- ✅ **Tokio support** - Async keypath chains through `Arc<tokio::sync::Mutex<T>>` and `Arc<tokio::sync::RwLock<T>>`
- ✅ **Compile-time type safety** - Invalid keypath compositions fail at compile time, preventing runtime errors

---

## 🚀 Examples

### Deep Nested Composition with Option, Arc<RwLock>, and Enum Casepaths

This example demonstrates fluent keypath chains through `Option<T>`, `Arc<std::sync::RwLock<T>>`, and enum variants. Use `rust-key-paths` + `key-paths-derive`:

```toml
[dependencies]
rust-key-paths = "1.28"
key-paths-derive = "1.1"
```

```rust
use std::sync::Arc;
use key_paths_derive::Kp;

#[derive(Debug, Kp)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
    scfs2: Arc<std::sync::RwLock<SomeOtherStruct>>,
}

#[derive(Debug, Kp)]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Kp)]
enum SomeEnum {
    A(String),
    B(Box<DarkStruct>),
}

#[derive(Debug, Kp)]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Kp)]
struct DarkStruct {
    dsf: Option<String>,
}

fn main() {
    let mut instance = SomeComplexStruct::new();

    // Option chain: use scsf() -> .then() for nested access
    if let Some(omsf) = SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())  // Enum variant accessor from Kp derive
        .then(DarkStruct::dsf())
        .get_mut(&mut instance)
    {
        *omsf = String::from("Updated via Option chain");
    }

    // Arc<RwLock> chain: use scfs2_lock() -> .then() for lock-through access
    if let Some(omsf) = SomeComplexStruct::scfs2_lock()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf())
        .get_mut(&mut instance)
    {
        *omsf = String::from("Updated via RwLock chain");
    }

    println!("instance = {:?}", instance);
}
```

**Generated methods:**

| Field Type | Generated Methods | Usage |
|------------|-------------------|-------|
| `Option<T>` | `field()` | Kp that unwraps; chain with `.then()` |
| `Arc<std::sync::RwLock<T>>` | `field()` | Kp to container |
| `Arc<std::sync::RwLock<T>>` | `field_lock()` | LockKp; chain with `.then()` through lock |
| `Arc<std::sync::Mutex<T>>` | `field_lock()` | Same pattern for Mutex |

**Key patterns:**
- **Option fields**: `scsf()` returns a Kp; use `.then()` to chain into nested `Option` values.
- **Arc<RwLock> fields**: `scfs2_lock()` returns a LockKp that acquires the lock and chains with `.then()`.
- **Enum variants**: `SomeEnum::b()` (from Kp derive on enums) acts as a prism into the `B(Box<DarkStruct>)` variant.

**Running the example:**
```bash
cargo run --example basics_casepath
```

---

### Deep Nested Composition with Box and Enums

This example demonstrates keypath composition through deeply nested structures with `Box<T>` and enum variants:

```rust
use keypaths_proc::{Casepaths, Kp};

#[derive(Debug, Kp)]
#[Writable]
struct SomeComplexStruct {
    scsf: Box<SomeOtherStruct>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Box::new(SomeOtherStruct {
                sosf: OneMoreStruct {
                    omsf: String::from("no value for now"),
                    omse: SomeEnum::B(DarkStruct {
                        dsf: String::from("dark field"),
                    }),
                },
            }),
        }
    }
}

#[derive(Debug, Kp)]
#[Writable]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

#[derive(Debug, Casepaths)]
#[Writable]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

#[derive(Debug, Kp)]
#[Writable]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Kp)]
#[Writable]
struct DarkStruct {
    dsf: String,
}

fn main() {
    use rust_keypaths::WritableOptionalKeyPath;
    
    // Compose keypath through Box, nested structs, and enum variants
    // Using .then() method (works on stable Rust)
    let keypath = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_fw())
        .then(DarkStruct::dsf_fw());
    
    // Alternatively, use the >> operator (requires nightly feature):
    // #![feature(impl_trait_in_assoc_type)]
    // let keypath = SomeComplexStruct::scsf_fw()
    //     >> SomeOtherStruct::sosf_fw()
    //     >> OneMoreStruct::omse_fw()
    //     >> SomeEnum::b_fw()
    //     >> DarkStruct::dsf_fw();
    
    let mut instance = SomeComplexStruct::new();
    
    // Mutate deeply nested field through composed keypath
    if let Some(dsf) = keypath.get_mut(&mut instance) {
        *dsf = String::from("we can update the field of struct with the other way unlocked by keypaths");
        println!("instance = {:?}", instance);
    }
}
```

### Type Safety: Compile-Time Error Prevention

Keypaths provide **compile-time type safety** - if you try to compose keypaths that don't share the same root type, the compiler will catch the error before your code runs.

**The Rule:** When chaining keypaths with `.then()`, the `Value` type of the first keypath must match the `Root` type of the second keypath.

```rust
use keypaths_proc::Kp;

#[derive(Kp)]
#[All]
struct Person {
    name: String,
    address: Address,
}

#[derive(Kp)]
#[All]
struct Address {
    city: String,
}

#[derive(Kp)]
#[All]
struct Product {
    name: String,
}

fn main() {
    // ✅ CORRECT: Person -> Address -> city (all part of same hierarchy)
    let city_kp = Person::address_r()
        .then(Address::city_r());
    
    // ❌ COMPILE ERROR: Person::name_r() returns KeyPath<Person, String>
    //                   Product::name_r() expects Product as root, not String!
    // let invalid = Person::name_r()
    //     .then(Product::name_r());  // Error: expected `String`, found `Product`
}
```

**What happens:**
- ✅ **Valid compositions** compile successfully
- ❌ **Invalid compositions** fail at compile time with clear error messages
- 🛡️ **No runtime errors** - type mismatches are caught before execution
- 📝 **Clear error messages** - Rust compiler shows exactly what types are expected vs. found

This ensures that keypath chains are always type-safe and prevents bugs that would only be discovered at runtime.

**Running the example:**
```bash
cargo run --example type_safety_demo
```

### parking_lot Support (Default for `Mutex`/`RwLock`)

> ⚠️ **IMPORTANT**: When using the derive macro, `Mutex` and `RwLock` **default to `parking_lot`** unless you explicitly use `std::sync::Mutex` or `std::sync::RwLock`.

```toml
[dependencies]
rust-keypaths = { version = "1.9.0", features = ["parking_lot"] }
keypaths-proc = "1.9.0"
```

#### Derive Macro Generated Methods for Locks

The derive macro generates helper methods for `Arc<Mutex<T>>` and `Arc<RwLock<T>>` fields:

| Field Type | Generated Methods | Description |
|------------|-------------------|-------------|
| `Arc<Mutex<T>>` (parking_lot default) | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through parking_lot::Mutex |
| `Arc<RwLock<T>>` (parking_lot default) | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through parking_lot::RwLock |
| `Arc<std::sync::Mutex<T>>` | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through std::sync::Mutex |
| `Arc<std::sync::RwLock<T>>` | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through std::sync::RwLock |

```rust
use std::sync::Arc;
use parking_lot::RwLock;
use keypaths_proc::Kp;

#[derive(Kp)]
#[Writable]
struct Container {
    // This uses parking_lot::RwLock (default)
    data: Arc<RwLock<DataStruct>>,
    
    // This uses std::sync::RwLock (explicit)
    std_data: Arc<std::sync::RwLock<DataStruct>>,
}

#[derive(Kp)]
#[Writable]
struct DataStruct {
    name: String,
}

fn main() {
    let container = Container { /* ... */ };
    
    // Using generated _fr_at() for parking_lot (default)
    Container::data_fr_at(DataStruct::name_r())
        .get(&container, |value| {
            println!("Name: {}", value);
        });
    
    // Using generated _fw_at() for parking_lot (default)
    Container::data_fw_at(DataStruct::name_w())
        .get_mut(&container, |value| {
            *value = "New name".to_string();
        });
    
    // Using generated _fr_at() for std::sync::RwLock (explicit)
    Container::std_data_fr_at(DataStruct::name_r())
        .get(&container, |value| {
            println!("Name: {}", value);
        });
}
```

**Key advantage:** parking_lot locks **never fail** (no poisoning), so chain methods don't return `Option` for the lock operation itself.

**Running the example:**
```bash
cargo run --example parking_lot_chains --features parking_lot
cargo run --example parking_lot_nested_chain --features parking_lot
```

---

### Tokio Support (Async Locks)

> ⚠️ **IMPORTANT**: Tokio support requires the `tokio` feature and uses `tokio::sync::Mutex` and `tokio::sync::RwLock`. All operations are **async** and must be awaited.

```toml
[dependencies]
rust-keypaths = { version = "1.7.0", features = ["tokio"] }
keypaths-proc = "1.7.0"
tokio = { version = "1.38.0", features = ["sync", "rt", "rt-multi-thread", "macros"] }
```

#### Derive Macro Generated Methods for Tokio Locks

The derive macro generates helper methods for `Arc<tokio::sync::Mutex<T>>` and `Arc<tokio::sync::RwLock<T>>` fields:

| Field Type | Generated Methods | Description |
|------------|-------------------|-------------|
| `Arc<tokio::sync::Mutex<T>>` | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through tokio::sync::Mutex (async) |
| `Arc<tokio::sync::RwLock<T>>` | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through tokio::sync::RwLock (async, read/write locks) |

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use keypaths_proc::Kp;

#[derive(Kp)]
#[All]  // Generate all methods (readable, writable, owned)
struct AppState {
    user_data: Arc<tokio::sync::Mutex<UserData>>,
    config: Arc<tokio::sync::RwLock<Config>>,
    optional_cache: Option<Arc<tokio::sync::RwLock<Cache>>>,
}

#[derive(Kp)]
#[All]
struct UserData {
    name: String,
    email: String,
}

#[derive(Kp)]
#[All]
struct Config {
    api_key: String,
    timeout: u64,
}

#[derive(Kp)]
#[All]
struct Cache {
    entries: Vec<String>,
    size: usize,
}

#[tokio::main]
async fn main() {
    let state = AppState { /* ... */ };
    
    // Reading through Arc<tokio::sync::Mutex<T>> (async)
    AppState::user_data_fr_at(UserData::name_r())
        .get(&state, |name| {
            println!("User name: {}", name);
        })
        .await;
    
    // Writing through Arc<tokio::sync::Mutex<T>> (async)
    AppState::user_data_fw_at(UserData::name_w())
        .get_mut(&state, |name| {
            *name = "Bob".to_string();
        })
        .await;
    
    // Reading through Arc<tokio::sync::RwLock<T>> (async, read lock)
    AppState::config_fr_at(Config::api_key_r())
        .get(&state, |api_key| {
            println!("API key: {}", api_key);
        })
        .await;
    
    // Writing through Arc<tokio::sync::RwLock<T>> (async, write lock)
    AppState::config_fw_at(Config::timeout_w())
        .get_mut(&state, |timeout| {
            *timeout = 60;
        })
        .await;
    
    // Reading through optional Arc<tokio::sync::RwLock<T>> (async)
    if let Some(()) = AppState::optional_cache_fr()
        .chain_arc_tokio_rwlock_at_kp(Cache::size_r())
        .get(&state, |size| {
            println!("Cache size: {}", size);
        })
        .await
    {
        println!("Successfully read cache size");
    }
}
```

**Key features:**
- ✅ **Async operations**: All lock operations are async and must be awaited
- ✅ **Read/write locks**: `RwLock` supports concurrent reads with `_fr_at()` and exclusive writes with `_fw_at()`
- ✅ **Optional chaining**: Works seamlessly with `Option<Arc<tokio::sync::Mutex<T>>>` and `Option<Arc<tokio::sync::RwLock<T>>>`
- ✅ **Nested composition**: Chain through multiple levels of Tokio locks and nested structures

**Running the example:**
```bash
cargo run --example tokio_containers --features tokio
```

---

## 🌟 Showcase - Crates Using rust-key-paths

The rust-key-paths library is being used by several exciting crates in the Rust ecosystem:

- 🔍 [rust-queries-builder](https://crates.io/crates/rust-queries-builder) - Type-safe, SQL-like queries for in-memory collections
- 🎭 [rust-overture](https://crates.io/crates/rust-overture) - Functional programming utilities and abstractions  
- 🚀 [rust-prelude-plus](https://crates.io/crates/rust-prelude-plus) - Enhanced prelude with additional utilities and traits

---

## 🔗 Helpful Links & Resources

* 📘 [type-safe property paths](https://lodash.com/docs/4.17.15#get)
* 📘 [Swift KeyPath documentation](https://developer.apple.com/documentation/swift/keypath)
* 📘 [Elm Architecture & Functional Lenses](https://guide.elm-lang.org/architecture/)
* 📘 [Rust Macros Book](https://doc.rust-lang.org/book/ch19-06-macros.html)
* 📘 [Category Theory in FP (for intuition)](https://bartoszmilewski.com/2014/11/24/category-the-essence-of-composition/)

---

## 💡 Why use KeyPaths?

* Avoids repetitive `match` / `.` chains.
* Encourages **compositional design**.
* Plays well with **DDD (Domain-Driven Design)** and **Actor-based systems**.
* Useful for **reflection-like behaviors** in Rust (without unsafe).
* **High performance**: **essentially zero overhead** for deep nested writes (10 levels)!

## ⚡ Performance

KeyPaths are optimized for performance with minimal overhead. Below are benchmark results comparing **direct unwrap** vs **keypaths** for 10-level deep nested access:

| Operation | Direct Unwrap | KeyPath | Notes                         |
|-----------|---------------|---------|-------------------------------|
| **Read (10 levels)** | **384.07 ps** | **848.27 ps** | ~464 ps absolute difference   |
| **Write (10 levels)** | **19.306 ns** | **19.338 ns** | **Essentially identical!** ⚡ |

See [`benches/BENCHMARK_SUMMARY.md`](benches/BENCHMARK_SUMMARY.md) for detailed performance analysis.

### Benchmarking RwLock Operations

The library includes comprehensive benchmarks for both `parking_lot::RwLock` and `tokio::sync::RwLock` operations:

**parking_lot::RwLock benchmarks:**
```bash
cargo bench --bench rwlock_write_deeply_nested --features parking_lot
```

**Tokio RwLock benchmarks (read and write):**
```bash
cargo bench --bench rwlock_write_deeply_nested --features parking_lot,tokio
```

The benchmarks compare:
- ✅ **Keypath approach**: Using `_fr_at()` and `_fw_at()` methods for readable and writable access
- ⚙️ **Traditional approach**: Manual read/write guards with nested field access

Benchmarks include:
- Deeply nested read/write operations through `Arc<RwLock<T>>`
- Optional field access (`Option<T>`)
- Multiple sequential operations
- Both synchronous (`parking_lot`) and asynchronous (`tokio`) primitives

**Benchmark Results:**

| Operation | Keypath | Manual Guard | Overhead | Notes |
|-----------|---------|--------------|----------|-------|
| **parking_lot::RwLock - Deep Write** | 24.5 ns | 23.9 ns | 2.5% slower | Deeply nested write through `Arc<RwLock<T>>` |
| **parking_lot::RwLock - Simple Write** | 8.5 ns | 8.6 ns | **1.2% faster** ⚡ | Simple field write (`Option<i32>`) |
| **parking_lot::RwLock - Field Write** | 23.8 ns | 23.9 ns | **0.4% faster** ⚡ | Field write (`Option<String>`) |
| **parking_lot::RwLock - Multiple Writes** | 55.8 ns | 41.8 ns | 33.5% slower | Multiple sequential writes (single guard faster) |
| **tokio::sync::RwLock - Deep Read** | 104.8 ns | 104.6 ns | 0.2% slower | Deeply nested async read |
| **tokio::sync::RwLock - Deep Write** | 124.8 ns | 124.1 ns | 0.6% slower | Deeply nested async write |
| **tokio::sync::RwLock - Simple Write** | 103.8 ns | 105.0 ns | **1.2% faster** ⚡ | Simple async field write |
| **tokio::sync::RwLock - Field Read** | 103.3 ns | 103.2 ns | 0.1% slower | Simple async field read |
| **tokio::sync::RwLock - Field Write** | 125.7 ns | 124.6 ns | 0.9% slower | Simple async field write |

**Key findings:**
- ✅ **parking_lot::RwLock**: Keypaths show **essentially identical performance** (0-2.5% overhead) for single operations
- ✅ **tokio::sync::RwLock**: Keypaths show **essentially identical performance** (0-1% overhead) for async operations
- ⚡ **Simple operations**: Keypaths can be **faster** than manual guards in some cases (1-2% improvement)
- ⚠️ **Multiple writes**: Manual single guard is faster (33% overhead) - use single guard for multiple operations
- 🎯 **Type safety**: Minimal performance cost for significant type safety and composability benefits

**Detailed Analysis:**
- For detailed performance analysis, see [`benches/BENCHMARK_SUMMARY.md`](benches/BENCHMARK_SUMMARY.md)
- For performance optimization details, see [`benches/PERFORMANCE_ANALYSIS.md`](benches/PERFORMANCE_ANALYSIS.md)
- For complete benchmark results, see [`benches/BENCHMARK_RESULTS.md`](benches/BENCHMARK_RESULTS.md)

---

## 🔄 Comparison with Other Lens Libraries
| Feature | rust-keypaths | keypath | pl-lens | lens-rs |
|---------|--------------|---------|---------|---------|
| **Struct Field Access** | ✅ Readable/Writable | ✅ Readable/Writable | ✅ Readable/Writable | ✅ Partial |
| **Option<T> Chains** | ✅ Built-in (`_fr`/`_fw`) | ❌ Manual composition | ❌ Manual composition | ❌ Manual |
| **Enum Case Paths** | ✅ Built-in (CasePaths) | ❌ Not supported | ❌ Not supported | ❌ Limited |
| **Tuple Structs** | ✅ Full support | ⚠️ Unknown | ❌ Not supported | ❌ Not supported |
| **Composition** | ✅ `.then()` chaining | ⚠️ Less ergonomic | ⚠️ Manual | ⚠️ Complex |
| **Result<T, E>** | ✅ Built-in support | ❌ Not supported | ❌ Not supported | ❌ Not supported |
| **Mutex/RwLock** | ✅ Built-in (`with_mutex`, etc.) | ❌ Not supported | ❌ Not supported | ❌ Not supported |
| **Arc/Box/Rc** | ✅ Built-in support | ⚠️ Unknown | ⚠️ Limited | ⚠️ Limited |
| **Collections** | ✅ Vec, HashMap, HashSet, etc. | ❌ Not supported | ❌ Not supported | ❌ Not supported |
| **Derive Macros** | ✅ `#[derive(Kp)]`, `#[derive(Casepaths)]` | ✅ `#[derive(Keypath)]` | ✅ `#[derive(Lenses)]` | ⚠️ Limited |
| **Deep Nesting** | ✅ Works seamlessly | ⚠️ May require workarounds | ❌ Requires workarounds | ❌ Complex |
| **Type Safety** | ✅ Full compile-time checks | ✅ Good | ✅ Good | ⚠️ Moderate |
| **Performance** | ✅ Optimized (1.46x overhead reads, near-zero writes) | ⚠️ Unknown | ⚠️ Unknown | ⚠️ Unknown |
| **Readable Keypaths** | ✅ `KeyPath` | ✅ Supported | ✅ `RefLens` | ⚠️ Partial |
| **Writable Keypaths** | ✅ `WritableKeyPath` | ✅ Supported | ✅ `Lens` | ⚠️ Partial |
| **Failable Readable** | ✅ `OptionalKeyPath` | ❌ Manual | ❌ Manual | ❌ Manual |
| **Failable Writable** | ✅ `WritableOptionalKeyPath` | ❌ Manual | ❌ Manual | ❌ Manual |
| **Zero-cost Abstractions** | ✅ | ⚠️ Unknown | ⚠️ Depends | ⚠️ Depends |
| **Swift KeyPath-like API** | ✅ Inspired by Swift | ⚠️ Partial | ❌ No | ❌ No |
| **Container Methods** | ✅ `with_mutex`, `with_rwlock`, `with_arc`, etc. | ❌ Not supported | ❌ Not supported | ❌ Not supported |
| **Iteration Helpers** | ✅ `iter()`, `iter_mut()` | ❌ Not supported | ❌ Not supported | ❌ Not supported |
| **Derivable References** | ✅ Full support | ✅ Full support | ❌ Not supported | ❌ Not supported |
| **Active Maintenance** | ✅ Active | ⚠️ Unknown | ⚠️ Unknown | ⚠️ Unknown |

### Key Advantages of rust-keypaths

1. **✅ Native Option support**: Built-in failable keypaths (`_fr`/`_fw`) that compose seamlessly through `Option<T>` chains (unlike keypath, pl-lens, and lens-rs which require manual composition)
2. **✅ Enum CasePaths**: First-class support for enum variant access (prisms) with `#[derive(Casepaths)]` (unique feature not found in keypath, pl-lens, or lens-rs)
3. **✅ Container types**: Built-in support for `Result`, `Mutex`, `RwLock`, `Arc`, `Rc`, `Box`, and all standard collections (comprehensive container support unmatched by alternatives)
4. **✅ Functional chains for sync primitives**: Compose keypaths through `Arc<Mutex<T>>` and `Arc<RwLock<T>>` with a clean, functional API
5. **✅ parking_lot support**: Feature-gated support for faster `parking_lot::Mutex` and `parking_lot::RwLock`
6. **✅ Zero-cost abstractions**: Minimal overhead (1.46x for reads, near-zero for writes) - benchmarked and optimized
7. **✅ Comprehensive derive macros**: Automatic generation for structs (named and tuple), enums, and all container types
8. **✅ Swift-inspired API**: Familiar API for developers coming from Swift's KeyPath system with `.then()` composition
9. **✅ Deep composition**: Works seamlessly with 10+ levels of nesting without workarounds (tested and verified)
10. **✅ Type-safe composition**: Full compile-time type checking with `.then()` method
11. **✅ Active development**: Regularly maintained with comprehensive feature set and documentation

---

## 🛠 Roadmap

- [x] Inspired by Lenses: [Compositional Data Access And Manipulation](https://www.youtube.com/watch?v=dxGaKn4REaY&list=LL&index=7)
- [x] Compose across structs, options and enum cases
- [x] Derive macros for automatic keypath generation (`Keypaths`, `Keypaths`, `Casepaths`)
- [x] Optional chaining with failable keypaths
- [x] Smart pointer adapters (`.for_arc()`, `.for_box()`, `.for_rc()`)
- [x] Container support for `Result`, `Mutex`, `RwLock`, `Weak`, and collections
- [x] Helper derive macros (`ReadableKeypaths`, `WritableKeypaths`)
- [x] Functional chains for `Arc<Mutex<T>>` and `Arc<RwLock<T>>`
- [x] `parking_lot` support for faster synchronization primitives
- [x] **Tokio support** for async keypath chains through `Arc<tokio::sync::Mutex<T>>` and `Arc<tokio::sync::RwLock<T>>`
- [ ] Derive macros for complex multi-field enum variants
---

## 📜 License

* Mozilla Public License 2.0