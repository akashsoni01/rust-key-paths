use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

// Example struct to demonstrate FailableCombined keypath
#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u32,
    address: Option<String>,
}

impl Person {
    // Constructor
    fn new(name: String, age: u32, address: Option<String>) -> Self {
        Self { name, age, address }
    }
}

fn main() {
    println!("🔗 FailableCombined KeyPath Example");
    println!("=====================================");
    
    // Create a person with an address
    let mut person = Person::new("Alice".to_string(), 30, Some("123 Main St".to_string()));
    
    // Create a FailableCombined keypath for the address field
    // This keypath can handle all three access patterns: readable, writable, and owned
    let address_keypath = KeyPaths::<Person, String>::failable_combined(
        // Readable closure - returns Option<&String>
        |person: &Person| person.address.as_ref(),
        // Writable closure - returns Option<&mut String>  

        |person: &mut Person| person.address.as_mut(),
        // Owned closure - returns Option<String> (takes ownership of Person, moves only the address)
        |person: Person| person.address,
    );
    
    println!("\n📖 Testing Readable Access:");
    // Test readable access
    if let Some(address) = address_keypath.get(&person) {
        println!("✅ Address (readable): {}", address);
    } else {
        println!("❌ No address found");
    }
    
    println!("\n✏️  Testing Writable Access:");
    // Test writable access
    let address = address_keypath.get_mut(&mut person);
    {
        *address = "456 Oak Ave".to_string();
        println!("✅ Address updated to: {}", address);
    } else {
        println!("❌ Could not get mutable reference to address");
    }
    
    println!("\n📦 Testing Owned Access:");
    // Test owned access - this moves both the keypath and the root
    // We need to clone the keypath since get_failable_owned consumes it
    if let Some(owned_address) = address_keypath.clone().get_failable_owned(person.clone()) {
        println!("✅ Got owned address: {}", owned_address);
        // The person is still available since we cloned it
        println!("✅ Person is still available: {:?}", person);
    } else {
        println!("❌ Could not get owned address");
    }
    
    println!("\n🧪 Testing with Person without Address:");
    // Test with a person without an address
    let person_no_address = Person::new("Bob".to_string(), 25, None);
    
    println!("📖 Readable access (no address):");
    if let Some(_address) = address_keypath.get(&person_no_address) {
        println!("✅ Address found: {}", _address);
    } else {
        println!("❌ No address found (expected)");
    }
    
    println!("\n📦 Owned access (no address):");
    if let Some(_owned_address) = address_keypath.get_failable_owned(person_no_address) {
        println!("✅ Got owned address: {}", _owned_address);
    } else {
        println!("❌ No address found (expected)");
    }
    
    println!("\n✨ Key Benefits of FailableCombined:");
    println!("1. 🔍 Readable: Get immutable references when available");
    println!("2. ✏️  Writable: Get mutable references when available"); 
    println!("3. 📦 Owned: Get owned values without moving the root");
    println!("4. 🛡️  Failable: All operations return Option<T> for safe handling");
    println!("5. 🎯 Combined: One keypath handles all three access patterns");
}
