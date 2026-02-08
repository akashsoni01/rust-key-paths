//! Example: Filter PKp and AKp by TypeId and print root/value types.
//!
//! Run with: `cargo run --example pkp_akp_filter_typeid`

use key_paths_derive::{Akp, Kp, Pkp};
use rust_key_paths::{AKp, PKp};
use std::any::TypeId;

#[derive(Kp, Pkp, Akp, Debug)]
struct Person {
    name: String,
    age: i32,
    active: bool,
}

#[derive(Kp, Pkp, Akp, Debug)]
struct Product {
    title: String,
    price: f64,
}

/// Returns a human-readable type name for known TypeIds.
fn type_id_name(tid: TypeId) -> String {
    if tid == TypeId::of::<Person>() {
        "Person".into()
    } else if tid == TypeId::of::<Product>() {
        "Product".into()
    } else if tid == TypeId::of::<String>() {
        "String".into()
    } else if tid == TypeId::of::<i32>() {
        "i32".into()
    } else if tid == TypeId::of::<f64>() {
        "f64".into()
    } else if tid == TypeId::of::<bool>() {
        "bool".into()
    } else {
        format!("{:?}", tid)
    }
}

fn main() {
    println!("=== PKp and AKp: filter by TypeId, print root/value types ===\n");

    // ---- PKp (PartialKeyPath): Root is known (Person), Value is type-erased ----
    println!("--- PKp: Person::partial_kps() ---");
    let person_kps = Person::partial_kps();
    println!("All keypaths (count = {}):", person_kps.len());

    for pkp in &person_kps {
        println!("  root: Person, value: {}", type_id_name(pkp.value_type_id()));
    }

    println!("\nFilter: value_type_id == String");
    let string_kps: Vec<&PKp<Person>> = person_kps
        .iter()
        .filter(|pkp| pkp.value_type_id() == TypeId::of::<String>())
        .collect();
    for pkp in &string_kps {
        println!("  root: Person, value: {}", type_id_name(pkp.value_type_id()));
    }

    println!("\nFilter: value_type_id == i32");
    let i32_kps: Vec<&PKp<Person>> = person_kps
        .iter()
        .filter(|pkp| pkp.value_type_id() == TypeId::of::<i32>())
        .collect();
    for pkp in &i32_kps {
        println!("  root: Person, value: {}", type_id_name(pkp.value_type_id()));
    }

    // ---- AKp (AnyKeyPath): Both Root and Value are type-erased ----
    println!("\n--- AKp: heterogeneous collection ---");
    let person_akps = Person::any_kps();
    let product_akps = Product::any_kps();
    let all_akps: Vec<AKp> = person_akps
        .into_iter()
        .chain(product_akps.into_iter())
        .collect();

    println!("All AKps (count = {}):", all_akps.len());
    for akp in &all_akps {
        println!(
            "  root: {}, value: {}",
            type_id_name(akp.root_type_id()),
            type_id_name(akp.value_type_id())
        );
    }

    println!("\nFilter: root_type_id == Person");
    let person_root: Vec<&AKp> = all_akps
        .iter()
        .filter(|akp| akp.root_type_id() == TypeId::of::<Person>())
        .collect();
    for akp in &person_root {
        println!(
            "  root: {}, value: {}",
            type_id_name(akp.root_type_id()),
            type_id_name(akp.value_type_id())
        );
    }

    println!("\nFilter: value_type_id == String");
    let string_value: Vec<&AKp> = all_akps
        .iter()
        .filter(|akp| akp.value_type_id() == TypeId::of::<String>())
        .collect();
    for akp in &string_value {
        println!(
            "  root: {}, value: {}",
            type_id_name(akp.root_type_id()),
            type_id_name(akp.value_type_id())
        );
    }

    println!("\nFilter: root == Person AND value == String");
    let person_string: Vec<&AKp> = all_akps
        .iter()
        .filter(|akp| {
            akp.root_type_id() == TypeId::of::<Person>()
                && akp.value_type_id() == TypeId::of::<String>()
        })
        .collect();
    for akp in &person_string {
        println!(
            "  root: {}, value: {}",
            type_id_name(akp.root_type_id()),
            type_id_name(akp.value_type_id())
        );
    }

    println!("\n=== Done ===");
}
