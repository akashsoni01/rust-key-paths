// use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

// #[derive(Debug, Clone, PartialEq)]
// struct Person {
//     name: String,
//     age: u32,
// }

// #[derive(Debug, Clone, PartialEq)]
// struct Address {
//     street: String,
//     city: String,
// }

// /*
// there is no fom and om i.e. failable owned mutable and owned mutable keypaths. 
// because once the value moved it is up to you you wan to mutate it or not. 
// e.g. 
//     let name_kp = KeyPaths::owned(|p: Person| p.name);
//     let mut extracted_name = name_kp.get_owned(person.clone());
// */
// fn main() {
//     println!("=== Owned KeyPaths Test Suite ===\n");

//     let person = Person {
//         name: "Akash".to_string(),
//         age: 30,
//     };

//     let address = Address {
//         street: "123 Main St".to_string(),
//         city: "New York".to_string(),
//     };

//     // Test 1: Basic owned keypath
//     println!("Test 1: Basic owned keypath");
//     let name_kp = KeyPaths::owned(|p: Person| p.name);
//     let extracted_name = name_kp.get_owned(person.clone());
//     assert_eq!(extracted_name, "Akash");
//     println!("  ✓ Name extraction: {}", extracted_name);

//     // Test 2: Failable owned keypath
//     println!("Test 2: Failable owned keypath");
//     let age_kp = KeyPaths::failable_owned(|p: Person| Some(p.age));
//     let extracted_age = age_kp.get_failable_owned(person.clone());
//     assert_eq!(extracted_age, Some(30));
//     println!("  ✓ Age extraction: {:?}", extracted_age);

//     // Test 3: Owned keypath composition
//     println!("Test 3: Owned keypath composition");
//     let name_kp = KeyPaths::owned(|p: Person| p.name);
//     let length_kp = KeyPaths::owned(|s: String| s.len());
//     let composed_kp = name_kp.then(length_kp);
//     let name_length = composed_kp.get_owned(person.clone());
//     assert_eq!(name_length, 5);
//     println!("  ✓ Name length via composition: {}", name_length);

//     // Test 4: Failable owned keypath composition
//     println!("Test 4: Failable owned keypath composition");
//     let person_kp = KeyPaths::owned(|p: Person| p);
//     let age_kp = KeyPaths::failable_owned(|p: Person| Some(p.age));
//     let composed_failable_kp = person_kp.then(age_kp);
//     let extracted_age_composed = composed_failable_kp.get_failable_owned(person.clone());
//     assert_eq!(extracted_age_composed, Some(30));
//     println!("  ✓ Age via failable composition: {:?}", extracted_age_composed);

//     // Test 5: Iterator support
//     println!("Test 5: Iterator support");
//     #[derive(Debug, Clone)]
//     struct PersonWithAddresses {
//         addresses: Vec<Address>,
//     }
    
//     let person_with_addresses = PersonWithAddresses {
//         addresses: vec![address.clone(), address.clone()],
//     };
    
//     let addresses_kp = KeyPaths::owned(|p: PersonWithAddresses| p.addresses);
//     if let Some(iter) = addresses_kp.into_iter(person_with_addresses.clone()) {
//         let count = iter.count();
//         assert_eq!(count, 2);
//         println!("  ✓ Iterator count: {}", count);
//     }

//     // Test 6: KeyPath kind names
//     println!("Test 6: KeyPath kind names");
//     let name_kp = KeyPaths::owned(|p: Person| p.name);
//     let failable_age_kp = KeyPaths::failable_owned(|p: Person| Some(p.age));
    
//     assert_eq!(name_kp.kind_name(), "Owned");
//     assert_eq!(failable_age_kp.kind_name(), "FailableOwned");
//     println!("  ✓ Name keypath kind: {}", name_kp.kind_name());
//     println!("  ✓ Failable age keypath kind: {}", failable_age_kp.kind_name());

//     println!("\n=== All Tests Passed! ===");
// }

fn main() {
    
}