//! Example demonstrating parking_lot::Mutex and parking_lot::RwLock functional keypath chains
//! 
//! Run with: cargo run --example parking_lot_chains --features parking_lot

#[cfg(feature = "parking_lot")]
mod parking_lot_example {
    use keypaths_proc::{Keypaths, WritableKeypaths, Casepaths};
    use rust_keypaths::{KeyPath, OptionalKeyPath, keypath, opt_keypath};
    use std::sync::Arc;
    use parking_lot::{Mutex as ParkingMutex, RwLock as ParkingRwLock};

    // ========== Data Structures ==========
    
    #[derive(Debug)]
    struct Container {
        // Direct Arc<parking_lot::*> fields (for KeyPath chains)
        parking_mutex_data: Arc<ParkingMutex<DataStruct>>,
        parking_rwlock_data: Arc<ParkingRwLock<DataStruct>>,
        // Optional Arc<parking_lot::*> fields (for OptionalKeyPath chains)
        optional_mutex: Option<Arc<ParkingMutex<DataStruct>>>,
        optional_rwlock: Option<Arc<ParkingRwLock<DataStruct>>>,
        // Enum with Arc<parking_lot::*> case
        state: AppState,
    }

    #[derive(Debug, Casepaths)]
    enum AppState {
        Idle,
        Active(Arc<ParkingRwLock<Session>>),
    }

