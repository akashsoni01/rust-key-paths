// Example demonstrating the WithContainer trait usage
// Run with: cargo run --example with_container_trait_example

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath, WithContainer};
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
    println!("=== WithContainer Trait Example ===\n");

    // Create test data
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
    };

    // Create keypaths
    let name_path = KeyPath::new(|u: &User| &u.name);
    let age_path = KeyPath::new(|u: &User| &u.age);
    let name_path_w = WritableKeyPath::new(|u: &mut User| &mut u.name);

    // ===== Example 1: Trait Usage with Arc =====
    println!("--- Example 1: Trait Usage with Arc ---");
    
    let arc_user = Arc::new(user.clone());

    // Using the trait method
    name_path.clone().with_arc(&arc_user, |name| {
        println!("  Name from Arc (via trait): {}", name);
    });

    // ===== Example 2: Trait Usage with Box =====
    println!("--- Example 2: Trait Usage with Box ---");
    
    let mut boxed_user = Box::new(user.clone());

    // Read via trait
    name_path.clone().with_box(&boxed_user, |name| {
        println!("  Name from Box (via trait): {}", name);
    });

    // Write via trait
    name_path_w.clone().with_box_mut(&mut boxed_user, |name| {
        *name = "Alice Boxed".to_string();
        println!("  Updated name in Box (via trait): {}", name);
    });

    // ===== Example 3: Trait Usage with Rc =====
    println!("--- Example 3: Trait Usage with Rc ---");
    
    let rc_user = Rc::new(user.clone());

    // Using the trait method
    name_path.clone().with_rc(&rc_user, |name| {
        println!("  Name from Rc (via trait): {}", name);
    });

    // ===== Example 4: Trait Usage with Result =====
    println!("--- Example 4: Trait Usage with Result ---");
    
    let mut result_user: Result<User, String> = Ok(user.clone());

    // Read via trait
    if let Some(name) = name_path.clone().with_result(&result_user, |name| name.clone()) {
        println!("  Name from Result (via trait): {}", name);
    }

    // Write via trait
    if let Some(()) = name_path_w.clone().with_result_mut(&mut result_user, |name| {
        *name = "Alice Result".to_string();
        println!("  Updated name in Result (via trait): {}", name);
    }) {
        println!("  Successfully updated Result via trait");
    }

    // ===== Example 5: Trait Usage with Option =====
    println!("--- Example 5: Trait Usage with Option ---");
    
    let mut option_user: Option<User> = Some(user.clone());

    // Read via trait
    if let Some(name) = name_path.clone().with_option(&option_user, |name| name.clone()) {
        println!("  Name from Option (via trait): {}", name);
    }

    // Write via trait
    if let Some(()) = name_path_w.clone().with_option_mut(&mut option_user, |name| {
        *name = "Alice Option".to_string();
        println!("  Updated name in Option (via trait): {}", name);
    }) {
        println!("  Successfully updated Option via trait");
    }

    // ===== Example 6: Trait Usage with RefCell =====
    println!("--- Example 6: Trait Usage with RefCell ---");
    
    let refcell_user = RefCell::new(user.clone());

    // Read via trait
    if let Some(name) = name_path.clone().with_refcell(&refcell_user, |name| name.clone()) {
        println!("  Name from RefCell (via trait): {}", name);
    }

    // Write via trait
    if let Some(()) = name_path_w.clone().with_refcell_mut(&refcell_user, |name| {
        *name = "Alice RefCell".to_string();
        println!("  Updated name in RefCell (via trait): {}", name);
    }) {
        println!("  Successfully updated RefCell via trait");
    }

    // ===== Example 7: Trait Usage with Mutex =====
    println!("--- Example 7: Trait Usage with Mutex ---");
    
    let mutex_user = Mutex::new(user.clone());

    // Read via trait
    name_path.clone().with_mutex(&mutex_user, |name| {
        println!("  Name from Mutex (via trait): {}", name);
    });

    // Write via trait
    let mut mutex_user_mut = Mutex::new(user.clone());
    name_path_w.clone().with_mutex_mut(&mut mutex_user_mut, |name| {
        *name = "Alice Mutexed".to_string();
        println!("  Updated name in Mutex (via trait): {}", name);
    });

    // ===== Example 8: Trait Usage with RwLock =====
    println!("--- Example 8: Trait Usage with RwLock ---");
    
    let rwlock_user = RwLock::new(user.clone());

    // Read via trait
    name_path.clone().with_rwlock(&rwlock_user, |name| {
        println!("  Name from RwLock (via trait): {}", name);
    });

    // Write via trait
    let mut rwlock_user_mut = RwLock::new(user.clone());
    let age_path_w = WritableKeyPath::new(|u: &mut User| &mut u.age);
    age_path_w.clone().with_rwlock_mut(&mut rwlock_user_mut, |age| {
        *age += 1;
        println!("  Updated age in RwLock (via trait): {}", age);
    });

    // ===== Example 9: Generic Function Using Trait =====
    println!("--- Example 9: Generic Function Using Trait ---");
    
    fn process_user_name<T>(keypath: KeyPath<User, String, impl for<\'r> Fn(&\'r User) -> &\'r String>, container: T)
    where
        T: WithContainer<User, String>,
    {
        // This would work if we had a generic way to call the trait methods
        // For now, we'll demonstrate the concept
        println!("  Generic function would process user name via trait");
    }

    // ===== Example 10: Trait Benefits =====
    println!("--- Example 10: Trait Benefits ---");
    
    println!("  ✅ Clean API: All with_* methods are organized under one trait");
    println!("  ✅ Extensibility: Easy to add new container types");
    println!("  ✅ Consistency: All methods follow the same pattern");
    println!("  ✅ Documentation: Centralized documentation for all container methods");
    println!("  ✅ Type Safety: Compile-time guarantees for container access");

    println!("=== All Examples Completed Successfully! ===");
}
