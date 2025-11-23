# 🔑 KeyPaths & CasePaths in Rust

Key paths and case paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **Swift’s KeyPath / CasePath** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

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

```toml
[dependencies]
key-paths-core = "1.7.0"
key-paths-derive = "1.1.0"
```

## 🎯 Choose Your Macro

### `#[derive(Keypath)]` - Simple & Beginner-Friendly
- **One method per field**: `field_name()` 
- **Smart keypath selection**: Automatically chooses readable or failable readable based on field type
- **No option chaining**: Perfect for beginners and simple use cases
- **Clean API**: Just call `Struct::field_name()` and you're done!

```rust
use key_paths_derive::Keypath;
#[derive(Keypath)]
struct User {
    name: String,           // -> User::name() returns readable keypath
    email: Option<String>,  // -> User::email() returns failable readable keypath
}

// Usage
let user = User { name: "Alice".into(), email: Some("alice@example.com".into()) };
let name_keypath = User::name();
let email_keypath = User::email();
let name = name_keypath.get(&user);        // Some("Alice")
let email = email_keypath.get(&user);      // Some("alice@example.com")
```

### `#[derive(Keypaths)]` - Advanced & Feature-Rich
- **Multiple methods per field**: `field_r()`, `field_w()`, `field_fr()`, `field_fw()`, `field_o()`, `field_fo()`
- **Full control**: Choose exactly which type of keypath you need
- **Option chaining**: Perfect for intermediate and advanced developers
- **Comprehensive**: Supports all container types and access patterns

```rust
use key_paths_derive::Keypaths;

#[derive(Keypaths)]
#[Readable] // Default scope for every field is Readable, others Writable, Owned and All.
struct User {
    name: String,
    email: Option<String>,
}

// Usage - you choose the exact method
let user = User { name: "Alice".into(), email: Some("alice@example.com".into()) };
let name_keypath = User::name_r();
let email_keypath = User::email_fr();
let name = name_keypath.get(&user);      // Some("Alice") - readable
let email = email_keypath.get(&user);   // Some("alice@example.com") - failable readable
```
---

### Widely used - Deeply nested struct
```rust
use key_paths_derive::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
#[Writable] // Default scope for every field is Readable, others Writable, Owned and All.
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
}


#[derive(Debug, Keypaths)]
#[Writable] // Default scope for every field is Readable, others Writable, Owned and All.
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Keypaths)]
#[Writable] // Default scope for every field is Readable, others Writable, Owned and All.
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Casepaths)]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

#[derive(Debug, Keypaths)]
#[Writable] // Default scope for every field is Readable, others Writable, Owned and All.
struct DarkStruct {
    dsf: Option<String>,
}


impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    })),
                }),
            }),
        }
    }
}


fn main() {
    let dsf_kp = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw());

    let mut instance = SomeComplexStruct::new();
    
    if let Some(omsf) = dsf_kp.get_mut(&mut instance) {
        *omsf = String::from("This is changed 🖖🏿");
        println!("instance = {:?}", instance);

    }
}
```

**Recommendation**: Start with `#[derive(Keypath)]` for simplicity, upgrade to `#[derive(Keypaths)]` when you need more control!

### Keypath vs Keypaths - When to Use Which?

| Feature | `#[derive(Keypath)]` | `#[derive(Keypaths)]` |
|---------|---------------------|----------------------|
| **API Complexity** | Simple - one method per field | Advanced - multiple methods per field |
| **Learning Curve** | Beginner-friendly | Requires understanding of keypath types |
| **Container Support** | Basic containers only | Full container support including `Result`, `Mutex`, `RwLock`, `Wea****k` |
| **Option Chaining** | No - smart selection only | Yes - full control over failable vs non-failable |
| **Writable Access** | Limited | Full writable support |
| **Use Case** | Simple field access, beginners | Complex compositions, advanced users |

**When to use `Keypath`:**
- You're new to keypaths
- You want simple, clean field access
- You don't need complex option chaining
- You're working with basic types

**When to use `Keypaths`:**
- You need full control over keypath types
- You're composing complex nested structures
- You need writable access to fields
- You're working with advanced container types

---

## 🚀 Examples

See `examples/` for many runnable samples. Below are a few highlights.

### Quick Start - Simple Keypaths Usage
```rust
use key_paths_derive::Keypath;

#[derive(Keypath)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

fn main() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: Some("alice@example.com".to_string()),
    };

    // Access fields using keypaths
    let name_keypath = User::name();
    let age_keypath = User::age();
    let email_keypath = User::email();
    
    let name = name_keypath.get(&user);        // Some("Alice")
    let age = age_keypath.get(&user);          // Some(30)
    let email = email_keypath.get(&user);      // Some("alice@example.com")

    println!("Name: {:?}", name);
    println!("Age: {:?}", age);
    println!("Email: {:?}", email);
}
```
---

