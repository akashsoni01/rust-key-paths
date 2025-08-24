# 🔑 Rust KeyPaths & CasePaths

A lightweight experimental library to bring **Swift-style KeyPaths** and **CasePaths** into Rust.  
This makes it easier to work with **nested struct fields** and **enum variants** in a composable, functional style.

---

## ✨ Features

- **KeyPath**: Immutable (read-only) access to struct fields.
- **WritableKeyPath**: Read-write access to struct fields.
- **CasePath**: Extract and embed enum variants safely.
- **Composable**: KeyPaths and CasePaths can be chained for deep traversal.
- **Arc support**: Thread-safe sharing of extracted values.

---

## 📦 Installation

```toml
[dependencies]
casepath_keypath = { git = "https://github.com/your-username/casepath-keypath" }
````

---

## 🔑 KeyPath Example

```rust
use casepath_keypath::{KeyPath, WritableKeyPath};

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    // Read-only KeyPath
    let name_key = KeyPath::new(|u: &User| &u.name);
    let age_key = KeyPath::new(|u: &User| &u.age);

    let user = User { name: "Akash".into(), age: 25 };

    println!("name = {}", name_key.get(&user));
    println!("age = {}", age_key.get(&user));

    // Writable KeyPath
    let name_key = WritableKeyPath::new(|u: &User| &u.name, |u: &mut User| &mut u.name);
    let age_key = WritableKeyPath::new(|u: &User| &u.age, |u: &mut User| &mut u.age);

    let mut user = User { name: "Akash".into(), age: 25 };

    name_key.set(&mut user, "Soni".into());
    age_key.set(&mut user, 30);

    println!("updated = {:?}", user);
}
```

---

## 🔀 CasePath Example

```rust
use casepath_keypath::casepath;

#[derive(Debug)]
enum AppState {
    Loading,
    Loaded(String),
    Error(u32),
}

fn main() {
    let loaded = casepath!(AppState::Loaded(String));
    let error = casepath!(AppState::Error(u32));
    let loading = casepath!(AppState::Loading);

    let s = AppState::Loaded("Hello".into());

    if let Some(inner) = loaded.extract(&s) {
        println!("Loaded value = {}", inner);
    }

    let state = AppState::Loading;
    println!("Loading extract = {:?}", loading.extract(&state));

    println!("Embed Error = {:?}", error.embed(404));
    println!("Embed Loading = {:?}", loading.embed(()));
}
```

---

## 🧑‍💻 Why Use This?

* Makes code more **declarative**: instead of closures everywhere, reuse paths.
* **Functional programming friendly**: map, filter, and transform collections using KeyPaths.
* Helps with **pattern matching enums** without repeating boilerplate.
* Provides a **Swift-like developer experience** for Rust.

---

## 📚 Helpful Resources

* [Swift KeyPath Documentation](https://developer.apple.com/documentation/swift/keypath)
* [Swift CasePath proposal (SE-0259)](https://github.com/apple/swift-evolution/blob/main/proposals/0259-enum-cases-as-protocol-witnesses.md)
* [Point-Free’s CasePaths library for Swift](https://github.com/pointfreeco/swift-case-paths)
* [Rust `Arc` (std::sync)](https://doc.rust-lang.org/std/sync/struct.Arc.html)
* [Rust Macros by Example](https://doc.rust-lang.org/reference/macros-by-example.html)

---

## 🚧 Roadmap

* [ ] Composition for nested CasePaths (e.g. `Option<AppState>` → `Some` → `Loaded`)
* [ ] Derive macros for automatic KeyPath generation
* [ ] Integration with async contexts

---

## 🤝 Contributing

Contributions are welcome! Please open an issue or PR with improvements, examples, or bug fixes.

---

Licensed under either of:

* Mozilla Public License 2.0
