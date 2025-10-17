use key_paths_derive::Keypath;
use std::sync::{Mutex, RwLock};
use std::rc::Weak;

#[derive(Debug, Keypath)]
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
    println!("=== New Container Types Test ===");
    
    let container = ContainerTest {
        result: Ok("Success!".to_string()),
        result_int: Ok(42),
        mutex_data: Mutex::new("Mutex content".to_string()),
        rwlock_data: RwLock::new(100),
        weak_ref: Weak::new(), // Empty weak reference
        name: "Alice".to_string(),
        age: 30,
    };

    // Test Result<T, E> - returns Ok value if available
    if let Some(value) = ContainerTest::result().get(&container) {
        println!("Result value: {}", value);
    } else {
        println!("Result is Err or None");
    }

    if let Some(value) = ContainerTest::result_int().get(&container) {
        println!("Result int value: {}", value);
    } else {
        println!("Result int is Err or None");
    }

    // Test Mutex<T> - returns reference to the Mutex container
    if let Some(mutex_ref) = ContainerTest::mutex_data().get(&container) {
        println!("Mutex reference: {:?}", mutex_ref);
        // To access the inner data, you would need to lock it manually
        if let Ok(data) = mutex_ref.try_lock() {
            println!("Mutex data: {}", *data);
        } else {
            println!("Mutex is locked");
        }
    }

    // Test RwLock<T> - returns reference to the RwLock container
    if let Some(rwlock_ref) = ContainerTest::rwlock_data().get(&container) {
        println!("RwLock reference: {:?}", rwlock_ref);
        // To access the inner data, you would need to lock it manually
        if let Ok(data) = rwlock_ref.try_read() {
            println!("RwLock data: {}", *data);
        } else {
            println!("RwLock is locked");
        }
    }

    // Test Weak<T> - returns reference to the Weak container
    if let Some(weak_ref) = ContainerTest::weak_ref().get(&container) {
        println!("Weak reference: {:?}", weak_ref);
        // To access the inner data, you would need to upgrade it manually
        if let Some(rc) = weak_ref.upgrade() {
            println!("Weak ref upgraded to: {}", *rc);
        } else {
            println!("Weak ref upgrade failed");
        }
    }

    // Test basic types for comparison
    if let Some(name) = ContainerTest::name().get(&container) {
        println!("Name: {}", name);
    }

    if let Some(age) = ContainerTest::age().get(&container) {
        println!("Age: {}", age);
    }

    // Test with error cases
    println!("\n=== Error Cases ===");
    
    let error_container = ContainerTest {
        result: Err("Something went wrong".to_string()),
        result_int: Err("Invalid number".to_string()),
        mutex_data: Mutex::new("Error content".to_string()),
        rwlock_data: RwLock::new(200),
        weak_ref: Weak::new(),
        name: "Bob".to_string(),
        age: 25,
    };

    // Result with error should return None
    if let Some(value) = ContainerTest::result().get(&error_container) {
        println!("This should not print: {}", value);
    } else {
        println!("✓ Correctly returned None for Err result");
    }

    if let Some(value) = ContainerTest::result_int().get(&error_container) {
        println!("This should not print: {}", value);
    } else {
        println!("✓ Correctly returned None for Err result_int");
    }

    // Mutex and RwLock should still work
    if let Some(mutex_ref) = ContainerTest::mutex_data().get(&error_container) {
        println!("Error container mutex reference: {:?}", mutex_ref);
        if let Ok(data) = mutex_ref.try_lock() {
            println!("Error container mutex data: {}", *data);
        }
    }

    if let Some(rwlock_ref) = ContainerTest::rwlock_data().get(&error_container) {
        println!("Error container rwlock reference: {:?}", rwlock_ref);
        if let Ok(data) = rwlock_ref.try_read() {
            println!("Error container rwlock data: {}", *data);
        }
    }

    println!("\n=== Keypath Types ===");
    println!("result() returns: KeyPaths<ContainerTest, String> (failable readable)");
    println!("result_int() returns: KeyPaths<ContainerTest, i32> (failable readable)");
    println!("mutex_data() returns: KeyPaths<ContainerTest, Mutex<String>> (readable)");
    println!("rwlock_data() returns: KeyPaths<ContainerTest, RwLock<i32>> (readable)");
    println!("weak_ref() returns: KeyPaths<ContainerTest, Weak<String>> (readable)");
    println!("name() returns: KeyPaths<ContainerTest, String> (readable)");
    println!("age() returns: KeyPaths<ContainerTest, u32> (readable)");

    println!("\n=== All new container tests completed successfully! ===");
}
