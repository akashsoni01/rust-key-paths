//! Example demonstrating parking_lot::Mutex and parking_lot::RwLock functional keypath chains
//! 
//! Run with: cargo run --example parking_lot_chains --features parking_lot

#[cfg(feature = "parking_lot")]
mod parking_lot_example {
    use keypaths_proc::{Keypaths, WritableKeypaths};
    use rust_keypaths::{KeyPath, keypath};
    use std::sync::Arc;
    use parking_lot::{Mutex as ParkingMutex, RwLock as ParkingRwLock};

    #[derive(Debug)]
    struct Container {
        // parking_lot synchronization primitives
        parking_mutex_data: Arc<ParkingMutex<DataStruct>>,
        parking_rwlock_data: Arc<ParkingRwLock<DataStruct>>,
    }

    impl Container {
        fn new() -> Self {
            Self {
                parking_mutex_data: Arc::new(ParkingMutex::new(DataStruct {
                    name: "Mutex Hello".to_string(),
                    optional_value: Some("Mutex Optional".to_string()),
                    count: 42,
                })),
                parking_rwlock_data: Arc::new(ParkingRwLock::new(DataStruct {
                    name: "RwLock Hello".to_string(),
                    optional_value: Some("RwLock Optional".to_string()),
                    count: 100,
                })),
            }
        }
        
        // Manual keypath methods for Arc<parking_lot::Mutex/RwLock> fields
        fn parking_mutex_data_r() -> KeyPath<Container, Arc<ParkingMutex<DataStruct>>, impl for<'r> Fn(&'r Container) -> &'r Arc<ParkingMutex<DataStruct>>> {
            keypath!(|c: &Container| &c.parking_mutex_data)
        }
        
        fn parking_rwlock_data_r() -> KeyPath<Container, Arc<ParkingRwLock<DataStruct>>, impl for<'r> Fn(&'r Container) -> &'r Arc<ParkingRwLock<DataStruct>>> {
            keypath!(|c: &Container| &c.parking_rwlock_data)
        }
    }

    #[derive(Debug, Keypaths, WritableKeypaths)]
    struct DataStruct {
        name: String,
        optional_value: Option<String>,
        count: i32,
    }

    pub fn run() {
        println!("=== parking_lot Functional Keypath Chains Example ===\n");
        
        let container = Container::new();
        
        // ========== parking_lot::Mutex Examples ==========
        println!("=== Arc<parking_lot::Mutex<T>> Chains ===\n");
        
        // Example 1: Read through Arc<parking_lot::Mutex<T>>
        Container::parking_mutex_data_r()
            .chain_arc_parking_mutex(DataStruct::name_r())
            .get(&container, |value| {
                println!("✅ chain_arc_parking_mutex (read): name = {}", value);
            });
        
        // Example 2: Read optional field through Arc<parking_lot::Mutex<T>>
        Container::parking_mutex_data_r()
            .chain_arc_parking_mutex_optional(DataStruct::optional_value_fr())
            .get(&container, |value| {
                println!("✅ chain_arc_parking_mutex_optional (read): optional_value = {}", value);
            });
        
        // Example 3: Write through Arc<parking_lot::Mutex<T>>
        let write_container = Container::new();
        Container::parking_mutex_data_r()
            .chain_arc_parking_mutex_writable(DataStruct::name_w())
            .get_mut(&write_container, |value| {
                *value = "Modified via chain_arc_parking_mutex_writable".to_string();
                println!("✅ chain_arc_parking_mutex_writable (write): Modified name");
            });
        
        // Verify the write
        Container::parking_mutex_data_r()
            .chain_arc_parking_mutex(DataStruct::name_r())
            .get(&write_container, |value| {
                println!("   Verified: name = {}", value);
            });
        
        // Example 4: Write optional field through Arc<parking_lot::Mutex<T>>
        Container::parking_mutex_data_r()
            .chain_arc_parking_mutex_writable_optional(DataStruct::optional_value_fw())
            .get_mut(&write_container, |value| {
                *value = "Modified optional via parking_mutex".to_string();
                println!("✅ chain_arc_parking_mutex_writable_optional (write): Modified optional_value");
            });
        
        // Verify the write
        Container::parking_mutex_data_r()
            .chain_arc_parking_mutex_optional(DataStruct::optional_value_fr())
            .get(&write_container, |value| {
                println!("   Verified: optional_value = {}", value);
            });
        
        // ========== parking_lot::RwLock Examples ==========
        println!("\n=== Arc<parking_lot::RwLock<T>> Chains ===\n");
        
        // Example 5: Read through Arc<parking_lot::RwLock<T>>
        Container::parking_rwlock_data_r()
            .chain_arc_parking_rwlock(DataStruct::name_r())
            .get(&container, |value| {
                println!("✅ chain_arc_parking_rwlock (read): name = {}", value);
            });
        
        // Example 6: Read optional field through Arc<parking_lot::RwLock<T>>
        Container::parking_rwlock_data_r()
            .chain_arc_parking_rwlock_optional(DataStruct::optional_value_fr())
            .get(&container, |value| {
                println!("✅ chain_arc_parking_rwlock_optional (read): optional_value = {}", value);
            });
        
        // Example 7: Write through Arc<parking_lot::RwLock<T>>
        let rwlock_write_container = Container::new();
        Container::parking_rwlock_data_r()
            .chain_arc_parking_rwlock_writable(DataStruct::name_w())
            .get_mut(&rwlock_write_container, |value| {
                *value = "Modified via chain_arc_parking_rwlock_writable".to_string();
                println!("✅ chain_arc_parking_rwlock_writable (write): Modified name");
            });
        
        // Verify the write
        Container::parking_rwlock_data_r()
            .chain_arc_parking_rwlock(DataStruct::name_r())
            .get(&rwlock_write_container, |value| {
                println!("   Verified: name = {}", value);
            });
        
        // Example 8: Write optional field through Arc<parking_lot::RwLock<T>>
        Container::parking_rwlock_data_r()
            .chain_arc_parking_rwlock_writable_optional(DataStruct::optional_value_fw())
            .get_mut(&rwlock_write_container, |value| {
                *value = "Modified optional via parking_rwlock".to_string();
                println!("✅ chain_arc_parking_rwlock_writable_optional (write): Modified optional_value");
            });
        
        // Verify the write
        Container::parking_rwlock_data_r()
            .chain_arc_parking_rwlock_optional(DataStruct::optional_value_fr())
            .get(&rwlock_write_container, |value| {
                println!("   Verified: optional_value = {}", value);
            });
        
        // ========== Comparison: parking_lot vs std::sync ==========
        println!("\n=== Key Differences: parking_lot vs std::sync ===\n");
        println!("1. parking_lot locks NEVER fail - no Option return for lock operations");
        println!("2. parking_lot is faster - no poisoning overhead");
        println!("3. Same functional keypath chain pattern works for both!");
        
        println!("\n=== All parking_lot chain examples completed successfully! ===");
    }
}

#[cfg(feature = "parking_lot")]
fn main() {
    parking_lot_example::run();
}

#[cfg(not(feature = "parking_lot"))]
fn main() {
    // This will never run due to compile_error! above
    eprintln!("This example requires the 'parking_lot' feature.");
    eprintln!("Run with: cargo run --example parking_lot_chains --features parking_lot");
}
