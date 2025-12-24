//! Example demonstrating deeply nested parking_lot RwLock chains
//! 
//! Run with: cargo run --example parking_lot_nested_chain --features parking_lot
//! 
//! This example shows how to chain through multiple `Arc<parking_lot::RwLock<T>>` layers
//! using the functional chain methods.

#[cfg(not(feature = "parking_lot"))]
compile_error!("This example requires the 'parking_lot' feature. Run with: cargo run --example parking_lot_nested_chain --features parking_lot");

#[cfg(feature = "parking_lot")]
mod example {
    use std::sync::Arc;
    use keypaths_proc::Keypaths;
    use parking_lot::RwLock;
    use rust_keypaths::keypath;

    #[derive(Keypaths)]
    #[Writable]
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
    #[Writable]
    pub struct SomeOtherStruct {
        pub f3: Option<String>,
        pub f4: Arc<RwLock<DeeplyNestedStruct>>,
    }

    #[derive(Keypaths)]
    #[Writable]
    pub struct DeeplyNestedStruct {
        pub f1: Option<String>,
        pub f2: Option<i32>,
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
                })),
            })),
        };

        println!("Initial state:");
        println!("  f3 = {:?}", instance.f1.read().f3);
        println!("  f4.f1 = {:?}", instance.f1.read().f4.read().f1);
        println!("  f4.f2 = {:?}", instance.f1.read().f4.read().f2);

        // Create a keypath to the Arc<RwLock<SomeOtherStruct>> field
        let f1_kp = keypath!(|s: &SomeStruct| &s.f1);

        println!("\n--- Reading through first lock layer ---");

        // Read a non-optional field through Arc<RwLock<T>>
        // Note: f3 is Option<String>, so we use an OptionalKeyPath
        let f3_read_kp = rust_keypaths::OptionalKeyPath::new(
            |s: &SomeOtherStruct| s.f3.as_ref()
        );
        
        f1_kp.clone()
            .then_arc_parking_rwlock_optional_at_kp(f3_read_kp)
            .get(&instance, |value| {
                println!("✅ Read f3 via chain: {:?}", value);
            });

        // Read f4 (Arc<RwLock<DeeplyNestedStruct>>) through first lock
        let f4_read_kp = rust_keypaths::KeyPath::new(
            |s: &SomeOtherStruct| &s.f4
        );

        f1_kp.clone()
            .then_arc_parking_rwlock_at_kp(f4_read_kp.clone())
            .get(&instance, |f4_arc| {
                println!("✅ Got reference to f4 Arc<RwLock<...>>: {:p}", f4_arc);
                
                // Now chain through the second lock layer
                let identity_kp = keypath!(|s: &Arc<RwLock<DeeplyNestedStruct>>| s);
                
                let deep_f1_kp = rust_keypaths::OptionalKeyPath::new(
                    |s: &DeeplyNestedStruct| s.f1.as_ref()
                );
                let deep_f2_kp = rust_keypaths::OptionalKeyPath::new(
                    |s: &DeeplyNestedStruct| s.f2.as_ref()
                );

                identity_kp.clone()
                    .then_arc_parking_rwlock_optional_at_kp(deep_f1_kp)
                    .get(f4_arc, |value| {
                        println!("✅ Read f4.f1 via nested chain: {:?}", value);
                    });

                identity_kp
                    .then_arc_parking_rwlock_optional_at_kp(deep_f2_kp)
                    .get(f4_arc, |value| {
                        println!("✅ Read f4.f2 via nested chain: {:?}", value);
                    });
            });

        println!("\n--- Writing through lock layers ---");

        // Write to f3 through first lock
        let f3_write_kp = rust_keypaths::WritableOptionalKeyPath::new(
            |s: &mut SomeOtherStruct| s.f3.as_mut()
        );

        f1_kp.clone()
            .then_arc_parking_rwlock_writable_optional_at_kp(f3_write_kp)
            .get_mut(&instance, |value| {
                *value = String::from("updated_middle_value");
                println!("✅ Wrote to f3 via chain");
            });

        // Write to deeply nested values through both lock layers
        f1_kp.clone()
            .then_arc_parking_rwlock_at_kp(f4_read_kp)
            .get(&instance, |f4_arc| {
                let identity_kp = keypath!(|s: &Arc<RwLock<DeeplyNestedStruct>>| s);
                
                let deep_f1_w_kp = rust_keypaths::WritableOptionalKeyPath::new(
                    |s: &mut DeeplyNestedStruct| s.f1.as_mut()
                );
                let deep_f2_w_kp = rust_keypaths::WritableOptionalKeyPath::new(
                    |s: &mut DeeplyNestedStruct| s.f2.as_mut()
                );

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

        println!("\n--- Verifying writes ---");
        println!("  f3 = {:?}", instance.f1.read().f3);
        println!("  f4.f1 = {:?}", instance.f1.read().f4.read().f1);
        println!("  f4.f2 = {:?}", instance.f1.read().f4.read().f2);

        println!("\n=== No cloning occurred - all access was zero-copy! ===");
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
