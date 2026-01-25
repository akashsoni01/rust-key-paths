//! Example demonstrating functional keypath chains for Arc<Mutex<T>> and Arc<RwLock<T>>
//!
//! This example shows how to use the chain_arc_mutex_at_kp and chain_arc_rwlock_at_kp methods
//! to access and modify data through synchronization primitives in a functional style.
//!
//! Run with: cargo run --example readable_keypaths_new_containers_test

use keypaths_proc::{Kp, ReadableKp, WritableKeypaths};
use rust_keypaths::KeyPath;
use std::rc::Weak;
use std::sync::Arc;

#[derive(Debug, Kp)]
struct ContainerTest {
    // Error handling containers
    result: Result<String, String>,
    result_int: Result<i32, String>,
    
    // Synchronization primitives
    /// Important - it is mandatory to use std::sync::Mutex over Mutex and use std::sync::Mutex; statement
    /// as our parser for Keypaths written for parking_lot as default if you want to use std then use with full import syntax
    /// 
    mutex_data: Arc<std::sync::Mutex<SomeStruct>>,
    /// Important - it is mandatory to use std::sync::RwLock over RwLock and use std::sync::RwLock; statement
    /// as our parser for Keypaths written for parking_lot as default if you want to use std then use with full import syntax
    ///
    rwlock_data: Arc<std::sync::RwLock<SomeStruct>>,
    
    // Reference counting with weak references
    weak_ref: Weak<String>,
    
    // Basic types for comparison
    name: String,
    age: u32,
}

impl ContainerTest {
    fn new() -> Self {
        Self {
            result: Ok("Success!".to_string()),
            result_int: Ok(42),
            mutex_data: Arc::new(std::sync::Mutex::new(SomeStruct {
                data: "Hello".to_string(),
                optional_field: Some("Optional value".to_string()),
            })),
            rwlock_data: Arc::new(std::sync::RwLock::new(SomeStruct {
                data: "RwLock Hello".to_string(),
                optional_field: Some("RwLock Optional".to_string()),
            })),
            weak_ref: Weak::new(),
            name: "Akash".to_string(),
            age: 30,
        }
    }
}

#[derive(Debug, Kp, WritableKeypaths)]
struct SomeStruct {
    data: String,
    optional_field: Option<String>,
}

