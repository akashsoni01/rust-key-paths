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
* **High performance**: Only 1.46x overhead for reads, **93.6x faster** when reused, and **essentially zero overhead** for deep nested writes (10 levels)!

## ⚡ Performance

KeyPaths are optimized for performance with minimal overhead. Below are benchmark results comparing **direct unwrap** vs **keypaths** for 10-level deep nested access:

| Operation | Direct Unwrap | KeyPath | Overhead | Notes |
|-----------|---------------|---------|----------|-------|
| **Read (10 levels)** | **384.07 ps** | **848.27 ps** | **2.21x** | ~464 ps absolute difference |
| **Write (10 levels)** | **19.306 ns** | **19.338 ns** | **1.002x** | **Essentially identical!** ⚡ |

See [`benches/BENCHMARK_SUMMARY.md`](benches/BENCHMARK_SUMMARY.md) for detailed performance analysis.

---

## 🔄 Comparison with Other Lens Libraries

### Limitations of lens-rs, pl-lens, and keypath

Both **lens-rs**, **pl-lens** (Plausible Labs), and **keypath** have several limitations when working with Rust's type system, especially for nested structures:

#### keypath limitations:
1. **❌ No enum variant support**: No built-in support for enum case paths (prisms)
2. **❌ No Option<T> chain support**: Requires manual `.and_then()` composition for Option types
3. **❌ Limited container support**: No built-in support for `Result<T, E>`, `Mutex<T>`, `RwLock<T>`, or collection types
4. **❌ No failable keypaths**: Cannot easily compose through Option chains with built-in methods
5. **❌ No writable failable keypaths**: Missing support for composing writable access through Option chains
6. **❌ Limited composition API**: Less ergonomic composition compared to `.then()` chaining
7. **⚠️ Maintenance status**: May have limited active maintenance

#### pl-lens limitations:
1. **❌ No support for `Option<Struct>` nested compositions**: The `#[derive(Lenses)]` macro fails to generate proper lens types for nested structs wrapped in `Option<T>`, requiring manual workarounds
2. **❌ Limited enum support**: No built-in support for enum variant case paths (prisms)
3. **❌ No automatic failable composition**: Requires manual composition through `.and_then()` chains for Option types
4. **❌ Limited container support**: No built-in support for `Result<T, E>`, `Mutex<T>`, `RwLock<T>`, or collection types
5. **❌ Named fields only**: The derive macro only works with structs that have named fields, not tuple structs
6. **❌ No writable failable keypaths**: Cannot compose writable access through Option chains easily
7. **❌ Type system limitations**: The lens composition through Option types requires manual function composition, losing type safety

#### lens-rs limitations:
1. **❌ Different API design**: Uses a different lens abstraction that doesn't match Rust's ownership model as well
2. **❌ Limited ecosystem**: Less mature and fewer examples/documentation
3. **❌ Composition complexity**: More verbose composition syntax

### Feature Comparison Table

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
4. **✅ Zero-cost abstractions**: Static dispatch with minimal overhead (1.46x for reads, near-zero for writes) - benchmarked and optimized
5. **✅ Comprehensive derive macros**: Automatic generation for structs (named and tuple), enums, and all container types
6. **✅ Swift-inspired API**: Familiar API for developers coming from Swift's KeyPath system with `.then()` composition
7. **✅ Deep composition**: Works seamlessly with 10+ levels of nesting without workarounds (tested and verified)
8. **✅ Type-safe composition**: Full compile-time type checking with `.then()` method
9. **✅ Active development**: Regularly maintained with comprehensive feature set and documentation

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
- [] Derive macros for complex multi-field enum variants
---

## 📜 License

* Mozilla Public License 2.0