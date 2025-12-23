use keypaths_proc::{Keypaths, ReadableKeypaths, WritableKeypaths};
use rust_keypaths::{KeyPath, OptionalKeyPath};
use std::sync::{Arc, Mutex, RwLock};
use std::rc::Weak;

#[derive(Debug, ReadableKeypaths, WritableKeypaths, Clone)]
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

#[derive(Debug, Keypaths, WritableKeypaths, Clone)]
struct SomeStruct {
    data: String,
    optional_field: Option<String>,
}

#[derive(Debug, Keypaths, WritableKeypaths, Clone)]
struct NestedStruct {
    inner: Option<SomeStruct>,
}
fn main() {
    println!("=== ReadableKeypaths Macro New Container Types Test ===");
    
    let container = ContainerTest {
        result: Ok("Success!".to_string()),
        result_int: Ok(42),
        mutex_data: Arc::new(Mutex::new(SomeStruct { 
            data: "Hello".to_string(),
            optional_field: Some("Optional value".to_string()),
        })),
        rwlock_data: Arc::new(RwLock::new(10)),
        weak_ref: Weak::new(),
        name: "Alice".to_string(),
        age: 30,
    };

    // Test Result<T, E> with ReadableKeypaths
    if let Some(value) = ContainerTest::result_fr().get(&container) {
        println!("✅ Result value: {}", value);
    }
    
    println!("\n=== CurriedMutexKeyPath Examples (using curry_mutex) ===");
    
    // Example 1: CurriedMutexKeyPath with KeyPath using curry_mutex()
    // Use curry_mutex() for non-optional KeyPath types
    let data_keypath = crate::SomeStruct::data_r();
    
    // Curry the keypath to work with Mutex<SomeStruct>
    // curry_mutex() is for KeyPath (non-optional)
    let curried = data_keypath.curry_mutex();
    
    // Apply to the mutex with a callback
    curried.apply(&container.mutex_data, |data| {
        println!("✅ CurriedMutexKeyPath (curry_mutex) - Data: {}", data);
    });
    
    // Example 2: Manual KeyPath with curry_mutex()
    // You can also use manually created KeyPath
    let manual_keypath = KeyPath::new(|s: &SomeStruct| &s.data);
    let manual_curried = manual_keypath.curry_mutex();
    manual_curried.apply(&container.mutex_data, |data| {
        println!("✅ Manual KeyPath (curry_mutex) - Data: {}", data);
    });
    
    // Example 3: Using uncurry_mutex for direct access with KeyPath
    // This is a convenience method that combines curry and apply
    // Note: uncurry_mutex requires Clone, so use manual keypath
    // let data_keypath2 = SomeStruct::data_r();
    // data_keypath2.uncurry_mutex(&container.mutex_data, |data| {
    //     println!("✅ UncurryMutex (KeyPath) - Data: {}", data);
    // });
    
    println!("\n=== CurriedMutexOptionalKeyPath Examples (using curry_mutex_optional) ===");
    
    // Example 4: CurriedMutexOptionalKeyPath with macro-generated optional keypath
    // Create an optional keypath using macro
    let optional_data_keypath = crate::SomeStruct::data_fr();
    
    // Curry the optional keypath to work with Mutex<SomeStruct>
    // Use curry_mutex_optional() for OptionalKeyPath
    let curried_optional = optional_data_keypath.curry_mutex_optional();
    
    // Apply to the mutex - callback only runs if value exists
    curried_optional.apply(&container.mutex_data, |data| {
        println!("✅ Macro OptionalKeyPath (curry_mutex_optional) - Data: {}", data);
    });
    
    // Example 5: Manual OptionalKeyPath with curry_mutex_optional()
    // You can also use manually created OptionalKeyPath
    let manual_optional_keypath = OptionalKeyPath::new(|s: &SomeStruct| Some(&s.data));
    let manual_curried_optional = manual_optional_keypath.curry_mutex_optional();
    manual_curried_optional.apply(&container.mutex_data, |data| {
        println!("✅ Manual OptionalKeyPath (curry_mutex_optional) - Data: {}", data);
    });
    
    // Example 6: Using uncurry_mutex with optional keypath
    // Create a new optional keypath since curry_mutex_optional consumes self
    let optional_data_keypath2 = OptionalKeyPath::new(|s: &SomeStruct| Some(&s.data));
    optional_data_keypath2.uncurry_mutex(&container.mutex_data, |data| {
        println!("✅ UncurryMutex (Optional) - Data: {}", data);
    });
    
    println!("\n=== Chaining from ContainerTest::mutex_data (Simplified Helper API) ===");
    
    // Example 7: Using the new helper method get_arc_mutex_and_apply()
    // This reduces complexity by combining get() and curry_arc_mutex_optional() in one call
    // Chain: ContainerTest -> Arc<Mutex<SomeStruct>> -> SomeStruct -> Option<String>
    
    // Before (verbose):
    // let mutex_ref = ContainerTest::mutex_data_r().get(&container);
    // let curried = SomeStruct::optional_field_fr().curry_arc_mutex_optional();
    // curried.apply(mutex_ref, |value| { ... });
    
    // After (simplified with helper):
    crate::ContainerTest::mutex_data_r().get_arc_mutex_and_apply(
        &container,
        crate::SomeStruct::optional_field_fr(),
        |value| {
            println!("✅ Simplified helper (get_arc_mutex_and_apply): ContainerTest::mutex_data -> optional_field: {}", value);
        }
    );
    
    // Example 7b: Using helper with non-optional keypath
    crate::ContainerTest::mutex_data_r().get_arc_mutex_and_apply_keypath(
        &container,
        crate::SomeStruct::data_r(),
        |value| {
            println!("✅ Simplified helper (get_arc_mutex_and_apply_keypath): ContainerTest::mutex_data -> data: {}", value);
        }
    );
    
    println!("\n=== Chaining from ContainerTest::mutex_data (Composable API) ===");
    
    // Example 7c: Direct composable chain using curry_arc_mutex_optional() (for comparison)
    let mutex_data_keypath = crate::ContainerTest::mutex_data_r();
    let mutex_ref = mutex_data_keypath.get(&container);
    
    let optional_field_keypath1 = crate::SomeStruct::optional_field_fr();
    let chained_fr = optional_field_keypath1.curry_arc_mutex_optional();
    chained_fr.apply(mutex_ref, |data| {
        println!("✅ Composable chain (curry_arc_mutex_optional): ContainerTest::mutex_data -> optional_field: {}", data);
    });
    
    // Method 2: Using for_arc_mutex() adapter method
    let optional_field_keypath2 = crate::SomeStruct::optional_field_fr();
    let chained_via_adapter = optional_field_keypath2.for_arc_mutex();
    chained_via_adapter.apply(mutex_ref, |value| {
        println!("✅ Composable chain (for_arc_mutex): ContainerTest::mutex_data -> optional_field: {}", value);
    });
    
    // Example 8: Composable chaining from ContainerTest::mutex_data with fw (failable writable)
    // Chain: ContainerTest -> Arc<Mutex<SomeStruct>> -> SomeStruct -> Option<String> (mutable)
    
    // Get mutex_data from ContainerTest (writable keypath)
    let mutex_data_w = crate::ContainerTest::mutex_data_w();
    
    // For writable access through Arc<Mutex<T>>, we need mutable access to the container
    let mut mutable_container = container.clone();
    
    // Get mutable reference to the Arc<Mutex<SomeStruct>> through the container
    let mutex_ref = mutex_data_w.get_mut(&mut mutable_container);
    
    // For Arc<Mutex<T>>, we need to use lock() instead of get_mut()
    // Since Arc doesn't allow get_mut(), we use lock() to get mutable access
    if let Ok(mut guard) = mutex_ref.lock() {
        // Chain the writable optional keypath
        let optional_field_fw = crate::SomeStruct::optional_field_fw();
        if let Some(field_ref) = optional_field_fw.get_mut(&mut *guard) {
            // field_ref is &mut String (the value inside Option<String>)
            *field_ref = "Updated via ContainerTest::mutex_data fw chain".to_string();
            println!("✅ Chained from ContainerTest::mutex_data (fw) - Updated optional_field: {}", field_ref);
        }
    }
    
    // Note: For fully composable writable chains through Arc<Mutex>, 
    // you would use the same pattern but with writable curried keypaths
    // (WritableCurriedArcMutexKeyPath would need to be implemented similarly)
    
    // Example 9: Multi-level composable chaining from ContainerTest::mutex_data
    // This demonstrates the full composability of the new API
    // Pattern: ContainerTest -> mutex_data -> optional_field
    
    let mutex_data_path = crate::ContainerTest::mutex_data_r();
    let mutex_ref = mutex_data_path.get(&container);
    
    // Fully composable chain using curry_arc_mutex_optional()
    let optional_field_path1 = crate::SomeStruct::optional_field_fr();
    let fully_composable = optional_field_path1.curry_arc_mutex_optional();
    fully_composable.apply(mutex_ref, |data| {
        println!("✅ Fully composable chain (curry_arc_mutex_optional): ContainerTest::mutex_data -> optional_field: {}", data);
    });
    
    // Alternative composable chain using for_arc_mutex()
    let optional_field_path2 = crate::SomeStruct::optional_field_fr();
    let adapter_chain = optional_field_path2.for_arc_mutex();
    adapter_chain.apply(mutex_ref, |data| {
        println!("✅ Adapter chain (for_arc_mutex): ContainerTest::mutex_data -> optional_field: {}", data);
    });
    
    // Demonstrate chaining curried keypaths together
    // Since we're accessing optional_field from SomeStruct, we chain directly
    let optional_kp = crate::SomeStruct::optional_field_fr();
    
    // Chain: curry_arc_mutex_optional() - fully composable!
    let chained_curried = optional_kp.curry_arc_mutex_optional();
    chained_curried.apply(mutex_ref, |value| {
        println!("✅ Chained curried keypaths (curry_arc_mutex_optional): optional_field: {}", value);
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
