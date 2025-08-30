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
key_paths_core = "0.6"
```

---

## 🚀 Examples - Go to latest examples directory docs will be updated later

### 1. CasePaths with Enums

```rust
use key_paths_core::KeyPaths;
#[derive(Debug)]
enum Payment {
    Cash { amount: u32 },
    Card { number: String, cvv: String },
}

fn main() {
    let kp = KeyPaths::writable_enum(
        |v| Payment::Cash { amount: v },
        |p: &Payment| match p {
            Payment::Cash { amount } => Some(amount),
            _ => None,
        },
        |p: &mut Payment| match p {
            Payment::Cash { amount } => Some(amount),
            _ => None,
        },

    );

    let mut p = Payment::Cash { amount: 10 };

    println!("{:?}", p);

    if let Some(v) = kp.get_mut(&mut p) {
        *v = 34
    }
    println!("{:?}", p);
}
```

---

### 2. Readable KeyPaths - helper macros wip

```rust
use key_paths_core::KeyPaths;

#[derive(Debug)]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct Rectangle {
    size: Size,
    name: String,
}

fn main() {
    let mut rect = Rectangle {
        size: Size {
            width: 30,
            height: 50,
        },
        name: "MyRect".into(),
    };

    let width_direct = KeyPaths::readable(|r: &Rectangle| &r.size.width);
    println!("Width: {:?}", width_direct.get(&rect));
}
```

---

### 3. Writable KeyPaths - helper macros wip

```rust
use key_paths_core::KeyPaths;

#[derive(Debug)]
struct Size {
    width: u32,
    height: u32,
}
#[derive(Debug)]
struct Rectangle {
    size: Size,
    name: String,
}
fn main() {
    let mut rect = Rectangle {
        size: Size {
            width: 30,
            height: 50,
        },
        name: "MyRect".into(),
    };
    let width_mut = KeyPaths::writable(
        |r: &mut Rectangle| &mut r.size.width,
    );
    // Mutable
    if let Some(hp_mut) = width_mut.get_mut(&mut rect) {
        *hp_mut += 50;
    }
    println!("Updated rectangle: {:?}", rect);
}
```

### 4. Composability and failablity
 ```rust
use key_paths_core::KeyPaths;

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

    let kp_car = KeyPaths::failable_writable(|g: &mut Garage| g.car.as_mut());
    let kp_engine = KeyPaths::failable_writable(|c: &mut Car| c.engine.as_mut());
    let kp_hp = KeyPaths::failable_writable(|e: &mut Engine| Some(&mut e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.compose(kp_engine).compose(kp_hp);

    println!("{garage:?}");
    if let Some(hp) = kp.get_mut(&mut garage) {
        *hp = 200;
    }

    println!("{garage:?}");
}
```
### 4. Mutability
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

## 🔗 Helpful Links & Resources

* 📘 [type-safe property paths](https://lodash.com/docs/4.17.15#get)
* 📘 [Swift KeyPath documentation](https://developer.apple.com/documentation/swift/keypath)
* 📘 [Elm Architecture & Functional Lenses](https://guide.elm-lang.org/architecture/)
* 📘 [Rust Macros Book](https://doc.rust-lang.org/book/ch19-06-macros.html)
* 📘 [Category Theory in FP (for intuition)](https://bartoszmilewski.com/2014/11/24/category-the-essence-of-composition/)

---

## Support

* Struct Field Support
* Enum Variant Support
* Read / Write
* Mutability support 
* Full support of Composition with keypaths including enum
* Helper macros support (WIP)
* Proc macros support (WIP)
* Feature rich, error free and light weigh 3KB only 

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
* [ ] Optional chaining with failable.
---

## 📜 License

* Mozilla Public License 2.0