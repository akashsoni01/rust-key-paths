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
    use keypaths_proc::Kp;
    use parking_lot::RwLock;
    use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

    #[derive(Kp)]
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

    #[derive(Kp)]
    #[All]  // Generate both readable and writable keypaths
    pub struct SomeOtherStruct {
        pub f3: Option<String>,
        pub f4: Arc<RwLock<DeeplyNestedStruct>>,
    }

    #[derive(Kp)]
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
        println!("  • _fr_at()  -> Chain through lock for reading (parking_lot)");
        println!("  • _fw_at()  -> Chain through lock for writing (parking_lot)");
        println!("\nFor Arc<std::sync::RwLock<T>> fields (explicit prefix), generates:");
        println!("  • _fr_at()  -> Chain through lock for reading (std::sync)");
        println!("  • _fw_at()  -> Chain through lock for writing (std::sync)");

        // ============================================================
        // USING THE _fr_at() HELPER METHOD (parking_lot by default)
        // ============================================================
        println!("\n--- Using _fr_at() for reading (parking_lot) ---");

        // Create a keypath to the name field (non-optional)
        let name_kp = KeyPath::new(|s: &DeeplyNestedStruct| &s.name);
        
        // Use the generated f1_fr_at() to chain through the first lock
        // (defaults to parking_lot since we used `RwLock` without `std::sync::` prefix)
        SomeStruct::f1_fr_at(SomeOtherStruct::f4_r())
            .get(&instance, |f4_arc| {
                // Now chain through the second lock to get the name
                SomeOtherStruct::f4_fr_at(name_kp.clone())
                    .get(&instance.f1.read(), |name| {
                        println!("✅ Read name via _fr_at chain (parking_lot): {:?}", name);
                    });
            });
        
        // Alternative: Direct chaining through both locks
        // Using the identity keypath pattern for nested access
        let identity_kp = KeyPath::new(|s: &Arc<RwLock<DeeplyNestedStruct>>| s);
        
        SomeStruct::f1_r()
            .chain_arc_parking_rwlock_at_kp(SomeOtherStruct::f4_r())
            .get(&instance, |f4_arc| {
                identity_kp.clone()
                    .chain_arc_parking_rwlock_at_kp(name_kp.clone())
                    .get(f4_arc, |name| {
                        println!("✅ Read name via nested chain: {:?}", name);
                    });
            });

        // ============================================================
        // READING OPTIONAL FIELDS
        // ============================================================
        println!("\n--- Reading optional fields through locks ---");

        // Create keypaths for optional inner fields
        let f3_read_kp = OptionalKeyPath::new(|s: &SomeOtherStruct| s.f3.as_ref());

        // Chain through first lock to read f3 (Option<String>)
        SomeStruct::f1_r()
            .then_arc_parking_rwlock_optional_at_kp(f3_read_kp.clone())
            .get(&instance, |value| {
                println!("✅ Read f3 via chain: {:?}", value);
            });

        // Read through two lock layers to get optional fields
        SomeStruct::f1_r()
            .chain_arc_parking_rwlock_at_kp(SomeOtherStruct::f4_r())
            .get(&instance, |f4_arc| {
                let deep_f1_kp = OptionalKeyPath::new(|s: &DeeplyNestedStruct| s.f1.as_ref());
                let deep_f2_kp = OptionalKeyPath::new(|s: &DeeplyNestedStruct| s.f2.as_ref());

                identity_kp.clone()
                    .then_arc_parking_rwlock_optional_at_kp(deep_f1_kp)
                    .get(f4_arc, |value| {
                        println!("✅ Read f4.f1 via nested chain: {:?}", value);
                    });

                identity_kp.clone()
                    .then_arc_parking_rwlock_optional_at_kp(deep_f2_kp)
                    .get(f4_arc, |value| {
                        println!("✅ Read f4.f2 via nested chain: {:?}", value);
                    });
            });

        // ============================================================
        // USING THE NEW _parking_fw_at() HELPER METHOD FOR WRITING
        // ============================================================
        println!("\n--- Using _parking_fw_at() for writing ---");

        // Create a writable keypath to the name field (non-optional)
        let name_w_kp = WritableKeyPath::new(|s: &mut DeeplyNestedStruct| &mut s.name);

        // Use the generated method to write through both locks
        SomeStruct::f1_r()
            .chain_arc_parking_rwlock_at_kp(SomeOtherStruct::f4_r())
            .get(&instance, |f4_arc| {
                let identity_kp = KeyPath::new(|s: &Arc<RwLock<DeeplyNestedStruct>>| s);
                identity_kp
                    .chain_arc_parking_rwlock_writable_at_kp(name_w_kp.clone())
                    .get_mut(f4_arc, |name| {
                        *name = String::from("updated_name");
                        println!("✅ Wrote name via _parking_fw_at chain");
                    });
            });

        // ============================================================
        // WRITING OPTIONAL FIELDS
        // ============================================================
        println!("\n--- Writing optional fields through locks ---");

        // Write to f3 through first lock
        let f3_write_kp = WritableOptionalKeyPath::new(|s: &mut SomeOtherStruct| s.f3.as_mut());
        SomeStruct::f1_r()
            .then_arc_parking_rwlock_writable_optional_at_kp(f3_write_kp)
            .get_mut(&instance, |value| {
                *value = String::from("updated_middle_value");
                println!("✅ Wrote to f3 via chain");
            });

        // Write to deeply nested values through both lock layers
        SomeStruct::f1_r()
            .chain_arc_parking_rwlock_at_kp(SomeOtherStruct::f4_r())
            .get(&instance, |f4_arc| {
                let identity_kp = KeyPath::new(|s: &Arc<RwLock<DeeplyNestedStruct>>| s);
                
                let deep_f1_w_kp = WritableOptionalKeyPath::new(|s: &mut DeeplyNestedStruct| s.f1.as_mut());
                let deep_f2_w_kp = WritableOptionalKeyPath::new(|s: &mut DeeplyNestedStruct| s.f2.as_mut());

                identity_kp.clone()
                    .then_arc_parking_rwlock_writable_optional_at_kp(deep_f1_w_kp)
                    .get_mut(f4_arc, |value| {
                        *value = String::from("updated_deep_string");
                        println!("✅ Wrote to f4.f1 via nested chain");
                    });

                identity_kp
                    .then_arc_parking_rwlock_writable_optional_at_kp(deep_f2_w_kp)
                    .get_mut(f4_arc, |value| {
                        *value = 100;
                        println!("✅ Wrote to f4.f2 via nested chain");
                    });
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
        println!("✅ Used derive-generated keypaths: SomeStruct::f1_r(), SomeOtherStruct::f4_r()");
        println!("✅ Used _parking_fr_at() and _parking_fw_at() for simplified chaining");
        println!("✅ Chained through multiple Arc<RwLock<T>> layers safely");
        println!("✅ Works with parking_lot for better performance (no lock poisoning)");
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
