// Example demonstrating the for_option adapter method
// Run with: cargo run --example for_option_example

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath, WithContainer};

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

#[derive(Debug, Clone)]
struct Profile {
    user: Option<User>,
    settings: Option<String>,
}

fn main() {
    println!("=== For Option Adapter Example ===\n");

    // Create test data
    let user = User {
        name: "Akash".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
    };

    let profile = Profile {
        user: Some(user.clone()),
        settings: Some("dark_mode".to_string()),
    };

    // Create keypaths
    let name_path = KeyPath::new(|u: &User| &u.name);
    let age_path = KeyPath::new(|u: &User| &u.age);
    let email_path = OptionalKeyPath::new(|u: &User| u.email.as_ref());
    let name_path_w = WritableKeyPath::new(|u: &mut User| &mut u.name);
    let age_path_w = WritableKeyPath::new(|u: &mut User| &mut u.age);

    // ===== Example 1: Basic Option Usage =====
    println!("--- Example 1: Basic Option Usage ---");
    
    let mut option_user: Option<User> = Some(user.clone());

    // Use for_option to create a keypath that works with Option<User>
    let name_option_path = name_path.clone().for_option();
    
    // Access name from Option<User> using get_ref
    if let Some(name) = name_option_path.get(&option_user) {
        println!("  Name from Option: {}", name);
    }

    // ===== Example 2: Writable Option Usage =====
    println!("--- Example 2: Writable Option Usage ---");
    
    let mut option_user_mut: Option<User> = Some(user.clone());

    // Use for_option with writable keypath
    let name_option_path_w = name_path_w.clone().for_option();
    
    // Modify name in Option<User> using get_mut
    if let Some(name) = name_option_path_w.get_mut(&mut option_user_mut) {
        *name = "Akash Updated".to_string();
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
    
    // Access email from Option<User> using get_ref
    if let Some(email) = email_option_path.get(&option_user_with_email) {
        println!("  Email from Option: {}", email);
    } else {
        println!("  No email in user");
    }

    // ===== Example 4: None Option Handling =====
    println!("--- Example 4: None Option Handling ---");
    
    let none_user: Option<User> = None;

    // Try to access name from None Option using get_ref
    if name_option_path.get(&none_user).is_some() {
        println!("  Name from None Option");
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

    // Process names from collection of Options using get_ref
    let mut names = Vec::new();
    for option_user in &option_users {
        if let Some(name) = name_option_path.get(&option_user) {
            names.push(name.clone());
        }
    }
    println!("  User names from Option collection: {:?}", names);

    // ===== Example 6: Nested Option Structure =====
    println!("--- Example 6: Nested Option Structure ---");
    
    let mut option_profile: Option<Profile> = Some(profile.clone());

    // Create a keypath that goes through Option<Profile> -> Option<User> -> String
    let profile_user_name_path = OptionalKeyPath::new(|p: &Profile| p.user.as_ref())
        .then(name_path.clone().to_optional());

    // Use for_option to work with Option<Profile>
    let profile_name_option_path = profile_user_name_path.for_option();

    // Access nested name through Option<Profile> using get_ref
    if let Some(name) = profile_name_option_path.get(&option_profile) {
        println!("  Nested name from Option<Profile>: {}", name);
    }

    // ===== Example 7: Mutable Nested Option =====
    println!("--- Example 7: Mutable Nested Option ---");
    
    let mut option_profile_mut: Option<Profile> = Some(profile.clone());

    // Create a writable keypath for nested Option<Profile> -> Option<User> -> String
    let profile_user_name_path_w = WritableOptionalKeyPath::new(|p: &mut Profile| p.user.as_mut())
        .then(name_path_w.clone().to_optional());

    // Use for_option to work with Option<Profile>
    let profile_name_option_path_w = profile_user_name_path_w.for_option();

    // Modify nested name through Option<Profile> using get_mut
    if let Some(name) = profile_name_option_path_w.get_mut(&mut option_profile_mut) {
        *name = "Akash Profile".to_string();
        println!("  Updated nested name in Option<Profile>: {}", name);
    }

    // ===== Example 8: Composition with for_option =====
    println!("--- Example 8: Composition with for_option ---");
    
    let option_user_comp: Option<User> = Some(user.clone());

    // Compose keypaths: Option<User> -> User -> Option<String> -> String
    let composed_path = name_path.clone()
        .for_option()  // KeyPaths<Option<User>, &String>
        .then(OptionalKeyPath::new(|s: &String| Some(s))); // KeyPaths<Option<&String>, &String>

    // This creates a complex nested Option structure
    println!("  Composed keypath created successfully");

    // ===== Example 9: Error Handling =====
    println!("--- Example 9: Error Handling ---");
    
    // Test with None at different levels
    let none_profile: Option<Profile> = None;
    if profile_name_option_path.get(&none_profile).is_some() {
        println!("  Name from None Profile");
    } else {
        println!("  Correctly handled None Profile");
    }

    // Test with Profile containing None user
    let profile_with_none_user = Profile {
        user: None,
        settings: Some("light_mode".to_string()),
    };
    let option_profile_none_user: Option<Profile> = Some(profile_with_none_user);
    
    if profile_name_option_path.get(&option_profile_none_user).is_some() {
        println!("  Name from Profile with None user");
    } else {
        println!("  Correctly handled Profile with None user");
    }

    println!("=== All Examples Completed Successfully! ===");
}
