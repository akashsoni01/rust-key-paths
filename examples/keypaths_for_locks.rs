// use keypaths_proc::Keypaths;
// use std::sync::{Arc};

// // Level 1: Inner struct with simple fields
// #[derive(Debug, Clone, Keypaths)]
// #[All]
// struct UserData {
//     name: String,
//     age: u32,
//     email: String,
// }

// // Level 2: Struct containing Mutex and RwLock
// #[derive(Debug, Keypaths)]
// #[All]
// struct UserProfile {
//     data: std::sync::Mutex<UserData>,
//     preferences: std::sync::RwLock<Vec<String>>,
//     metadata: Arc<std::sync::Mutex<HashMap<String, String>>>,
// }

// // Level 3: Container struct
// #[derive(Debug, Keypaths)]
// #[All]
// struct UserAccount {
//     profile: Option<UserProfile>,
//     account_id: u64,
// }

// // Level 4: Top-level struct
// #[derive(Debug, Keypaths)]
// #[All]
// struct ApplicationState {
//     user: Option<UserAccount>,
//     system_config: Arc<std::sync::RwLock<SystemConfig>>,
// }

// #[derive(Debug, Clone, Keypaths)]
// #[All]
// struct SystemConfig {
//     theme: String,
//     language: String,
// }

// use std::collections::HashMap;

// fn main() {
//     println!("=== KeyPaths for Locks Example ===\n");

//     // Create a multi-level structure with locks
//     let mut app_state = ApplicationState {
//         user: Some(UserAccount {
//             profile: Some(UserProfile {
//                 data: std::sync::Mutex::new(UserData {
//                     name: "Alice".to_string(),
//                     age: 30,
//                     email: "alice@example.com".to_string(),
//                 }),
//                 preferences: std::sync::RwLock::new(vec!["dark_mode".to_string(), "notifications".to_string()]),
//                 metadata: Arc::new(std::sync::Mutex::new({
//                     let mut map = HashMap::new();
//                     map.insert("created".to_string(), "2024-01-01".to_string());
//                     map.insert("last_login".to_string(), "2024-12-01".to_string());
//                     map
//                 })),
//             }),
//             account_id: 12345,
//         }),
//         system_config: Arc::new(std::sync::RwLock::new(SystemConfig {
//             theme: "dark".to_string(),
//             language: "en".to_string(),
//         })),
//     };

//     // ==========================================
//     // Example 1: Reading from Mutex with keypath composition and chaining
//     // ==========================================
//     println!("1. Reading user name from Mutex<UserData> using keypath chaining:");
    
//     // Chain through Option types and Mutex using keypath composition
//     // ApplicationState -> Option<UserAccount> -> Option<UserProfile> -> Mutex<UserData> -> name
//     // Pattern: L1::f1_fr().then(L2::f1_fr()).get() to get UserProfile, then use fr_at with the lock
//     if let Some(user_profile) = ApplicationState::user_fr()
//         .then(UserAccount::profile_fr())
//         .get(&app_state)
//     {
//         // Use fr_at to get cloned value from Mutex
//         let get_name = UserProfile::data_r(UserData::name_r());
//         if let Some(name) = get_name(&user_profile.data) {
//             println!("   User name: {}", name);
//         }
//     }

//     // ==========================================
//     // Example 2: Reading preferences from RwLock<Vec<String>> using keypath chaining
//     // ==========================================
//     println!("\n2. Reading preferences from RwLock<Vec<String>> using keypath chaining:");
    
//     // Chain through Option types and RwLock using keypath composition
//     // ApplicationState -> Option<UserAccount> -> Option<UserProfile> -> RwLock<Vec<String>> -> Vec<String>
//     if let Some(user_profile) = ApplicationState::user_fr()
//         .then(UserAccount::profile_fr())
//         .get(&app_state)
//     {
//         let vec_kp = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
//         let get_prefs = UserProfile::preferences_rwlock_fr_at(vec_kp);
//         if let Some(prefs) = get_prefs(&user_profile.preferences) {
//             println!("   All preferences: {:?}", prefs);
//             if let Some(first) = prefs.first() {
//                 println!("   First preference: {}", first);
//             }
//         }
//     }

//     // ==========================================
//     // Example 3: Writing to Mutex with direct value
//     // ==========================================
//     println!("\n3. Updating user age in Mutex<UserData>:");
    
//     // Chain through Option types using failable writable keypaths
//     if let Some(user_profile) = ApplicationState::user_fw()
//         .then(UserAccount::profile_fw())
//         .get_mut(&mut app_state)
//     {
//         // Create writable keypath to age field
//         let age_kp = UserData::age_w();
        
//         // Use the helper method to update value directly
//         let update_age = UserProfile::data_mutex_fw_at(age_kp, 31u32);
        
