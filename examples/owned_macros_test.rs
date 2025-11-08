use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Keypaths, Debug, Clone)]
#[All]
struct Person {
    name: String,
    age: u32,
    address: Option<Address>,
    tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct Address {
    street: String,
    city: String,
}

fn main() {
    println!("=== Owned KeyPaths with Macros Test ===");
    
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        address: Some(Address {
            street: "123 Main St".to_string(),
            city: "New York".to_string(),
        }),
        tags: vec!["developer".to_string(), "rust".to_string()],
    };
    
    // Test owned keypath methods
    println!("1. Basic owned keypath usage:");
    let name_kp = Person::name_o();
    let extracted_name = name_kp.get_owned(person.clone());
    println!("  Extracted name: {}", extracted_name);
    
    println!("\n2. Failable owned keypath usage:");
    let address_kp = Person::address_fo();
    let extracted_address = address_kp.get_failable_owned(person.clone());
    println!("  Extracted address: {:?}", extracted_address);
    
    println!("\n3. Vec owned keypath usage:");
    let tags_kp = Person::tags_o();
    let extracted_tags = tags_kp.get_owned(person.clone());
    println!("  Extracted tags: {:?}", extracted_tags);
    
    println!("\n4. Vec failable owned keypath usage:");
    let first_tag_kp = Person::tags_fo();
    let first_tag = first_tag_kp.get_failable_owned(person.clone());
    println!("  First tag: {:?}", first_tag);
    
    println!("\n5. Owned keypath composition:");
    // Create a keypath that gets the length of a string
    let string_length_kp = KeyPaths::owned(|s: String| s.len());
    let name_length_kp = Person::name_o().then(string_length_kp);
    let name_length = name_length_kp.get_owned(person.clone());
    println!("  Name length via composition: {}", name_length);
    
    println!("\n6. KeyPath kind information:");
    println!("  Name keypath kind: {}", Person::name_o().kind_name());
    println!("  Address failable keypath kind: {}", Person::address_fo().kind_name());
    println!("  Tags keypath kind: {}", Person::tags_o().kind_name());
    
    println!("\n=== All Tests Completed Successfully! ===");
}
