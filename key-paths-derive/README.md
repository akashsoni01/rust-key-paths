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
key-paths-core = "0.8"
key-paths-derive = "0.1"
```

---

## 🚀 Examples

See `examples/` for many runnable samples. Below are a few highlights.

### Widely used - Deeply nested struct
```
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
    let op = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw());
    let mut instance = SomeComplexStruct::new();
    let omsf = op.get_mut(&mut instance);
    *omsf.unwrap() =
        String::from("we can change the field with the other way unlocked by keypaths");
    println!("instance = {:?}", instance);

}
```
### 1) Structs with #[derive(Keypaths)]

```rust
use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct Size { width: u32, height: u32 }

#[derive(Debug, Keypaths)]
struct Rectangle { size: Size, name: String }

fn main() {
    let mut rect = Rectangle { size: Size { width: 30, height: 50 }, name: "MyRect".into() };

    // Readable/writable
    println!("width = {:?}", Size::width_r().get(&rect.size));
    if let Some(w) = Size::width_w().get_mut(&mut rect.size) { *w += 10; }

    // Compose: Rectangle -> Size -> width
    let rect_width = Rectangle::size_r().compose(Size::width_r());
    println!("rect.width = {:?}", rect_width.get(&rect));
}
```

### 2) Optional chaining (failable keypaths)

```rust
use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct Engine { horsepower: u32 }
#[derive(Debug, Keypaths)]
struct Car { engine: Option<Engine> }
#[derive(Debug, Keypaths)]
struct Garage { car: Option<Car> }

fn main() {
    let mut g = Garage { car: Some(Car { engine: Some(Engine { horsepower: 120 }) }) };

    // Read horsepower if present
    let hp = Garage::car_fr()
        .compose(Car::engine_fr())
        .compose(Engine::horsepower_r());
    println!("hp = {:?}", hp.get(&g));

    // Mutate horsepower if present
    if let Some(car) = Garage::car_fw().get_mut(&mut g) {
        if let Some(engine) = Car::engine_fw().get_mut(car) {
            if let Some(hp) = Engine::horsepower_w().get_mut(engine) { *hp += 30; }
        }
    }
}
```

### 3) Enum CasePaths (readable/writable prisms)

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

### 4) Compose enum prisms with struct fields
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
### 5) Iteration via keypaths
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

## 💡 Why use KeyPaths?

* Avoids repetitive `match` / `.` chains.
* Encourages **compositional design**.
* Plays well with **DDD (Domain-Driven Design)** and **Actor-based systems**.
* Useful for **reflection-like behaviors** in Rust (without unsafe).

---

## 🛠 Roadmap

- [x] Compose across structs, options and enum cases
- [x] Derive macros for automatic keypath generation
- [x] Optional chaining with failable keypaths
- [ ] Derive macros for complex multi-field enum variants
---

## 📜 License

* Mozilla Public License 2.0