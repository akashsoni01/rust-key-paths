// Example demonstrating all container types with no-clone callback methods
// Run with: cargo run --example all_containers_no_clone_example

use key_paths_core::{KeyPaths, WithContainer};
use std::sync::{Arc, Mutex, RwLock};
use std::rc::Rc;

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

fn main() {
    println!("=== All Containers No-Clone Example ===\n");

    // Create test data
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: Some("alice@example.com".to_string()),
    };

    // Create keypaths
    let name_path = KeyPaths::readable(|u: &User| &u.name);
    let age_path = KeyPaths::readable(|u: &User| &u.age);
    let email_path = KeyPaths::failable_readable(|u: &User| u.email.as_ref());
    let name_path_w = KeyPaths::writable(|u: &mut User| &mut u.name);
    let age_path_w = KeyPaths::writable(|u: &mut User| &mut u.age);

    // ===== Example 1: Arc (Read-only) =====
    println!("--- Example 1: Arc (Read-only) ---");
    
    let arc_user = Arc::new(user.clone());

    // Access data from Arc - no cloning!
    name_path.clone().with_arc(&arc_user, |name| {
        println!("  Name from Arc: {}", name);
    });

    age_path.clone().with_arc(&arc_user, |age| {
        println!("  Age from Arc: {}", age);
    });

    // ===== Example 2: Box (Read and Write) =====
    println!("--- Example 2: Box (Read and Write) ---");
    
    let mut boxed_user = Box::new(user.clone());

    // Read from Box - no cloning!
    name_path.clone().with_box(&boxed_user, |name| {
        println!("  Name from Box: {}", name);
    });

    // Write to Box - no cloning!
    name_path_w.clone().with_box_mut(&mut boxed_user, |name| {
        *name = "Alice Boxed".to_string();
        println!("  Updated name in Box: {}", name);
    });

    // ===== Example 3: Rc (Read-only) =====
    println!("--- Example 3: Rc (Read-only) ---");
    
    let rc_user = Rc::new(user.clone());

    // Access data from Rc - no cloning!
    name_path.clone().with_rc(&rc_user, |name| {
        println!("  Name from Rc: {}", name);
    });

    email_path.clone().with_rc(&rc_user, |email| {
        println!("  Email from Rc: {}", email);
    });

    // ===== Example 4: Result (Read and Write) =====
    println!("--- Example 4: Result (Read and Write) ---");
    
    let mut result_user: Result<User, String> = Ok(user.clone());

    // Read from Result - no cloning!
    if let Some(name) = name_path.clone().with_result(&result_user, |name| name.clone()) {
        println!("  Name from Result: {}", name);
    }

    // Write to Result - no cloning!
    if let Some(()) = name_path_w.clone().with_result_mut(&mut result_user, |name| {
        *name = "Alice Result".to_string();
        println!("  Updated name in Result: {}", name);
    }) {
        println!("  Successfully updated Result");
    }

    // Test with Err Result
    let err_result: Result<User, String> = Err("User not found".to_string());
    if name_path.clone().with_result(&err_result, |name| name.clone()).is_none() {
        println!("  Correctly handled Err Result");
    }

    // ===== Example 5: Mutex (Read and Write) =====
    println!("--- Example 5: Mutex (Read and Write) ---");
    
    let mutex_user = Mutex::new(user.clone());

    // Read from Mutex - no cloning!
    name_path.clone().with_mutex(&mutex_user, |name| {
        println!("  Name from Mutex: {}", name);
    });

    // Write to Mutex - no cloning!
    let mut mutex_user_mut = Mutex::new(user.clone());
    name_path_w.clone().with_mutex_mut(&mut mutex_user_mut, |name| {
        *name = "Alice Mutexed".to_string();
        println!("  Updated name in Mutex: {}", name);
    });

    // ===== Example 6: RwLock (Read and Write) =====
    println!("--- Example 6: RwLock (Read and Write) ---");
    
    let rwlock_user = RwLock::new(user.clone());

    // Read from RwLock - no cloning!
    name_path.clone().with_rwlock(&rwlock_user, |name| {
        println!("  Name from RwLock: {}", name);
    });

    // Write to RwLock - no cloning!
    let mut rwlock_user_mut = RwLock::new(user.clone());
    age_path_w.clone().with_rwlock_mut(&mut rwlock_user_mut, |age| {
        *age += 1;
        println!("  Updated age in RwLock: {}", age);
    });

    // ===== Example 7: Collection Processing (No Clone) =====
    println!("--- Example 7: Collection Processing (No Clone) ---");
    
    let arc_users: Vec<Arc<User>> = vec![
        Arc::new(User {
            name: "Bob".to_string(),
            age: 25,
            email: Some("bob@example.com".to_string()),
        }),
        Arc::new(User {
            name: "Charlie".to_string(),
            age: 35,
            email: None,
        }),
        Arc::new(User {
            name: "Diana".to_string(),
            age: 28,
            email: Some("diana@example.com".to_string()),
        }),
    ];

    // Process names from Arc collection - no cloning!
    let mut names = Vec::new();
    for arc_user in &arc_users {
        name_path.clone().with_arc(arc_user, |name| {
            names.push(name.clone()); // Only clone when we need to store
        });
    }
    println!("  User names from Arc collection: {:?}", names);

    // ===== Example 8: Box Collection Processing =====
    println!("--- Example 8: Box Collection Processing ---");
    
    let mut boxed_users: Vec<Box<User>> = vec![
        Box::new(User {
            name: "Eve".to_string(),
            age: 32,
            email: Some("eve@example.com".to_string()),
        }),
        Box::new(User {
            name: "Frank".to_string(),
            age: 45,
            email: None,
        }),
    ];

    // Read and modify Box collection - no cloning!
    for (i, boxed_user) in boxed_users.iter_mut().enumerate() {
        name_path.clone().with_box(boxed_user, |name| {
            println!("  User {}: {}", i + 1, name);
        });
        
        age_path_w.clone().with_box_mut(boxed_user, |age| {
            *age += 1; // Increment age
        });
    }

    // ===== Example 9: Mixed Container Types =====
    println!("--- Example 9: Mixed Container Types ---");
    
    let containers: Vec<Box<dyn std::fmt::Debug>> = vec![
        Box::new(Arc::new(user.clone())),
        Box::new(Box::new(user.clone())),
        Box::new(Rc::new(user.clone())),
    ];

    println!("  Created mixed container collection with {} items", containers.len());

    // ===== Example 10: Error Handling =====
    println!("--- Example 10: Error Handling ---");
    
    // Test with poisoned Mutex
    let poisoned_mutex = Mutex::new(user.clone());
    {
        let _guard = poisoned_mutex.lock().unwrap();
        std::panic::catch_unwind(|| {
            panic!("This will poison the mutex");
        }).ok();
    }

    // Try to access poisoned mutex (should return None)
    if name_path.clone().with_mutex(&poisoned_mutex, |name| name.clone()).is_some() {
        println!("  Successfully accessed poisoned Mutex");
    } else {
        println!("  Failed to access poisoned Mutex (as expected)");
    }

    // Test with Err Result
    let err_result: Result<User, String> = Err("Database error".to_string());
    if name_path.clone().with_result(&err_result, |name| name.clone()).is_none() {
        println!("  Correctly handled Err Result");
    }

    println!("=== All Examples Completed Successfully! ===");
}
