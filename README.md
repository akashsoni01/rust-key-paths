# 🔑 KeyPaths in Rust

Key paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **Swift's KeyPath ** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

---

## 🚀 New: Static Dispatch Implementation
### `rust-keypaths` + `keypaths-proc` (Recommended)
- ✅ **Static dispatch** - Faster performance, better compiler optimizations
- ✅ **Write operations can be faster than manual unwrapping** at deeper nesting levels
- ✅ **Zero runtime overhead** - No dynamic dispatch costs
- ✅ **Better inlining** - Compiler can optimize more aggressively
- ✅ **Functional chains for `Arc<Mutex<T>>`/`Arc<RwLock<T>>`** - Compose keypaths through sync primitives
- ✅ **parking_lot support** - Optional feature for faster locks

```toml
[dependencies]
rust-keypaths = "1.3.0"
keypaths-proc = "1.3.0"
```
---

## ✨ Features

- ✅ **Readable/Writable keypaths** for struct fields
- ✅ **Failable keypaths** for `Option<T>` chains (`_fr`/`_fw`)
- ✅ **Enum CasePaths** (readable and writable prisms)
- ✅ **Composition** across structs, options and enum cases
- ✅ **Iteration helpers** over collections via keypaths
- ✅ **Proc-macros**: `#[derive(Keypaths)]` for structs/tuple-structs and enums, `#[derive(Casepaths)]` for enums
- ✅ **Functional chains for `Arc<Mutex<T>>` and `Arc<RwLock<T>>`** - Compose-first, apply-later pattern
- ✅ **parking_lot support** - Feature-gated support for faster synchronization primitives

---

## 🚀 Examples

### Deep Nested Composition with Box and Enums

This example demonstrates keypath composition through deeply nested structures with `Box<T>` and enum variants:

```rust
use keypaths_proc::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
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

#[derive(Debug, Keypaths)]
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

#[derive(Debug, Keypaths)]
#[Writable]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Keypaths)]
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
        .then(SomeEnum::b_case_fw())
        .then(DarkStruct::dsf_fw());
    
    // Alternatively, use the >> operator (requires nightly feature):
    // #![feature(impl_trait_in_assoc_type)]
    // let keypath = SomeComplexStruct::scsf_fw()
    //     >> SomeOtherStruct::sosf_fw()
    //     >> OneMoreStruct::omse_fw()
    //     >> SomeEnum::b_case_fw()
    //     >> DarkStruct::dsf_fw();
    
    let mut instance = SomeComplexStruct::new();
    
    // Mutate deeply nested field through composed keypath
    if let Some(dsf) = keypath.get_mut(&mut instance) {
        *dsf = String::from("we can update the field of struct with the other way unlocked by keypaths");
        println!("instance = {:?}", instance);
    }
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
        .then_arc_mutex_at_kp(DataStruct::name_r())
        .get(&container, |value| {
            println!("Name: {}", value);
        });
    
    // Write through Arc<Mutex<T>>
    Container::mutex_data_r()
        .then_arc_mutex_writable_at_kp(DataStruct::name_w())
        .get_mut(&container, |value| {
            *value = "New name".to_string();
        });
    
    // Read through Arc<RwLock<T>> (read lock)
    Container::rwlock_data_r()
        .then_arc_rwlock_at_kp(DataStruct::name_r())
        .get(&container, |value| {
            println!("Name: {}", value);
        });
    
    // Write through Arc<RwLock<T>> (write lock)
    Container::rwlock_data_r()
        .then_arc_rwlock_writable_at_kp(DataStruct::name_w())
        .get_mut(&container, |value| {
            *value = "New name".to_string();
        });
}
```

**Running the example:**
```bash
cargo run --example readable_keypaths_new_containers_test
```

### parking_lot Support

For faster synchronization with `parking_lot::Mutex` and `parking_lot::RwLock`:

```toml
[dependencies]
rust-keypaths = { version = "1.3.0", features = ["parking_lot"] }
```

```rust
use std::sync::Arc;
use parking_lot::{Mutex as ParkingMutex, RwLock as ParkingRwLock};

// Chain methods for parking_lot (feature-gated)
Container::parking_mutex_data_r()
    .then_arc_parking_mutex_at_kp(DataStruct::name_r())
    .get(&container, |value| {
        println!("Name: {}", value);
    });

Container::parking_rwlock_data_r()
    .then_arc_parking_rwlock_writable_at_kp(DataStruct::name_w())
    .get_mut(&container, |value| {
        *value = "New name".to_string();
    });
```

**Key advantage:** parking_lot locks **never fail** (no poisoning), so chain methods don't return `Option` for the lock operation itself.

