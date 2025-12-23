use keypaths_proc::{Keypaths, ReadableKeypaths, WritableKeypaths};
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
    
    // Example 7: Chaining CurriedMutexOptionalKeyPath with fr (failable readable)
    // Create nested structure wrapped in Mutex for chaining demonstration
    let nested_mutex = Arc::new(Mutex::new(NestedStruct {
        inner: Some(SomeStruct { 
            data: "Nested data".to_string(),
            optional_field: Some("Nested optional".to_string()),
        }),
    }));
    
    // Create keypaths for chaining:
    // 1. NestedStruct::inner_fr() - gets Option<&SomeStruct> from NestedStruct
    // 2. SomeStruct::optional_field_fr() - gets Option<&String> from SomeStruct
    let inner_keypath = crate::NestedStruct::inner_fr();
    let optional_field_keypath = crate::SomeStruct::optional_field_fr();
    
    // Chain: Mutex<NestedStruct> -> Option<SomeStruct> -> Option<String>
    // First curry the inner optional keypath with curry_mutex_optional(), then chain with optional_field keypath
    let chained_fr = inner_keypath.curry_mutex_optional().then(optional_field_keypath);
    
    // Apply the chained keypath - callback only runs if both steps succeed
    chained_fr.apply(&nested_mutex, |data| {
        println!("✅ Chained fr keypaths (curry_mutex_optional) - Optional field: {}", data);
    });
    
    // Example 8: Chaining with fw (failable writable) keypaths
    // For writable keypaths, we need to use mutable mutex access
    // Note: We use Mutex directly (not Arc) for mutable access
    let mut nested_mutex_mut = Mutex::new(NestedStruct {
        inner: Some(SomeStruct { 
            data: "Writable nested data".to_string(),
            optional_field: Some("Original optional".to_string()),
        }),
    });
    
    // Create writable optional keypaths for optional fields
    // inner_fw() - gets Option<&mut SomeStruct> from NestedStruct
    // optional_field_fw() - gets Option<&mut String> from SomeStruct
    let inner_fw = crate::NestedStruct::inner_fw();
    let optional_field_fw = crate::SomeStruct::optional_field_fw();
    
    // Note: For writable keypaths through Mutex, we need to use get_mut() pattern
    // Since Mutex requires mutable access for writable operations
    if let Ok(guard) = nested_mutex_mut.get_mut() {
        // Chain the writable optional keypaths directly (not through mutex curry)
        if let Some(field_ref) = inner_fw.get_mut(&mut *guard)
            .and_then(|inner| optional_field_fw.get_mut(inner)) {
            *field_ref = "Updated via fw chain".to_string();
            println!("✅ Chained fw keypaths - Updated optional_field: {}", field_ref);
        }
    }
    
    // Example 9: Chaining through Mutex with fr -> fr pattern (different fields)
    // This shows how to chain multiple optional keypaths through a mutex
    let mutex_with_nested = Arc::new(Mutex::new(NestedStruct {
        inner: Some(SomeStruct { 
            data: "Chain test".to_string(),
            optional_field: Some("Chain optional".to_string()),
        }),
    }));
    
    // Chain fr keypaths: NestedStruct::inner_fr() -> SomeStruct::optional_field_fr()
    let chain1 = crate::NestedStruct::inner_fr();
    let chain2 = crate::SomeStruct::optional_field_fr();
    let chained_through_mutex = chain1.curry_mutex_optional().then(chain2);
    
    chained_through_mutex.apply(&mutex_with_nested, |data| {
        println!("✅ Chained fr->fr through Mutex - Optional field: {}", data);
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
