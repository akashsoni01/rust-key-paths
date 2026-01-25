//! Simple example demonstrating Tokio RwLock with keypaths
//! 
//! Run with: cargo run --example tokio_simple --features tokio
//! 
//! This example shows how to:
//! 1. Use derive-generated keypath methods for lock fields
//! 2. Read and write through a single Arc<tokio::sync::RwLock<T>>
//! 3. Chain keypaths through the lock to access inner fields (async)

// #[cfg(not(feature = "tokio"))]
// compile_error!("This example requires the 'tokio' feature. Run with: cargo run --example tokio_simple --features tokio");

#[cfg(feature = "tokio")]
mod example {
    use std::sync::Arc;
    use keypaths_proc::Kp;

    #[derive(Kp)]
    #[All]  // Generate all methods (readable, writable, owned)
    pub struct AppState {
        pub user_data: Arc<tokio::sync::RwLock<UserData>>,
    }

    impl Clone for AppState {
        fn clone(&self) -> Self {
            panic!("AppState should not be cloned!")
        }
    }

    #[derive(Kp)]
    #[All]  // Generate all methods (readable, writable, owned)
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

    pub async fn run() {
        println!("=== Simple Tokio RwLock Example ===\n");

        let state = AppState {
            user_data: Arc::new(tokio::sync::RwLock::new(UserData {
                name: String::from("Akash"),
                age: 30,
                email: Some(String::from("alice@example.com")),
            })),
        };

        println!("Initial state:");
        {
            let guard = state.user_data.read().await;
            println!("  name = {:?}", guard.name);
            println!("  age = {:?}", guard.age);
            println!("  email = {:?}", guard.email);
        }

        println!("\n=== Using Generated Keypath Methods ===");
        println!("For Arc<tokio::sync::RwLock<T>> fields, the macro generates:");
        println!("  • _r()      -> KeyPath<Struct, Arc<RwLock<T>>>  (readable)");
        println!("  • _w()      -> WritableKeyPath<Struct, Arc<RwLock<T>>>  (writable)");
        println!("  • _fr_at()  -> Chain through lock for reading (tokio, async)");
        println!("  • _fw_at()  -> Chain through lock for writing (tokio, async)\n");

        // ============================================================
        // READING THROUGH THE LOCK (async)
        // ============================================================
        println!("--- Reading through lock (async) ---");

        // Read name field through the lock
        AppState::user_data_fr_at(UserData::name_r())
            .get(&state, |name| {
                println!("✅ Read name via keypath: {:?}", name);
            })
            .await;

        // Read age field through the lock
        AppState::user_data_fr_at(UserData::age_r())
            .get(&state, |age| {
                println!("✅ Read age via keypath: {:?}", age);
            })
            .await;

        // Read optional email field through the lock
        AppState::user_data_fr_at(UserData::email_r())
            .get(&state, |email| {
                println!("✅ Read email via keypath: {:?}", email);
            })
            .await;

        // ============================================================
        // WRITING THROUGH THE LOCK (async)
        // ============================================================
        println!("\n--- Writing through lock (async) ---");

        // Write to name field
        AppState::user_data_fw_at(UserData::name_w())
            .get_mut(&state, |name| {
                *name = String::from("Bob");
                println!("✅ Updated name to: {:?}", name);
            })
            .await;

        // Write to age field
        AppState::user_data_fw_at(UserData::age_w())
            .get_mut(&state, |age| {
                *age = 35;
                println!("✅ Updated age to: {:?}", age);
            })
            .await;

        // Write to optional email field
        AppState::user_data_fw_at(UserData::email_w())
            .get_mut(&state, |email| {
                *email = Some(String::from("bob@example.com"));
                println!("✅ Updated email to: {:?}", email);
            })
            .await;

        // ============================================================
        // VERIFY THE WRITES
        // ============================================================
        println!("\n--- Final state ---");
        {
            let guard = state.user_data.read().await;
            println!("  name = {:?}", guard.name);
            println!("  age = {:?}", guard.age);
            println!("  email = {:?}", guard.email);
        }

        println!("\n=== Key Takeaways ===");
        println!("✅ No cloning occurred - all access was zero-copy!");
        println!("✅ Used derive-generated keypaths: AppState::user_data_fr_at(), AppState::user_data_fw_at()");
        println!("✅ Chained through Arc<tokio::sync::RwLock<T>> safely");
        println!("✅ All operations are async and must be awaited");
        println!("✅ Works with Tokio for async runtime compatibility");
    }
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() {
    example::run().await;
}

#[cfg(not(feature = "tokio"))]
fn main() {
    eprintln!("This example requires the 'tokio' feature.");
    eprintln!("Run with: cargo run --example tokio_simple --features tokio");
}

