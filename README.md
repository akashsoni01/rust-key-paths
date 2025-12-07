# 🔑 KeyPaths & CasePaths in Rust

Key paths and case paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **Swift's KeyPath / CasePath** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

---

## 🚀 New: Static Dispatch Implementation

**We now provide two implementations:**

### Primary: `rust-keypaths` + `keypaths-proc` (Recommended)
- ✅ **Static dispatch** - Faster performance, better compiler optimizations
- ✅ **Write operations can be faster than manual unwrapping** at deeper nesting levels
- ✅ **Zero runtime overhead** - No dynamic dispatch costs
- ✅ **Better inlining** - Compiler can optimize more aggressively

```toml
[dependencies]
rust-keypaths = "1.0.2"
keypaths-proc = "1.0.1"
```

### Legacy: `key-paths-core` + `key-paths-derive` (v1.6.0)
- ⚠️ **Dynamic dispatch** - Use only if you need:
  - `Send + Sync` bounds for multithreaded scenarios
  - Dynamic dispatch with trait objects
  - Compatibility with existing code using the enum-based API

```toml
[dependencies]
key-paths-core = "1.6.0"  # Use 1.6.0 for dynamic dispatch
key-paths-derive = "1.1.0"
```
---

## ✨ Features

- ✅ **Readable/Writable keypaths** for struct fields
- ✅ **Failable keypaths** for `Option<T>` chains (`_fr`/`_fw`)
- ✅ **Enum CasePaths** (readable and writable prisms)
- ✅ **Composition** across structs, options and enum cases
- ✅ **Iteration helpers** over collections via keypaths
- ✅ **Proc-macros**: `#[derive(Keypaths)]` for structs/tuple-structs and enums, `#[derive(Casepaths)]` for enums

---

## 📦 Installation

### Recommended: Static Dispatch (rust-keypaths)

```toml
[dependencies]
rust-keypaths = "1.0.0"
keypaths-proc = "1.0.0"
```

### Legacy: Dynamic Dispatch (key-paths-core)

```toml
[dependencies]
key-paths-core = "1.6.0"  # Use 1.6.0 for dynamic dispatch
key-paths-derive = "1.1.0"
```

### API Differences

**rust-keypaths (Static Dispatch):**
```rust
use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath};

let kp = KeyPath::new(|s: &Struct| &s.field);
let opt_kp = OptionalKeyPath::new(|s: &Struct| s.opt_field.as_ref());
let writable_kp = WritableKeyPath::new(|s: &mut Struct| &mut s.field);
```

**key-paths-core (Dynamic Dispatch):**
```rust
use key_paths_core::KeyPaths;

let kp = KeyPaths::readable(|s: &Struct| &s.field);
let opt_kp = KeyPaths::failable_readable(|s: &Struct| s.opt_field.as_ref());
let writable_kp = KeyPaths::writable(|s: &mut Struct| &mut s.field);
```

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
    let keypath = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_fw())
        .then(DarkStruct::dsf_fw());
    
    let mut instance = SomeComplexStruct::new();
    
    // Mutate deeply nested field through composed keypath
    if let Some(dsf) = keypath.get_mut(&mut instance) {
        *dsf = String::from("we can update the field of struct with the other way unlocked by keypaths");
        println!("instance = {:?}", instance);
    }
}
```

Run it yourself:

```bash
cargo run --example box_keypath
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
* **High performance**: Only 1.43x overhead for reads, **98.3x faster** when reused!

## ⚡ Performance

KeyPaths are optimized for performance with minimal overhead. Below are benchmark results comparing **direct unwrap** vs **keypaths** for different operations (all times in picoseconds):

| Operation | Direct Unwrap | KeyPath | Overhead | Notes |
|-----------|---------------|---------|----------|-------|
| **Read (3 levels)** | 379.28 ps | 820.81 ps | 2.16x | ~441 ps absolute difference |
| **Write (3 levels)** | 377.04 ps | 831.65 ps | 2.21x | ~454 ps absolute difference |
| **Deep Read (5 levels, no enum)** | 379.37 ps | 926.83 ps | 2.44x | Pure Option chain |
| **Deep Read (5 levels, with enum)** | 384.10 ps | 1,265.3 ps | 3.29x | Includes enum case path + Box adapter |
| **Write (5 levels, with enum)** | 385.23 ps | 1,099.7 ps | 2.85x | Writable with enum case path |
| **Keypath Creation** | N/A | 325.60 ps | N/A | One-time cost, negligible |
| **Reused Read (100x)** | 36,808 ps | 36,882 ps | 1.00x | **Near-zero overhead when reused!** ⚡ |
| **Pre-composed** | N/A | 848.26 ps | N/A | 1.45x faster than on-the-fly |
| **Composed on-the-fly** | N/A | 1,234.0 ps | N/A | Composition overhead |

**Key Findings:**
- ✅ **Reused keypaths** have near-zero overhead (1.00x vs baseline)
- ✅ **Pre-composition** provides 1.45x speedup over on-the-fly composition
- ✅ **Write operations** show similar overhead to reads (2.21x vs 2.16x)
- ✅ **Deep nesting** with enums has higher overhead (3.29x) but remains manageable
- ✅ Single-use overhead is minimal (~400-500 ps for typical operations)

**Best Practices:**
- **Pre-compose keypaths** before loops/iterations (1.45x faster)
- **Reuse keypaths** whenever possible (near-zero overhead)
- Single-use overhead is negligible (< 1 ns for reads)
- Deep nested paths with enums have higher overhead but still manageable

See [`benches/BENCHMARK_SUMMARY.md`](benches/BENCHMARK_SUMMARY.md) for detailed performance analysis.

---

## 🛠 Roadmap

- [x] Compose across structs, options and enum cases
- [x] Derive macros for automatic keypath generation (`Keypaths`, `Keypaths`, `Casepaths`)
- [x] Optional chaining with failable keypaths
- [x] Smart pointer adapters (`.for_arc()`, `.for_box()`, `.for_rc()`)
- [x] Container support for `Result`, `Mutex`, `RwLock`, `Weak`, and collections
- [x] Helper derive macros (`ReadableKeypaths`, `WritableKeypaths`)
- [] Derive macros for complex multi-field enum variants
---

## 📜 License

* Mozilla Public License 2.0