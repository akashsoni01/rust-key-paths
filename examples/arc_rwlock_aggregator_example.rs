use key_paths_core::WithContainer;
use key_paths_derive::Keypaths;
use std::sync::{Arc, RwLock};

#[derive(Keypaths, Clone, Debug)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

#[derive(Keypaths, Clone, Debug)]
struct Profile {
    user: User,
    #[Writable]
    bio: String,
    #[Writable]
    settings: Settings,
}

#[derive(Keypaths, Clone, Debug)]
struct Settings {
    theme: String,
    notifications: bool,
}

fn main() {
    println!("🔒 Arc<RwLock> Aggregator Example");
    println!("=================================");

    // Create Arc<RwLock> containers
    let arc_rwlock_user = Arc::new(RwLock::new(User {
        name: "Alice Johnson".to_string(),
        age: 30,
        email: Some("alice@example.com".to_string()),
    }));

    let arc_rwlock_profile = Arc::new(RwLock::new(Profile {
        user: User {
            name: "Akash Soni".to_string(),
            age: 25,
            email: None,
        },
        bio: "Software developer with passion for Rust".to_string(),
        settings: Settings {
            theme: "dark".to_string(),
            notifications: true,
        },
    }));

    println!("\n🎯 Testing for_arc_rwlock() Aggregator");
    println!("-------------------------------------");

    // Test 1: Simple field access with for_arc_rwlock()
    println!("\n1️⃣  Simple Field Access");
    println!("----------------------");

    let name_keypath = User::name();
    let arc_rwlock_name_keypath = name_keypath.for_arc_rwlock();

    // Use get_failable_owned() since for_arc_rwlock() returns FailableOwned
    if let Some(name) = arc_rwlock_name_keypath.get_failable_owned(arc_rwlock_user.clone()) {
        println!("✅ User name from Arc<RwLock>: {}", name);
    }

    // Test 2: Optional field access
    println!("\n2️⃣  Optional Field Access");
    println!("-------------------------");

    let email_keypath = User::email();
    let arc_rwlock_email_keypath = email_keypath.for_arc_rwlock();

    if let Some(email) = arc_rwlock_email_keypath.get_failable_owned(arc_rwlock_user.clone()) {
        println!("✅ User email from Arc<RwLock>: {}", email);
    }

    // Test 3: Nested field access (readable chain)
    println!("\n3️⃣  Nested Field Access");
    println!("----------------------");

    let profile_user_name_keypath = Profile::user().then(User::name());
    let arc_rwlock_user_name_keypath = profile_user_name_keypath.for_arc_rwlock();

    if let Some(name) = arc_rwlock_user_name_keypath.get_failable_owned(arc_rwlock_profile.clone())
    {
        println!("✅ Nested user name from Arc<RwLock>: {}", name);
    }

    // Test 4: Deeply nested field access
    println!("\n4️⃣  Deeply Nested Field Access");
    println!("-----------------------------");

    let nested_email_keypath = Profile::user().then(User::email());
    let arc_rwlock_email_nested_keypath = nested_email_keypath.for_arc_rwlock();

    if let Some(email) =
        arc_rwlock_email_nested_keypath.get_failable_owned(arc_rwlock_profile.clone())
    {
        println!("✅ Nested user email from Arc<RwLock>: {}", email);
    }

    println!("\n🔄 Testing with_arc_rwlock() Methods");
    println!("-----------------------------------");

    // Test 5: Using with_arc_rwlock() for read access
    println!("\n5️⃣  Read Access with with_arc_rwlock()");
    println!("-------------------------------------");

    let name_keypath = User::name();
    if let Some(name) = name_keypath.with_arc_rwlock(&arc_rwlock_user, |name| name.clone()) {
        println!("✅ User name via with_arc_rwlock(): {}", name);
    }

    // Test 6: Using with_arc_rwlock() for nested access
    println!("\n6️⃣  Nested Access with with_arc_rwlock()");
    println!("---------------------------------------");

    let user_name_keypath = Profile::user().then(User::name());
    if let Some(name) = user_name_keypath.with_arc_rwlock(&arc_rwlock_profile, |name| name.clone())
    {
        println!("✅ Profile user name via with_arc_rwlock(): {}", name);
    }

    // Test 7: Using with_arc_rwlock_mut() for write access
    println!("\n7️⃣  Write Access with with_arc_rwlock_mut()");
    println!("-----------------------------------------");

    let bio_keypath = Profile::bio();
    if let Some(new_bio) = bio_keypath.with_arc_rwlock_mut(&arc_rwlock_profile, |bio| {
        let old_bio = bio.clone();
        *bio =
            "Senior software engineer with expertise in Rust and systems programming".to_string();
        old_bio
    }) {
        println!("✅ Updated bio from Arc<RwLock>, old was: {}", new_bio);
    }

    // Verify the change
    println!("///////========================================================================");
    let bio_keypath = Profile::bio();
    if let Some(bio) = bio_keypath.with_arc_rwlock_mut(&arc_rwlock_profile, |bio| bio.clone()) {
        println!("✅ New bio after update: {}", bio);
    }

    // Test 8: Using with_arc_rwlock_mut() for nested write access
    println!("\n8️⃣  Nested Write Access with with_arc_rwlock_mut()");
    println!("-----------------------------------------------");

    // We need to use a writable keypath for the entire path
    let settings_keypath = Profile::settings();
    if let Some(old_theme) = settings_keypath.with_arc_rwlock_mut(&arc_rwlock_profile, |settings| {
        let old_theme = settings.theme.clone();
        settings.theme = "light".to_string();
        old_theme
    }) {
        println!("✅ Updated theme from Arc<RwLock>, old was: {}", old_theme);
    }

    // Verify the change
    println!("///////========================================================================");
    let settings_keypath = Profile::settings();
    if let Some(theme) =
        settings_keypath.with_arc_rwlock_mut(&arc_rwlock_profile, |settings| settings.theme.clone())
    {
        println!("✅ New theme after update: {}", theme);
    }

    println!("\n🎯 Performance Comparison");
    println!("------------------------");

    // Test 9: Performance comparison between for_arc_rwlock() and with_arc_rwlock()
    println!("\n9️⃣  Performance Comparison");
    println!("-------------------------");

    let name_keypath = User::name();

    // Method 1: Using for_arc_rwlock() (clones the value)
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let arc_rwlock_name_keypath = name_keypath.clone().for_arc_rwlock();
        let _ = arc_rwlock_name_keypath.get_failable_owned(arc_rwlock_user.clone());
    }
    let for_arc_rwlock_time = start.elapsed();

    // Method 2: Using with_arc_rwlock() (no cloning, just reference access)
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = name_keypath
            .clone()
            .with_arc_rwlock(&arc_rwlock_user, |name| name.clone());
    }
    let with_arc_rwlock_time = start.elapsed();

    println!("✅ for_arc_rwlock() time: {:?}", for_arc_rwlock_time);
    println!("✅ with_arc_rwlock() time: {:?}", with_arc_rwlock_time);
    println!(
        "✅ Performance difference: {:.2}x",
        for_arc_rwlock_time.as_nanos() as f64 / with_arc_rwlock_time.as_nanos() as f64
    );

    println!("\n💡 Key Takeaways");
    println!("================");
    println!("1. for_arc_rwlock() creates FailableOwned keypaths that clone values");
    println!("2. with_arc_rwlock() provides no-clone access via closures");
    println!("3. with_arc_rwlock_mut() enables safe mutable access");
    println!("4. Use for_arc_rwlock() when you need to store the adapted keypath");
    println!("5. Use with_arc_rwlock() for better performance when you don't need to store");
    println!("6. Both methods handle Arc<RwLock> poisoning gracefully");
    println!("7. Deep nesting works seamlessly with both approaches");
    println!("8. Type safety is maintained throughout all operations");
}
