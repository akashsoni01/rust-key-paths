//! Example demonstrating Tokio keypath chains with Arc<tokio::sync::Mutex<T>> and Arc<tokio::sync::RwLock<T>>
//!
//! This example shows how to use keypaths with Tokio's async synchronization primitives.
//! Tokio locks are async, so all operations must be awaited.
/// cargo run --example tokio_containers 2>&1

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath};
use keypaths_proc::Keypaths;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Keypaths, Debug)]
#[All]  // Generate all methods (readable, writable, owned)
struct AppState {
    user_data: Arc<tokio::sync::Mutex<UserData>>,
    config: Arc<tokio::sync::RwLock<Config>>,
    optional_cache: Option<Arc<tokio::sync::RwLock<Cache>>>,
    optional_mutex_cache: Option<Arc<tokio::sync::Mutex<Cache>>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        panic!("AppState::clone() should never be called")
    }
}

#[derive(Keypaths, Debug)]
#[All]  // Generate all methods (readable, writable, owned)
struct UserData {
    name: String,
    email: String,
    settings: UserSettings,
}

impl Clone for UserData {
    fn clone(&self) -> Self {
        panic!("UserData::clone() should never be called")
    }
}

#[derive(Keypaths, Debug)]
#[All]  // Generate all methods (readable, writable, owned)
struct UserSettings {
    theme: String,
    notifications: bool,
}

impl Clone for UserSettings {
    fn clone(&self) -> Self {
        panic!("UserSettings::clone() should never be called")
    }
}

#[derive(Keypaths, Debug)]
#[All]  // Generate all methods (readable, writable, owned)
struct Config {
    api_key: String,
    timeout: u64,
    features: FeatureFlags,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        panic!("Config::clone() should never be called")
    }
}

#[derive(Keypaths, Debug)]
#[All]  // Generate all methods (readable, writable, owned)
struct FeatureFlags {
    enable_logging: bool,
    enable_metrics: bool,
}

impl Clone for FeatureFlags {
    fn clone(&self) -> Self {
        panic!("FeatureFlags::clone() should never be called")
    }
}

#[derive(Keypaths, Debug)]
#[All]  // Generate all methods (readable, writable, owned)
struct Cache {
    entries: Vec<String>,
    size: usize,
}

impl Clone for Cache {
    fn clone(&self) -> Self {
        panic!("Cache::clone() should never be called")
    }
}

