use key_paths_derive::Keypaths;
use key_paths_core::WithContainer;
use std::sync::Arc;

#[cfg(feature = "parking_lot")]
use parking_lot::{Mutex, RwLock};

#[derive(Keypaths, Clone, Debug)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

#[derive(Keypaths, Clone, Debug)]
struct Profile {
    user: User,
    bio: String,
    settings: Option<Settings>,
}

#[derive(Keypaths, Clone, Debug)]
struct Settings {
    theme: String,
    notifications: bool,
}

fn main() {
    println!("🚗 Parking Lot Support Example");
    println!("==============================");

    #[cfg(feature = "parking_lot")]
    {
        // Create test data wrapped in Arc<parking_lot::Mutex<T>> and Arc<parking_lot::RwLock<T>>
        let parking_mutex_user = Arc::new(Mutex::new(User {
            name: "Alice".to_string(),
            age: 30,
            email: Some("akash@example.com".to_string()),
        }));

        let parking_rwlock_profile = Arc::new(RwLock::new(Profile {
            user: User {
                name: "Bob".to_string(),
                age: 25,
                email: None,
            },
            bio: "Software developer".to_string(),
            settings: Some(Settings {
                theme: "dark".to_string(),
                notifications: true,
            }),
        }));

        println!("\n🎯 Testing Arc<parking_lot::Mutex<T>> Support");
        println!("--------------------------------------------");

        // Test for_arc_parking_mutex aggregator
        let name_keypath = User::name_r();
        let age_keypath = User::age_r();
        let email_keypath = User::email_fr();

        // Convert to Arc<parking_lot::Mutex<T>> keypaths
        let name_parking_mutex_path = name_keypath.for_arc_parking_mutex();
        let age_parking_mutex_path = age_keypath.for_arc_parking_mutex();
        let email_parking_mutex_path = email_keypath.for_arc_parking_mutex();

        // Test reading values
        if let Some(name) = name_parking_mutex_path.get_failable_owned(parking_mutex_user.clone()) {
            println!("✅ Name from Arc<parking_lot::Mutex<User>>: {}", name);
        }

        if let Some(age) = age_parking_mutex_path.get_failable_owned(parking_mutex_user.clone()) {
            println!("✅ Age from Arc<parking_lot::Mutex<User>>: {}", age);
        }

        if let Some(email) = email_parking_mutex_path.get_failable_owned(parking_mutex_user.clone()) {
            println!("✅ Email from Arc<parking_lot::Mutex<User>>: {:?}", email);
        }

        println!("\n🎯 Testing Arc<parking_lot::RwLock<T>> Support");
        println!("---------------------------------------------");

        // Test for_arc_parking_rwlock aggregator
        let bio_keypath = Profile::bio_r();
        let user_name_keypath = Profile::user_r().then(User::name_r());
        let user_age_keypath = Profile::user_r().then(User::age_r());
        let settings_theme_keypath = Profile::settings_fr().then(Settings::theme_r());

        // Convert to Arc<parking_lot::RwLock<T>> keypaths
        let bio_parking_rwlock_path = bio_keypath.for_arc_parking_rwlock();
        let user_name_parking_rwlock_path = user_name_keypath.for_arc_parking_rwlock();
        let user_age_parking_rwlock_path = user_age_keypath.for_arc_parking_rwlock();
        let settings_theme_parking_rwlock_path = settings_theme_keypath.for_arc_parking_rwlock();

        // Test reading values
        if let Some(bio) = bio_parking_rwlock_path.get_failable_owned(parking_rwlock_profile.clone()) {
            println!("✅ Bio from Arc<parking_lot::RwLock<Profile>>: {}", bio);
        }

        if let Some(name) = user_name_parking_rwlock_path.get_failable_owned(parking_rwlock_profile.clone()) {
            println!("✅ User name from Arc<parking_lot::RwLock<Profile>>: {}", name);
        }

        if let Some(age) = user_age_parking_rwlock_path.get_failable_owned(parking_rwlock_profile.clone()) {
            println!("✅ User age from Arc<parking_lot::RwLock<Profile>>: {}", age);
        }

        if let Some(theme) = settings_theme_parking_rwlock_path.get_failable_owned(parking_rwlock_profile.clone()) {
            println!("✅ Settings theme from Arc<parking_lot::RwLock<Profile>>: {}", theme);
        }

        println!("\n🔄 Testing Composition with Parking Lot Support");
        println!("-----------------------------------------------");

        // Test composition with parking lot aggregators
        let nested_email_path = Profile::user_r()
            .then(User::email_fr())
            .for_arc_parking_rwlock();

        if let Some(email) = nested_email_path.get_failable_owned(parking_rwlock_profile.clone()) {
            println!("✅ Nested email from Arc<parking_lot::RwLock<Profile>>: {:?}", email);
        }

        println!("\n📊 Testing with Collections");
        println!("--------------------------");

        // Test with collections of Arc<parking_lot::Mutex<T>>
        let users = vec![
            Arc::new(Mutex::new(User {
                name: "Charlie".to_string(),
                age: 35,
                email: Some("charlie@example.com".to_string()),
            })),
            Arc::new(Mutex::new(User {
                name: "Diana".to_string(),
                age: 28,
                email: None,
            })),
        ];

        let name_aggregator = User::name_r().for_arc_parking_mutex();
        let email_aggregator = User::email_fr().for_arc_parking_mutex();

        for (i, user) in users.iter().enumerate() {
            if let Some(name) = name_aggregator.clone().get_failable_owned(user.clone()) {
                if let Some(email) = email_aggregator.clone().get_failable_owned(user.clone()) {
                    println!("✅ User {}: {} (email: {:?})", i + 1, name, email);
                }
            }
        }

        println!("\n💡 Key Takeaways");
        println!("================");
        println!("1. for_arc_parking_mutex() adapts keypaths to work with Arc<parking_lot::Mutex<T>>");
        println!("2. for_arc_parking_rwlock() adapts keypaths to work with Arc<parking_lot::RwLock<T>>");
        println!("3. Both return FailableOwned keypaths that clone values");
        println!("4. Use get_failable_owned() to access values from these keypaths");
        println!("5. Composition works naturally: path.then(other).for_arc_parking_mutex()");
        println!("6. Perfect for working with collections of Arc<parking_lot::Mutex<T>> or Arc<parking_lot::RwLock<T>>");
        println!("7. Parking lot locks are faster than std::sync locks");
        println!("8. Requires the 'parking_lot' feature to be enabled");
    }

    #[cfg(not(feature = "parking_lot"))]
    {
        println!("\n⚠️  Parking Lot Support Not Available");
        println!("====================================");
        println!("This example requires the 'parking_lot' feature to be enabled.");
        println!("Add 'parking_lot' to your Cargo.toml features to use this functionality.");
        println!("\nExample Cargo.toml:");
        println!("[dependencies]");
        println!("key-paths-core = {{ version = \"1.0.6\", features = [\"parking_lot\"] }}");
    }
}
