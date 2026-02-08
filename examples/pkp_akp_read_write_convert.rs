//! Example: Read/write using PKp and AKp, and convert back to typed Kp for mutation.
//!
//! PKp and AKp are read-only (get/get_as). For mutation, we dispatch to the typed Kp
//! (e.g. Person::name()) based on TypeId—that is the "convert back" pattern.
//!
//! Run with: `cargo run --example pkp_akp_read_write_convert`

use key_paths_derive::{Akp, Kp, Pkp};
use rust_key_paths::KpType;
use std::any::TypeId;

#[derive(Kp, Pkp, Akp, Debug)]
struct Person {
    name: String,
    age: i32,
}

#[derive(Kp, Pkp, Akp, Debug)]
struct Product {
    title: String,
    price: f64,
}

/// Convert PKp<Person> to typed KpType for the given value type.
/// Returns None if the value type doesn't match any known field.
fn pkp_to_kp_person(value_tid: TypeId) -> Option<PersonKp> {
    if value_tid == TypeId::of::<String>() {
        Some(PersonKp::Name(Person::name()))
    } else if value_tid == TypeId::of::<i32>() {
        Some(PersonKp::Age(Person::age()))
    } else {
        None
    }
}

enum PersonKp {
    Name(KpType<'static, Person, String>),
    Age(KpType<'static, Person, i32>),
}

impl PersonKp {
    fn get_mut_name<'a>(&self, p: &'a mut Person) -> Option<&'a mut String> {
        match self {
            PersonKp::Name(kp) => kp.get_mut(p),
            _ => None,
        }
    }
    fn get_mut_age<'a>(&self, p: &'a mut Person) -> Option<&'a mut i32> {
        match self {
            PersonKp::Age(kp) => kp.get_mut(p),
            _ => None,
        }
    }
}

/// Convert back to typed Kp based on root and value TypeIds.
/// For a heterogeneous AKp collection, we dispatch to the right typed Kp.
fn akp_to_typed_kp(root_tid: TypeId, value_tid: TypeId) -> Option<&'static str> {
    if root_tid == TypeId::of::<Person>() && value_tid == TypeId::of::<String>() {
        Some("Person::name()")
    } else if root_tid == TypeId::of::<Person>() && value_tid == TypeId::of::<i32>() {
        Some("Person::age()")
    } else if root_tid == TypeId::of::<Product>() && value_tid == TypeId::of::<String>() {
        Some("Product::title()")
    } else if root_tid == TypeId::of::<Product>() && value_tid == TypeId::of::<f64>() {
        Some("Product::price()")
    } else {
        None
    }
}

fn main() {
    println!("=== PKp/AKp: read, write via convert-back to Kp ===\n");

    // ---- 1. READ using PKp ----
    println!("--- 1. Read using PKp::get_as ---");
    let mut person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let pkps = Person::partial_kps();
    for pkp in &pkps {
        if let Some(name) = pkp.get_as::<String>(&person) {
            println!("  name (via PKp): {}", name);
        }
        if let Some(age) = pkp.get_as::<i32>(&person) {
            println!("  age (via PKp): {}", age);
        }
    }

    // ---- 2. WRITE: convert PKp back to typed Kp, then use get_mut ----
    println!("\n--- 2. Write: convert PKp -> KpType, use get_mut ---");
    let string_pkp = pkps
        .iter()
        .find(|p| p.value_type_id() == TypeId::of::<String>())
        .unwrap();

    if let Some(person_kp) = pkp_to_kp_person(string_pkp.value_type_id()) {
        if let Some(name_ref) = person_kp.get_mut_name(&mut person) {
            *name_ref = "Bob".to_string();
            println!("  Mutated name via Person::name() -> {}", person.name);
        }
    }

    let i32_pkp = pkps
        .iter()
        .find(|p| p.value_type_id() == TypeId::of::<i32>())
        .unwrap();
    if let Some(person_kp) = pkp_to_kp_person(i32_pkp.value_type_id()) {
        if let Some(age_ref) = person_kp.get_mut_age(&mut person) {
            *age_ref = 35;
            println!("  Mutated age via Person::age() -> {}", person.age);
        }
    }

    // Verify read after write
    println!("\n  After mutations: {:?}", person);

    // ---- 3. READ using AKp (heterogeneous) ----
    println!("\n--- 3. Read using AKp::get_as (heterogeneous) ---");
    let person = Person {
        name: "Eve".to_string(),
        age: 28,
    };
    let product = Product {
        title: "Widget".to_string(),
        price: 19.99,
    };

    let all_akps = Person::any_kps();
    for akp in &all_akps {
        if let Some(Some(name)) = akp.get_as::<Person, String>(&person) {
            println!("  Person name: {}", name);
        }
        if let Some(Some(age)) = akp.get_as::<Person, i32>(&person) {
            println!("  Person age: {}", age);
        }
    }

    let product_akps = Product::any_kps();
    for akp in &product_akps {
        if let Some(Some(title)) = akp.get_as::<Product, String>(&product) {
            println!("  Product title: {}", title);
        }
        if let Some(Some(price)) = akp.get_as::<Product, f64>(&product) {
            println!("  Product price: {}", price);
        }
    }

    // ---- 4. Convert AKp back to typed Kp (identify which Kp to use) ----
    println!("\n--- 4. Convert AKp -> typed Kp (identify for write) ---");
    for akp in all_akps.iter().chain(product_akps.iter()) {
        if let Some(kp_name) = akp_to_typed_kp(akp.root_type_id(), akp.value_type_id()) {
            println!("  AKp maps to: {}", kp_name);
        }
    }

    // ---- 5. Write via typed Kp (the "convert back" in practice) ----
    println!("\n--- 5. Write: use typed Kp directly (result of convert-back) ---");
    let mut product = Product {
        title: "Gadget".to_string(),
        price: 29.99,
    };

    // After filtering AKp by root+value, we "convert" by using the known typed Kp
    let title_kp = Product::title();
    title_kp.get_mut(&mut product).map(|t| *t = "Super Gadget".to_string());
    println!("  Product title after write: {}", product.title);

    let price_kp = Product::price();
    price_kp.get_mut(&mut product).map(|p| *p = 39.99);
    println!("  Product price after write: {}", product.price);

    println!("\n=== Done ===");
}
