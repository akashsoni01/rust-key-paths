// Demonstrates reference support for KeyPaths
// This example shows how to work with slices and iterators using keypaths
// cargo run --example reference_support_example

use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Clone, Keypaths)]
#[All]
struct Person {
    name: String,
    age: u32,
    email: String,
    active: bool,
}

#[derive(Debug, Clone, Keypaths)]
struct Company {
    name: String,
    employees: Vec<Person>,
    founded_year: u32,
}

fn main() {
    println!("=== Reference Support Example ===\n");

    // Create sample data
    let people = vec![
        Person {
            name: "Alice Johnson".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
            active: true,
        },
        Person {
            name: "Bob Smith".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
            active: true,
        },
        Person {
            name: "Charlie Brown".to_string(),
            age: 35,
            email: "charlie@example.com".to_string(),
            active: false,
        },
        Person {
            name: "Diana Prince".to_string(),
            age: 28,
            email: "diana@example.com".to_string(),
            active: true,
        },
    ];

    println!("Sample people: {}", people.len());

    // Example 1: Extract names from slice using extract_from_slice
    println!("\n--- Example 1: Extract Names from Slice ---");
    let names: Vec<&String> = Person::name_r().extract_from_slice(&people);
    for name in &names {
        println!("  • {}", name);
    }

    // Example 2: Extract ages from slice
    println!("\n--- Example 2: Extract Ages from Slice ---");
    let ages: Vec<&u32> = Person::age_r().extract_from_slice(&people);
    for age in &ages {
        println!("  • Age: {}", age);
    }

    // Example 3: Extract emails from iterator
    println!("\n--- Example 3: Extract Emails from Iterator ---");
    let emails: Vec<&String> = Person::email_r().extract_from_iter(people.iter());
    for email in &emails {
        println!("  • {}", email);
    }

    // Example 4: Extract active status (boolean)
    println!("\n--- Example 4: Extract Active Status ---");
    let active_status: Vec<&bool> = Person::active_r().extract_from_slice(&people);
    for (i, status) in active_status.iter().enumerate() {
        println!("  • Person {}: {}", i + 1, if **status { "Active" } else { "Inactive" });
    }

    // Example 5: Filter active people and extract their names
    println!("\n--- Example 5: Names of Active People ---");
    let active_people: Vec<&Person> = people.iter().filter(|p| p.active).collect();
    println!("  Active people count: {}", active_people.len());
    for person in &active_people {
        println!("  • {} (age: {})", person.name, person.age);
    }

    // Example 6: Extract ages of people over 30
    println!("\n--- Example 6: Ages of People Over 30 ---");
    let older_people: Vec<&Person> = people.iter().filter(|p| p.age > 30).collect();
    println!("  People over 30: {}", older_people.len());
    for person in &older_people {
        println!("  • {} (age: {})", person.name, person.age);
    }

    // Example 7: Working with nested structures
    println!("\n--- Example 7: Nested Structure Example ---");
    let company = Company {
        name: "TechCorp".to_string(),
        employees: people.clone(),
        founded_year: 2020,
    };

    // Extract company name
    if let Some(company_name) = Company::name_r().get_ref(&&company) {
        println!("  Company: {}", company_name);
    }

    // Extract founded year
    if let Some(year) = Company::founded_year_r().get_ref(&&company) {
        println!("  Founded: {}", year);
    }

    // Example 8: Chain operations with references
    println!("\n--- Example 8: Chain Operations ---");
    let all_ages: Vec<&u32> = Person::age_r().extract_from_slice(&people);
    let total_age: u32 = all_ages.iter().map(|&&age| age).sum();
    let average_age = total_age as f64 / all_ages.len() as f64;
    println!("  Total age: {}", total_age);
    println!("  Average age: {:.1}", average_age);

    // Example 9: Working with failable keypaths
    println!("\n--- Example 9: Failable KeyPath with References ---");
    // Create a person with optional field
    #[derive(Debug, Clone, Keypaths)]
    struct PersonWithOptional {
        name: String,
        age: u32,
        nickname: Option<String>,
    }

    let people_with_nicknames = vec![
        PersonWithOptional {
            name: "Alice".to_string(),
            age: 30,
            nickname: Some("Ally".to_string()),
        },
        PersonWithOptional {
            name: "Bob".to_string(),
            age: 25,
            nickname: None,
        },
        PersonWithOptional {
            name: "Charlie".to_string(),
            age: 35,
            nickname: Some("Chuck".to_string()),
        },
    ];

    // Extract nicknames (only those that exist)
    let nicknames: Vec<&String> = PersonWithOptional::nickname_fr().extract_from_slice(&people_with_nicknames);
    println!("  Nicknames found: {}", nicknames.len());
    for nickname in &nicknames {
        println!("    • {}", nickname);
    }

    // Example 10: Mutable reference support
    println!("\n--- Example 10: Mutable Reference Support ---");
    let mut mutable_people = vec![
        Person {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
            active: true,
        },
        Person {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
            active: true,
        },
    ];

    println!("  Before modification:");
    for person in &mutable_people {
        println!("    • {} (age: {})", person.name, person.age);
    }

    // Extract mutable references to names and modify them
    let mut names: Vec<&mut String> = Person::name_w().extract_mut_from_slice(&mut mutable_people);
    for name in &mut names {
        name.push_str(" (Updated)");
    }

    println!("  After modification:");
    for person in &mutable_people {
        println!("    • {} (age: {})", person.name, person.age);
    }

    // Example 11: Performance comparison
    println!("\n--- Example 11: Performance Note ---");
    println!("  Using extract_from_slice(), extract_from_iter(), and extract_mut_from_slice() is efficient because:");
    println!("  • No cloning of the original data");
    println!("  • Direct reference access to fields");
    println!("  • Works with any slice or iterator");
    println!("  • Perfect for read-only and mutable operations on collections");
    println!("  • Zero-cost abstraction for field access");

    println!("\n✅ Reference support example completed!");
}
