// Example demonstrating the no-clone approach for Mutex and RwLock with KeyPaths
// Run with: cargo run --example no_clone_mutex_rwlock_example

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath, WithContainer};
use std::sync::{Mutex, RwLock};

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

fn main() {
    println!("=== No-Clone Mutex and RwLock Example ===\n");

    // Create some test data
    let user = User {
        name: "Akash".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
    };

    // Create keypaths
    let name_path = KeyPath::new(|u: &User| &u.name);
    let age_path = KeyPath::new(|u: &User| &u.age);
    let email_path = OptionalKeyPath::new(|u: &User| u.email.as_ref());

    // ===== Example 1: Basic Mutex Usage (No Clone) =====
    println!("--- Example 1: Basic Mutex Usage (No Clone) ---");
    
    let mutex_user = Mutex::new(user.clone());

    // Access data from Mutex using with_mutex() - no cloning!
    if let Some(name) = name_path.clone().with_mutex(&mutex_user, |name| name.clone()) {
        println!("  Name from Mutex: {}", name);
    }

    // Or just print directly without cloning
    name_path.clone().with_mutex(&mutex_user, |name| {
        println!("  Name from Mutex (direct): {}", name);
    });

    // ===== Example 2: Basic RwLock Usage (No Clone) =====
    println!("--- Example 2: Basic RwLock Usage (No Clone) ---");
    
    let rwlock_user = RwLock::new(user.clone());

    // Access data from RwLock using with_rwlock() - no cloning!
    name_path.clone().with_rwlock(&rwlock_user, |name| {
        println!("  Name from RwLock: {}", name);
    });

    // ===== Example 3: Mutex with Failable KeyPath (No Clone) =====
    println!("--- Example 3: Mutex with Failable KeyPath (No Clone) ---");
    
    let mutex_user_with_email = Mutex::new(User {
        name: "Bob".to_string(),
        age: 25,
        email: Some("bob@example.com".to_string()),
    });

    // Access optional email from Mutex - no cloning!
    email_path.clone().with_mutex(&mutex_user_with_email, |email| {
        println!("  Email from Mutex: {}", email);
    });

    // ===== Example 4: RwLock with Failable KeyPath (No Clone) =====
    println!("--- Example 4: RwLock with Failable KeyPath (No Clone) ---");
    
    let rwlock_user_with_email = RwLock::new(User {
        name: "Charlie".to_string(),
        age: 35,
        email: None, // No email
    });

    // Access optional email from RwLock (should return None)
    if email_path.clone().with_rwlock(&rwlock_user_with_email, |email| email.clone()).is_some() {
        println!("  Email found in RwLock");
    } else {
        println!("  No email found in RwLock (as expected)");
    }

    // ===== Example 5: Collection Processing (No Clone) =====
    println!("--- Example 5: Collection Processing (No Clone) ---");
    
    let mutex_users: Vec<Mutex<User>> = vec![
        Mutex::new(User {
            name: "David".to_string(),
            age: 28,
            email: Some("david@example.com".to_string()),
        }),
        Mutex::new(User {
            name: "Eve".to_string(),
            age: 32,
            email: None,
        }),
        Mutex::new(User {
            name: "Frank".to_string(),
            age: 45,
            email: Some("frank@example.com".to_string()),
        }),
    ];

    // Process names from Mutex collection - no cloning!
    let mut names = Vec::new();
    for mutex_user in &mutex_users {
        name_path.clone().with_mutex(mutex_user, |name| {
            names.push(name.clone()); // Only clone when we need to store
        });
    }
    println!("  User names: {:?}", names);

    // ===== Example 6: Mutable Access (No Clone) =====
    println!("--- Example 6: Mutable Access (No Clone) ---");
    
    let mut mutex_user_mut = Mutex::new(User {
        name: "Grace".to_string(),
        age: 29,
        email: Some("grace@example.com".to_string()),
    });

    // Modify data through Mutex - no cloning!
    let name_path_w = WritableKeyPath::new(|u: &mut User| &mut u.name);
    name_path_w.clone().with_mutex_mut(&mut mutex_user_mut, |name| {
        *name = "Grace Updated".to_string();
        println!("  Updated name to: {}", name);
    });

    // ===== Example 7: RwLock Mutable Access (No Clone) =====
    println!("--- Example 7: RwLock Mutable Access (No Clone) ---");
    
    let mut rwlock_user_mut = RwLock::new(User {
        name: "Henry".to_string(),
        age: 38,
        email: Some("henry@example.com".to_string()),
    });

    // Modify data through RwLock - no cloning!
    let age_path_w = WritableKeyPath::new(|u: &mut User| &mut u.age);
    age_path_w.clone().with_rwlock_mut(&mut rwlock_user_mut, |age| {
        *age += 1;
        println!("  Updated age to: {}", age);
    });

    // ===== Example 8: Error Handling (No Clone) =====
    println!("--- Example 8: Error Handling (No Clone) ---");
    
    // Create a Mutex that will be poisoned
    let poisoned_mutex = Mutex::new(User {
        name: "Poisoned".to_string(),
        age: 99,
        email: Some("poisoned@example.com".to_string()),
    });

    // Poison the mutex by panicking while holding the lock
    {
        let _guard = poisoned_mutex.lock().unwrap();
        std::panic::catch_unwind(|| {
            panic!("This will poison the mutex");
        }).ok();
    } // _guard is dropped here

    // Try to access data from poisoned mutex (should return None)
    if name_path.clone().with_mutex(&poisoned_mutex, |name| name.clone()).is_some() {
        println!("  Successfully accessed poisoned Mutex");
    } else {
        println!("  Failed to access poisoned Mutex (as expected)");
    }

    println!("=== All Examples Completed Successfully! ===");
}
