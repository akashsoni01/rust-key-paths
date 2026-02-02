//! Simple example demonstrating parking_lot RwLock with keypaths
//!
//! Run with: cargo run --example parking_lot_simple --features parking_lot
//!
//! This example shows how to:
//! 1. Use derive-generated keypath methods for lock fields
//! 2. Read and write through a single Arc<parking_lot::RwLock<T>>
//! 3. Chain keypaths through the lock to access inner fields

#[cfg(not(feature = "parking_lot"))]
compile_error!(
    "This example requires the 'parking_lot' feature. Run with: cargo run --example parking_lot_simple --features parking_lot"
);

#[cfg(feature = "parking_lot")]
mod example {
    use keypaths_proc::Kp;
    use parking_lot::RwLock;
    use std::sync::Arc;

    #[derive(Kp)]
    #[All] // Generate all methods (readable, writable, owned)
    pub struct AppState {
        pub user_data: Arc<RwLock<UserData>>,
    }

    impl Clone for AppState {
        fn clone(&self) -> Self {
            panic!("AppState should not be cloned!")
        }
    }

    #[derive(Kp)]
    #[All] // Generate all methods (readable, writable, owned)
    pub struct UserData {
        pub name: String,
        pub age: u32,
        pub email: Option<String>,
    }

    impl Clone for UserData {
        fn clone(&self) -> Self {
            panic!("UserData should not be cloned!")
        }
    }

    pub fn run() {
        println!("=== Simple parking_lot RwLock Example ===\n");

        let state = AppState {
            user_data: Arc::new(RwLock::new(UserData {
                name: String::from("Akash"),
                age: 30,
                email: Some(String::from("alice@example.com")),
            })),
        };

        println!("Initial state:");
        println!("  name = {:?}", state.user_data.read().name);
        println!("  age = {:?}", state.user_data.read().age);
        println!("  email = {:?}", state.user_data.read().email);

        println!("\n=== Using Generated Keypath Methods ===");
        println!("For Arc<RwLock<T>> fields, the macro generates:");
        println!("  • _r()      -> KeyPath<Struct, Arc<RwLock<T>>>  (readable)");
        println!("  • _w()      -> WritableKeyPath<Struct, Arc<RwLock<T>>>  (writable)");
        println!("  • _fr_at()  -> Chain through lock for reading (parking_lot)");
        println!("  • _fw_at()  -> Chain through lock for writing (parking_lot)\n");

        // ============================================================
        // READING THROUGH THE LOCK
        // ============================================================
        println!("--- Reading through lock ---");

        // Read name field through the lock
        AppState::user_data_fr_at(UserData::name_r()).get(&state, |name| {
            println!("✅ Read name via keypath: {:?}", name);
        });

        // Read age field through the lock
        AppState::user_data_fr_at(UserData::age_r()).get(&state, |age| {
            println!("✅ Read age via keypath: {:?}", age);
        });

        // Read optional email field through the lock
        AppState::user_data_fr_at(UserData::email_r()).get(&state, |email| {
            println!("✅ Read email via keypath: {:?}", email);
        });

        // ============================================================
        // WRITING THROUGH THE LOCK
        // ============================================================
        println!("\n--- Writing through lock ---");

        // Write to name field
        AppState::user_data_fw_at(UserData::name_w()).get_mut(&state, |name| {
            *name = String::from("Bob");
            println!("✅ Updated name to: {:?}", name);
        });

        // Write to age field
        AppState::user_data_fw_at(UserData::age_w()).get_mut(&state, |age| {
            *age = 35;
            println!("✅ Updated age to: {:?}", age);
        });

        // Write to optional email field
        AppState::user_data_fw_at(UserData::email_w()).get_mut(&state, |email| {
            *email = Some(String::from("bob@example.com"));
            println!("✅ Updated email to: {:?}", email);
        });

        // ============================================================
        // VERIFY THE WRITES
        // ============================================================
        println!("\n--- Final state ---");
        println!("  name = {:?}", state.user_data.read().name);
        println!("  age = {:?}", state.user_data.read().age);
        println!("  email = {:?}", state.user_data.read().email);

        println!("\n=== Key Takeaways ===");
        println!("✅ No cloning occurred - all access was zero-copy!");
        println!(
            "✅ Used derive-generated keypaths: AppState::user_data_fr_at(), AppState::user_data_fw_at()"
        );
        println!("✅ Chained through Arc<RwLock<T>> safely");
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
    eprintln!("Run with: cargo run --example parking_lot_simple --features parking_lot");
}