    #[derive(Debug, Keypaths, WritableKeypaths)]
    struct Session {
        user_name: String,
        logged_in: bool,
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
                optional_mutex: Some(Arc::new(ParkingMutex::new(DataStruct {
                    name: "Optional Mutex".to_string(),
                    optional_value: Some("Opt Mutex Value".to_string()),
                    count: 1,
                }))),
                optional_rwlock: Some(Arc::new(ParkingRwLock::new(DataStruct {
                    name: "Optional RwLock".to_string(),
                    optional_value: Some("Opt RwLock Value".to_string()),
                    count: 2,
                }))),
                state: AppState::Active(Arc::new(ParkingRwLock::new(Session {
                    user_name: "Alice".to_string(),
                    logged_in: true,
                }))),
            }
        }
        
        fn new_idle() -> Self {
            let mut c = Self::new();
            c.state = AppState::Idle;
            c
        }
        
        // Manual keypath methods for Arc<parking_lot::Mutex/RwLock> fields
        fn parking_mutex_data_r() -> KeyPath<Container, Arc<ParkingMutex<DataStruct>>, impl for<'r> Fn(&'r Container) -> &'r Arc<ParkingMutex<DataStruct>>> {
            keypath!(|c: &Container| &c.parking_mutex_data)
        }
        
        fn parking_rwlock_data_r() -> KeyPath<Container, Arc<ParkingRwLock<DataStruct>>, impl for<'r> Fn(&'r Container) -> &'r Arc<ParkingRwLock<DataStruct>>> {
            keypath!(|c: &Container| &c.parking_rwlock_data)
        }
        
        // OptionalKeyPath methods for Option<Arc<parking_lot::*>> fields
        fn optional_mutex_fr() -> OptionalKeyPath<Container, Arc<ParkingMutex<DataStruct>>, impl for<'r> Fn(&'r Container) -> Option<&'r Arc<ParkingMutex<DataStruct>>>> {
            opt_keypath!(|c: &Container| c.optional_mutex.as_ref())
        }
        
        fn optional_rwlock_fr() -> OptionalKeyPath<Container, Arc<ParkingRwLock<DataStruct>>, impl for<'r> Fn(&'r Container) -> Option<&'r Arc<ParkingRwLock<DataStruct>>>> {
            opt_keypath!(|c: &Container| c.optional_rwlock.as_ref())
        }
        
        // OptionalKeyPath for enum -> Arc<parking_lot::RwLock<T>> chain
        fn state_fr() -> OptionalKeyPath<Container, AppState, impl for<'r> Fn(&'r Container) -> Option<&'r AppState>> {
            opt_keypath!(|c: &Container| Some(&c.state))
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
        
        // ========== PART 1: KeyPath chains (direct Arc fields) ==========
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("PART 1: KeyPath chains (direct Arc<parking_lot::*> fields)");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        // ========== parking_lot::Mutex Examples ==========
        println!("=== Arc<parking_lot::Mutex<T>> Chains ===\n");
        
        // Example 1: Read through Arc<parking_lot::Mutex<T>>
        Container::parking_mutex_data_r()
            .then_arc_parking_mutex_at_kp(DataStruct::name_r())
            .get(&container, |value| {
                println!("✅ then_arc_parking_mutex_at_kp (read): name = {}", value);
            });
        
        // Example 2: Read optional field through Arc<parking_lot::Mutex<T>>
        Container::parking_mutex_data_r()
            .then_arc_parking_mutex_optional_at_kp(DataStruct::optional_value_fr())
            .get(&container, |value| {
                println!("✅ then_arc_parking_mutex_optional_at_kp (read): optional_value = {}", value);
            });
        
        // Example 3: Write through Arc<parking_lot::Mutex<T>>
        let write_container = Container::new();
        Container::parking_mutex_data_r()
            .then_arc_parking_mutex_writable_at_kp(DataStruct::name_w())
            .get_mut(&write_container, |value| {
                *value = "Modified via then_arc_parking_mutex_writable_at_kp".to_string();
                println!("✅ then_arc_parking_mutex_writable_at_kp (write): Modified name");
            });
        
        // Verify the write
        Container::parking_mutex_data_r()
            .then_arc_parking_mutex_at_kp(DataStruct::name_r())
            .get(&write_container, |value| {
                println!("   Verified: name = {}", value);
            });
        
        // Example 4: Write optional field through Arc<parking_lot::Mutex<T>>
        Container::parking_mutex_data_r()
            .then_arc_parking_mutex_writable_optional_at_kp(DataStruct::optional_value_fw())
            .get_mut(&write_container, |value| {
                *value = "Modified optional via parking_mutex".to_string();
                println!("✅ then_arc_parking_mutex_writable_optional_at_kp (write): Modified optional_value");
            });
        
        // Verify the write
        Container::parking_mutex_data_r()
            .then_arc_parking_mutex_optional_at_kp(DataStruct::optional_value_fr())
            .get(&write_container, |value| {
                println!("   Verified: optional_value = {}", value);
            });
        
        // ========== parking_lot::RwLock Examples ==========
        println!("\n=== Arc<parking_lot::RwLock<T>> Chains ===\n");
        
        // Example 5: Read through Arc<parking_lot::RwLock<T>>
        Container::parking_rwlock_data_r()
            .then_arc_parking_rwlock_at_kp(DataStruct::name_r())
            .get(&container, |value| {
                println!("✅ then_arc_parking_rwlock_at_kp (read): name = {}", value);
            });
        
        // Example 6: Read optional field through Arc<parking_lot::RwLock<T>>
        Container::parking_rwlock_data_r()
            .then_arc_parking_rwlock_optional_at_kp(DataStruct::optional_value_fr())
            .get(&container, |value| {
                println!("✅ then_arc_parking_rwlock_optional_at_kp (read): optional_value = {}", value);
            });
        
        // Example 7: Write through Arc<parking_lot::RwLock<T>>
        let rwlock_write_container = Container::new();
        Container::parking_rwlock_data_r()
            .then_arc_parking_rwlock_writable_at_kp(DataStruct::name_w())
            .get_mut(&rwlock_write_container, |value| {
                *value = "Modified via then_arc_parking_rwlock_writable_at_kp".to_string();
                println!("✅ then_arc_parking_rwlock_writable_at_kp (write): Modified name");
            });
        
        // Verify the write
        Container::parking_rwlock_data_r()
            .then_arc_parking_rwlock_at_kp(DataStruct::name_r())
            .get(&rwlock_write_container, |value| {
                println!("   Verified: name = {}", value);
            });
        
        // Example 8: Write optional field through Arc<parking_lot::RwLock<T>>
        Container::parking_rwlock_data_r()
            .then_arc_parking_rwlock_writable_optional_at_kp(DataStruct::optional_value_fw())
            .get_mut(&rwlock_write_container, |value| {
                *value = "Modified optional via parking_rwlock".to_string();
                println!("✅ then_arc_parking_rwlock_writable_optional_at_kp (write): Modified optional_value");
            });
        
        // Verify the write
        Container::parking_rwlock_data_r()
            .then_arc_parking_rwlock_optional_at_kp(DataStruct::optional_value_fr())
            .get(&rwlock_write_container, |value| {
                println!("   Verified: optional_value = {}", value);
            });
        
        // ========== PART 2: OptionalKeyPath chains (Option<Arc<*>> fields) ==========
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("PART 2: OptionalKeyPath chains (Option<Arc<parking_lot::*>> fields)");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        println!("=== OptionalKeyPath -> Arc<parking_lot::Mutex<T>> Chains ===\n");
        
        // Example 9: Read through Option<Arc<parking_lot::Mutex<T>>>
        Container::optional_mutex_fr()
            .then_arc_parking_mutex_at_kp(DataStruct::name_r())
            .get(&container, |value| {
                println!("✅ optional -> then_arc_parking_mutex_at_kp (read): name = {}", value);
            });
        
        // Example 10: Read optional field through Option<Arc<parking_lot::Mutex<T>>>
        Container::optional_mutex_fr()
            .then_arc_parking_mutex_optional_at_kp(DataStruct::optional_value_fr())
            .get(&container, |value| {
                println!("✅ optional -> then_arc_parking_mutex_optional_at_kp (read): optional_value = {}", value);
            });
        
        // Example 11: Write through Option<Arc<parking_lot::Mutex<T>>>
        let opt_write_container = Container::new();
        Container::optional_mutex_fr()
            .then_arc_parking_mutex_writable_at_kp(DataStruct::name_w())
            .get_mut(&opt_write_container, |value| {
                *value = "Modified via optional parking_mutex chain".to_string();
                println!("✅ optional -> then_arc_parking_mutex_writable_at_kp (write): Modified name");
            });
        
        // Verify the write
        Container::optional_mutex_fr()
            .then_arc_parking_mutex_at_kp(DataStruct::name_r())
            .get(&opt_write_container, |value| {
                println!("   Verified: name = {}", value);
            });
        
        // Example 12: Write optional field through Option<Arc<parking_lot::Mutex<T>>>
        Container::optional_mutex_fr()
            .then_arc_parking_mutex_writable_optional_at_kp(DataStruct::optional_value_fw())
            .get_mut(&opt_write_container, |value| {
                *value = "Modified optional via optional parking_mutex".to_string();
                println!("✅ optional -> then_arc_parking_mutex_writable_optional_at_kp (write): Modified optional_value");
            });
        
        // Verify the write
        Container::optional_mutex_fr()
            .then_arc_parking_mutex_optional_at_kp(DataStruct::optional_value_fr())
            .get(&opt_write_container, |value| {
                println!("   Verified: optional_value = {}", value);
            });
        
        println!("\n=== OptionalKeyPath -> Arc<parking_lot::RwLock<T>> Chains ===\n");
        
        // Example 13: Read through Option<Arc<parking_lot::RwLock<T>>>
        Container::optional_rwlock_fr()
            .then_arc_parking_rwlock_at_kp(DataStruct::name_r())
            .get(&container, |value| {
                println!("✅ optional -> then_arc_parking_rwlock_at_kp (read): name = {}", value);
            });
        
        // Example 14: Read optional field through Option<Arc<parking_lot::RwLock<T>>>
        Container::optional_rwlock_fr()
            .then_arc_parking_rwlock_optional_at_kp(DataStruct::optional_value_fr())
            .get(&container, |value| {
                println!("✅ optional -> then_arc_parking_rwlock_optional_at_kp (read): optional_value = {}", value);
            });
        
        // Example 15: Write through Option<Arc<parking_lot::RwLock<T>>>
        let opt_rwlock_write_container = Container::new();
        Container::optional_rwlock_fr()
            .then_arc_parking_rwlock_writable_at_kp(DataStruct::name_w())
            .get_mut(&opt_rwlock_write_container, |value| {
                *value = "Modified via optional parking_rwlock chain".to_string();
                println!("✅ optional -> then_arc_parking_rwlock_writable_at_kp (write): Modified name");
            });
        
        // Verify the write
        Container::optional_rwlock_fr()
            .then_arc_parking_rwlock_at_kp(DataStruct::name_r())
            .get(&opt_rwlock_write_container, |value| {
                println!("   Verified: name = {}", value);
            });
        
        // Example 16: Write optional field through Option<Arc<parking_lot::RwLock<T>>>
        Container::optional_rwlock_fr()
            .then_arc_parking_rwlock_writable_optional_at_kp(DataStruct::optional_value_fw())
            .get_mut(&opt_rwlock_write_container, |value| {
                *value = "Modified optional via optional parking_rwlock".to_string();
                println!("✅ optional -> then_arc_parking_rwlock_writable_optional_at_kp (write): Modified optional_value");
            });
        
        // Verify the write
        Container::optional_rwlock_fr()
            .then_arc_parking_rwlock_optional_at_kp(DataStruct::optional_value_fr())
            .get(&opt_rwlock_write_container, |value| {
                println!("   Verified: optional_value = {}", value);
            });
        
        // ========== PART 3: Enum -> Arc<parking_lot::RwLock<T>> chains ==========
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("PART 3: Enum case -> Arc<parking_lot::RwLock<T>> chains");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        // Example 17: Read through enum case -> Arc<parking_lot::RwLock<T>>
        Container::state_fr()
            .then(AppState::active_case_r())
            .then_arc_parking_rwlock_at_kp(Session::user_name_r())
            .get(&container, |value| {
                println!("✅ enum -> then_arc_parking_rwlock_at_kp (read): user_name = {}", value);
            });
        
        // Example 18: Write through enum case -> Arc<parking_lot::RwLock<T>>
        let enum_container = Container::new();
        Container::state_fr()
            .then(AppState::active_case_r())
            .then_arc_parking_rwlock_writable_at_kp(Session::user_name_w())
            .get_mut(&enum_container, |value| {
                *value = "Bob (Updated via enum chain)".to_string();
                println!("✅ enum -> then_arc_parking_rwlock_writable_at_kp (write): Modified user_name");
            });
        
        // Verify the write
        Container::state_fr()
            .then(AppState::active_case_r())
            .then_arc_parking_rwlock_at_kp(Session::user_name_r())
            .get(&enum_container, |value| {
                println!("   Verified: user_name = {}", value);
            });
        
        // Example 19: Non-matching enum variant returns None
        let idle_container = Container::new_idle();
        let result = Container::state_fr()
            .then(AppState::active_case_r())
            .then_arc_parking_rwlock_at_kp(Session::user_name_r())
            .get(&idle_container, |_| ());
        
        if result.is_none() {
            println!("✅ enum (Idle) -> None: Correctly returned None for non-matching variant");
        }
        
        // ========== Comparison: parking_lot vs std::sync ==========
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Key Differences: parking_lot vs std::sync");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        println!("1. parking_lot locks NEVER fail - no Option return for lock operations");
        println!("2. parking_lot is faster - no poisoning overhead");
        println!("3. Same functional keypath chain pattern works for both!");
        println!("4. Both KeyPath AND OptionalKeyPath have full chain support");
        
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
