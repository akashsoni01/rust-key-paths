use key_paths_derive::Keypaths;
use key_paths_core::WithContainer;
use std::sync::{Arc, Mutex, RwLock};

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
}

fn main() {
    println!("🔒 Arc Sync Aggregator Example");
    println!("==============================");

    // Create test data wrapped in Arc<Mutex<T>> and Arc<RwLock<T>>
    let arc_mutex_user = Arc::new(Mutex::new(User {
        name: "Alice".to_string(),
        age: 30,
        email: Some("alice@example.com".to_string()),
    }));

    let arc_rwlock_profile = Arc::new(RwLock::new(Profile {
        user: User {
            name: "Bob".to_string(),
            age: 25,
            email: None,
        },
        bio: "Software developer".to_string(),
    }));

    println!("\n🎯 Testing Arc<Mutex<T>> - No Clone Approach");
    println!("--------------------------------------------");

    // Method 1: Using with_arc_mutex (no cloning)
    let name_keypath = User::name_r();
    let age_keypath = User::age_r();
    let email_keypath = User::email_fr();

    // Use with_mutex for no-clone access
    name_keypath.clone().with_mutex(&arc_mutex_user, |name| {
        println!("✅ Name from Arc<Mutex<User>> (no clone): {}", name);
    });

    age_keypath.clone().with_mutex(&arc_mutex_user, |age| {
        println!("✅ Age from Arc<Mutex<User>> (no clone): {}", age);
    });

    email_keypath.clone().with_mutex(&arc_mutex_user, |email| {
        println!("✅ Email from Arc<Mutex<User>> (no clone): {:?}", email);
    });

    println!("\n🎯 Testing Arc<Mutex<T>> - Aggregator Approach");
    println!("----------------------------------------------");

    // Method 2: Using for_arc_mutex aggregator (with cloning)
    let name_arc_mutex_path = name_keypath.for_arc_mutex();
    let age_arc_mutex_path = age_keypath.for_arc_mutex();
    let email_arc_mutex_path = email_keypath.for_arc_mutex();

    // Test reading values with cloning
    if let Some(name) = name_arc_mutex_path.get_failable_owned(arc_mutex_user.clone()) {
        println!("✅ Name from Arc<Mutex<User>> (with clone): {}", name);
    }

    if let Some(age) = age_arc_mutex_path.get_failable_owned(arc_mutex_user.clone()) {
        println!("✅ Age from Arc<Mutex<User>> (with clone): {}", age);
    }

    if let Some(email) = email_arc_mutex_path.get_failable_owned(arc_mutex_user.clone()) {
        println!("✅ Email from Arc<Mutex<User>> (with clone): {:?}", email);
    }

    println!("\n🎯 Testing Arc<RwLock<T>> - No Clone Approach");
    println!("---------------------------------------------");

    // Method 1: Using with_arc_rwlock (no cloning)
    let bio_keypath = Profile::bio_r();
    let user_name_keypath = Profile::user_r().then(User::name_r());
    let user_age_keypath = Profile::user_r().then(User::age_r());

    // Use with_rwlock for no-clone access
    bio_keypath.clone().with_rwlock(&arc_rwlock_profile, |bio| {
        println!("✅ Bio from Arc<RwLock<Profile>> (no clone): {}", bio);
    });

    user_name_keypath.clone().with_rwlock(&arc_rwlock_profile, |name| {
        println!("✅ User name from Arc<RwLock<Profile>> (no clone): {}", name);
    });

    user_age_keypath.clone().with_rwlock(&arc_rwlock_profile, |age| {
        println!("✅ User age from Arc<RwLock<Profile>> (no clone): {}", age);
    });

    println!("\n🎯 Testing Arc<RwLock<T>> - Aggregator Approach");
    println!("-----------------------------------------------");

    // Method 2: Using for_arc_rwlock aggregator (with cloning)
    let bio_arc_rwlock_path = bio_keypath.for_arc_rwlock();
    let user_name_arc_rwlock_path = user_name_keypath.for_arc_rwlock();
    let user_age_arc_rwlock_path = user_age_keypath.for_arc_rwlock();

    // Test reading values with cloning
    if let Some(bio) = bio_arc_rwlock_path.get_failable_owned(arc_rwlock_profile.clone()) {
        println!("✅ Bio from Arc<RwLock<Profile>> (with clone): {}", bio);
    }

    if let Some(name) = user_name_arc_rwlock_path.get_failable_owned(arc_rwlock_profile.clone()) {
        println!("✅ User name from Arc<RwLock<Profile>> (with clone): {}", name);
    }

    if let Some(age) = user_age_arc_rwlock_path.get_failable_owned(arc_rwlock_profile.clone()) {
        println!("✅ User age from Arc<RwLock<Profile>> (with clone): {}", age);
    }

    println!("\n🔄 Advanced Composition Examples");
    println!("=================================");

    // Example 1: Multi-level composition with no-clone approach
    println!("\n📝 Example 1: Multi-level Composition (No Clone)");
    println!("-----------------------------------------------");
    
    let nested_email_path = Profile::user_r().then(User::email_fr());
    nested_email_path.with_rwlock(&arc_rwlock_profile, |email| {
        println!("✅ Nested email from Arc<RwLock<Profile>> (no clone): {:?}", email);
    });

    // Example 2: Complex composition with multiple levels
    println!("\n📝 Example 2: Complex Multi-level Composition");
    println!("--------------------------------------------");
    
    // Create a more complex nested structure
    let complex_profile = Arc::new(RwLock::new(Profile {
        user: User {
            name: "Complex User".to_string(),
            age: 42,
            email: Some("complex@example.com".to_string()),
        },
        bio: "Complex bio with multiple levels".to_string(),
    }));

    // Multi-level composition: Profile -> User -> Email
    let complex_email_path = Profile::user_r()
        .then(User::email_fr());
    
    complex_email_path.with_rwlock(&complex_profile, |email| {
        println!("✅ Complex nested email (no clone): {:?}", email);
    });

    // Example 3: Composition with aggregators (with cloning)
    println!("\n📝 Example 3: Composition with Aggregators (With Clone)");
    println!("----------------------------------------------------");
    
    let nested_email_aggregator = Profile::user_r()
        .then(User::email_fr())
        .for_arc_rwlock();

    if let Some(email) = nested_email_aggregator.get_failable_owned(arc_rwlock_profile.clone()) {
        println!("✅ Nested email from Arc<RwLock<Profile>> (with clone): {:?}", email);
    }

    // Example 4: Reusable composition patterns
    println!("\n📝 Example 4: Reusable Composition Patterns");
    println!("-------------------------------------------");
    
    // Create reusable base paths
    let user_base = Profile::user_r();
    let user_name_path = user_base.clone().then(User::name_r());
    let user_age_path = user_base.clone().then(User::age_r());
    let user_email_path = user_base.then(User::email_fr());

    // Use the same base paths with different containers
    user_name_path.with_rwlock(&arc_rwlock_profile, |name| {
        println!("✅ Reusable name path (no clone): {}", name);
    });

    user_age_path.with_rwlock(&arc_rwlock_profile, |age| {
        println!("✅ Reusable age path (no clone): {}", age);
    });

    user_email_path.with_rwlock(&arc_rwlock_profile, |email| {
        println!("✅ Reusable email path (no clone): {:?}", email);
    });

    // Example 5: Composition with different container types
    println!("\n📝 Example 5: Mixed Container Types");
    println!("----------------------------------");
    
    // Use the same keypath with different container types
    let name_path = User::name_r();
    
    // With Arc<Mutex<T>>
    name_path.with_mutex(&arc_mutex_user, |name| {
        println!("✅ Name from Arc<Mutex<User>> (no clone): {}", name);
    });
    
    // With Arc<RwLock<T>> (through Profile)
    let profile_name_path = Profile::user_r().then(User::name_r());
    profile_name_path.with_rwlock(&arc_rwlock_profile, |name| {
        println!("✅ Name from Arc<RwLock<Profile>> (no clone): {}", name);
    });

    println!("\n📊 Testing with Collections");
    println!("===========================");

    // Test with collections of Arc<Mutex<T>>
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

    println!("\n📝 Collections - No Clone Approach");
    println!("----------------------------------");
    
    let name_path = User::name_r();
    let email_path = User::email_fr();

    for (i, user) in users.iter().enumerate() {
        name_path.clone().with_mutex(user, |name| {
            email_path.clone().with_mutex(user, |email| {
                println!("✅ User {}: {} (email: {:?}) - no clone", i + 1, name, email);
            });
        });
    }

    println!("\n📝 Collections - Aggregator Approach (With Clone)");
    println!("------------------------------------------------");
    
    let name_aggregator = User::name_r().for_arc_mutex();
    let email_aggregator = User::email_fr().for_arc_mutex();

    for (i, user) in users.iter().enumerate() {
        if let Some(name) = name_aggregator.clone().get_failable_owned(user.clone()) {
            if let Some(email) = email_aggregator.clone().get_failable_owned(user.clone()) {
                println!("✅ User {}: {} (email: {:?}) - with clone", i + 1, name, email);
            }
        }
    }

    println!("\n💡 Key Takeaways");
    println!("================");
    println!("1. Two approaches for Arc<Mutex<T>> and Arc<RwLock<T>>:");
    println!("   - with_arc_mutex()/with_arc_rwlock(): No cloning, callback-based");
    println!("   - for_arc_mutex()/for_arc_rwlock(): With cloning, aggregator-based");
    println!("2. No-clone approach is preferred for performance:");
    println!("   - Use with_arc_mutex() and with_arc_rwlock() from WithContainer trait");
    println!("   - Access values through callbacks without cloning");
    println!("3. Aggregator approach for when you need owned values:");
    println!("   - Use for_arc_mutex() and for_arc_rwlock()");
    println!("   - Returns FailableOwned keypaths that clone values");
    println!("   - Use get_failable_owned() to access values");
    println!("4. Composition works naturally with both approaches:");
    println!("   - path.then(other).with_arc_mutex() (no clone)");
    println!("   - path.then(other).for_arc_mutex() (with clone)");
    println!("5. Perfect for working with collections of Arc<Mutex<T>> or Arc<RwLock<T>>");
    println!("6. Handles lock poisoning gracefully by returning None");
    println!("7. Reusable composition patterns work with both approaches");
    println!("8. Mixed container types supported seamlessly");
}
