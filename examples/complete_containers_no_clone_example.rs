// Complete example demonstrating ALL container types with no-clone callback methods
// Run with: cargo run --example complete_containers_no_clone_example

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use std::sync::{Arc, Mutex, RwLock};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

fn main() {
    println!("=== Complete Containers No-Clone Example ===\n");

    // Create test data
    let user = User {
        name: "Akash".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
    };

    // Create keypaths
    let name_path = KeyPath::new(|u: &User| &u.name);
    let age_path = KeyPath::new(|u: &User| &u.age);
    let email_path = OptionalKeyPath::new(|u: &User| u.email.as_ref());
    let name_path_w = WritableKeyPath::new(|u: &mut User| &mut u.name);
    let age_path_w = WritableKeyPath::new(|u: &mut User| &mut u.age);

    // ===== Example 1: Arc (Read-only) =====
    println!("--- Example 1: Arc (Read-only) ---");
    
    let arc_user = Arc::new(user.clone());
    name_path.clone().with_arc(&arc_user, |name| {
        println!("  Name from Arc: {}", name);
    });

    // ===== Example 2: Box (Read and Write) =====
    println!("--- Example 2: Box (Read and Write) ---");
    
    let mut boxed_user = Box::new(user.clone());
    name_path.clone().with_box(&boxed_user, |name| {
        println!("  Name from Box: {}", name);
    });
    
    name_path_w.clone().with_box_mut(&mut boxed_user, |name| {
        *name = "Akash Boxed".to_string();
        println!("  Updated name in Box: {}", name);
    });

    // ===== Example 3: Rc (Read-only) =====
    println!("--- Example 3: Rc (Read-only) ---");
    
    let rc_user = Rc::new(user.clone());
    name_path.clone().with_rc(&rc_user, |name| {
        println!("  Name from Rc: {}", name);
    });

    // ===== Example 4: Result (Read and Write) =====
    println!("--- Example 4: Result (Read and Write) ---");
    
    let mut result_user: Result<User, String> = Ok(user.clone());
    if let Some(name) = name_path.clone().with_result(&result_user, |name| name.clone()) {
        println!("  Name from Result: {}", name);
    }
    
    if let Some(()) = name_path_w.clone().with_result_mut(&mut result_user, |name| {
        *name = "Akash Result".to_string();
        println!("  Updated name in Result: {}", name);
    }) {
        println!("  Successfully updated Result");
    }

    // Test with Err Result
    let err_result: Result<User, String> = Err("User not found".to_string());
    if name_path.clone().with_result(&err_result, |name| name.clone()).is_none() {
        println!("  Correctly handled Err Result");
    }

    // ===== Example 5: Option (Read and Write) =====
    println!("--- Example 5: Option (Read and Write) ---");
    
    let mut option_user: Option<User> = Some(user.clone());
    if let Some(name) = name_path.clone().with_option(&option_user, |name| name.clone()) {
        println!("  Name from Option: {}", name);
    }
    
    if let Some(()) = name_path_w.clone().with_option_mut(&mut option_user, |name| {
        *name = "Akash Option".to_string();
        println!("  Updated name in Option: {}", name);
    }) {
        println!("  Successfully updated Option");
    }

    // Test with None Option
    let none_option: Option<User> = None;
    if name_path.clone().with_option(&none_option, |name| name.clone()).is_none() {
        println!("  Correctly handled None Option");
    }

    // ===== Example 6: RefCell (Read and Write) =====
    println!("--- Example 6: RefCell (Read and Write) ---");
    
    let refcell_user = RefCell::new(user.clone());
    if let Some(name) = name_path.clone().with_refcell(&refcell_user, |name| name.clone()) {
        println!("  Name from RefCell: {}", name);
    }
    
    if let Some(()) = name_path_w.clone().with_refcell_mut(&refcell_user, |name| {
        *name = "Akash RefCell".to_string();
        println!("  Updated name in RefCell: {}", name);
    }) {
        println!("  Successfully updated RefCell");
    }

    // ===== Example 7: Mutex (Read and Write) =====
    println!("--- Example 7: Mutex (Read and Write) ---");
    
    let mutex_user = Mutex::new(user.clone());
    name_path.clone().with_mutex(&mutex_user, |name| {
        println!("  Name from Mutex: {}", name);
    });
    
    let mut mutex_user_mut = Mutex::new(user.clone());
    name_path_w.clone().with_mutex_mut(&mut mutex_user_mut, |name| {
        *name = "Akash Mutexed".to_string();
        println!("  Updated name in Mutex: {}", name);
    });

    // ===== Example 8: RwLock (Read and Write) =====
    println!("--- Example 8: RwLock (Read and Write) ---");
    
    let rwlock_user = RwLock::new(user.clone());
    name_path.clone().with_rwlock(&rwlock_user, |name| {
        println!("  Name from RwLock: {}", name);
    });
    
    let mut rwlock_user_mut = RwLock::new(user.clone());
    age_path_w.clone().with_rwlock_mut(&mut rwlock_user_mut, |age| {
        *age += 1;
        println!("  Updated age in RwLock: {}", age);
    });

    // ===== Example 9: Collection Processing (No Clone) =====
    println!("--- Example 9: Collection Processing (No Clone) ---");
    
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
    ];

    // Process names from Arc collection - no cloning!
    let mut names = Vec::new();
    for arc_user in &arc_users {
        name_path.clone().with_arc(arc_user, |name| {
            names.push(name.clone()); // Only clone when we need to store
        });
    }
    println!("  User names from Arc collection: {:?}", names);

    // ===== Example 10: Mixed Container Types =====
    println!("--- Example 10: Mixed Container Types ---");
    
    let mixed_containers = vec![
        ("Arc", Box::new(Arc::new(user.clone())) as Box<dyn std::fmt::Debug>),
        ("Box", Box::new(Box::new(user.clone())) as Box<dyn std::fmt::Debug>),
        ("Rc", Box::new(Rc::new(user.clone())) as Box<dyn std::fmt::Debug>),
        ("Option", Box::new(Some(user.clone())) as Box<dyn std::fmt::Debug>),
        ("RefCell", Box::new(RefCell::new(user.clone())) as Box<dyn std::fmt::Debug>),
    ];

    println!("  Created mixed container collection with {} types:", mixed_containers.len());
    for (name, _) in &mixed_containers {
        println!("    - {}", name);
    }

    // ===== Example 11: Error Handling =====
    println!("--- Example 11: Error Handling ---");
    
    // Test with poisoned Mutex
    let poisoned_mutex = Mutex::new(user.clone());
    {
        let _guard = poisoned_mutex.lock().unwrap();
        std::panic::catch_unwind(|| {
            panic!("This will poison the mutex");
        }).ok();
    }

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

    // Test with None Option
    let none_option: Option<User> = None;
    if name_path.clone().with_option(&none_option, |name| name.clone()).is_none() {
        println!("  Correctly handled None Option");
    }

    // Test with RefCell borrow failure
    let refcell_user = RefCell::new(user.clone());
    let _borrow = refcell_user.borrow(); // Hold a borrow
    if name_path.clone().with_refcell(&refcell_user, |name| name.clone()).is_none() {
        println!("  Correctly handled RefCell borrow failure");
    }

    println!("=== All Examples Completed Successfully! ===");
}
