//! Example demonstrating deeply nested parking_lot RwLock chains
//! 
//! Run with: cargo run --example parking_lot_nested_chain --features parking_lot
//! 
//! This example shows how to:
//! 1. Use derive-generated keypath methods for lock fields
//! 2. Use the new `_parking_fr_at()` and `_parking_fw_at()` helper methods
//! 3. Chain through multiple `Arc<parking_lot::RwLock<T>>` layers
//! 4. Read and write through nested locks without cloning

#[cfg(not(feature = "parking_lot"))]
compile_error!("This example requires the 'parking_lot' feature. Run with: cargo run --example parking_lot_nested_chain --features parking_lot");

#[cfg(feature = "parking_lot")]
mod example {
    use std::sync::Arc;
    use keypaths_proc::Keypaths;
    use parking_lot::RwLock;
    use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

    #[derive(Keypaths)]
    #[All]  // Generate both readable and writable keypaths
    pub struct SomeStruct {
        pub f1: Arc<RwLock<SomeOtherStruct>>,
    }

    impl Clone for SomeStruct {
        fn clone(&self) -> Self {
            panic!("SomeStruct should not be cloned!")
        }
    }

    impl Clone for SomeOtherStruct {
        fn clone(&self) -> Self {
            panic!("SomeOtherStruct should not be cloned!")
        }
    }

    #[derive(Keypaths)]
    #[All]  // Generate both readable and writable keypaths
    pub struct SomeOtherStruct {
        pub f3: Option<String>,
        pub f4: Arc<RwLock<DeeplyNestedStruct>>,
    }

    #[derive(Keypaths)]
    #[All]  // Generate both readable and writable keypaths
    pub struct DeeplyNestedStruct {
        pub f1: Option<String>,
        pub f2: Option<i32>,
        pub name: String,  // Non-optional field
    }

    impl Clone for DeeplyNestedStruct {
        fn clone(&self) -> Self {
            panic!("DeeplyNestedStruct should not be cloned!")
        }
    }

