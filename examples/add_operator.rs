//! Example demonstrating the `+` (Add) operator for keypath chaining
//!
//! ## Requirements
//!
//! This example requires:
//! 1. Rust nightly toolchain
//! 2. The `nightly` feature enabled:
//!    ```toml
//!    [dependencies]
//!    rust-keypaths = { version = "1.0.6", features = ["nightly"] }
//!    ```
//! 3. The feature gate enabled in your code:
//!    ```rust
//!    #![feature(impl_trait_in_assoc_type)]
//!    ```
//!
//! ## Running the example
//!
//! **IMPORTANT**: You must use the nightly toolchain:
//! ```bash
//! cargo +nightly run --example add_operator --features nightly
//! ```
//!
//! On stable Rust, use `keypath1.then(keypath2)` instead, which provides
//! the same functionality without requiring nightly features.

// Enable the feature gate when nightly feature is enabled
// NOTE: This requires Rust nightly toolchain - it will fail on stable Rust
#![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

use rust_keypaths::{keypath, opt_keypath, writable_keypath, writable_opt_keypath, 
                    KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

#[derive(Debug, Clone)]
struct Address {
    street: String,
    city: String,
    zip_code: Option<String>,
}

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    address: Address,
    metadata: Option<String>,
}

#[derive(Debug, Clone)]
struct Company {
    name: String,
    owner: Option<User>,
}

fn main() {
    println!("=== Add Operator (+) Examples ===\n");

    // Example 1: KeyPath + KeyPath
    example_keypath_chaining();
    
    // Example 2: KeyPath + OptionalKeyPath
    example_keypath_to_optional();
    
    // Example 3: OptionalKeyPath + OptionalKeyPath
    example_optional_chaining();
    
    // Example 4: WritableKeyPath + WritableKeyPath
    example_writable_chaining();
    
    // Example 5: WritableKeyPath + WritableOptionalKeyPath
    example_writable_to_optional();
    
    // Example 6: WritableOptionalKeyPath + WritableOptionalKeyPath
    example_writable_optional_chaining();
    
    // Example 7: Comparison with then() method
    example_comparison();
}

#[cfg(feature = "nightly")]
fn example_keypath_chaining() {
    use std::ops::Add;
    
    println!("1. KeyPath + KeyPath");
    
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
            zip_code: Some("10001".to_string()),
        },
        metadata: None,
    };
    
    // Create keypaths using macros
    let address_kp = keypath!(|u: &User| &u.address);
    let street_kp = keypath!(|a: &Address| &a.street);
    
    // Chain using + operator (requires nightly feature)
    let user_street_kp = address_kp + street_kp;
    
    println!("   User street: {}", user_street_kp.get(&user));
    println!("   ✓ KeyPath + KeyPath works!\n");
}

#[cfg(not(feature = "nightly"))]
fn example_keypath_chaining() {
    println!("1. KeyPath + KeyPath (requires nightly feature)");
    println!("   Use keypath1.then(keypath2) instead on stable Rust\n");
}

#[cfg(feature = "nightly")]
fn example_keypath_to_optional() {
    use std::ops::Add;
    
    println!("2. KeyPath + OptionalKeyPath");
    
    let user = User {
        name: "Bob".to_string(),
        age: 25,
        address: Address {
            street: "456 Oak Ave".to_string(),
            city: "London".to_string(),
            zip_code: Some("SW1A 1AA".to_string()),
        },
        metadata: Some("admin".to_string()),
    };
    
    let address_kp = keypath!(|u: &User| &u.address);
    let zip_code_kp = opt_keypath!(|a: &Address| a.zip_code.as_ref());
    
    // Chain KeyPath with OptionalKeyPath using +
    let user_zip_kp = address_kp + zip_code_kp;
    
    if let Some(zip) = user_zip_kp.get(&user) {
        println!("   User zip code: {}", zip);
    }
    println!("   ✓ KeyPath + OptionalKeyPath works!\n");
}

#[cfg(not(feature = "nightly"))]
fn example_keypath_to_optional() {
    println!("2. KeyPath + OptionalKeyPath (requires nightly feature)");
    println!("   Use keypath1.then_optional(opt_keypath2) instead on stable Rust\n");
}

#[cfg(feature = "nightly")]
fn example_optional_chaining() {
    use std::ops::Add;
    
    println!("3. OptionalKeyPath + OptionalKeyPath");
    
    let company = Company {
        name: "Acme Corp".to_string(),
        owner: Some(User {
            name: "Charlie".to_string(),
            age: 40,
            address: Address {
                street: "789 Pine Rd".to_string(),
                city: "Paris".to_string(),
                zip_code: Some("75001".to_string()),
            },
            metadata: Some("founder".to_string()),
        }),
    };
    
    let owner_kp = opt_keypath!(|c: &Company| c.owner.as_ref());
    let address_kp = opt_keypath!(|u: &User| Some(&u.address));
    let street_kp = opt_keypath!(|a: &Address| Some(&a.street));
    
    // Chain multiple OptionalKeyPaths using +
    let company_owner_street_kp = owner_kp + address_kp + street_kp;
    
    if let Some(street) = company_owner_street_kp.get(&company) {
        println!("   Company owner's street: {}", street);
    }
    println!("   ✓ OptionalKeyPath + OptionalKeyPath works!\n");
}

#[cfg(not(feature = "nightly"))]
fn example_optional_chaining() {
    println!("3. OptionalKeyPath + OptionalKeyPath (requires nightly feature)");
    println!("   Use opt_keypath1.then(opt_keypath2) instead on stable Rust\n");
}

