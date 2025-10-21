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
key-paths-core = "1.0.5"
key-paths-derive = "0.9"
```

## 🎯 Choose Your Macro

### `#[derive(Keypaths)]` - Simple & Beginner-Friendly
- **One method per field**: `field_name()` 
- **Smart keypath selection**: Automatically chooses readable or failable readable based on field type
- **No option chaining**: Perfect for beginners and simple use cases
- **Clean API**: Just call `Struct::field_name()` and you're done!

```rust
use key_paths_derive::Keypaths;

#[derive(Keypaths)]
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

**Recommendation**: Start with `#[derive(Keypaths)]` for simplicity, upgrade to `#[derive(Keypaths)]` when you need more control!

### Keypaths vs Keypaths - When to Use Which?

| Feature | `#[derive(Keypaths)]` | `#[derive(Keypaths)]` |
|---------|---------------------|----------------------|
| **API Complexity** | Simple - one method per field | Advanced - multiple methods per field |
| **Learning Curve** | Beginner-friendly | Requires understanding of keypath types |
| **Container Support** | Basic containers only | Full container support including `Result`, `Mutex`, `RwLock`, `Weak` |
| **Option Chaining** | No - smart selection only | Yes - full control over failable vs non-failable |
| **Writable Access** | Limited | Full writable support |
| **Use Case** | Simple field access, beginners | Complex compositions, advanced users |

**When to use `Keypaths`:**
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
use key_paths_derive::Keypaths;

#[derive(Keypaths)]
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

### Widely used - Deeply nested struct
```rust
use key_paths_core::KeyPaths;
use key_paths_derive::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
    // scsf2: Option<SomeOtherStruct>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: OneMoreStruct {
                    omsf: String::from("no value for now"),
                    omse: SomeEnum::B(DarkStruct { dsf: String::from("dark field") }),
                },
            }),
        }
    }
}

#[derive(Debug, Keypaths)]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

#[derive(Debug, Casepaths)]
enum SomeEnum {
    A(String), 
    B(DarkStruct)
}

#[derive(Debug, Keypaths)]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum
}

#[derive(Debug, Keypaths)]
struct DarkStruct {
    dsf: String
}

fn main() {    
    let op = SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf());
    let mut instance = SomeComplexStruct::new();
    let omsf = op.get_mut(&mut instance);
    *omsf.unwrap() =
        String::from("we can change the field with the other way unlocked by keypaths");
    println!("instance = {:?}", instance);

}
```

### Iteration via keypaths
 ```rust
use key_paths_core::KeyPaths;

#[derive(Debug)]
struct Size {
    width: u32,
    height: u32,
}
#[derive(Debug)]
enum Color {
    Red,
    Green,
    Blue,
    Other(RGBU8),
}
#[derive(Debug)]
struct RGBU8(u8, u8, u8);

#[derive(Debug)]
struct ABox {
    name: String,
    size: Size,
    color: Color,
}
#[derive(Debug)]
struct Rectangle {
    size: Size,
    name: String,
}
fn main() {
    let mut a_box = ABox {
        name: String::from("A box"),
        size: Size {
            width: 10,
            height: 20,
        },
        color: Color::Other(
            RGBU8(10, 20, 30)
        ),
    };

    let color_kp: KeyPaths<ABox, Color> = KeyPaths::failable_writable(|x: &mut ABox| Some(&mut x.color));
    let case_path = KeyPaths::writable_enum(
        {
            |v| Color::Other(v)
        },
        |p: &Color| match p {
            Color::Other(rgb) => Some(rgb),
            _ => None,
        },
        |p: &mut Color| match p {
            Color::Other(rgb) => Some(rgb),
            _ => None,
        },

    );
    
    println!("{:?}", a_box);
    let color_rgb_kp = color_kp.compose(case_path);
    if let Some(value) = color_rgb_kp.get_mut(&mut a_box) {
        *value = RGBU8(0, 0, 0);
    }
    println!("{:?}", a_box);
}
/*
ABox { name: "A box", size: Size { width: 10, height: 20 }, color: Other(RGBU8(10, 20, 30)) }
ABox { name: "A box", size: Size { width: 10, height: 20 }, color: Other(RGBU8(0, 0, 0)) }
*/
```

---

## 📦 Container Adapters & References (NEW!)

KeyPaths now support smart pointers, containers, and references via adapter methods:

### Smart Pointer Adapters

Use `.for_arc()`, `.for_box()`, or `.for_rc()` to adapt keypaths for wrapped types:

```rust
use key_paths_derive::Keypaths;
use std::sync::Arc;

#[derive(Keypaths)]
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

#[derive(Keypaths)]
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