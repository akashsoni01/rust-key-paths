// Example demonstrating the for_result() adapter for KeyPaths
// Run with: cargo run --example result_adapter_example

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

fn main() {
    println!("=== Result Adapter Example ===\n");

    // Create some test data
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
    };

    // Create keypaths
    let name_path = KeyPath::new(|u: &User| &u.name);
    let age_path = KeyPath::new(|u: &User| &u.age);
    let email_path = OptionalKeyPath::new(|u: &User| u.email.as_ref());

    // ===== Example 1: Basic Result Usage =====
    println!("--- Example 1: Basic Result Usage ---");
    
    let ok_result = Ok(user.clone());
    let err_result: Result<User, String> = Err("User not found".to_string());

    // Adapt keypaths for Result
    let name_path_result = name_path.clone().for_result::<String>();
    let age_path_result = age_path.clone().for_result::<String>();
    let email_path_result = email_path.clone().for_result::<String>();

    // Access data from Ok result
    if let Some(name) = name_path_result.get(&ok_result) {
        println!("  Name from Ok result: {}", name);
    }

    if let Some(&age) = age_path_result.get(&ok_result) {
        println!("  Age from Ok result: {}", age);
    }

    if let Some(email) = email_path_result.get(&ok_result) {
        println!("  Email from Ok result: {}", email);
    }

    // Access data from Err result (should return None)
    if let Some(_) = name_path_result.get(&err_result) {
        println!("  This should not print!");
    } else {
        println!("  Name from Err result: None (as expected)");
    }

    println!("✓ Example 1 completed\n");

    // ===== Example 2: Collection of Results =====
    println!("--- Example 2: Collection of Results ---");
    
    let results: Vec<Result<User, String>> = vec![
        Ok(User {
            name: "Bob".to_string(),
            age: 25,
            email: Some("bob@example.com".to_string()),
        }),
        Err("Database error".to_string()),
        Ok(User {
            name: "Charlie".to_string(),
            age: 35,
            email: None,
        }),
        Err("Network timeout".to_string()),
        Ok(User {
            name: "Diana".to_string(),
            age: 28,
            email: Some("diana@example.com".to_string()),
        }),
    ];

    // Extract names from successful results
    let successful_names: Vec<&String> = results
        .iter()
        .filter_map(|result| name_path_result.get(result))
        .collect();

    println!("  Successful user names: {:?}", successful_names);

    // Calculate average age from successful results
    let ages: Vec<u32> = results
        .iter()
        .filter_map(|result| age_path_result.get(result).copied())
        .collect();

    let avg_age = if !ages.is_empty() {
        ages.iter().sum::<u32>() as f64 / ages.len() as f64
    } else {
        0.0
    };

    println!("  Average age from successful results: {:.1}", avg_age);

    // Count users with email addresses
    let users_with_email = results
        .iter()
        .filter(|result| email_path_result.get(result).is_some())
        .count();

    println!("  Users with email addresses: {}", users_with_email);
    println!("✓ Example 2 completed\n");

    // ===== Example 3: Error Handling Patterns =====
    println!("--- Example 3: Error Handling Patterns ---");
    
    let api_results: Vec<Result<User, &str>> = vec![
        Ok(User {
            name: "Eve".to_string(),
            age: 32,
            email: Some("eve@example.com".to_string()),
        }),
        Err("Invalid JSON"),
        Ok(User {
            name: "Frank".to_string(),
            age: 45,
            email: Some("frank@example.com".to_string()),
        }),
        Err("Rate limit exceeded"),
    ];

    let name_path_result_str = name_path.clone().for_result::<&str>();

    // Process results with different error types
    for (i, result) in api_results.iter().enumerate() {
        match name_path_result_str.get(result) {
            Some(name) => println!("  User {}: {} (success)", i + 1, name),
            None => println!("  User {}: Failed to load (error)", i + 1),
        }
    }

    println!("✓ Example 3 completed\n");

    println!("=== All Examples Completed Successfully! ===");
}
