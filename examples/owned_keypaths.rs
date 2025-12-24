// use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

// #[derive(Debug, Clone)]
// struct Person {
//     name: String,
//     age: u32,
//     address: Address,
// }

// #[derive(Debug, Clone)]
// struct Address {
//     street: String,
//     city: String,
//     zip: String,
// }

// fn main() {
//     println!("=== Owned KeyPaths Examples ===\n");

//     // Create a sample person
//     let person = Person {
//         name: "Alice".to_string(),
//         age: 30,
//         address: Address {
//             street: "123 Main St".to_string(),
//             city: "New York".to_string(),
//             zip: "10001".to_string(),
//         },
//     };

//     // ===== Basic Owned KeyPath Usage =====
//     println!("1. Basic Owned KeyPath Usage:");
    
//     // Create owned keypaths
//     let name_kp = KeyPaths::owned(|p: Person| p.name);
//     let age_kp = KeyPaths::owned(|p: Person| p.age);
//     let address_kp = KeyPaths::owned(|p: Person| p.address);
    
//     // Use owned keypaths
//     let extracted_name = name_kp.get_owned(person.clone());
//     let extracted_age = age_kp.get_owned(person.clone());
//     let extracted_address = address_kp.get_owned(person.clone());
    
//     println!("  Extracted name: {}", extracted_name);
//     println!("  Extracted age: {}", extracted_age);
//     println!("  Extracted address: {:?}", extracted_address);
//     println!();

//     // ===== Failable Owned KeyPath Usage =====
//     println!("2. Failable Owned KeyPath Usage:");
    
//     // Create failable owned keypaths
//     let street_kp = KeyPaths::failable_owned(|p: Person| {
//         Some(p.address.street)
//     });
    
//     let city_kp = KeyPaths::failable_owned(|p: Person| {
//         Some(p.address.city)
//     });
    
//     // Use failable owned keypaths
//     let extracted_street = street_kp.get_failable_owned(person.clone());
//     let extracted_city = city_kp.get_failable_owned(person.clone());
    
//     println!("  Extracted street: {:?}", extracted_street);
//     println!("  Extracted city: {:?}", extracted_city);
//     println!();

//     // ===== Owned KeyPath Composition =====
//     println!("3. Owned KeyPath Composition:");
    
//     // Compose owned keypaths
//     let name_from_person = KeyPaths::owned(|p: Person| p.name);
//     let first_char_kp = KeyPaths::owned(|s: String| s.chars().next().unwrap_or('?'));
    
//     let composed_kp = name_from_person.then(first_char_kp);
//     let first_char = composed_kp.get_owned(person.clone());
    
//     println!("  First character of name: {}", first_char);
//     println!();

//     // Compose failable owned keypaths
//     let address_from_person = KeyPaths::owned(|p: Person| p.address);
//     let street_from_address = KeyPaths::failable_owned(|a: Address| Some(a.street));
    
//     let composed_failable_kp = address_from_person.then(street_from_address);
//     let extracted_street_composed = composed_failable_kp.get_failable_owned(person.clone());
    
//     println!("  Street via composition: {:?}", extracted_street_composed);
//     println!();

//     // ===== Iterator Support =====
//     println!("4. Iterator Support:");
    
//     // Create a person with a vector of addresses
//     #[derive(Debug, Clone)]
//     struct PersonWithAddresses {
//         name: String,
//         addresses: Vec<Address>,
//     }
    
//     let person_with_addresses = PersonWithAddresses {
//         name: "Bob".to_string(),
//         addresses: vec![
//             Address {
//                 street: "456 Oak Ave".to_string(),
//                 city: "Boston".to_string(),
//                 zip: "02101".to_string(),
//             },
//             Address {
//                 street: "789 Pine St".to_string(),
//                 city: "Seattle".to_string(),
//                 zip: "98101".to_string(),
//             },
//         ],
//     };
    
//     // Create owned keypath for addresses
//     let addresses_kp = KeyPaths::owned(|p: PersonWithAddresses| p.addresses);
    
//     // Iterate over addresses
//     if let Some(iter) = addresses_kp.into_iter(person_with_addresses.clone()) {
//         println!("  Addresses:");
//         for (i, address) in iter.enumerate() {
//             println!("    {}: {:?}", i + 1, address);
//         }
//     }
//     println!();

//     // ===== Failable Iterator Support =====
//     println!("5. Failable Iterator Support:");
    
//     // Create failable owned keypath for addresses
//     let failable_addresses_kp = KeyPaths::failable_owned(|p: PersonWithAddresses| {
//         Some(p.addresses)
//     });
    
//     // Iterate over addresses with failable access
//     if let Some(iter) = failable_addresses_kp.into_iter(person_with_addresses.clone()) {
//         println!("  Failable addresses:");
//         for (i, address) in iter.enumerate() {
//             println!("    {}: {:?}", i + 1, address);
//         }
//     }
//     println!();

//     // ===== KeyPath Kind Information =====
//     println!("6. KeyPath Kind Information:");
    
//     let name_kp = KeyPaths::owned(|p: Person| p.name);
//     let failable_age_kp = KeyPaths::failable_owned(|p: Person| Some(p.age));
    
//     println!("  Name keypath kind: {}", name_kp.kind_name());
//     println!("  Failable age keypath kind: {}", failable_age_kp.kind_name());
//     println!();

//     println!("=== All Examples Completed Successfully! ===");
// }

fn main() {
    
}