**Running the example:**
```bash
cargo run --example parking_lot_chains --features parking_lot
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

---

## 🔄 Comparison with Other Lens Libraries
| Feature | rust-keypaths | keypath | pl-lens | lens-rs |
|---------|---------------|---------|---------|---------|
| **Struct Field Access** | ✅ Readable/Writable | ✅ Readable/Writable | ✅ Readable/Writable | ✅ Partial |
| **Option<T> Chains** | ✅ Built-in (`_fr`/`_fw`) | ❌ Manual composition | ❌ Manual composition | ❌ Manual |
| **Enum Case Paths** | ✅ Built-in (CasePaths) | ❌ Not supported | ❌ Not supported | ❌ Limited |
| **Tuple Structs** | ✅ Full support | ⚠️ Unknown | ❌ Not supported | ❌ Not supported |
| **Composition** | ✅ `.then()` chaining | ⚠️ Less ergonomic | ⚠️ Manual | ⚠️ Complex |
| **Result<T, E>** | ✅ Built-in support | ❌ Not supported | ❌ Not supported | ❌ Not supported |
| **Mutex/RwLock** | ✅ Built-in (`with_mutex`, etc.) | ❌ Not supported | ❌ Not supported | ❌ Not supported |
| **Arc/Box/Rc** | ✅ Built-in support | ⚠️ Unknown | ⚠️ Limited | ⚠️ Limited |
| **Collections** | ✅ Vec, HashMap, HashSet, etc. | ❌ Not supported | ❌ Not supported | ❌ Not supported |
| **Derive Macros** | ✅ `#[derive(Keypaths)]`, `#[derive(Casepaths)]` | ✅ `#[derive(Keypath)]` | ✅ `#[derive(Lenses)]` | ⚠️ Limited |
| **Deep Nesting** | ✅ Works seamlessly | ⚠️ May require workarounds | ❌ Requires workarounds | ❌ Complex |
| **Type Safety** | ✅ Full compile-time checks | ✅ Good | ✅ Good | ⚠️ Moderate |
| **Performance** | ✅ Optimized (1.46x overhead reads, near-zero writes) | ⚠️ Unknown | ⚠️ Unknown | ⚠️ Unknown |
| **Readable Keypaths** | ✅ `KeyPath` | ✅ Supported | ✅ `RefLens` | ⚠️ Partial |
| **Writable Keypaths** | ✅ `WritableKeyPath` | ✅ Supported | ✅ `Lens` | ⚠️ Partial |
| **Failable Readable** | ✅ `OptionalKeyPath` | ❌ Manual | ❌ Manual | ❌ Manual |
| **Failable Writable** | ✅ `WritableOptionalKeyPath` | ❌ Manual | ❌ Manual | ❌ Manual |
| **Zero-cost Abstractions** | ✅ Static dispatch | ⚠️ Unknown | ⚠️ Depends | ⚠️ Depends |
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
6. **✅ Zero-cost abstractions**: Static dispatch with minimal overhead (1.46x for reads, near-zero for writes) - benchmarked and optimized
7. **✅ Comprehensive derive macros**: Automatic generation for structs (named and tuple), enums, and all container types
8. **✅ Swift-inspired API**: Familiar API for developers coming from Swift's KeyPath system with `.then()` composition
9. **✅ Deep composition**: Works seamlessly with 10+ levels of nesting without workarounds (tested and verified)
10. **✅ Type-safe composition**: Full compile-time type checking with `.then()` method
11. **✅ Active development**: Regularly maintained with comprehensive feature set and documentation

### Example: Why rust-keypaths is Better for Nested Option Chains

**pl-lens approach** (requires manual work):
```rust
// Manual composition - verbose and error-prone
let result = struct_instance
    .level1_field
    .as_ref()
    .and_then(|l2| l2.level2_field.as_ref())
    .and_then(|l3| l3.level3_field.as_ref())
    // ... continues for 10 levels
```

**rust-keypaths approach** (composable and type-safe):
```rust
// Clean composition - type-safe and reusable
let keypath = Level1::level1_field_fr()
    .then(Level2::level2_field_fr())
    .then(Level3::level3_field_fr())
    // ... continues for 10 levels
    .then(Level10::level10_field_fr());
    
let result = keypath.get(&instance); // Reusable, type-safe, fast
```

---

## 🛠 Roadmap

- [x] Compose across structs, options and enum cases
- [x] Derive macros for automatic keypath generation (`Keypaths`, `Keypaths`, `Casepaths`)
- [x] Optional chaining with failable keypaths
- [x] Smart pointer adapters (`.for_arc()`, `.for_box()`, `.for_rc()`)
- [x] Container support for `Result`, `Mutex`, `RwLock`, `Weak`, and collections
- [x] Helper derive macros (`ReadableKeypaths`, `WritableKeypaths`)
- [x] Functional chains for `Arc<Mutex<T>>` and `Arc<RwLock<T>>`
- [x] `parking_lot` support for faster synchronization primitives
- [ ] Derive macros for complex multi-field enum variants
---

## 📜 License

* Mozilla Public License 2.0