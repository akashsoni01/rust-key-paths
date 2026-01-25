// Demonstrates the new reference support methods
// This example shows how to work with collections of references using keypaths
// cargo run --example simple_ref_support_example

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::Kp;

#[derive(Debug, Clone, Kp)]
#[All]
struct Person {
    name: String,
    age: u32,
    email: String,
}

fn main() {
    println!("=== Simple Reference Support Example ===\n");

    let person1 = Person {
        name: "Akash Johnson".to_string(),
        age: 30,
        email: "akash@example.com".to_string(),
    };

    let person2 = Person {
        name: "Bob Smith".to_string(),
        age: 25,
        email: "bob@example.com".to_string(),
    };

    let person3 = Person {
        name: "Charlie Brown".to_string(),
        age: 35,
        email: "charlie@example.com".to_string(),
    };

    // Example 1: Working with collections of references
    println!("--- Example 1: Collections of References ---");
    let people_refs = vec![&person1, &person2, &person3];
    
    // Extract names using the new extract_from_ref_slice method
    let name_path = Person::name_r();
    let names: Vec<&String> = name_path.extract_from_ref_slice(&people_refs);
    
    println!("  Names from references:");
    for name in &names {
        println!("    • {}", name);
    }

    // Example 2: Working with collections of mutable references
    println!("\n--- Example 2: Collections of Mutable References ---");
    let mut person1_mut = person1.clone();
    let mut person2_mut = person2.clone();
    let mut person3_mut = person3.clone();
    
    let mut people_mut_refs = vec![&mut person1_mut, &mut person2_mut, &mut person3_mut];
    
    // Extract mutable names using the new extract_mut_from_ref_slice method
    let name_mut_path = Person::name_w();
    let names_mut: Vec<&mut String> = name_mut_path.extract_mut_from_ref_slice(&mut people_mut_refs);
    
    // Modify the names
    for name in names_mut {
        name.push_str(" (Modified)");
    }
    
    println!("  Modified names:");
    println!("    • {}", person1_mut.name);
    println!("    • {}", person2_mut.name);
    println!("    • {}", person3_mut.name);

    // Example 3: Working with existing extract_from_slice method
    println!("\n--- Example 3: Existing extract_from_slice Method ---");
    let people_owned = vec![person1.clone(), person2.clone(), person3.clone()];
    
    // Extract ages using the existing method
    let age_path = Person::age_r();
    let ages: Vec<&u32> = age_path.extract_from_slice(&people_owned);
    
    println!("  Ages from owned values:");
    for age in &ages {
        println!("    • {}", age);
    }

    // Example 4: Working with existing extract_mut_from_slice method
    println!("\n--- Example 4: Existing extract_mut_from_slice Method ---");
    let mut people_owned_mut = vec![person1.clone(), person2.clone(), person3.clone()];
    
    // Extract mutable ages using the existing method
    let age_mut_path = Person::age_w();
    let ages_mut: Vec<&mut u32> = age_mut_path.extract_mut_from_slice(&mut people_owned_mut);
    
    // Modify the ages
    for age in ages_mut {
        *age += 1;
    }
    
    println!("  Modified ages:");
    for person in &people_owned_mut {
        println!("    • {}: {}", person.name, person.age);
    }

    // Example 5: Comparison of different approaches
    println!("\n--- Example 5: Comparison of Approaches ---");
    
    // Direct access (traditional approach)
    let direct_names: Vec<&String> = people_refs.iter().map(|p| &p.name).collect();
    println!("  Direct access: {:?}", direct_names);
    
    // KeyPath approach (new approach)
    let keypath_names: Vec<&String> = name_path.extract_from_ref_slice(&people_refs);
    println!("  KeyPath approach: {:?}", keypath_names);
    
    println!("\n✅ Simple reference support example completed!");
    println!("📝 Note: The new methods provide a clean way to work with collections of references");
    println!("   without needing to create KeyPaths<&Root, Value> types directly.");
}