fn main() {
    println!("=== Functional Keypath Chains for Arc<Mutex<T>> and Arc<RwLock<T>> ===\n");
    
    let container = ContainerTest::new();

    // Test Result<T, E> with ReadableKeypaths
    if let Some(value) = ContainerTest::result_fr().get(&container) {
        println!("✅ Result value: {}", value);
    }
    
    // ==========================================================
    println!("\n=== Arc<Mutex<T>> Chain Examples ===\n");
    // ==========================================================

    // Example 1: Read through Arc<Mutex<T>> with chain_arc_mutex_at_kp
    // let x = ContainerTest::rwlock_data_r().to;
    let x = ContainerTest::rwlock_data_r().to_arc_rwlock_chain();
    ContainerTest::rwlock_data_fr_at(SomeStruct::data_r()).get(&container, |value| {
        println!("asdf = {}", value);
    });
    ContainerTest::mutex_data_r()
        .chain_arc_mutex_at_kp(SomeStruct::data_r())
        .get(&container, |value| {
            println!("✅ chain_arc_mutex_at_kp (read): data = {}", value);
        });
    
    // Example 2: Read optional field through Arc<Mutex<T>>
    ContainerTest::mutex_data_r()
        .chain_arc_mutex_optional_at_kp(SomeStruct::optional_field_fr())
        .get(&container, |value| {
            println!("✅ chain_arc_mutex_optional_at_kp (read): optional_field = {}", value);
        });
    
    // Example 3: Write through Arc<Mutex<T>>
    let write_container = ContainerTest::new();
    ContainerTest::mutex_data_r()
        .chain_arc_mutex_writable_at_kp(SomeStruct::data_w())
        .get_mut(&write_container, |value| {
            *value = "Modified via chain_arc_mutex_writable_at_kp".to_string();
            println!("✅ chain_arc_mutex_writable_at_kp (write): Modified data");
        });
    
    // Verify the write
    ContainerTest::mutex_data_r()
        .chain_arc_mutex_at_kp(SomeStruct::data_r())
        .get(&write_container, |value| {
            println!("   Verified: data = {}", value);
        });
    
    // Example 4: Write optional field through Arc<Mutex<T>>
    ContainerTest::mutex_data_r()
        .chain_arc_mutex_writable_optional_at_kp(SomeStruct::optional_field_fw())
        .get_mut(&write_container, |value| {
            *value = "Modified optional via chain".to_string();
            println!("✅ chain_arc_mutex_writable_optional_at_kp (write): Modified optional_field");
        });
    
    // Verify the write
    ContainerTest::mutex_data_r()
        .chain_arc_mutex_optional_at_kp(SomeStruct::optional_field_fr())
        .get(&write_container, |value| {
            println!("   Verified: optional_field = {}", value);
        });

    // ==========================================================
    println!("\n=== Arc<RwLock<T>> Chain Examples ===\n");
    // ==========================================================
    
    // Example 5: Read through Arc<RwLock<T>> with chain_arc_rwlock_at_kp
    ContainerTest::rwlock_data_r()
        .chain_arc_rwlock_at_kp(SomeStruct::data_r())
        .get(&container, |value| {
            println!("✅ chain_arc_rwlock_at_kp (read): data = {}", value);
        });
    
    // Example 6: Read optional field through Arc<RwLock<T>>
    ContainerTest::rwlock_data_r()
        .chain_arc_rwlock_optional_at_kp(SomeStruct::optional_field_fr())
        .get(&container, |value| {
            println!("✅ chain_arc_rwlock_optional_at_kp (read): optional_field = {}", value);
        });
    
    // Example 7: Write through Arc<RwLock<T>>
    let rwlock_write_container = ContainerTest::new();
    ContainerTest::rwlock_data_r()
        .chain_arc_rwlock_writable_at_kp(SomeStruct::data_w())
        .get_mut(&rwlock_write_container, |value| {
            *value = "Modified via chain_arc_rwlock_writable_at_kp".to_string();
            println!("✅ chain_arc_rwlock_writable_at_kp (write): Modified data");
        });
    
    // Verify the write
    ContainerTest::rwlock_data_r()
        .chain_arc_rwlock_at_kp(SomeStruct::data_r())
        .get(&rwlock_write_container, |value| {
            println!("   Verified: data = {}", value);
        });
    
    // Example 8: Write optional field through Arc<RwLock<T>>
    ContainerTest::rwlock_data_r()
        .chain_arc_rwlock_writable_optional_at_kp(SomeStruct::optional_field_fw())
        .get_mut(&rwlock_write_container, |value| {
            *value = "Modified optional via rwlock chain".to_string();
            println!("✅ chain_arc_rwlock_writable_optional_at_kp (write): Modified optional_field");
        });
    
    // Verify the write
    ContainerTest::rwlock_data_r()
        .chain_arc_rwlock_optional_at_kp(SomeStruct::optional_field_fr())
        .get(&rwlock_write_container, |value| {
            println!("   Verified: optional_field = {}", value);
        });
    
    // ==========================================================
    println!("\n=== Summary ===\n");
    // ==========================================================
    
    println!("Available chain methods from KeyPath:");
    println!("  • chain_arc_mutex_at_kp(inner_keypath) -> read through Arc<Mutex<T>>");
    println!("  • chain_arc_mutex_optional_at_kp(inner_optional_keypath) -> read optional through Arc<Mutex<T>>");
    println!("  • chain_arc_mutex_writable_at_kp(inner_writable_keypath) -> write through Arc<Mutex<T>>");
    println!("  • chain_arc_mutex_writable_optional_at_kp(inner_writable_optional_keypath) -> write optional through Arc<Mutex<T>>");
    println!("  • chain_arc_rwlock_at_kp(inner_keypath) -> read through Arc<RwLock<T>>");
    println!("  • chain_arc_rwlock_optional_at_kp(inner_optional_keypath) -> read optional through Arc<RwLock<T>>");
    println!("  • chain_arc_rwlock_writable_at_kp(inner_writable_keypath) -> write through Arc<RwLock<T>>");
    println!("  • chain_arc_rwlock_writable_optional_at_kp(inner_writable_optional_keypath) -> write optional through Arc<RwLock<T>>");
    
    println!("\n=== All functional chain examples completed successfully! ===");
}
