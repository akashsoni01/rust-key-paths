use keypaths_proc::Keypaths;
use std::sync::{Mutex, RwLock};
use std::rc::Weak;

#[derive(Debug, Keypaths)]
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
    if let Some(value) = ContainerTest::result_fr().get(&container) {
        println!("Result value: {}", value);
    } else {
        println!("Result is Err or None");
    }

    if let Some(value) = ContainerTest::result_int_fr().get(&container) {
        println!("Result int value: {}", value);
    } else {
        println!("Result int is Err or None");
    }

    // Test Mutex<T> - returns reference to the Mutex container
    if let Some(mutex_ref) = ContainerTest::mutex_data_r().get(&container) {
        println!("Mutex reference: {:?}", mutex_ref);
        // To access the inner data, you would need to lock it manually
        if let Ok(data) = mutex_ref.try_lock() {
            println!("Mutex data: {}", *data);
        } else {
            println!("Mutex is locked");
        }
    }

    // Test RwLock<T> - returns reference to the RwLock container
    if let Some(rwlock_ref) = ContainerTest::rwlock_data_r().get(&container) {
        println!("RwLock reference: {:?}", rwlock_ref);
        // To access the inner data, you would need to lock it manually
        if let Ok(data) = rwlock_ref.try_read() {
            println!("RwLock data: {}", *data);
        } else {
            println!("RwLock is locked");
        }
    }

    /*
    The code creates Weak::new(), which is an empty weak reference with no associated strong reference. Since there's no Rc or Arc backing it, .upgrade() returns None.
    To see a successful upgrade, create the Weak from an Rc or Arc. For example:
    let rc = Rc::new("Shared reference".to_string());
    let weak_ref = Rc::downgrade(&rc);  // Create Weak from Rc
    // Now weak_ref.upgrade() will return Some(Rc)
    The current example uses Weak::new() (empty), so the upgrade fails as expected. This demonstrates that Weak references can be empty and that .upgrade() may return None.
    The keypath correctly returns a reference to the Weak container; the upgrade failure is due to the Weak being empty, not a keypath issue.
    */
    // Test Weak<T> - returns reference to the Weak container
    if let Some(weak_ref) = ContainerTest::weak_ref_r().get(&container) {
        println!("Weak reference: {:?}", weak_ref);
        // To access the inner data, you would need to upgrade it manually
        if let Some(rc) = weak_ref.upgrade() {
            println!("Weak ref upgraded to: {}", *rc);
        } else {
            println!("Weak ref upgrade failed");
        }
    }

    // Test basic types for comparison
    if let Some(name) = ContainerTest::name_r().get(&container) {
        println!("Name: {}", name);
    }

    if let Some(age) = ContainerTest::age_r().get(&container) {
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
    if let Some(value) = ContainerTest::result_fr().get(&error_container) {
        println!("This should not print: {}", value);
    } else {
        println!("✓ Correctly returned None for Err result");
    }

    if let Some(value) = ContainerTest::result_int_fr().get(&error_container) {
        println!("This should not print: {}", value);
    } else {
        println!("✓ Correctly returned None for Err result_int");
    }

    // Mutex and RwLock should still work
    if let Some(mutex_ref) = ContainerTest::mutex_data_r().get(&error_container) {
        println!("Error container mutex reference: {:?}", mutex_ref);
        if let Ok(data) = mutex_ref.try_lock() {
            println!("Error container mutex data: {}", *data);
        }
    }

    if let Some(rwlock_ref) = ContainerTest::rwlock_data_r().get(&error_container) {
        println!("Error container rwlock reference: {:?}", rwlock_ref);
        if let Ok(data) = rwlock_ref.try_read() {
            println!("Error container rwlock data: {}", *data);
        }
    }

    println!("\n=== Keypaths Types ===");
    println!("result() returns: KeyPath<ContainerTest, String, impl for<\'r> Fn(&\'r ContainerTest) -> &\'r String> (failable readable)");
    println!("result_int() returns: KeyPath<ContainerTest, i32, impl for<\'r> Fn(&\'r ContainerTest) -> &\'r i32> (failable readable)");
    println!("mutex_data() returns: KeyPath<ContainerTest, Mutex<String, impl for<\'r> Fn(&\'r ContainerTest) -> &\'r Mutex<String>> (readable)");
    println!("rwlock_data() returns: KeyPath<ContainerTest, RwLock<i32, impl for<\'r> Fn(&\'r ContainerTest) -> &\'r RwLock<i32>> (readable)");
    println!("weak_ref() returns: KeyPath<ContainerTest, Weak<String, impl for<\'r> Fn(&\'r ContainerTest) -> &\'r Weak<String>> (readable)");
    println!("name() returns: KeyPath<ContainerTest, String, impl for<\'r> Fn(&\'r ContainerTest) -> &\'r String> (readable)");
    println!("age() returns: KeyPath<ContainerTest, u32, impl for<\'r> Fn(&\'r ContainerTest) -> &\'r u32> (readable)");

    println!("\n=== All new container tests completed successfully! ===");
}
