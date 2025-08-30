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
rust_key_paths_ = "0.7"
```

---

## 🚀 Examples - Go to latest examples directory docs will be updated later

### 1. CasePaths with Enums

```rust
use rust_key_paths::{FailableWritable, ReadKeyPath, Writable, WriteKeyPath};
use rust_key_paths::Compose;

// ========== EXAMPLES ==========

// Example 1: Nested structs
#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    address: Address,
}

#[derive(Debug, Clone)]
struct Address {
    street: String,
    city: String,
    zip_code: String,
}

// Create keypaths for nested struct access
fn user_name_keypath() -> Writable<User, String> {
    Writable::new(
        |user: &User| Some(&user.name),
        |user: &mut User| Some(&mut user.name),
        |user: &mut User, name: String| user.name = name,
    )
}

fn user_address_keypath() -> Writable<User, Address> {
    Writable::new(
        |user: &User| Some(&user.address),
        |user: &mut User| Some(&mut user.address),
        |user: &mut User, address: Address| user.address = address,
    )
}

fn address_city_keypath() -> Writable<Address, String> {
    Writable::new(
        |addr: &Address| Some(&addr.city),
        |addr: &mut Address| Some(&mut addr.city),
        |addr: &mut Address, city: String| addr.city = city,
    )
}

// Example 2: Enum with variants
#[derive(Debug, Clone)]
enum Contact {
    Email(String),
    Phone(String),
    Address(Address),
    Unknown,
}

#[derive(Debug, Clone)]
struct Profile {
    name: String,
    contact: Contact,
}

// Keypath for enum variant access (failable since variant might not match)
fn contact_email_keypath() -> FailableWritable<Contact, String> {
    FailableWritable::new(|contact: &mut Contact| {
        match contact {
            Contact::Email(email) => Some(email),
            _ => None,
        }
    })
}

fn profile_contact_keypath() -> Writable<Profile, Contact> {
    Writable::new(
        |profile: &Profile| Some(&profile.contact),
        |profile: &mut Profile| Some(&mut profile.contact),
        |profile: &mut Profile, contact: Contact| profile.contact = contact,
    )
}

// Example 3: Complex nested structure
#[derive(Debug, Clone)]
struct Company {
    name: String,
    employees: Vec<Employee>,
}

#[derive(Debug, Clone)]
struct Employee {
    id: u32,
    profile: Profile,
    salary: f64,
}

fn main() {
    println!("=== Nested Struct Example ===");

    let mut user = User {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            zip_code: "12345".to_string(),
        },
    };

    // Basic keypath usage
    let name_kp = user_name_keypath();
    if let Some(name) = name_kp.get(&user) {
        println!("User name: {}", name);
    }

    name_kp.set(&mut user, "Bob".to_string());
    println!("User after name change: {:?}", user);

    // Composition: User -> Address -> City
    let user_city_kp = user_address_keypath().then(address_city_keypath());

    if let Some(city) = user_city_kp.get(&user) {
        println!("User city: {}", city);
    }

    user_city_kp.set(&mut user, "Metropolis".to_string());
    println!("User after city change: {:?}", user);

    println!("\n=== Enum Example ===");

    let mut profile = Profile {
        name: "Charlie".to_string(),
        contact: Contact::Email("charlie@example.com".to_string()),
    };

    let contact_kp = profile_contact_keypath();
    let email_kp = contact_email_keypath();

    // Compose profile -> contact -> email (failable)
    let profile_email_kp = contact_kp.then(email_kp);

    if let Some(email) = profile_email_kp.get(&profile) {
        println!("Profile email: {}", email);
    }

    // This will work because contact is Email variant
    profile_email_kp.set(&mut profile, "charlie.new@example.com".to_string());
    println!("Profile after email change: {:?}", profile);

    // // Change contact to Phone variant (email access will now fail)
    // contact_kp.set(&mut profile, Contact::Phone("555-1234".to_string()));

    if let Some(email) = profile_email_kp.get(&profile) {
        println!("Profile email: {}", email);
    } else {
        println!("No email found (contact is now Phone variant)");
    }

    println!("\n=== Complex Nested Example ===");

    let mut company = Company {
        name: "Tech Corp".to_string(),
        employees: vec![
            Employee {
                id: 1,
                profile: Profile {
                    name: "Dave".to_string(),
                    contact: Contact::Email("dave@tech.com".to_string()),
                },
                salary: 50000.0,
            },
            Employee {
                id: 2,
                profile: Profile {
                    name: "Eve".to_string(),
                    contact: Contact::Phone("555-6789".to_string()),
                },
                salary: 60000.0,
            },
        ],
    };

    // Create keypath for first employee's email
    let first_employee_kp = Writable::new(
        |company: &Company| company.employees.first(),
        |company: &mut Company| company.employees.first_mut(),
        |company: &mut Company, employee: Employee| {
            if !company.employees.is_empty() {
                company.employees[0] = employee;
            }
        },
    );

    let employee_profile_kp = Writable::new(
        |employee: &Employee| Some(&employee.profile),
        |employee: &mut Employee| Some(&mut employee.profile),
        |employee: &mut Employee, profile: Profile| employee.profile = profile,
    );

    // Compose: Company -> first Employee -> Profile -> Contact -> Email
    let company_first_employee_email_kp = first_employee_kp
        .then(employee_profile_kp)
        .then(profile_contact_keypath())
        .then(contact_email_keypath());

    if let Some(email) = company_first_employee_email_kp.get(&company) {
        println!("First employee email: {}", email);
    }

    // This will work for the first employee (who has email)
    company_first_employee_email_kp.set(
        &mut company,
        "dave.new@tech.com".to_string()
    );

    println!("Company after email change: {:?}", company);
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