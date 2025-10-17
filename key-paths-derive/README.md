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
- ✅ **Readable-only macro**: `#[derive(ReadableKeypaths)]` for read-only access patterns

---

---

## 🚀 Examples

See `examples/` for many runnable samples. Below are a few highlights.

### Readable-only keypaths for safe data access
```rust
use key_paths_derive::ReadableKeypaths;

#[derive(Debug, ReadableKeypaths)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    hobbies: Vec<String>,
    scores: std::collections::HashMap<String, u32>,
}

fn main() {
    let person = Person {
        name: "John Doe".to_string(),
        age: 25,
        email: Some("john@example.com".to_string()),
        hobbies: vec!["reading".to_string(), "coding".to_string()],
        scores: {
            let mut map = std::collections::HashMap::new();
            map.insert("math".to_string(), 95);
            map.insert("science".to_string(), 88);
            map
        },
    };

    // Basic readable keypaths
    println!("Name: {:?}", Person::name_r().get(&person));
    println!("Age: {:?}", Person::age_r().get(&person));

    // Failable readable keypaths
    if let Some(email) = Person::email_fr().get(&person) {
        println!("Email: {}", email);
    }

    if let Some(hobby) = Person::hobbies_fr().get(&person) {
        println!("First hobby: {}", hobby);
    }

    if let Some(score) = Person::scores_fr("math".to_string()).get(&person) {
        println!("Math score: {}", score);
    }

    // Indexed access for Vec
    if let Some(hobby) = Person::hobbies_fr_at(1).get(&person) {
        println!("Second hobby: {}", hobby);
    }
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
- [] Derive macros for complex multi-field enum variants
---

## 📜 License

* Mozilla Public License 2.0