### Attribute-Scoped Generation (NEW!)
Struct-level and field-level attributes let you control which keypath methods are emitted. The default scope is `Readable`, but you can opt into `Writable`, `Owned`, or `All` on individual fields or the entire type.

```rust
use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Clone, Debug, Keypaths)]
#[Readable] // default scope for every field
struct Account {
    nickname: Option<String>, // inherits #[Readable]
    #[Writable]
    balance: i64, // writable accessors only
    #[Owned]
    recovery_token: Option<String>, // owned accessors only
}

fn main() {
    let mut account = Account {
        nickname: Some("ace".into()),
        balance: 1_000,
        recovery_token: Some("token-123".into()),
    };

    let nickname = Account::nickname_fr().get(&account);
    let owned_token = Account::recovery_token_fo().get_failable_owned(account.clone());

    if let Some(balance) = Account::balance_w().get_mut(&mut account) {
        *balance += 500;
    }

    println!("nickname: {:?}", nickname);
    println!("balance: {}", account.balance);
    println!("recovery token: {:?}", owned_token);
}
```

Run it yourself:

```
cargo run --example attribute_scopes
```

---

## 📦 Container Adapters & References (NEW!)

KeyPaths now support smart pointers, containers, and references via adapter methods:

### Smart Pointer Adapters

Use `.for_arc()`, `.for_box()`, or `.for_rc()` to adapt keypaths for wrapped types:

```rust
use key_paths_derive::Keypaths;
use std::sync::Arc;

#[derive(Keypath)]
struct Product {
    name: String,
    price: f64,
}

let products: Vec<Arc<Product>> = vec![
    Arc::new(Product { name: "Laptop".into(), price: 999.99 }),
];

// Adapt keypath to work with Arc<Product>
let price_path = Product::price().for_arc();

let affordable: Vec<&Arc<Product>> = products
    .iter()
    .filter(|p| price_path.get(p).map_or(false, |&price| price < 100.0))
    .collect();
```

### Reference Support

Use `.get_ref()` and `.get_mut_ref()` for collections of references:

```rust
use key_paths_derive::Keypaths;

#[derive(Keypath)]
struct Product {
    name: String,
    price: f64,
}

let products: Vec<&Product> = hashmap.values().collect();
let price_path = Product::price();

for product_ref in &products {
    if let Some(&price) = price_path.get_ref(product_ref) {
        println!("Price: ${}", price);
    }
}
```

**Supported Adapters:**
- `.for_arc()` - Works with `Arc<T>` (read-only)
- `.for_box()` - Works with `Box<T>` (read & write)
- `.for_rc()` - Works with `Rc<T>` (read-only)
- `.get_ref()` - Works with `&T` references
- `.get_mut_ref()` - Works with `&mut T` references

**Examples:**
- [`examples/container_adapters.rs`](examples/container_adapters.rs) - Smart pointer usage
- [`examples/reference_keypaths.rs`](examples/reference_keypaths.rs) - Reference collections
- [`key-paths-core/examples/container_adapter_test.rs`](key-paths-core/examples/container_adapter_test.rs) - Test suite

**Documentation:** See [`CONTAINER_ADAPTERS.md`](CONTAINER_ADAPTERS.md) and [`REFERENCE_SUPPORT.md`](REFERENCE_SUPPORT.md)

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

KeyPaths are optimized for performance with minimal overhead:

| Operation | Overhead | Notes |
|-----------|----------|-------|
| **Read (3 levels)** | 1.43x (43% slower) | Only ~170 ps absolute difference |
| **Write (3 levels)** | 10.8x slower | ~3.8 ns absolute difference |
| **Reused Read** | **98.3x faster** ⚡ | Primary benefit - reuse keypaths! |
| **Pre-composed** | Optimal | 390x faster than on-the-fly composition |

**Key Optimizations Applied:**
- ✅ Direct `match` composition (Phase 1) - eliminated `and_then` overhead
- ✅ `Rc` instead of `Arc` - faster for single-threaded use
- ✅ Aggressive inlining - `#[inline(always)]` on hot paths

**Best Practices:**
- **Pre-compose keypaths** before loops/iterations
- **Reuse keypaths** whenever possible to get the 98x speedup
- Single-use overhead is negligible (< 1 ns for reads)

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