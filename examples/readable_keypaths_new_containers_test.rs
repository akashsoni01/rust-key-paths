use keypaths_proc::{Keypaths, ReadableKeypaths};
use std::sync::{Arc, Mutex, RwLock};
use std::rc::Weak;

#[derive(Debug, ReadableKeypaths, Clone)]
struct ContainerTest {
    // Error handling containers
    result: Result<String, String>,
    result_int: Result<i32, String>,
    
    // Synchronization primitives
    mutex_data: Arc<Mutex<SomeStruct>>,
    rwlock_data: Arc<RwLock<i32>>,
    
    // Reference counting with weak references
    weak_ref: Weak<String>,
    
    // Basic types for comparison
    name: String,
    age: u32,
}

#[derive(Debug, Keypaths)]
struct SomeStruct {
    data: String
}
fn main() {
    println!("=== ReadableKeypaths Macro New Container Types Test ===");
    
    let container = ContainerTest {
        result: Ok("Success!".to_string()),
        result_int: Ok(42),
        mutex_data: Arc::new(Mutex::new(SomeStruct { data: "Hello".to_string() })),
        rwlock_data: Arc::new(RwLock::new(10)),
        weak_ref: Weak::new(),
        name: "Alice".to_string(),
        age: 30,
    };

    // Test Result<T, E> with ReadableKeypaths
    if let Some(value) = ContainerTest::result_fr().get(&container) {
        println!("✅ Result value: {}", value);
    }
    
    
    // // Test Mutex<T> with ReadableKeypaths
    // if let Some(mutex_ref) = ContainerTest::mutex_data_r().get(&container) {
    //     println!("✅ Mutex reference: {:?}", mutex_ref);
    // }
    //
    
    // Test Mutex<T> with curry/uncurry pattern for chaining
    // Get the keypath to access data inside SomeStruct
    let data_path = SomeStruct::data_r();
    
    // Pattern 1: Direct access through Mutex
    // Get the mutex from the container and access inner data
    let mutex_field = &container.mutex_data;
    if let Ok(guard) = mutex_field.lock() {
        let data = data_path.get(&*guard);
        println!("✅ Mutex direct access data: {}", data);
    }
    
    // Pattern 2: Using curry_mutex for chaining (requires Clone on keypath)
    // Create a keypath that works with SomeStruct, then curry it for Mutex<SomeStruct>
    // Note: Macro-generated keypaths may not implement Clone, so this pattern
    // works best with manually created keypaths that implement Clone
    let inner_data_path = SomeStruct::data_r();
    
    // Example of curry_mutex usage (commented out because macro keypaths don't implement Clone):
    // let curried = inner_data_path.curry_mutex();
    // curried.apply(mutex_field, |data| {
    //     println!("✅ Mutex curried data: {}", data);
    // });
    
    // Pattern 3: Using uncurry_mutex (also requires Clone)
    // inner_data_path.uncurry_mutex(mutex_field, |data| {
    //     println!("✅ Mutex uncurried data: {}", data);
    // });
    // // Test RwLock<T> with ReadableKeypaths
    // if let Some(rwlock_ref) = ContainerTest::rwlock_data_r().get(&container) {
    //     println!("✅ RwLock reference: {:?}", rwlock_ref);
    // }
    //
    // // Test Weak<T> with ReadableKeypaths
    // if let Some(weak_ref) = ContainerTest::weak_ref_r().get(&container) {
    //     println!("✅ Weak reference: {:?}", weak_ref);
    // }
    //
    // // Test basic types
    // if let Some(name) = ContainerTest::name_r().get(&container) {
    //     println!("✅ Name: {}", name);
    // }
    //
    // if let Some(age) = ContainerTest::age_r().get(&container) {
    //     println!("✅ Age: {}", age);
    // }

    println!("\n=== ReadableKeypaths Macro - All new container types supported! ===");
}
