use keypaths_proc::Keypaths;
use std::sync::{Mutex, RwLock, Arc};

// Level 1: Inner struct with simple fields
#[derive(Debug, Clone, Keypaths)]
#[All]
struct UserData {
    name: String,
    age: u32,
    email: String,
}

// Level 2: Struct containing Mutex and RwLock
#[derive(Debug, Keypaths)]
#[All]
struct UserProfile {
    data: Mutex<UserData>,
    preferences: RwLock<Vec<String>>,
    metadata: Arc<Mutex<HashMap<String, String>>>,
}

// Level 3: Container struct
#[derive(Debug, Keypaths)]
#[All]
struct UserAccount {
    profile: Option<UserProfile>,
    account_id: u64,
}

// Level 4: Top-level struct
#[derive(Debug, Keypaths)]
#[All]
struct ApplicationState {
    user: Option<UserAccount>,
    system_config: Arc<RwLock<SystemConfig>>,
}

#[derive(Debug, Clone, Keypaths)]
#[All]
struct SystemConfig {
    theme: String,
    language: String,
}

use std::collections::HashMap;

fn main() {
    println!("=== KeyPaths for Locks Example ===\n");

    // Create a multi-level structure with locks
    let mut app_state = ApplicationState {
        user: Some(UserAccount {
            profile: Some(UserProfile {
                data: Mutex::new(UserData {
                    name: "Alice".to_string(),
                    age: 30,
                    email: "alice@example.com".to_string(),
                }),
                preferences: RwLock::new(vec!["dark_mode".to_string(), "notifications".to_string()]),
                metadata: Arc::new(Mutex::new({
                    let mut map = HashMap::new();
                    map.insert("created".to_string(), "2024-01-01".to_string());
                    map.insert("last_login".to_string(), "2024-12-01".to_string());
                    map
                })),
            }),
            account_id: 12345,
        }),
        system_config: Arc::new(RwLock::new(SystemConfig {
            theme: "dark".to_string(),
            language: "en".to_string(),
        })),
    };

    // ==========================================
    // Example 1: Reading from Mutex with keypath composition
    // ==========================================
    println!("1. Reading user name from Mutex<UserData>:");
    
    // Chain through Option types using failable keypaths
    // ApplicationState -> Option<UserAccount> -> Option<UserProfile> -> Mutex<UserData> -> name
    if let Some(user_profile) = ApplicationState::user_fr()
        .then(UserAccount::profile_fr())
        .get(&app_state)
    {
        // Create keypath to access name field
        let name_kp = UserData::name_r();
        
        // Use the helper method to get cloned value from Mutex
        let get_name = UserProfile::data_mutex_fr_at(name_kp);
        if let Some(name) = get_name(&user_profile.data) {
            println!("   User name: {}", name);
        }
    }

    // ==========================================
    // Example 2: Reading preferences from RwLock<Vec<String>>
    // ==========================================
    println!("\n2. Reading preferences from RwLock<Vec<String>>:");
    
    // Chain through Option types to get to UserProfile
    if let Some(user_profile) = ApplicationState::user_fr()
        .then(UserAccount::profile_fr())
        .get(&app_state)
    {
        // Create keypath to access the entire Vec (which is Clone)
        let vec_kp = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
        
        // Use the helper method to get cloned value from RwLock
        let get_prefs = UserProfile::preferences_rwlock_fr_at(vec_kp);
        if let Some(prefs) = get_prefs(&user_profile.preferences) {
            println!("   All preferences: {:?}", prefs);
            if let Some(first) = prefs.first() {
                println!("   First preference: {}", first);
            }
        }
    }

    // ==========================================
    // Example 3: Writing to Mutex with closure
    // ==========================================
    println!("\n3. Updating user age in Mutex<UserData>:");
    
    // Chain through Option types using failable writable keypaths
    if let Some(user_profile) = ApplicationState::user_fw()
        .then(UserAccount::profile_fw())
        .get_mut(&mut app_state)
    {
        // Create writable keypath to age field
        let age_kp = UserData::age_w();
        
        // Use the helper method to update value via closure
        let update_age = UserProfile::data_mutex_fw_at(age_kp, |age: &mut u32| {
            *age += 1;
            println!("   Updated age to: {}", *age);
        });
        
        if update_age(&user_profile.data).is_some() {
            // Verify the update
            let age_kp_read = UserData::age_r();
            let get_age = UserProfile::data_mutex_fr_at(age_kp_read);
            if let Some(age) = get_age(&user_profile.data) {
                println!("   Verified age: {}", age);
            }
        }
    }

    // ==========================================
    // Example 4: Writing to RwLock with closure
    // ==========================================
    println!("\n4. Adding preference to RwLock<Vec<String>>:");
    
    // Chain through Option types to get mutable access to UserProfile
    if let Some(user_profile) = ApplicationState::user_fw()
        .then(UserAccount::profile_fw())
        .get_mut(&mut app_state)
    {
        // Create writable keypath to the Vec
        let vec_kp = rust_keypaths::WritableKeyPath::new(|v: &mut Vec<String>| v);
        
        // Use the helper method to update via closure
        let add_preference = UserProfile::preferences_rwlock_fw_at(vec_kp, |prefs: &mut Vec<String>| {
            prefs.push("accessibility".to_string());
            println!("   Added new preference");
        });
        
        if add_preference(&user_profile.preferences).is_some() {
            // Verify the update
            let vec_kp_read = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
            let get_prefs = UserProfile::preferences_rwlock_fr_at(vec_kp_read);
            if let Some(prefs) = get_prefs(&user_profile.preferences) {
                println!("   All preferences: {:?}", prefs);
            }
        }
    }

    // ==========================================
    // Example 5: Working with Arc<Mutex<T>>
    // ==========================================
    println!("\n5. Reading from Arc<Mutex<HashMap>>:");
    
    // Chain through Option types to get to UserProfile
    if let Some(user_profile) = ApplicationState::user_fr()
        .then(UserAccount::profile_fr())
        .get(&app_state)
    {
        // Create keypath to get a value from HashMap
        // We'll get the entire HashMap and then extract the value
        let map_kp = rust_keypaths::KeyPath::new(|m: &HashMap<String, String>| m);
        
        // Use the helper method for Arc<Mutex<T>> to get the HashMap
        let get_map = UserProfile::metadata_arc_mutex_fr_at(map_kp);
        if let Some(map) = get_map(&user_profile.metadata) {
            if let Some(last_login) = map.get("last_login") {
                println!("   Last login: {}", last_login);
            }
        }
    }

    // ==========================================
    // Example 6: Working with Arc<RwLock<T>>
    // ==========================================
    println!("\n6. Reading and writing to Arc<RwLock<SystemConfig>>:");
    
    // Read theme
    let theme_kp = SystemConfig::theme_r();
    let get_theme = ApplicationState::system_config_arc_rwlock_fr_at(theme_kp);
    if let Some(theme) = get_theme(&app_state.system_config) {
        println!("   Current theme: {}", theme);
    }
    
    // Update language
    let language_kp = SystemConfig::language_w();
    let update_language = ApplicationState::system_config_arc_rwlock_fw_at(language_kp, |lang: &mut String| {
        *lang = "fr".to_string();
        println!("   Updated language to: {}", *lang);
    });
    
    if update_language(&app_state.system_config).is_some() {
        // Verify the update
        let language_kp_read = SystemConfig::language_r();
        let get_language = ApplicationState::system_config_arc_rwlock_fr_at(language_kp_read);
        if let Some(lang) = get_language(&app_state.system_config) {
            println!("   Verified language: {}", lang);
        }
    }

    // ==========================================
    // Example 7: Deep multi-level access through Option chains
    // ==========================================
    println!("\n7. Deep multi-level access through Option chains:");
    
    // Access nested fields through multiple Option levels using keypath chaining
    // ApplicationState -> Option<UserAccount> -> Option<UserProfile> -> Mutex<UserData> -> email
    if let Some(user_profile) = ApplicationState::user_fr()
        .then(UserAccount::profile_fr())
        .get(&app_state)
    {
        let email_kp = UserData::email_r();
        let get_email = UserProfile::data_mutex_fr_at(email_kp);
        if let Some(email) = get_email(&user_profile.data) {
            println!("   User email (deep access via keypath chain): {}", email);
        }
    }

    // ==========================================
    // Example 8: Complex update with multiple fields
    // ==========================================
    println!("\n8. Complex update with multiple fields:");
    
    // Chain through Option types to get mutable access
    if let Some(user_profile) = ApplicationState::user_fw()
        .then(UserAccount::profile_fw())
        .get_mut(&mut app_state)
    {
        // Update multiple fields in a single lock acquisition
        let name_kp = UserData::name_w();
        let update_name = UserProfile::data_mutex_fw_at(name_kp, |name: &mut String| {
            *name = "Alice Updated".to_string();
            println!("   Updated name to: {}", *name);
        });
        
        if update_name(&user_profile.data).is_some() {
            // Then update age in a separate operation
            let age_kp = UserData::age_w();
            let update_age = UserProfile::data_mutex_fw_at(age_kp, |age: &mut u32| {
                *age = 31;
                println!("   Updated age to: {}", *age);
            });
            
            if update_age(&user_profile.data).is_some() {
                // Read both back to verify
                let name_kp_read = UserData::name_r();
                let age_kp_read = UserData::age_r();
                let get_name = UserProfile::data_mutex_fr_at(name_kp_read);
                let get_age = UserProfile::data_mutex_fr_at(age_kp_read);
                
                if let (Some(name), Some(age)) = (get_name(&user_profile.data), get_age(&user_profile.data)) {
                    println!("   Verified - Name: {}, Age: {}", name, age);
                }
            }
        }
    }

    // ==========================================
    // Example 9: Working with collections inside locks
    // ==========================================
    println!("\n9. Working with collections inside locks:");
    
    // Chain through Option types
    if let Some(user_profile) = ApplicationState::user_fw()
        .then(UserAccount::profile_fw())
        .get_mut(&mut app_state)
    {
        // Read all preferences
        let vec_kp = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
        let get_prefs = UserProfile::preferences_rwlock_fr_at(vec_kp);
        if let Some(prefs) = get_prefs(&user_profile.preferences) {
            println!("   Current preferences count: {}", prefs.len());
        }
        
        // Modify the collection
        let vec_kp_mut = rust_keypaths::WritableKeyPath::new(|v: &mut Vec<String>| v);
        let modify_prefs = UserProfile::preferences_rwlock_fw_at(vec_kp_mut, |prefs: &mut Vec<String>| {
            prefs.retain(|p| p != "notifications");
            prefs.push("high_contrast".to_string());
            println!("   Modified preferences list");
        });
        
        if modify_prefs(&user_profile.preferences).is_some() {
            // Create a new keypath for reading after modification
            let vec_kp_after = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
            let get_prefs_after = UserProfile::preferences_rwlock_fr_at(vec_kp_after);
            if let Some(prefs) = get_prefs_after(&user_profile.preferences) {
                println!("   Updated preferences: {:?}", prefs);
            }
        }
    }

    // ==========================================
    // Example 10: Concurrent-safe access patterns
    // ==========================================
    println!("\n10. Concurrent-safe access patterns:");
    
    // Demonstrate that locks are properly acquired and released
    // Chain through Option types using keypaths
    if let Some(user_profile) = ApplicationState::user_fr()
        .then(UserAccount::profile_fr())
        .get(&app_state)
    {
        // Multiple read operations can happen (RwLock allows concurrent reads)
        let name_kp = UserData::name_r();
        let email_kp = UserData::email_r();
        
        let get_name = UserProfile::data_mutex_fr_at(name_kp);
        let get_email = UserProfile::data_mutex_fr_at(email_kp);
        
        // These would work in parallel in a real concurrent scenario
        // Each lock acquisition is independent and safe
        if let Some(name) = get_name(&user_profile.data) {
            println!("   Read name: {}", name);
        }
        if let Some(email) = get_email(&user_profile.data) {
            println!("   Read email: {}", email);
        }
    }

    // ==========================================
    // Example 11: Error handling with lock acquisition
    // ==========================================
    println!("\n11. Error handling with lock acquisition:");
    
    // Chain through Option types - if any is None, the chain short-circuits
    if let Some(user_profile) = ApplicationState::user_fr()
        .then(UserAccount::profile_fr())
        .get(&app_state)
    {
        let name_kp = UserData::name_r();
        let get_name = UserProfile::data_mutex_fr_at(name_kp);
        
        // The helper methods return Option, handling lock acquisition failures gracefully
        match get_name(&user_profile.data) {
            Some(name) => println!("   Successfully acquired lock and read name: {}", name),
            None => println!("   Failed to acquire lock (would happen if lock was poisoned)"),
        }
    } else {
        println!("   Keypath chain short-circuited (user or profile is None)");
    }

    // ==========================================
    // Example 12: Composition with .then() for nested structures
    // ==========================================
    println!("\n12. Composition pattern for nested structures:");
    
    // This demonstrates how you can compose keypaths before using them with locks
    // Chain through Option types using keypath composition
    if let Some(user_profile) = ApplicationState::user_fr()
        .then(UserAccount::profile_fr())
        .get(&app_state)
    {
        // Create a keypath that accesses a nested field
        // Then use it with the lock helper
        let name_kp = UserData::name_r();
        
        // The helper method accepts any keypath that works with the inner type
        let get_name = UserProfile::data_mutex_fr_at(name_kp);
        
        if let Some(name) = get_name(&user_profile.data) {
            println!("   Composed keypath result: {}", name);
        }
        
        // You can also create keypaths on-the-fly
        let custom_kp = rust_keypaths::KeyPath::new(|data: &UserData| &data.email);
        let get_custom = UserProfile::data_mutex_fr_at(custom_kp);
        if let Some(email) = get_custom(&user_profile.data) {
            println!("   Custom keypath result: {}", email);
        }
    }

    // ==========================================
    // Example 13: Real-world scenario - User profile update
    // ==========================================
    println!("\n13. Real-world scenario - Complete user profile update:");
    
    // Chain through Option types to get mutable access
    if let Some(user_profile) = ApplicationState::user_fw()
        .then(UserAccount::profile_fw())
        .get_mut(&mut app_state)
    {
        println!("   Performing complete profile update...");
        
        // Update user data
        let name_kp = UserData::name_w();
        let update_name = UserProfile::data_mutex_fw_at(name_kp, |name: &mut String| {
            *name = "Alice Smith".to_string();
        });
        update_name(&user_profile.data);
        
        let age_kp = UserData::age_w();
        let update_age = UserProfile::data_mutex_fw_at(age_kp, |age: &mut u32| {
            *age = 32;
        });
        update_age(&user_profile.data);
        
        // Update preferences
        let prefs_kp = rust_keypaths::WritableKeyPath::new(|v: &mut Vec<String>| v);
        let update_prefs = UserProfile::preferences_rwlock_fw_at(prefs_kp, |prefs: &mut Vec<String>| {
            prefs.clear();
            prefs.extend(vec!["dark_mode".to_string(), "compact_view".to_string()]);
        });
        update_prefs(&user_profile.preferences);
        
        // Update metadata
        let metadata_kp = rust_keypaths::WritableKeyPath::new(|m: &mut HashMap<String, String>| m);
        let update_metadata = UserProfile::metadata_arc_mutex_fw_at(metadata_kp, |meta: &mut HashMap<String, String>| {
            meta.insert("last_updated".to_string(), "2024-12-15".to_string());
        });
        update_metadata(&user_profile.metadata);
        
        println!("   Profile update complete!");
        
        // Verify all updates
        let name_kp_read = UserData::name_r();
        let age_kp_read = UserData::age_r();
        let get_name = UserProfile::data_mutex_fr_at(name_kp_read);
        let get_age = UserProfile::data_mutex_fr_at(age_kp_read);
        
        if let (Some(name), Some(age)) = (get_name(&user_profile.data), get_age(&user_profile.data)) {
            println!("   Final state - Name: {}, Age: {}", name, age);
        }
    }

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("1. Use keypath chaining (.then()) to traverse Option types instead of manual if-let chains");
    println!("2. Helper methods (_mutex_fr_at, _rwlock_fr_at, etc.) safely acquire locks");
    println!("3. Read operations return cloned values (no lifetime issues)");
    println!("4. Write operations use closures for safe mutation");
    println!("5. All lock types (Mutex, RwLock, Arc<Mutex>, Arc<RwLock>) are supported");
    println!("6. Methods return Option to handle lock acquisition failures");
    println!("7. Keypath chaining works seamlessly with lock helper methods");
    println!("8. Pattern: chain through Options with .then(), then use lock helpers on the result");
}

