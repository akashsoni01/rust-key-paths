// Simple example demonstrating the for_mutex() adapter for KeyPaths
// Run with: cargo run --example simple_mutex_example

use key_paths_core::KeyPaths;
use std::sync::Mutex;

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    println!("=== Simple Mutex Adapter Example ===\n");

    // Create some test data
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    // Create keypaths
    let name_path = KeyPaths::readable(|u: &User| &u.name);
    let age_path = KeyPaths::readable(|u: &User| &u.age);

    // ===== Example 1: Basic Mutex Usage =====
    println!("--- Example 1: Basic Mutex Usage ---");
    
    let mutex_user = Mutex::new(user.clone());

    // Note: We'll create the adapted keypaths inline since they can't be cloned

    // Access data from Mutex using the no-clone approach
    name_path.clone().with_mutex(&mutex_user, |name| {
        println!("  Name from Mutex: {}", name);
    });

    // ===== Example 2: Collection of Mutex =====
    println!("--- Example 2: Collection of Mutex ---");
    
    let mutex_users: Vec<Mutex<User>> = vec![
        Mutex::new(User {
            name: "Bob".to_string(),
            age: 25,
        }),
        Mutex::new(User {
            name: "Charlie".to_string(),
            age: 35,
        }),
        Mutex::new(User {
            name: "Diana".to_string(),
            age: 28,
        }),
    ];

    // Extract names from Mutex collection using the no-clone approach
    let mut successful_names = Vec::new();
    for mutex_user in mutex_users {
        name_path.clone().with_mutex(&mutex_user, |name| {
            successful_names.push(name.clone()); // Only clone when we need to store
        });
    }

    println!("  Successful user names: {:?}", successful_names);

    println!("=== Example Completed Successfully! ===");
}