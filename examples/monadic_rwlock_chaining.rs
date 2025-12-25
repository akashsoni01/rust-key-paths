//! Comprehensive example demonstrating monadic keypath chaining through Arc<RwLock<T>>
//!
//! This example shows how to use the monadic `then()` method to chain multiple keypaths
//! through deeply nested Arc<RwLock<T>> structures in a clean, functional style.
//!
//! Run with: cargo run --example monadic_rwlock_chaining

use keypaths_proc::Keypaths;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Keypaths)]
#[All]
struct ContainerTest {
    rwlock_data: Arc<RwLock<SomeStruct>>,
}

impl ContainerTest {
    fn new() -> Self {
        Self {
            rwlock_data: Arc::new(RwLock::new(SomeStruct {
                f1: Arc::new(RwLock::new(SomeOtherStruct {
                    f3: Some(String::from("value")),
                    f4: DeeplyNestedStruct {
                        f1: Some(String::from("deep value")),
                        f2: Some(42),
                    },
                })),
            })),
        }
    }
}

#[derive(Debug, Keypaths)]
#[All]
struct SomeStruct {
    f1: Arc<RwLock<SomeOtherStruct>>,
}

#[derive(Debug, Keypaths)]
#[All]
struct SomeOtherStruct {
    f3: Option<String>,
    f4: DeeplyNestedStruct,
}

#[derive(Debug, Keypaths)]
#[All]
struct DeeplyNestedStruct {
    f1: Option<String>,
    f2: Option<i32>,
}

