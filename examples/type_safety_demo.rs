//! Example demonstrating type safety when composing keypaths
//!
//! This example shows what happens when you try to compose keypaths
//! that don't share the same root type - it fails at compile time!

use keypaths_proc::Kp;

#[derive(Kp, Debug)]
#[All]
struct Person {
    name: String,
    age: u32,
    address: Address,
}

#[derive(Kp, Debug)]
#[All]
struct Address {
    street: String,
    city: String,
}

#[derive(Kp, Debug)]
#[All]
struct Company {
    name: String,
    employees: Vec<Person>,
}

#[derive(Kp, Debug)]
#[All]
struct Product {
    name: String,
    price: f64,
}

fn main() {
    let person = Person {
        name: "Akash".to_string(),
        age: 30,
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
        },
    };

    // ✅ CORRECT: Chaining keypaths that share the same root
    // Person -> Address -> city (all part of the same type hierarchy)
    let city_kp = Person::address_r()
        .then(Address::city_r());
    
    println!("City: {}", city_kp.get(&person));

    // ❌ COMPILE ERROR: Trying to chain keypaths from different roots
    // Person::name_r() returns KeyPath<Person, String>
    // Product::name_r() expects Product as root, not String!
    
    // Uncomment the following to see the compile error:
    /*
    let invalid_kp = Person::name_r()
        .then(Product::name_r());  // ERROR: expected `String`, found `Product`
    */
    
    // ❌ COMPILE ERROR: Trying to use a keypath from a completely different struct
    // Person::age_r() returns KeyPath<Person, u32>
    // Company::name_r() expects Company as root, not u32!
    
    // Uncomment the following to see the compile error:
    /*
    let invalid_kp2 = Person::age_r()
        .then(Company::name_r());  // ERROR: expected `u32`, found `Company`
    */
    
    // ❌ COMPILE ERROR: Type mismatch in the chain
    // Person::address_r() returns KeyPath<Person, Address>
    // Person::name_r() expects Person as root, not Address!
    
    // Uncomment the following to see the compile error:
    /*
    let invalid_kp3 = Person::address_r()
        .then(Person::name_r());  // ERROR: expected `Address`, found `Person`
    */

    println!("\n✅ All valid keypath compositions compiled successfully!");
    println!("❌ Invalid compositions would fail at compile time with clear error messages.");
    println!("\nThe Rust compiler ensures type safety:");
    println!("  - The Value type of the first keypath must match the Root type of the second");
    println!("  - This prevents runtime errors and ensures correctness");
    println!("\n📝 Example compile errors you would see:");
    println!("   error[E0308]: mismatched types");
    println!("   --> expected `String`");
    println!("   --> found `Product`");
    println!("\n   This happens because:");
    println!("   - Person::name_r() returns KeyPath<Person, String>");
    println!("   - Product::name_r() expects KeyPath<Product, String>");
    println!("   - When chaining, the Value (String) must match the Root (Product)");
    println!("   - Since String ≠ Product, the compiler rejects it");
}