//         if update_age(&user_profile.data).is_some() {
//             println!("   Updated age to: 31");
//             // Verify the update using keypath chaining
//             let get_age = UserProfile::data_mutex_fr_at(UserData::age_r());
//             if let Some(age) = get_age(&user_profile.data) {
//                 println!("   Verified age: {}", age);
//             }
//         }
//     }

//     // ==========================================
//     // Example 4: Writing to RwLock with direct value
//     // ==========================================
//     println!("\n4. Updating preferences in RwLock<Vec<String>>:");
    
//     // Chain through Option types to get mutable access to UserProfile
//     if let Some(user_profile) = ApplicationState::user_fw()
//         .then(UserAccount::profile_fw())
//         .get_mut(&mut app_state)
//     {
//         // Create writable keypath to the Vec
//         let vec_kp = rust_keypaths::WritableKeyPath::new(|v: &mut Vec<String>| v);
        
//         // Create new preferences list with added item
//         let mut new_prefs = vec!["dark_mode".to_string(), "notifications".to_string(), "accessibility".to_string()];
        
//         // Use the helper method to update with new value directly
//         let update_preferences = UserProfile::preferences_rwlock_fw_at(vec_kp, new_prefs);
        
//         if update_preferences(&user_profile.preferences).is_some() {
//             println!("   Updated preferences list");
//             // Verify the update using keypath chaining
//             let vec_kp_read = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
//             let get_prefs = UserProfile::preferences_rwlock_fr_at(vec_kp_read);
//             if let Some(prefs) = get_prefs(&user_profile.preferences) {
//                 println!("   All preferences: {:?}", prefs);
//             }
//         }
//     }

//     // ==========================================
//     // Example 5: Working with Arc<Mutex<T>> using keypath chaining
//     // ==========================================
//     println!("\n5. Reading from Arc<Mutex<HashMap>> using keypath chaining:");
    
//     // Chain through Option types and Arc<Mutex> using keypath composition
//     if let Some(user_profile) = ApplicationState::user_fr()
//         .then(UserAccount::profile_fr())
//         .get(&app_state)
//     {
//         let map_kp = rust_keypaths::KeyPath::new(|m: &HashMap<String, String>| m);
//         let get_map = UserProfile::metadata_arc_mutex_fr_at(map_kp);
//         if let Some(map) = get_map(&user_profile.metadata) {
//             if let Some(last_login) = map.get("last_login") {
//                 println!("   Last login: {}", last_login);
//             }
//         }
//     }

//     // ==========================================
//     // Example 6: Working with Arc<RwLock<T>>
//     // ==========================================
//     println!("\n6. Reading and writing to Arc<RwLock<SystemConfig>>:");
    
//     // Read theme using keypath chaining (direct access since system_config is not Option)
//     let theme_kp = SystemConfig::theme_r();
//     // Note: Since system_config is not Option, we access it directly
//     let theme_keypath = ApplicationState::system_config_r();
//     // For demonstration, we'll use the lock helper method
//     // In practice, you'd chain: ApplicationState::system_config_arc_rwlock_fr_at(theme_kp)
//     // But since system_config is direct (not Option), we need to handle it differently
//     let get_theme = ApplicationState::system_config_arc_rwlock_fr_at(theme_kp);
//     if let Some(theme) = get_theme(&app_state.system_config) {
//         println!("   Current theme: {}", theme);
//     }
    
//     // Update language
//     let language_kp = SystemConfig::language_w();
//     let update_language = ApplicationState::system_config_arc_rwlock_fw_at(language_kp, "fr".to_string());
    
//     if update_language(&app_state.system_config).is_some() {
//         println!("   Updated language to: fr");
//         // Verify the update using keypath chaining
//         let language_kp_read = SystemConfig::language_r();
//         let get_language = ApplicationState::system_config_arc_rwlock_fr_at(language_kp_read);
//         if let Some(lang) = get_language(&app_state.system_config) {
//             println!("   Verified language: {}", lang);
//         }
//     }

//     // ==========================================
//     // Example 7: Deep multi-level access through Option chains
//     // ==========================================
//     println!("\n7. Deep multi-level access through Option chains:");
    
//     // Access nested fields through multiple Option levels using keypath chaining
//     // ApplicationState -> Option<UserAccount> -> Option<UserProfile> -> Mutex<UserData> -> email
//     // Pattern: L1::f1_fr().then(L2::f1_fr()).get() to get UserProfile, then use fr_at with the lock
//     if let Some(user_profile) = ApplicationState::user_fr()
//         .then(UserAccount::profile_fr())
//         .get(&app_state)
//     {
//         let get_email = UserProfile::data_mutex_fr_at(UserData::email_r());
//         if let Some(email) = get_email(&user_profile.data) {
//             println!("   User email (deep access via keypath chain): {}", email);
//         }
//     }

