use key_paths_derive::WritableKeypaths;
use std::sync::{Mutex, RwLock};
use std::rc::Weak;

#[derive(Debug, WritableKeypaths)]
struct ContainerTest {
    // Error handling containers
    result: Result<String, String>,
    result_int: Result<i32, String>,
    
    // Synchronization primitives
    mutex_data: Mutex<String>,
    rwlock_data: RwLock<i32>,
    
    // Reference counting with weak references
    weak_ref: Weak<String>,
    
    // Basic types for comparison
    name: String,
    age: u32,
}

fn main() {
    println!("=== WritableKeypaths Macro New Container Types Test ===");
    
    let mut container = ContainerTest {
        result: Ok("Success!".to_string()),
        result_int: Ok(42),
        mutex_data: Mutex::new("Mutex content".to_string()),
        rwlock_data: RwLock::new(100),
        weak_ref: Weak::new(),
        name: "Alice".to_string(),
        age: 30,
    };

    // Test Result<T, E> with WritableKeypaths
    if let Some(result_ref) = ContainerTest::result_w().get(&mut container) {
        println!("✅ Result reference: {:?}", result_ref);
    }
    
    // Test Mutex<T> with WritableKeypaths
    if let Some(mutex_ref) = ContainerTest::mutex_data_w().get(&mut container) {
        println!("✅ Mutex reference: {:?}", mutex_ref);
    }
    
    // Test RwLock<T> with WritableKeypaths
    if let Some(rwlock_ref) = ContainerTest::rwlock_data_w().get(&mut container) {
        println!("✅ RwLock reference: {:?}", rwlock_ref);
    }
    
    // Note: Weak<T> doesn't have writable methods (it's immutable)

    // Test basic types
    if let Some(name_ref) = ContainerTest::name_w().get(&mut container) {
        println!("✅ Name reference: {:?}", name_ref);
    }

    if let Some(age_ref) = ContainerTest::age_w().get(&mut container) {
        println!("✅ Age reference: {:?}", age_ref);
    }

    println!("\n=== WritableKeypaths Macro - All new container types supported! ===");
    println!("Note: Weak<T> doesn't support writable access (it's immutable)");
}
