# 🔑 KeyPaths & CasePaths in Rust

Key paths and case paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **Swift’s KeyPath / CasePath** system, this crate lets you work with **struct fields** and **enum variants** as *first-class values*.

---

## ✨ Features

* ✅ **ReadableKeyPath** → safely read struct fields.
* ✅ **WritableKeyPath** → safely read/write struct fields.
* ✅ **EnumKeyPath (CasePaths)** → extract and embed enum variants.
* ✅ **Composable** → chain key paths together(Upcoming).
* ✅ **Iterable** → iterate or mutate values across collections.
* ✅ **Macros** → concise `readable_keypath!`, `writable_keypath!`, `enum_keypath!`.

---

## 📦 Installation

```toml
[dependencies]
key_paths_core = "0.3"
```

---

## 🚀 Examples

### 1. CasePaths with Enums

```rust
use key_paths_core::enum_keypath;
use key_paths_core::EnumKeyPath;

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

#[derive(Debug)]
enum Status {
    Active(User),
    Inactive(()),
}

fn main() {
    let cp = enum_keypath!(Status::Active(User));

    let status = Status::Active(User {
        id: 42,
        name: "Charlie".to_string(),
    });

    if let Some(u) = cp.extract(&status) {
        println!("Extracted user: {:?}", u);
    }

    let new_status = cp.embed(User {
        id: 99,
        name: "Diana".to_string(),
    });
    println!("Embedded back: {:?}", new_status);
}
```

---

### 2. Readable KeyPaths

```rust
use key_paths_core::Readable;
use key_paths_core::ReadableKeyPath;
use key_paths_core::readable_keypath;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    let users = vec![
        User { name: "Akash".into(), age: 25 },
        User { name: "Soni".into(), age: 30 },
        User { name: "Neha".into(), age: 20 },
    ];

    let name_key = readable_keypath!(User, name);

    println!("Names:");
    for name in name_key.iter(&users) {
        println!("{}", name);
    }
}
```

---

### 3. Writable KeyPaths

```rust
use key_paths_core::writable_keypath;
use key_paths_core::WritableKeyPath;
use key_paths_core::Readable;
use key_paths_core::Writable;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    let mut users = vec![
        User { name: "Akash".into(), age: 25 },
        User { name: "Soni".into(), age: 30 },
        User { name: "Neha".into(), age: 20 },
    ];

    let age_key = writable_keypath!(User, age);

    println!("Ages before:");
    for age in age_key.iter(&users) {
        println!("{}", age);
    }

    for age in age_key.iter_mut(&mut users) {
        *age += 1;
    }

    println!("Ages after:");
    for age in age_key.iter(&users) {
        println!("{}", age);
    }
}
```

---

## 🔗 Helpful Links & Resources

* 📘 [Swift KeyPath documentation](https://developer.apple.com/documentation/swift/keypath)
* 📘 [Swift CasePath library (pointfreeco)](https://github.com/pointfreeco/swift-case-paths)
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

* [ ] `zip` support for combining multiple key paths (Upcoming).
* [ ] Derive macros for automatic KeyPath generation (Upcoming).
* [ ] Nested struct & enum traversal (Upcoming).
* [ ] Optional chaining (`User?.profile?.name`) (Upcoming).

---

## 📜 License

* Mozilla Public License 2.0