//     // ==========================================
//     // Example 8: Complex update with multiple fields
//     // ==========================================
//     println!("\n8. Complex update with multiple fields:");
    
//     // Chain through Option types to get mutable access
//     if let Some(user_profile) = ApplicationState::user_fw()
//         .then(UserAccount::profile_fw())
//         .get_mut(&mut app_state)
//     {
//         // Update multiple fields in separate lock acquisitions
//         let name_kp = UserData::name_w();
//         let update_name = UserProfile::data_mutex_fw_at(name_kp, "Alice Updated".to_string());
        
//         if update_name(&user_profile.data).is_some() {
//             println!("   Updated name to: Alice Updated");
//             // Then update age in a separate operation
//             let age_kp = UserData::age_w();
//             let update_age = UserProfile::data_mutex_fw_at(age_kp, 31u32);
            
//             if update_age(&user_profile.data).is_some() {
//                 println!("   Updated age to: 31");
//                 // Read both back to verify using keypath chaining
//                 let get_name = UserProfile::data_mutex_fr_at(UserData::name_r());
//                 let get_age = UserProfile::data_mutex_fr_at(UserData::age_r());
//                 if let (Some(name), Some(age)) = (
//                     get_name(&user_profile.data),
//                     get_age(&user_profile.data),
//                 ) {
//                     println!("   Verified - Name: {}, Age: {}", name, age);
//                 }
//             }
//         }
//     }

//     // ==========================================
//     // Example 9: Working with collections inside locks
//     // ==========================================
//     println!("\n9. Working with collections inside locks:");
    
//     // Chain through Option types
//     if let Some(user_profile) = ApplicationState::user_fw()
//         .then(UserAccount::profile_fw())
//         .get_mut(&mut app_state)
//     {
//         // Read all preferences using keypath chaining
//         let vec_kp = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
//         let get_prefs = UserProfile::preferences_rwlock_fr_at(vec_kp);
//         if let Some(prefs) = get_prefs(&user_profile.preferences) {
//             println!("   Current preferences count: {}", prefs.len());
//         }
        
//         // Modify the collection - read current, modify, then write back
//         let vec_kp_read = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
//         let get_prefs_read = UserProfile::preferences_rwlock_fr_at(vec_kp_read);
//         if let Some(mut prefs) = get_prefs_read(&user_profile.preferences) {
//             prefs.retain(|p| p != "notifications");
//             prefs.push("high_contrast".to_string());
            
//             let vec_kp_mut = rust_keypaths::WritableKeyPath::new(|v: &mut Vec<String>| v);
//             let modify_prefs = UserProfile::preferences_rwlock_fw_at(vec_kp_mut, prefs);
            
//             if modify_prefs(&user_profile.preferences).is_some() {
//                 println!("   Modified preferences list");
//                 // Read after modification using keypath chaining
//                 let vec_kp_after = rust_keypaths::KeyPath::new(|v: &Vec<String>| v);
//                 let get_prefs_after = UserProfile::preferences_rwlock_fr_at(vec_kp_after);
//                 if let Some(prefs) = get_prefs_after(&user_profile.preferences) {
//                     println!("   Updated preferences: {:?}", prefs);
//                 }
//             }
//         }
//     }

//     // ==========================================
//     // Example 10: Concurrent-safe access patterns
//     // ==========================================
//     println!("\n10. Concurrent-safe access patterns:");
    
//     // Demonstrate that locks are properly acquired and released
//     // Chain through Option types using keypaths
//     if let Some(user_profile) = ApplicationState::user_fr()
//         .then(UserAccount::profile_fr())
//         .get(&app_state)
//     {
//         // Multiple read operations can happen (RwLock allows concurrent reads)
//         // Using keypath chaining for both reads
//         let get_name = UserProfile::data_mutex_fr_at(UserData::name_r());
//         let get_email = UserProfile::data_mutex_fr_at(UserData::email_r());
//         if let Some(name) = get_name(&user_profile.data) {
//             println!("   Read name: {}", name);
//         }
//         if let Some(email) = get_email(&user_profile.data) {
//             println!("   Read email: {}", email);
//         }
//     }

//     // ==========================================
//     // Example 11: Error handling with lock acquisition
//     // ==========================================
//     println!("\n11. Error handling with lock acquisition:");
    
//     // Chain through Option types - if any is None, the chain short-circuits
//     if let Some(user_profile) = ApplicationState::user_fr()
//         .then(UserAccount::profile_fr())
//         .get(&app_state)
//     {
//         // The helper methods return Option, handling lock acquisition failures gracefully
//         let get_name = UserProfile::data_mutex_fr_at(UserData::name_r());
//         match get_name(&user_profile.data) {
//             Some(name) => println!("   Successfully acquired lock and read name: {}", name),
//             None => println!("   Failed to acquire lock (would happen if lock was poisoned)"),
//         }
//     } else {
//         println!("   Keypath chain short-circuited (user or profile is None)");
//     }

