use keypaths_proc::{Keypaths, ReadableKeypaths};
use rust_keypaths::{KeyPath, OptionalKeyPath};
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
    
    println!("\n=== CurriedMutexKeyPath Examples ===");
    
    // Example 1: CurriedMutexKeyPath with manual KeyPath
    // Create a manual keypath that implements Clone
    let data_keypath = KeyPath::new(|s: &SomeStruct| &s.data);
    
    // Curry the keypath to work with Mutex<SomeStruct>
    let curried = data_keypath.curry_mutex();
    
    // Apply to the mutex with a callback
    curried.apply(&container.mutex_data, |data| {
        println!("✅ CurriedMutexKeyPath - Data: {}", data);
    });
    
    // Example 2: Using uncurry_mutex for direct access
    // This is a convenience method that combines curry and apply
    // Create a new keypath since curry_mutex consumes self
    let data_keypath2 = KeyPath::new(|s: &SomeStruct| &s.data);
    data_keypath2.uncurry_mutex(&container.mutex_data, |data| {
        println!("✅ UncurryMutex - Data: {}", data);
    });
    
    println!("\n=== CurriedMutexOptionalKeyPath Examples ===");
    
    // Example 3: CurriedMutexOptionalKeyPath with optional keypath
    // Create an optional keypath (simulating a field that might not exist)
    // let optional_data_keypath = crate::SomeStruct::data_fr();
    //
    // Curry the optional keypath to work with Mutex<SomeStruct>
    // let curried_optional = optional_data_keypath.curry_mutex();
    
    // Apply to the mutex - callback only runs if value exists
    // curried_optional.apply(&container.mutex_data, |data| {
    //     println!("✅ CurriedMutexOptionalKeyPath - Data: {}", data);
    // });
    
    // Example 4: Using uncurry_mutex with optional keypath
    // Create a new optional keypath since curry_mutex consumes self
    let optional_data_keypath2 = OptionalKeyPath::new(|s: &SomeStruct| Some(&s.data));
    optional_data_keypath2.uncurry_mutex(&container.mutex_data, |data| {
        println!("✅ UncurryMutex (Optional) - Data: {}", data);
    });
    
    // Example 5: Chaining CurriedMutexOptionalKeyPath
    // Create nested structure for chaining demonstration
    #[derive(Debug)]
    struct NestedStruct {
        inner: Option<SomeStruct>,
    }
    
    let nested_mutex = Arc::new(Mutex::new(NestedStruct {
        inner: Some(SomeStruct { data: "Nested data".to_string() }),
    }));
    
    // Create keypaths for chaining
    let inner_keypath = OptionalKeyPath::new(|n: &NestedStruct| n.inner.as_ref());
    let data_keypath3 = crate::SomeStruct::data_fr();
    
    // Chain: NestedStruct -> Option<SomeStruct> -> String
    // First curry the inner keypath, then chain with data keypath
    let chained = inner_keypath.curry_mutex().then(data_keypath3);
    
    // Apply the chained keypath
    chained.apply(&nested_mutex, |data| {
        println!("✅ Chained CurriedMutexOptionalKeyPath - Data: {}", data);
    });
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
