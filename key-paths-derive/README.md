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
- ✅ **Writable-only macro**: `#[derive(WritableKeypaths)]` for write-only access patterns
- ✅ **Smart keypath macro**: `#[derive(Keypath)]` for intelligent keypath selection

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

### Writable-only keypaths for safe data mutation
```rust
use key_paths_derive::WritableKeypaths;

#[derive(Debug, WritableKeypaths)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    hobbies: Vec<String>,
    scores: std::collections::HashMap<String, u32>,
}

fn main() {
    let mut person = Person {
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

    // Basic writable keypaths
    if let Some(name_ref) = Person::name_w().get_mut(&mut person) {
        *name_ref = "John Smith".to_string();
    }

    if let Some(age_ref) = Person::age_w().get_mut(&mut person) {
        *age_ref = 26;
    }

    // Failable writable keypaths
    if let Some(email_ref) = Person::email_fw().get_mut(&mut person) {
        *email_ref = "john.smith@example.com".to_string();
    }

    if let Some(hobby_ref) = Person::hobbies_fw().get_mut(&mut person) {
        *hobby_ref = "gaming".to_string();
    }

    if let Some(score_ref) = Person::scores_fw("math".to_string()).get_mut(&mut person) {
        *score_ref = 98;
    }

    // Indexed access for Vec
    if let Some(hobby_ref) = Person::hobbies_fw_at(1).get_mut(&mut person) {
        *hobby_ref = "swimming".to_string();
    }
}
```

### Smart keypath selection for intuitive access
```rust
use key_paths_derive::Keypath;

#[derive(Debug, Keypath)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    hobbies: Vec<String>,
    scores: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Keypath)]
enum Status {
    Loading,
    Success(String),
    Error(Option<String>),
    Data(Vec<i32>),
    Position(f64, f64),
    User { name: String, age: u32 },
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

    // Each field gets a single method with the same name
    // The macro intelligently chooses the best keypath type:
    
    // Basic types -> readable keypath
    println!("Name: {:?}", Person::name().get(&person));
    println!("Age: {:?}", Person::age().get(&person));

    // Option<T> -> failable readable keypath to inner type
    if let Some(email) = Person::email().get(&person) {
        println!("Email: {}", email);
    }

    // Vec<T> -> failable readable keypath to first element
    if let Some(hobby) = Person::hobbies().get(&person) {
        println!("First hobby: {}", hobby);
    }

    // HashMap<K,V> -> readable keypath to container
    if let Some(scores) = Person::scores().get(&person) {
        println!("Scores: {:?}", scores);
    }

    // Enum keypath access
    let success = Status::Success("Operation completed".to_string());
    if let Some(message) = Status::success().get(&success) {
        println!("Success message: {}", message);
    }

    let error = Status::Error(Some("Something went wrong".to_string()));
    if let Some(error_msg) = Status::error().get(&error) {
        println!("Error message: {}", error_msg);
    }

    let data = Status::Data(vec![1, 2, 3, 4, 5]);
    if let Some(first_value) = Status::data().get(&data) {
        println!("First data value: {}", first_value);
    }

    let position = Status::Position(10.5, 20.3);
    if let Some(pos) = Status::position().get(&position) {
        println!("Position: {:?}", pos);
    }
}
```

### Keypath vs Keypaths: Choosing the Right Tool

The `Keypath` macro is designed for **simplicity and beginner-friendly access**, while `Keypaths` provides **full control and advanced features** for experienced developers.

#### **Keypath Limitations:**
- ❌ **No option chaining** - Can't compose keypaths with `.then()`
- ❌ **No indexed access** - No `_at()` methods for collections
- ❌ **No writable access** - Only provides readable/failable readable keypaths
- ❌ **No composition** - Can't chain multiple keypaths together
- ❌ **Fixed behavior** - Macro decides the keypath type, not you
- ❌ **Limited control** - No access to intermediate keypath variants