#[cfg(feature = "nightly")]
fn example_writable_chaining() {
    use std::ops::Add;
    
    println!("4. WritableKeyPath + WritableKeyPath");
    
    let mut user = User {
        name: "David".to_string(),
        age: 35,
        address: Address {
            street: "321 Elm St".to_string(),
            city: "Tokyo".to_string(),
            zip_code: None,
        },
        metadata: None,
    };
    
    let address_wkp = writable_keypath!(|u: &mut User| &mut u.address);
    let city_wkp = writable_keypath!(|a: &mut Address| &mut a.city);
    
    // Chain writable keypaths using +
    let user_city_wkp = address_wkp + city_wkp;
    
    *user_city_wkp.get_mut(&mut user) = "Osaka".to_string();
    
    println!("   Updated city: {}", user.address.city);
    println!("   ✓ WritableKeyPath + WritableKeyPath works!\n");
}

#[cfg(not(feature = "nightly"))]
fn example_writable_chaining() {
    println!("4. WritableKeyPath + WritableKeyPath (requires nightly feature)");
    println!("   Use writable_keypath1.then(writable_keypath2) instead on stable Rust\n");
}

#[cfg(feature = "nightly")]
fn example_writable_to_optional() {
    use std::ops::Add;
    
    println!("5. WritableKeyPath + WritableOptionalKeyPath");
    
    let mut user = User {
        name: "Eve".to_string(),
        age: 28,
        address: Address {
            street: "654 Maple Dr".to_string(),
            city: "Berlin".to_string(),
            zip_code: Some("10115".to_string()),
        },
        metadata: Some("developer".to_string()),
    };
    
    let address_wkp = writable_keypath!(|u: &mut User| &mut u.address);
    let zip_code_wokp = writable_opt_keypath!(|a: &mut Address| a.zip_code.as_mut());
    
    // Chain WritableKeyPath with WritableOptionalKeyPath using +
    let user_zip_wokp = address_wkp + zip_code_wokp;
    
    if let Some(zip) = user_zip_wokp.get_mut(&mut user) {
        *zip = "10116".to_string();
        println!("   Updated zip code: {}", zip);
    }
    println!("   ✓ WritableKeyPath + WritableOptionalKeyPath works!\n");
}

#[cfg(not(feature = "nightly"))]
fn example_writable_to_optional() {
    println!("5. WritableKeyPath + WritableOptionalKeyPath (requires nightly feature)");
    println!("   Use writable_keypath1.then_optional(writable_opt_keypath2) instead on stable Rust\n");
}

#[cfg(feature = "nightly")]
fn example_writable_optional_chaining() {
    use std::ops::Add;
    
    println!("6. WritableOptionalKeyPath + WritableOptionalKeyPath");
    
    let mut company = Company {
        name: "Tech Inc".to_string(),
        owner: Some(User {
            name: "Frank".to_string(),
            age: 45,
            address: Address {
                street: "987 Cedar Ln".to_string(),
                city: "Sydney".to_string(),
                zip_code: Some("2000".to_string()),
            },
            metadata: Some("CEO".to_string()),
        }),
    };
    
    let owner_wokp = writable_opt_keypath!(|c: &mut Company| c.owner.as_mut());
    let metadata_wokp = writable_opt_keypath!(|u: &mut User| u.metadata.as_mut());
    
    // Chain WritableOptionalKeyPaths using +
    let company_owner_metadata_wokp = owner_wokp + metadata_wokp;
    
    if let Some(metadata) = company_owner_metadata_wokp.get_mut(&mut company) {
        *metadata = "Founder & CEO".to_string();
        println!("   Updated owner metadata: {}", metadata);
    }
    println!("   ✓ WritableOptionalKeyPath + WritableOptionalKeyPath works!\n");
}

#[cfg(not(feature = "nightly"))]
fn example_writable_optional_chaining() {
    println!("6. WritableOptionalKeyPath + WritableOptionalKeyPath (requires nightly feature)");
    println!("   Use writable_opt_keypath1.then(writable_opt_keypath2) instead on stable Rust\n");
}

fn example_comparison() {
    println!("7. Comparison: + operator vs then() method");
    
    let user = User {
        name: "Grace".to_string(),
        age: 32,
        address: Address {
            street: "111 Willow Way".to_string(),
            city: "San Francisco".to_string(),
            zip_code: Some("94102".to_string()),
        },
        metadata: None,
    };
    
    let address_kp = keypath!(|u: &User| &u.address);
    let street_kp = keypath!(|a: &Address| &a.street);
    
    // Using then() method (works on stable Rust)
    let user_street_then = address_kp.clone().then(street_kp.clone());
    println!("   Using then(): {}", user_street_then.get(&user));
    
    #[cfg(feature = "nightly")]
    {
        use std::ops::Add;
        
        // Using + operator (requires nightly feature)
        let user_street_add = address_kp + street_kp;
        println!("   Using +: {}", user_street_add.get(&user));
        
        println!("   ✓ Both methods produce the same result!\n");
    }
    
    #[cfg(not(feature = "nightly"))]
    {
        println!("   Using +: (requires nightly feature)");
        println!("   ✓ Use then() method on stable Rust for the same functionality!\n");
    }
    
    println!("=== Summary ===");
    println!("The + operator provides a convenient syntax for chaining keypaths.");
    println!("The + operator requires nightly Rust with the 'nightly' feature.");
    println!("On stable Rust, use the then() methods which provide the same functionality.");
}

