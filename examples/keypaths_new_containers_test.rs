use keypaths_proc::Kp;
use std::rc::Weak;
use std::sync::{Mutex, RwLock};

#[derive(Debug, Kp)]
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
    println!("=== Keypaths Macro New Container Types Test ===");

    let container = ContainerTest {
        result: Ok("Success!".to_string()),
        result_int: Ok(42),
        mutex_data: Mutex::new("Mutex content".to_string()),
        rwlock_data: RwLock::new(100),
        weak_ref: Weak::new(),
        name: "Akash".to_string(),
        age: 30,
    };

    // Test Result<T, E> with Keypaths
    if let Some(value) = ContainerTest::result_fr().get(&container) {
        println!("✅ Result value: {}", value);
    }

    // Test Mutex<T> with Keypaths
    let mutex_ref = ContainerTest::mutex_data_r().get(&container);
    println!("✅ Mutex reference: {:?}", mutex_ref);

    // Test RwLock<T> with Keypaths
    let rwlock_ref = ContainerTest::rwlock_data_r().get(&container);
    println!("✅ RwLock reference: {:?}", rwlock_ref);

    // Test Weak<T> with Keypaths
    let weak_ref = ContainerTest::weak_ref_r().get(&container);
    println!("✅ Weak reference: {:?}", weak_ref);

    // Test basic types
    let name = ContainerTest::name_r().get(&container);
    println!("✅ Name: {}", name);

    let age = ContainerTest::age_r().get(&container);
    println!("✅ Age: {}", age);

    println!("\n=== Keypaths Macro - All new container types supported! ===");
}
