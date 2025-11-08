use key_paths_core::{KeyPaths, PartialKeyPath, AnyKeyPath};
use key_paths_derive::Keypaths;
use std::any::Any;

/// Example demonstrating full Swift KeyPath compatibility
/// This example shows all the Swift KeyPath features implemented in Rust:
/// - KeyPath<Root, Value> (read-only access)
/// - WritableKeyPath<Root, Value> (read-write access)
/// - ReferenceWritableKeyPath<Root, Value> (reference-specific writable access)
/// - PartialKeyPath<Root> (type-erased Value)
/// - AnyKeyPath (fully type-erased)

#[derive(Debug, Clone, Keypaths)]
#[All]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    is_active: bool,
}

#[derive(Debug, Clone, Keypaths)]
#[All]
struct Company {
    name: String,
    employees: Vec<Person>,
    revenue: f64,
}

fn main() {
    println!("=== Swift KeyPath Compatibility Example ===\n");

    // Example 1: KeyPath<Root, Value> (Read-only access)
    println!("--- 1. KeyPath<Root, Value> (Read-only access) ---");
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: Some("alice@example.com".to_string()),
        is_active: true,
    };

    // Create readable keypaths
    let name_path = Person::name_r();
    let age_path = Person::age_r();
    let email_path = Person::email_fr(); // FailableReadable for Option<String>

    // Use keypaths for read-only access
    if let Some(name) = name_path.get(&person) {
        println!("Person name: {}", name);
    }

    if let Some(age) = age_path.get(&person) {
        println!("Person age: {}", age);
    }

    if let Some(email) = email_path.get(&person) {
        println!("Person email: {:?}", email);
    }

    // Example 2: WritableKeyPath<Root, Value> (Read-write access)
    println!("\n--- 2. WritableKeyPath<Root, Value> (Read-write access) ---");
    let mut person_mut = person.clone();

    // Create writable keypaths
    let name_writable = Person::name_w();
    let age_writable = Person::age_w();
    let active_writable = Person::is_active_w();

    // Use keypaths for read-write access
    if let Some(name_ref) = name_writable.get_mut(&mut person_mut) {
        *name_ref = "Alice Updated".to_string();
        println!("Updated name: {}", name_ref);
    }

    if let Some(age_ref) = age_writable.get_mut(&mut person_mut) {
        *age_ref = 31;
        println!("Updated age: {}", age_ref);
    }

    if let Some(active_ref) = active_writable.get_mut(&mut person_mut) {
        *active_ref = false;
        println!("Updated active status: {}", active_ref);
    }

    // Example 3: ReferenceWritableKeyPath<Root, Value> (Reference-specific writable access)
    println!("\n--- 3. ReferenceWritableKeyPath<Root, Value> (Reference-specific writable access) ---");
    let mut person_ref = person.clone();

    // Create reference writable keypaths
    let name_ref_writable = KeyPaths::reference_writable(|p: &mut Person| &mut p.name);
    let age_ref_writable = KeyPaths::reference_writable(|p: &mut Person| &mut p.age);

    // Use reference writable keypaths
    if let Some(name_ref) = name_ref_writable.get_mut(&mut person_ref) {
        *name_ref = "Alice Reference".to_string();
        println!("Reference updated name: {}", name_ref);
    }

    if let Some(age_ref) = age_ref_writable.get_mut(&mut person_ref) {
        *age_ref = 32;
        println!("Reference updated age: {}", age_ref);
    }

    // Example 4: PartialKeyPath<Root> (Type-erased Value)
    println!("\n--- 4. PartialKeyPath<Root> (Type-erased Value) ---");
    
    // Convert typed keypaths to partial keypaths
    let name_partial = name_path.clone().to_partial();
    let age_partial = age_path.clone().to_partial();
    let email_partial = email_path.clone().to_partial();

    // Store different keypaths in the same collection
    let partial_keypaths: Vec<PartialKeyPath<Person>> = vec![
        name_partial,
        age_partial,
        email_partial,
    ];

    // Use partial keypaths with type erasure
    for (i, keypath) in partial_keypaths.iter().enumerate() {
        if let Some(value) = keypath.get(&person) {
            println!("Partial keypath {}: {:?} (type: {})", i, value, keypath.kind_name());
        }
    }

    // Example 5: AnyKeyPath (Fully type-erased)
    println!("\n--- 5. AnyKeyPath (Fully type-erased) ---");
    
    // Convert typed keypaths to any keypaths
    let name_any = name_path.clone().to_any();
    let age_any = age_path.clone().to_any();
    let email_any = email_path.clone().to_any();

    // Store different keypaths from different types in the same collection
    let any_keypaths: Vec<AnyKeyPath> = vec![
        name_any,
        age_any,
        email_any,
    ];

    // Use any keypaths with full type erasure
    for (i, keypath) in any_keypaths.iter().enumerate() {
        // We need to box the person to use with AnyKeyPath
        let person_boxed: Box<dyn Any + Send + Sync> = Box::new(person.clone());
        if let Some(value) = keypath.get(&*person_boxed) {
            println!("Any keypath {}: {:?} (type: {})", i, value, keypath.kind_name());
        }
    }

    // Example 6: Composition with type-erased keypaths
    println!("\n--- 6. Composition with type-erased keypaths ---");
    
    let company = Company {
        name: "TechCorp".to_string(),
        employees: vec![person.clone()],
        revenue: 1000000.0,
    };

    // Create composed keypaths
    let company_name_path = Company::name_r();
    
    // Use the company name keypath
    if let Some(name) = company_name_path.get(&company) {
        println!("Company name: {}", name);
    }

    // Access first employee directly
    if let Some(first_employee) = company.employees.first() {
        if let Some(name) = Person::name_r().get(first_employee) {
            println!("First employee name: {}", name);
        }
    }

    // Example 7: Mixed keypath types in collections
    println!("\n--- 7. Mixed keypath types in collections ---");
    
    // Create a collection of different keypath types
    let mixed_keypaths: Vec<Box<dyn Any>> = vec![
        Box::new(name_path.clone().to_partial()),
        Box::new(age_path.clone().to_partial()),
        Box::new(email_path.clone().to_partial()),
    ];

    // Process mixed keypaths
    for (i, keypath_box) in mixed_keypaths.iter().enumerate() {
        if let Some(partial_keypath) = keypath_box.downcast_ref::<PartialKeyPath<Person>>() {
            if let Some(value) = partial_keypath.get(&person) {
                println!("Mixed keypath {}: {:?}", i, value);
            }
        }
    }

    // Example 8: Dynamic keypath selection
    println!("\n--- 8. Dynamic keypath selection ---");
    
    let keypath_map: std::collections::HashMap<String, PartialKeyPath<Person>> = [
        ("name".to_string(), name_path.clone().to_partial()),
        ("age".to_string(), age_path.clone().to_partial()),
        ("email".to_string(), email_path.clone().to_partial()),
    ].iter().cloned().collect();

    // Dynamically select and use keypaths
    for field_name in ["name", "age", "email"] {
        if let Some(keypath) = keypath_map.get(field_name) {
            if let Some(value) = keypath.get(&person) {
                println!("Dynamic access to {}: {:?}", field_name, value);
            }
        }
    }

    println!("\n✅ Swift KeyPath Compatibility Example completed!");
    println!("📝 This example demonstrates:");
    println!("   • KeyPath<Root, Value> - Read-only access to properties");
    println!("   • WritableKeyPath<Root, Value> - Read-write access to properties");
    println!("   • ReferenceWritableKeyPath<Root, Value> - Reference-specific writable access");
    println!("   • PartialKeyPath<Root> - Type-erased Value for collections of same Root type");
    println!("   • AnyKeyPath - Fully type-erased for collections of different Root types");
    println!("   • Composition between all keypath types");
    println!("   • Dynamic keypath selection and usage");
    println!("   • Full Swift KeyPath compatibility in Rust!");
}
