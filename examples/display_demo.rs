// Demonstrates the Display trait implementation for keypaths
// cargo run --example display_demo

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

struct Person {
    name: String,
    age: Option<u32>,
    address: Address,
}

struct Address {
    street: String,
    city: Option<String>,
}

fn main() {
    println!("=== KeyPath Display Examples ===\n");

    // KeyPath (readable, non-optional)
    let name_path: KeyPath<Person, String, _> = KeyPath::new(|p: &Person| &p.name);
    println!("KeyPath: {}", name_path);
    println!("Debug: {:?}\n", name_path);

    // OptionalKeyPath (readable, optional)
    let age_path: OptionalKeyPath<Person, u32, _> =
        OptionalKeyPath::new(|p: &Person| p.age.as_ref());
    println!("OptionalKeyPath: {}", age_path);
    println!("Debug: {:?}\n", age_path);

    // WritableKeyPath (mutable, non-optional)
    let name_writable: WritableKeyPath<Person, String, _> =
        WritableKeyPath::new(|p: &mut Person| &mut p.name);
    println!("WritableKeyPath: {}", name_writable);
    println!("Debug: {:?}\n", name_writable);

    // WritableOptionalKeyPath (mutable, optional)
    let age_writable: WritableOptionalKeyPath<Person, u32, _> =
        WritableOptionalKeyPath::new(|p: &mut Person| p.age.as_mut());
    println!("WritableOptionalKeyPath: {}", age_writable);
    println!("Debug: {:?}\n", age_writable);

    // Chained keypaths
    let street_path: KeyPath<Person, String, _> = KeyPath::new(|p: &Person| &p.address.street);
    println!("Chained KeyPath: {}", street_path);

    let city_path: OptionalKeyPath<Person, String, _> =
        OptionalKeyPath::new(|p: &Person| p.address.city.as_ref());
    println!("Chained OptionalKeyPath: {}", city_path);

    println!("\n=== Display Format Benefits ===");
    println!("- Shows keypath type (KeyPath, OptionalKeyPath, etc.)");
    println!("- Shows source and target types");
    println!("- Simplified type names (without full module paths)");
    println!("- Useful for debugging and logging");
}