#### **Keypaths Advantages:**
- ✅ **Full composition** - Chain keypaths with `.then()` and `.compose()`
- ✅ **Indexed access** - `_at()` methods for collections and maps
- ✅ **All access types** - Readable, writable, failable, owned variants
- ✅ **Advanced patterns** - Option chaining, nested access, complex compositions
- ✅ **Developer control** - Choose exactly which keypath type you need
- ✅ **Intermediate/Advanced features** - Iteration, mutation, complex data access

#### **When to Use Each:**

**Use `Keypath` for:**
- 🎯 **Beginners** learning keypath concepts
- 🎯 **Simple access patterns** where you just need basic field access
- 🎯 **Prototyping** and quick development
- 🎯 **Read-only operations** on simple data structures
- 🎯 **Clean, minimal APIs** without complexity

**Use `Keypaths` for:**
- 🚀 **Intermediate/Advanced developers** who need full control
- 🚀 **Complex data access patterns** with option chaining
- 🚀 **Composition-heavy code** with nested keypath chains
- 🚀 **Performance-critical applications** where you need specific keypath types
- 🚀 **Advanced features** like iteration, mutation, and complex compositions

#### **Example Comparison:**

```rust
// Keypath - Simple, beginner-friendly
#[derive(Keypath)]
struct User {
    profile: Option<Profile>,
}

// Simple access - no composition possible
if let Some(profile) = User::profile().get(&user) {
    // Can't chain further - this is the limit
}

// Keypaths - Full power for advanced developers
#[derive(Keypaths)]
struct User {
    profile: Option<Profile>,
}

#[derive(Keypaths)]
struct Profile {
    name: Option<String>,
    settings: Settings,
}

#[derive(Keypaths)]
struct Settings {
    theme: String,
}

// Advanced composition with option chaining
let theme_path = User::profile_fr()
    .then(Profile::settings_r())
    .then(Settings::theme_r());

// Or even more complex chains
let name_path = User::profile_fr()
    .then(Profile::name_fr());

// Full control over keypath types
let writable_theme = User::profile_fw()
    .then(Profile::settings_w())
    .then(Settings::theme_w());
```

#### **Learning Progression:**

**Beginner Level (Keypath):**
```rust
// Start here - simple, intuitive access
#[derive(Keypath)]
struct User {
    name: String,
    email: Option<String>,
}

// Easy to understand - one method per field
let name = User::name().get(&user);
let email = User::email().get(&user);
```

**Intermediate Level (Keypaths - Basic):**
```rust
// Graduate to full control when you need more features
#[derive(Keypaths)]
struct User {
    name: String,
    email: Option<String>,
    profile: Option<Profile>,
}

// Now you have multiple variants to choose from
let name = User::name_r().get(&user);        // Readable
let email = User::email_fr().get(&user);     // Failable readable
let profile = User::profile_fr().get(&user); // Failable readable
```

**Advanced Level (Keypaths - Composition):**
```rust
// Master level - complex compositions and option chaining
#[derive(Keypaths)]
struct User {
    profile: Option<Profile>,
}

#[derive(Keypaths)]
struct Profile {
    settings: Option<Settings>,
}

#[derive(Keypaths)]
struct Settings {
    theme: String,
}

// Advanced option chaining - the real power of keypaths
let theme_path = User::profile_fr()
    .then(Profile::settings_fr())
    .then(Settings::theme_r());

// This safely navigates: user.profile?.settings?.theme
if let Some(theme) = theme_path.get(&user) {
    println!("User theme: {}", theme);
}
```

**Expert Level (Keypaths - All Features):**
```rust
// Full mastery - using all keypath features
let writable_theme = User::profile_fw()
    .then(Profile::settings_fw())
    .then(Settings::theme_w());

// Indexed access for collections
let first_hobby = User::hobbies_fr_at(0).get(&user);

// Complex compositions with owned keypaths
let owned_name = User::profile_fr()
    .then(Profile::name_fo())
    .get(&user);

// Iteration over collections
for hobby in User::hobbies_r().iter(&user) {
    println!("Hobby: {}", hobby);
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