//     // ==========================================
//     // Example 12: Composition with .then() for nested structures
//     // ==========================================
//     println!("\n12. Composition pattern for nested structures:");
    
//     // This demonstrates how you can compose keypaths before using them with locks
//     // Chain through Option types using keypath composition
//     if let Some(user_profile) = ApplicationState::user_fr()
//         .then(UserAccount::profile_fr())
//         .get(&app_state)
//     {
//         // Create a keypath that accesses a nested field
//         // The helper method accepts any keypath that works with the inner type
//         let get_name = UserProfile::data_mutex_fr_at(UserData::name_r());
//         if let Some(name) = get_name(&user_profile.data) {
//             println!("   Composed keypath result: {}", name);
//         }
        
//         // You can also create keypaths on-the-fly
//         let custom_kp = rust_keypaths::KeyPath::new(|data: &UserData| &data.email);
//         let get_email = UserProfile::data_mutex_fr_at(custom_kp);
//         if let Some(email) = get_email(&user_profile.data) {
//             println!("   Custom keypath result: {}", email);
//         }
//     }

//     // ==========================================
//     // Example 13: Real-world scenario - User profile update
//     // ==========================================
//     println!("\n13. Real-world scenario - Complete user profile update:");
    
//     // Chain through Option types to get mutable access
//     if let Some(user_profile) = ApplicationState::user_fw()
//         .then(UserAccount::profile_fw())
//         .get_mut(&mut app_state)
//     {
//         println!("   Performing complete profile update...");
        
//         // Update user data
//         let name_kp = UserData::name_w();
//         let update_name = UserProfile::data_mutex_fw_at(name_kp, "Alice Smith".to_string());
//         update_name(&user_profile.data);
        
//         let age_kp = UserData::age_w();
//         let update_age = UserProfile::data_mutex_fw_at(age_kp, 32u32);
//         update_age(&user_profile.data);
        
//         // Update preferences
//         let prefs_kp = rust_keypaths::WritableKeyPath::new(|v: &mut Vec<String>| v);
//         let new_prefs = vec!["dark_mode".to_string(), "compact_view".to_string()];
//         let update_prefs = UserProfile::preferences_rwlock_fw_at(prefs_kp, new_prefs);
//         update_prefs(&user_profile.preferences);
        
//         // Update metadata - read current, modify, then write back
//         let metadata_kp_read = rust_keypaths::KeyPath::new(|m: &HashMap<String, String>| m);
//         let get_metadata = UserProfile::metadata_arc_mutex_fr_at(metadata_kp_read);
//         if let Some(mut meta) = get_metadata(&user_profile.metadata) {
//             meta.insert("last_updated".to_string(), "2024-12-15".to_string());
//             let metadata_kp = rust_keypaths::WritableKeyPath::new(|m: &mut HashMap<String, String>| m);
//             let update_metadata = UserProfile::metadata_arc_mutex_fw_at(metadata_kp, meta);
//             update_metadata(&user_profile.metadata);
//         }
        
//         println!("   Profile update complete!");
        
//         // Verify all updates using keypath chaining
//         let get_name = UserProfile::data_mutex_fr_at(UserData::name_r());
//         let get_age = UserProfile::data_mutex_fr_at(UserData::age_r());
//         if let (Some(name), Some(age)) = (
//             get_name(&user_profile.data),
//             get_age(&user_profile.data),
//         ) {
//             println!("   Final state - Name: {}, Age: {}", name, age);
//         }
//     }

//     println!("\n=== Example Complete ===");
//     println!("\nKey Takeaways:");
//     println!("1. Use keypath chaining (.then()) to traverse Option types to get to the lock");
//     println!("2. Helper methods (_mutex_fr_at, _rwlock_fr_at, etc.) take a keypath and return a closure");
//     println!("3. Pattern for reading: L1::f1_fr().then(L2::f1_fr()).get() to get lock, then use fr_at(keypath)(&lock)");
//     println!("4. Pattern for writing: L1::f1_fr().then(L2::f1_fr()).get_mut() to get lock, then use fw_at(keypath, new_value)(&lock)");
//     println!("5. Read operations (_fr_at) take KeyPath<T, Value> and return Fn(&Lock<T>) -> Option<Value> (cloned)");
//     println!("6. Write operations (_fw_at) take WritableKeyPath<T, Value> and new_value, return FnOnce(&Lock<T>) -> Option<()>");
//     println!("7. All lock types (Mutex, RwLock, Arc<Mutex>, Arc<RwLock>) are supported");
//     println!("8. Methods return Option to handle lock acquisition failures");
//     println!("9. Deep keypath composition: pass nested keypaths (e.g., L3::f1().then(L4::f1())) to _fr_at/_fw_at methods");
// }

fn main() {}