    pub fn run() {
        println!("=== Deeply Nested parking_lot RwLock Chain Example ===\n");

        let instance = SomeStruct {
            f1: Arc::new(RwLock::new(SomeOtherStruct {
                f3: Some(String::from("middle_value")),
                f4: Arc::new(RwLock::new(DeeplyNestedStruct {
                    f1: Some(String::from("deep_string")),
                    f2: Some(42),
                    name: String::from("initial_name"),
                })),
            })),
        };

        println!("Initial state:");
        println!("  f3 = {:?}", instance.f1.read().f3);
        println!("  f4.f1 = {:?}", instance.f1.read().f4.read().f1);
        println!("  f4.f2 = {:?}", instance.f1.read().f4.read().f2);
        println!("  f4.name = {:?}", instance.f1.read().f4.read().name);

        println!("\n=== Generated Methods for Arc<RwLock<T>> Fields ===");
        println!("⚠️  IMPORTANT: RwLock and Mutex DEFAULT to parking_lot!");
        println!("    Use `std::sync::RwLock` or `std::sync::Mutex` for std::sync types.\n");
        println!("For Arc<RwLock<T>> fields (parking_lot default), the macro generates:");
        println!("  • _r()      -> KeyPath<Struct, Arc<RwLock<T>>>  (readable)");
        println!("  • _w()      -> WritableKeyPath<Struct, Arc<RwLock<T>>>  (writable)");
        println!("  • _fr()     -> LockedKeyPath (supports .then() and .then_optional())");
        println!("  • _fw()     -> LockedWritableKeyPath (supports .then() and .then_optional())");
        println!("  • _fr_at()  -> Chain through lock for reading (deprecated, use _fr().then())");
        println!("  • _fw_at()  -> Chain through lock for writing (deprecated, use _fw().then())");

        // ============================================================
        // USING THE NEW _fr() METHOD WITH MONADIC CHAINING
        // ============================================================
        println!("\n--- Using _fr().then() for monadic chaining (parking_lot) ---");

        // Simple chain: Read through one lock level (f3 is optional, so use then_optional)
        SomeStruct::f1_fr()
            .then_optional(SomeOtherStruct::f3_fr())
            .get(&instance, |value| {
                println!("✅ Read f3 via _fr().then_optional(): {:?}", value);
            });

        // Deep chain: Read through multiple lock levels using .then().then()
        // First chain through f1 to get SomeOtherStruct, then through f4 to get DeeplyNestedStruct
        SomeStruct::f1_fr()
            .then(SomeOtherStruct::f4_fr())
            .then(DeeplyNestedStruct::name_r())
            .get(&instance, |name| {
                println!("✅ Read name via _fr().then().then(): {:?}", name);
            });

        // Chain with optional fields using .then_optional()
        SomeStruct::f1_fr()
            .then(SomeOtherStruct::f4_fr())
            .then_optional(DeeplyNestedStruct::f1_fr())
            .get(&instance, |value| {
                println!("✅ Read f4.f1 (optional) via _fr().then().then_optional(): {:?}", value);
            });

        // ============================================================
        // READING OPTIONAL FIELDS WITH then_optional()
        // ============================================================
        println!("\n--- Reading optional fields through locks with then_optional() ---");

        // Chain through first lock to read f3 (Option<String>) using then_optional()
        SomeStruct::f1_fr()
            .then_optional(SomeOtherStruct::f3_fr())
            .get(&instance, |value| {
                println!("✅ Read f3 via _fr().then_optional(): {:?}", value);
            });

        // Read through two lock layers to get optional fields
        SomeStruct::f1_fr()
            .then(SomeOtherStruct::f4_fr())
            .then_optional(DeeplyNestedStruct::f1_fr())
            .get(&instance, |value| {
                println!("✅ Read f4.f1 via _fr().then().then_optional(): {:?}", value);
            });

        SomeStruct::f1_fr()
            .then(SomeOtherStruct::f4_fr())
            .then_optional(DeeplyNestedStruct::f2_fr())
            .get(&instance, |value| {
                println!("✅ Read f4.f2 via _fr().then().then_optional(): {:?}", value);
            });

        // ============================================================
        // USING THE NEW _fw() METHOD WITH MONADIC CHAINING FOR WRITING
        // ============================================================
        println!("\n--- Using _fw().then() for monadic chaining (writing) ---");

        // Deep chain: Write through multiple lock levels using .then().then()
        SomeStruct::f1_fw()
            .then(SomeOtherStruct::f4_fr())
            .then(DeeplyNestedStruct::name_w())
            .get_mut(&instance, |name| {
                *name = String::from("updated_name");
                println!("✅ Wrote name via _fw().then().then()");
            });

        // ============================================================
        // WRITING OPTIONAL FIELDS WITH then_optional()
        // ============================================================
        println!("\n--- Writing optional fields through locks with then_optional() ---");

        // Write to f3 through first lock using then_optional()
        SomeStruct::f1_fw()
            .then_optional(SomeOtherStruct::f3_fw())
            .get_mut(&instance, |value| {
                *value = Some(String::from("updated_middle_value"));
                println!("✅ Wrote to f3 via _fw().then_optional()");
            });

        // Write to deeply nested values through both lock layers
        SomeStruct::f1_fw()
            .then(SomeOtherStruct::f4_fr())
            .then_optional(DeeplyNestedStruct::f1_fw())
            .get_mut(&instance, |value| {
                *value = Some(String::from("updated_deep_string"));
                println!("✅ Wrote to f4.f1 via _fw().then().then_optional()");
            });

        SomeStruct::f1_fw()
            .then(SomeOtherStruct::f4_fr())
            .then_optional(DeeplyNestedStruct::f2_fw())
            .get_mut(&instance, |value| {
                *value = Some(100);
                println!("✅ Wrote to f4.f2 via _fw().then().then_optional()");
            });

        // ============================================================
        // VERIFY THE WRITES
        // ============================================================
        println!("\n--- Verifying writes ---");
        println!("  f3 = {:?}", instance.f1.read().f3);
        println!("  f4.f1 = {:?}", instance.f1.read().f4.read().f1);
        println!("  f4.f2 = {:?}", instance.f1.read().f4.read().f2);
        println!("  f4.name = {:?}", instance.f1.read().f4.read().name);

        println!("\n=== Key Takeaways ===");
        println!("✅ No cloning occurred - all access was zero-copy!");
        println!("✅ Used derive-generated locked keypaths: SomeStruct::f1_fr(), SomeStruct::f1_fw()");
        println!("✅ Used monadic chaining: .then() and .then_optional() for clean syntax");
        println!("✅ Chained through multiple Arc<RwLock<T>> layers safely");
        println!("✅ Works with parking_lot for better performance (no lock poisoning)");
        println!("✅ Example: SomeStruct::f1_fr().then(SomeOtherStruct::f4_r()).then(DeeplyNestedStruct::name_r())");
    }
}

#[cfg(feature = "parking_lot")]
fn main() {
    example::run();
}

#[cfg(not(feature = "parking_lot"))]
fn main() {
    eprintln!("This example requires the 'parking_lot' feature.");
    eprintln!("Run with: cargo run --example parking_lot_nested_chain --features parking_lot");
}
