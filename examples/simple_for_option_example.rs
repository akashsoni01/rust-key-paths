// Simple example demonstrating the for_option adapter method
// Run with: cargo run --example simple_for_option_example

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath, WithContainer};

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

fn main() {
    println!("=== Simple For Option Adapter Example ===\n");

    // Create test data
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
    };

    // Create keypaths
    let name_path = KeyPath::new(|u: &User| &u.name);
    let age_path = KeyPath::new(|u: &User| &u.age);
    let email_path = OptionalKeyPath::new(|u: &User| u.email.as_ref());
    let name_path_w = WritableKeyPath::new(|u: &mut User| &mut u.name);

    // ===== Example 1: Basic Option Usage =====
    println!("--- Example 1: Basic Option Usage ---");
    
    let option_user: Option<User> = Some(user.clone());

    // Use for_option to create a keypath that works with Option<User>
    let name_option_path = name_path.clone().for_option();
    
    // Access name from Option<User> - returns Option<&String>
    if let Some(name) = name_option_path.get(&option_user) {
        println!("  Name from Option: {}", name);
    }

    // ===== Example 2: Writable Option Usage =====
    println!("--- Example 2: Writable Option Usage ---");
    
    let mut option_user_mut: Option<User> = Some(user.clone());

    // Use for_option with writable keypath
    let name_option_path_w = name_path_w.clone().for_option();
    
    // Modify name in Option<User>
    let name = name_option_path_w.get_mut(&mut &mut option_user_mut);
    {
        *name = "Alice Updated".to_string();
        println!("  Updated name in Option: {}", name);
    }

    // ===== Example 3: Failable KeyPath with Option =====
    println!("--- Example 3: Failable KeyPath with Option ---");
    
    let option_user_with_email: Option<User> = Some(User {
        name: "Bob".to_string(),
        age: 25,
        email: Some("bob@example.com".to_string()),
    });

    // Use failable keypath with for_option
    let email_option_path = email_path.clone().for_option();
    
    // Access email from Option<User> - returns Option<Option<&String>>
    if let Some(email) = email_option_path.get(&option_user_with_email) {
        println!("  Email from Option: {}", email);
    } else {
        println!("  No user in Option");
    }

    // ===== Example 4: None Option Handling =====
    println!("--- Example 4: None Option Handling ---");
    
    let none_user: Option<User> = None;

    // Try to access name from None Option
    if let Some(name) = name_option_path.get(&none_user) {
        println!("  Name from None Option: {}", name);
    } else {
        println!("  Correctly handled None Option");
    }

    // ===== Example 5: Collection of Options =====
    println!("--- Example 5: Collection of Options ---");
    
    let option_users: Vec<Option<User>> = vec![
        Some(User {
            name: "Charlie".to_string(),
            age: 35,
            email: Some("charlie@example.com".to_string()),
        }),
        None,
        Some(User {
            name: "Diana".to_string(),
            age: 28,
            email: None,
        }),
    ];

    // Process names from collection of Options
    let mut names = Vec::new();
    for option_user in &option_users {
        if let Some(name) = name_option_path.get(&option_user) {
            names.push(name.clone());
        }
    }
    println!("  User names from Option collection: {:?}", names);

    // ===== Example 6: Using with_option (No-Clone Approach) =====
    println!("--- Example 6: Using with_option (No-Clone Approach) ---");
    
    let option_user_for_with: Option<User> = Some(user.clone());

    // Use the original keypath with with_option for no-clone access
    name_path.clone().with_option(&option_user_for_with, |name| {
        println!("  Name from Option (no-clone): {}", name);
    });

    // ===== Example 7: Comparison: for_option vs with_option =====
    println!("--- Example 7: Comparison: for_option vs with_option ---");
    
    let option_user_comp: Option<User> = Some(user.clone());

    // Method 1: for_option + get_ref (creates new keypath type)
    println!("  Method 1 - for_option + get_ref:");
    if let Some(name) = name_path.clone().for_option().get(&option_user_comp) {
        println!("    Name: {}", name);
    }

    // Method 2: with_option (no-clone callback)
    println!("  Method 2 - with_option (no-clone):");
    name_path.clone().with_option(&option_user_comp, |name| {
        println!("    Name: {}", name);
    });

    println!("=== All Examples Completed Successfully! ===");
}
