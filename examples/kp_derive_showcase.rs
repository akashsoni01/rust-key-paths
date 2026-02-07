// Example demonstrating the extended Kp derive macro with all wrapper types
use key_paths_derive::Kp;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Kp, Debug, Clone)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Kp, Debug)]
struct Person {
    name: String,
    age: i32,
    email: Option<String>,
    address: Box<Address>,
}

#[derive(Kp, Debug)]
struct Company {
    name: String,
    employees: Vec<Person>,
    shared_resources: Arc<String>,
    metadata: HashMap<String, String>,
}

fn main() {
    // Create sample data
    let company = Company {
        name: "Tech Corp".to_string(),
        employees: vec![
            Person {
                name: "Alice".to_string(),
                age: 30,
                email: Some("alice@example.com".to_string()),
                address: Box::new(Address {
                    street: "123 Main St".to_string(),
                    city: "New York".to_string(),
                    zip: "10001".to_string(),
                }),
            },
            Person {
                name: "Bob".to_string(),
                age: 25,
                email: None,
                address: Box::new(Address {
                    street: "456 Oak Ave".to_string(),
                    city: "San Francisco".to_string(),
                    zip: "94102".to_string(),
                }),
            },
        ],
        shared_resources: Arc::new("Shared Company Data".to_string()),
        metadata: {
            let mut map = HashMap::new();
            map.insert("founded".to_string(), "2020".to_string());
            map.insert("industry".to_string(), "Technology".to_string());
            map
        },
    };

    println!("=== Kp Derive Macro Examples ===\n");

    // Example 1: Basic field access
    println!("1. Basic Field Access:");
    let company_name_kp = Company::name();
    if let Some(name) = company_name_kp.get(&company) {
        println!("   Company name: {}", name);
    }

    // Example 2: Vec access (gets first element)
    println!("\n2. Vec Access (first element):");
    let employees_kp = Company::employees();
    if let Some(first_employee) = employees_kp.get(&company) {
        println!("   First employee: {}", first_employee.name);
    }

    // Example 3: Option unwrapping
    println!("\n3. Option Unwrapping:");
    let employees_kp = Company::employees();

    if let Some(first_employee) = employees_kp.get(&company) {
        let email_kp = Person::email();
        if let Some(email) = email_kp.get(first_employee) {
            println!("   First employee email: {}", email);
        } else {
            println!("   First employee has no email");
        }
    }

    // Example 4: Box dereferencing
    println!("\n4. Box Dereferencing:");
    let employees_kp = Company::employees();

    if let Some(first_employee) = employees_kp.get(&company) {
        let address_kp = Person::address();
        if let Some(address) = address_kp.get(first_employee) {
            println!(
                "   First employee's address: {} {}, {}",
                address.street, address.city, address.zip
            );
        }
    }

    // Example 5: Arc access
    println!("\n5. Arc Access:");
    let resources_kp = Company::shared_resources();
    if let Some(resources) = resources_kp.get(&company) {
        println!("   Shared resources: {}", resources);
    }

    // Example 6: HashMap access
    println!("\n6. HashMap Access:");
    let metadata_kp = Company::metadata();
    if let Some(metadata) = metadata_kp.get(&company) {
        println!("   Metadata: {:?}", metadata);
    }

    // Example 7: Mutable access
    println!("\n7. Mutable Access:");
    let mut mutable_person = Person {
        name: "Charlie".to_string(),
        age: 35,
        email: Some("charlie@example.com".to_string()),
        address: Box::new(Address {
            street: "789 Pine Rd".to_string(),
            city: "Seattle".to_string(),
            zip: "98101".to_string(),
        }),
    };

    println!("   Before: age = {}", mutable_person.age);

    let age_kp = Person::age();
    age_kp.get_mut(&mut mutable_person).map(|age| *age = 36);

    println!("   After:  age = {}", mutable_person.age);

    // Example 8: Modifying Option inner value
    println!("\n8. Modifying Option Inner Value:");
    println!("   Before: email = {:?}", mutable_person.email);

    let email_kp = Person::email();
    email_kp
        .get_mut(&mut mutable_person)
        .map(|email| *email = "newemail@example.com".to_string());

    println!("   After:  email = {:?}", mutable_person.email);

    // Example 9: Modifying Box inner value
    println!("\n9. Modifying Box Inner Value:");

    println!("   Before: city = {}", mutable_person.address.city);

    let address_kp = Person::address();
    if let Some(address) = address_kp.get_mut(&mut mutable_person) {
        address.city = "Portland".to_string();
    }

    println!("   After:  city = {}", mutable_person.address.city);

    println!("\n=== All Examples Complete ===");
}