#[tokio::main]
async fn main() {
    println!("=== Tokio Keypath Chains Example ===\n");

    // Create initial state
    let state = AppState {
        user_data: Arc::new(tokio::sync::Mutex::new(UserData {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            settings: UserSettings {
                theme: "dark".to_string(),
                notifications: true,
            },
        })),
        config: Arc::new(tokio::sync::RwLock::new(Config {
            api_key: "secret-key-123".to_string(),
            timeout: 30,
            features: FeatureFlags {
                enable_logging: true,
                enable_metrics: false,
            },
        })),
        optional_cache: Some(Arc::new(tokio::sync::RwLock::new(Cache {
            entries: vec!["entry1".to_string(), "entry2".to_string()],
            size: 2,
        }))),
        optional_mutex_cache: Some(Arc::new(tokio::sync::Mutex::new(Cache {
            entries: vec!["mutex_entry1".to_string(), "mutex_entry2".to_string(), "mutex_entry3".to_string()],
            size: 3,
        }))),
    };

    // Example 1: Reading through Arc<tokio::sync::Mutex<T>> using proc macro _fr_at() method
    println!("1. Reading through Arc<tokio::sync::Mutex<T>> using proc macro:");
    AppState::user_data_fr_at(UserData::name_r())
        .get(&state, |name| {
            println!("   User name: {}", name);
        })
        .await;

    // Example 2: Reading nested fields through Arc<tokio::sync::Mutex<T>> using proc macro
    println!("\n2. Reading nested fields through Arc<tokio::sync::Mutex<T>> using proc macro:");
    AppState::user_data_fr_at(UserData::settings_r())
        .then(UserSettings::theme_r())
        .get(&state, |theme| {
            println!("   User theme: {}", theme);
        })
        .await;

    // Example 3: Writing through Arc<tokio::sync::Mutex<T>> using proc macro _fw_at() method
    println!("\n3. Writing through Arc<tokio::sync::Mutex<T>> using proc macro:");
    AppState::user_data_fw_at(UserData::name_w())
        .get_mut(&state, |name| {
            *name = "Bob".to_string();
            println!("   Updated user name to: {}", name);
        })
        .await;

    // Verify the change
    AppState::user_data_fr_at(UserData::name_r())
        .get(&state, |name| {
            println!("   Verified name is now: {}", name);
        })
        .await;

    // Example 4: Reading through Arc<tokio::sync::RwLock<T>> (read lock) using proc macro
    println!("\n4. Reading through Arc<tokio::sync::RwLock<T>> (read lock) using proc macro:");
    AppState::config_fr_at(Config::api_key_r())
        .get(&state, |api_key| {
            println!("   API key: {}", api_key);
        })
        .await;

    // Example 5: Reading nested fields through Arc<tokio::sync::RwLock<T>> using proc macro
    println!("\n5. Reading nested fields through Arc<tokio::sync::RwLock<T>> using proc macro:");
    AppState::config_fr_at(Config::features_r())
        .then(FeatureFlags::enable_logging_r())
        .get(&state, |enable_logging| {
            println!("   Enable logging: {}", enable_logging);
        })
        .await;

    // Example 6: Writing through Arc<tokio::sync::RwLock<T>> (write lock) using proc macro
    println!("\n6. Writing through Arc<tokio::sync::RwLock<T>> (write lock) using proc macro:");
    AppState::config_fw_at(Config::timeout_w())
        .get_mut(&state, |timeout| {
            *timeout = 60;
            println!("   Updated timeout to: {}", timeout);
        })
        .await;

    // Verify the change
    AppState::config_fr_at(Config::timeout_r())
        .get(&state, |timeout| {
            println!("   Verified timeout is now: {}", timeout);
        })
        .await;

    // Example 7: Reading through optional Arc<tokio::sync::RwLock<T>> using proc macro
    println!("\n7. Reading through optional Arc<tokio::sync::RwLock<T>> using proc macro:");
    if let Some(()) = AppState::optional_cache_fr()
        .then_arc_tokio_rwlock_at_kp(Cache::size_r())
        .get(&state, |size| {
            println!("   Cache size: {}", size);
        })
        .await
    {
        println!("   Successfully read cache size");
    } else {
        println!("   Cache is None");
    }

    // Example 8: Writing through optional Arc<tokio::sync::RwLock<T>> using proc macro
    println!("\n8. Writing through optional Arc<tokio::sync::RwLock<T>> using proc macro:");
    if let Some(()) = AppState::optional_cache_fr()
        .then_arc_tokio_rwlock_writable_at_kp(Cache::size_w())
        .get_mut(&state, |size| {
            *size = 100;
            println!("   Updated cache size to: {}", size);
        })
        .await
    {
        println!("   Successfully updated cache size");
    } else {
        println!("   Cache is None");
    }

    // Example 9: Chaining multiple levels through Tokio Mutex using proc macro
    println!("\n9. Chaining multiple levels through Tokio Mutex using proc macro:");
    AppState::user_data_fr_at(UserData::settings_r())
        .then(UserSettings::notifications_r())
        .get(&state, |notifications| {
            println!("   Notifications enabled: {}", notifications);
        })
        .await;

    // Example 10: Chaining multiple levels through Tokio RwLock using proc macro
    println!("\n10. Chaining multiple levels through Tokio RwLock using proc macro:");
    AppState::config_fr_at(Config::features_r())
        .then(FeatureFlags::enable_metrics_r())
        .get(&state, |enable_metrics| {
            println!("   Enable metrics: {}", enable_metrics);
        })
        .await;

    // Example 11: Writing nested fields through Tokio RwLock using proc macro
    println!("\n11. Writing nested fields through Tokio RwLock using proc macro:");
    AppState::config_fw_at(Config::features_w())
        .then(FeatureFlags::enable_metrics_w())
        .get_mut(&state, |enable_metrics| {
            *enable_metrics = true;
            println!("   Updated enable_metrics to: {}", enable_metrics);
        })
        .await;

    // Verify the change
    AppState::config_fr_at(Config::features_r())
        .then(FeatureFlags::enable_metrics_r())
        .get(&state, |enable_metrics| {
            println!("   Verified enable_metrics is now: {}", enable_metrics);
        })
        .await;

    // Example 12: Reading through optional Arc<tokio::sync::Mutex<T>> using proc macro
    println!("\n12. Reading through optional Arc<tokio::sync::Mutex<T>> using proc macro:");
    if let Some(()) = AppState::optional_mutex_cache_fr()
        .then_arc_tokio_mutex_at_kp(Cache::size_r())
        .get(&state, |size| {
            println!("   Mutex cache size: {}", size);
        })
        .await
    {
        println!("   Successfully read mutex cache size");
    } else {
        println!("   Mutex cache is None");
    }

    // Example 13: Writing through optional Arc<tokio::sync::Mutex<T>> using proc macro
    println!("\n13. Writing through optional Arc<tokio::sync::Mutex<T>> using proc macro:");
    if let Some(()) = AppState::optional_mutex_cache_fr()
        .then_arc_tokio_mutex_writable_at_kp(Cache::size_w())
        .get_mut(&state, |size| {
            *size = 200;
            println!("   Updated mutex cache size to: {}", size);
        })
        .await
    {
        println!("   Successfully updated mutex cache size");
    } else {
        println!("   Mutex cache is None");
    }

    // Example 14: Reading nested fields through optional Arc<tokio::sync::Mutex<T>>
    println!("\n14. Reading nested fields through optional Arc<tokio::sync::Mutex<T>>:");
    if let Some(()) = AppState::optional_mutex_cache_fr()
        .then_arc_tokio_mutex_at_kp(Cache::entries_r())
        .get(&state, |entries| {
            println!("   Mutex cache entries count: {}", entries.len());
            if let Some(first) = entries.first() {
                println!("   First entry: {}", first);
            }
        })
        .await
    {
        println!("   Successfully read mutex cache entries");
    } else {
        println!("   Mutex cache is None");
    }

    println!("\n=== Example Complete ===");
}