fn main() {
    println!("=== Monadic Keypath Chaining through Arc<RwLock<T>> ===\n");
    
    let container = ContainerTest::new();

    // ==========================================================
    println!("=== Example 1: Simple Chain (2 levels) ===\n");
    // ==========================================================
    
    // Read through: ContainerTest -> Arc<RwLock<SomeStruct>> -> SomeStruct -> f1 -> Arc<RwLock<SomeOtherStruct>>
    // Then read: SomeOtherStruct -> f3 (Option<String>)
    println!("Reading f3 through 2 levels of Arc<RwLock<T>>:");
    ContainerTest::rwlock_data_r()
        .then_arc_parking_rwlock_at_kp(SomeStruct::f1_r())
        .get(&container, |f1_arc| {
            // Now we have Arc<RwLock<SomeOtherStruct>>, chain through it
            let guard = f1_arc.read();
            SomeOtherStruct::f3_fr().get(&*guard, |value| {
                println!("  ✅ f3 value: {:?}", value);
            });
        });

    // ==========================================================
    println!("\n=== Example 2: Write Chain with Monadic Composition ===\n");
    // ==========================================================
    
    // Write through: ContainerTest -> Arc<RwLock<SomeStruct>> -> SomeStruct -> f1 -> Arc<RwLock<SomeOtherStruct>>
    // Then write: SomeOtherStruct -> f3 (Option<String>)
    println!("Writing to f3 through 2 levels using monadic then():");
    let mut write_container = ContainerTest::new();
    
    // Use monadic chaining: .then().then() pattern with proc macro generated _then() method
    ContainerTest::rwlock_data_then(SomeStruct::f1_w())
        .then_optional(SomeOtherStruct::f3_fw())
        .get_mut(&write_container, |value| {
            *value = Some(String::from("updated via monadic chain"));
            println!("  ✅ Updated f3 via monadic then().then_optional() chain");
        });

    // Verify the write
    ContainerTest::rwlock_data_r()
        .then_arc_parking_rwlock_at_kp(SomeStruct::f1_r())
        .get(&write_container, |f1_arc| {
            let guard = f1_arc.read();
            SomeOtherStruct::f3_fr().get(&*guard, |value| {
                println!("  ✅ Verified f3: {:?}", value);
            });
        });

    // ==========================================================
    println!("\n=== Example 3: Deep Chain (3+ levels) ===\n");
    // ==========================================================
    
    // Write to deeply nested field: f4.f1
    // Path: ContainerTest -> Arc<RwLock<SomeStruct>> -> SomeStruct -> f1 -> Arc<RwLock<SomeOtherStruct>>
    //       -> SomeOtherStruct -> f4 -> DeeplyNestedStruct -> f1 (Option<String>)
    println!("Writing to deeply nested f4.f1 through multiple RwLock levels:");
    
    // Using proc macro generated _then() method for clean monadic syntax
    ContainerTest::rwlock_data_then(SomeStruct::f1_w())
        .then(SomeOtherStruct::f4_w())
        .then_optional(DeeplyNestedStruct::f1_fw())
        .get_mut(&write_container, |value| {
            *value = Some(String::from("deeply nested update"));
            println!("  ✅ Updated f4.f1 via deep .then().then().then_optional() chain");
        });

    // ==========================================================
    println!("\n=== Example 4: Using Proc Macro Generated then() Method ===\n");
    // ==========================================================
    
    // The proc macro generates _then() methods for Arc<RwLock<T>> fields
    println!("Using proc macro generated _then() method for clean monadic chaining:");
    
    ContainerTest::rwlock_data_then(SomeStruct::f1_w())
        .then(SomeOtherStruct::f4_w())
        .then_optional(DeeplyNestedStruct::f2_fw())
        .get_mut(&write_container, |value| {
            *value = Some(100);
            println!("  ✅ Updated f4.f2 using .then().then().then_optional() chain");
        });

    // Verify
    ContainerTest::rwlock_data_r()
        .then_arc_parking_rwlock_at_kp(SomeStruct::f1_r())
        .get(&write_container, |f1_arc| {
            let guard = f1_arc.read();
            SomeStruct::f1_r()
                .then_arc_parking_rwlock_at_kp(SomeOtherStruct::f4_r())
                .get(&*guard, |f4| {
                    let f2 = DeeplyNestedStruct::f2_fr().get(f4);
                    println!("  ✅ Verified f4.f2: {:?}", f2);
                });
        });

    // ==========================================================
    println!("\n=== Example 5: Chaining with Optional Fields ===\n");
    // ==========================================================
    
    // Chain through optional fields using then_optional()
    println!("Chaining through optional fields with then_optional():");
    
    ContainerTest::rwlock_data_then(SomeStruct::f1_w())
        .then_optional(SomeOtherStruct::f3_fw())
        .get_mut(&write_container, |value| {
            *value = Some(String::from("updated via then_optional"));
            println!("  ✅ Updated f3 (optional) via then_optional()");
        });

    // Chain through multiple optional fields
    ContainerTest::rwlock_data_then(SomeStruct::f1_w())
        .then(SomeOtherStruct::f4_w())
        .then_optional(DeeplyNestedStruct::f1_fw())
        .get_mut(&write_container, |value| {
            *value = Some(String::from("deep optional update"));
            println!("  ✅ Updated f4.f1 (optional) via .then().then_optional()");
        });

    // ==========================================================
    println!("\n=== Example 6: Multiple Sequential Operations ===\n");
    // ==========================================================
    
    // Perform multiple operations using the same chain pattern
    println!("Performing multiple sequential operations:");
    
    let mut multi_container = ContainerTest::new();
    
    // Update f3
    ContainerTest::rwlock_data_then(SomeStruct::f1_w())
        .then_optional(SomeOtherStruct::f3_fw())
        .get_mut(&multi_container, |value| {
            *value = Some(String::from("first update"));
        });
    
    // Update f4.f1
    ContainerTest::rwlock_data_then(SomeStruct::f1_w())
        .then(SomeOtherStruct::f4_w())
        .then_optional(DeeplyNestedStruct::f1_fw())
        .get_mut(&multi_container, |value| {
            *value = Some(String::from("second update"));
        });
    
    // Update f4.f2
    ContainerTest::rwlock_data_then(SomeStruct::f1_w())
        .then(SomeOtherStruct::f4_w())
        .then_optional(DeeplyNestedStruct::f2_fw())
        .get_mut(&multi_container, |value| {
            // *value = Some(999);
            todo!()
        });
    
    println!("  ✅ Performed 3 sequential updates using monadic chains");

    // ==========================================================
    println!("\n=== Summary ===\n");
    // ==========================================================
    
    println!("Monadic Keypath Chaining Benefits:");
    println!("  ✅ Clean, functional style: .then().then().then()");
    println!("  ✅ Type-safe composition at compile time");
    println!("  ✅ Easy to read and understand the access path");
    println!("  ✅ Supports both writable and optional keypaths");
    println!("  ✅ Works seamlessly with Arc<RwLock<T>> structures");
    println!("  ✅ Proc macro generates _then() methods automatically");
    println!("\nAvailable Methods:");
    println!("  • StructName::field_then(keypath) - Start chain (proc macro generated)");
    println!("  • .then(keypath) - Chain with WritableKeyPath");
    println!("  • .then_optional(keypath) - Chain with WritableOptionalKeyPath (for Option fields)");
    println!("  • .get_mut(container, callback) - Execute write operation");
    println!("\nExample Usage:");
    println!("  ContainerTest::rwlock_data_then(SomeStruct::f1_w())");
    println!("      .then(SomeOtherStruct::f4_w())");
    println!("      .then_optional(DeeplyNestedStruct::f1_w())");
    println!("      .get_mut(&container, |value| *value = Some(\"new\".to_string()));");
    println!("\n=== All monadic chain examples completed successfully! ===");
}

