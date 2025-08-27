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
use key_paths_core::{readable_keypath, ReadableKeyPath};

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    let users = vec![
        User {
            name: "Akash".into(),
            age: 25,
        },
        User {
            name: "Soni".into(),
            age: 30,
        },
        User {
            name: "Neha".into(),
            age: 20,
        },
    ];

    // Read-only keypath
    // let name_key = ReadableKeyPath::new(|u: &User| &u.name);
    let name_key = readable_keypath!(User, name);

    // Writable keypath
    // let age_key = WritableKeyPath::new(
    //     |u: &User| &u.age,
    //     |u: &mut User| &mut u.age,
    // );
    // let age_key = writable_keypath!(User, age);

    println!("Names:");
    for name in name_key.iter(&users) {
        println!("{}", name);
    }
}
```

---

### 3. Writable KeyPaths

```rust
use key_paths_core::{writable_keypath, WritableKeyPath};

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    let mut users = vec![
        User {
            name: "Akash".into(),
            age: 25,
        },
        User {
            name: "Soni".into(),
            age: 30,
        },
        User {
            name: "Neha".into(),
            age: 20,
        },
    ];

    // Read-only keypath
    // let name_key = ReadableKeyPath::new(|u: &User| &u.name);
    // let name_key = readable_keypath!(User, name);

    // Writable keypath
    // let age_key = WritableKeyPath::new(
    //     |u: & User| & u.age,
    //     |u: &mut User| &mut u.age,
    // );
    let age_key = writable_keypath!(User, age);

    // println!("Names:");
    // for name in name_key.iter(&users) {
    //     println!("{}", name);
    // }

    println!("Ages before:");
    for age in age_key.iter(&users) {
        println!("{}", age);
    }

    // Mutate agesiter
    for age in age_key.iter_mut(&mut users) {
        *age += 1;
    }

    println!("Ages after:");
    for age in age_key.iter(&mut users) {
        println!("{}", age);
    }
}
```

### 4. Composability and failablity
 ```rust
 use key_paths_core::{FailableReadableKeyPath};

#[derive(Debug)]
struct Engine {
    horsepower: u32,
}
#[derive(Debug)]
struct Car {
    engine: Option<Engine>,
}
#[derive(Debug)]
struct Garage {
    car: Option<Car>,
}

fn main() {
    let garage = Garage {
        car: Some(Car {
            engine: Some(Engine { horsepower: 120 }),
        }),
    };

    let kp_car = FailableReadableKeyPath::new(|g: &Garage| g.car.as_ref());
    let kp_engine = FailableReadableKeyPath::new(|c: &Car| c.engine.as_ref());
    let kp_hp = FailableReadableKeyPath::new(|e: &Engine| Some(&e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.compose(kp_engine).compose(kp_hp);

    let kp2 = FailableReadableKeyPath::new(|g: &Garage| {
        g.car
            .as_ref()
            .and_then(|c| c.engine.as_ref())
            .and_then(|e| Some(&e.horsepower))
    });

    if let Some(hp) = kp.try_get(&garage) {
        println!("{hp:?}");
    }

    if let Some(hp) = kp2.try_get(&garage) {
        println!("{hp:?}");
    }

    println!("{garage:?}");
}
```
### 4. Mutablity
 ```rust
 use key_paths_core::{FailableWritableKeyPath};

#[derive(Debug)]
struct Engine {
    horsepower: u32,
}
#[derive(Debug)]
struct Car {
    engine: Option<Engine>,
}
#[derive(Debug)]
struct Garage {
    car: Option<Car>,
}

fn main() {
    let mut garage = Garage {
        car: Some(Car {
            engine: Some(Engine { horsepower: 120 }),
        }),
    };

    let kp_car = FailableWritableKeyPath::new(|g: &Garage| g.car.as_ref(), |g: &mut Garage| g.car.as_mut());
    let kp_engine = FailableWritableKeyPath::new(|c: &Car| c.engine.as_ref(), |c: &mut Car| c.engine.as_mut());
    let kp_hp = FailableWritableKeyPath::new(|e: &Engine| Some(&e.horsepower), |e: &mut Engine| Some(&mut e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.compose(kp_engine).compose(kp_hp);

    println!("{garage:?}");
    if let Some(hp) = kp.try_get_mut(&mut garage) {
        *hp = 200;
    }

    println!("{garage:?}");
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

* [ ] `compose` support for combining multiple key paths.
* [ ] Derive macros for automatic KeyPath generation (Upcoming).
* [ ] Nested struct & enum traversal.
* [ ] Optional chaining (`User?.profile?.name`).

---

## 📜 License

* Mozilla Public License 2.0