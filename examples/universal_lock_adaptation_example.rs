use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::Kp;
use std::sync::Arc;
use parking_lot::{RwLock, Mutex};

#[derive(Kp, Clone)]
#[All]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

#[derive(Kp, Clone)]
#[All]
struct Profile {
    user: User,
    bio: String,
}

fn main() {
    println!("🔒 Universal Lock Adaptation Example");
    println!("====================================");
    
    // Create data wrapped in parking_lot synchronization primitives
    let user = User {
        name: "Akash".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
    };
    
    let profile = Profile {
        user: user.clone(),
        bio: "Software engineer with passion for Rust".to_string(),
    };
    
    let parking_mutex_user = Arc::new(Mutex::new(user));
    let parking_rwlock_profile = Arc::new(RwLock::new(profile));
    
    println!("\n📝 Working with parking_lot::Mutex");
    println!("----------------------------------");
    
    // Method 1: Direct access with parking_lot::Mutex
    let name_keypath = User::name_r();
    let name_keypath_w = User::name_w();
    
    // Access name through parking_lot::Mutex
    {
        let guard = parking_mutex_user.lock();
        let name = name_keypath.get(&*guard);
        println!("✅ Name from parking_lot::Mutex: {}", name);
    }
    
    // Modify name through parking_lot::Mutex
    {
        let mut guard = parking_mutex_user.lock();
        let name = name_keypath_w.get_mut(&mut *guard);
        *name = "Akash Updated".to_string();
        println!("✅ Updated name in parking_lot::Mutex: {}", name);
    }
    
    println!("\n📝 Working with parking_lot::RwLock");
    println!("-----------------------------------");
    
    // Method 2: Direct access with parking_lot::RwLock
    let bio_keypath = Profile::bio_r();
    let bio_keypath_w = Profile::bio_w();
    let user_name_keypath = Profile::user_r().to_optional().then(User::name_r().to_optional());
    
    // Read access through parking_lot::RwLock
    {
        let guard = parking_rwlock_profile.read();
        let bio = bio_keypath.get(&*guard);
        println!("✅ Bio from parking_lot::RwLock: {}", bio);
        
        if let Some(name) = user_name_keypath.get(&*guard) {
            println!("✅ Nested name from parking_lot::RwLock: {}", name);
        }
    }
    
    // Write access through parking_lot::RwLock
    {
        let mut guard = parking_rwlock_profile.write();
        let bio = bio_keypath_w.get_mut(&mut *guard);
        *bio = "Senior software engineer with passion for Rust and systems programming".to_string();
        println!("✅ Updated bio in parking_lot::RwLock: {}", bio);
    }
    
    println!("\n🔧 Creating Universal Lock Adapters");
    println!("-----------------------------------");
    
    // Method 3: Create adapter functions for universal locks
    let name_keypath = User::name_r();
    
    // Adapter for parking_lot::Mutex
    fn parking_mutex_adapter<F>(keypath: KeyPath<User, String, impl for<'r> Fn(&'r User) -> &'r String>, mutex: &Mutex<User>, f: F) 
    where F: FnOnce(&String) {
        let guard = mutex.lock();
        let value = keypath.get(&*guard);
        f(value);
    }
    
    // Adapter for parking_lot::RwLock
    fn parking_rwlock_adapter<F>(keypath: KeyPath<Profile, String, impl for<'r> Fn(&'r Profile) -> &'r String>, rwlock: &RwLock<Profile>, f: F) 
    where F: FnOnce(&String) {
        let guard = rwlock.read();
        let value = keypath.get(&*guard);
        f(value);
    }
    
    // Use the adapters
    parking_mutex_adapter(name_keypath, &parking_mutex_user, |name| {
        println!("✅ Adapter - Name from parking_lot::Mutex: {}", name);
    });
    
    parking_rwlock_adapter(bio_keypath, &parking_rwlock_profile, |bio| {
        println!("✅ Adapter - Bio from parking_lot::RwLock: {}", bio);
    });
    
    println!("\n🔄 Simple Universal Lock Adapter");
    println!("--------------------------------");
    
    // Method 4: Simple adapter that works with parking_lot locks
    fn with_parking_mutex<T, V, F, R>(
        keypath: KeyPath<T, V, impl for<'r> Fn(&'r T) -> &'r V>,
        mutex: &Mutex<T>,
        f: F,
    ) -> R
    where
        F: FnOnce(&V) -> R,
    {
        let guard = mutex.lock();
        f(keypath.get(&*guard))
    }
    
    fn with_parking_rwlock<T, V, F, R>(
        keypath: KeyPath<T, V, impl for<'r> Fn(&'r T) -> &'r V>,
        rwlock: &RwLock<T>,
        f: F,
    ) -> R
    where
        F: FnOnce(&V) -> R,
    {
        let guard = rwlock.read();
        f(keypath.get(&*guard))
    }
    
    // Use the simple adapters
    {
        let name_keypath = User::name_r();
        let name = with_parking_mutex(name_keypath, &parking_mutex_user, |name: &String| name.clone());
        println!("✅ Simple adapter - Name from parking_lot::Mutex: {}", name);
    }
    
    {
        let bio_keypath = Profile::bio_r();
        let bio = with_parking_rwlock(bio_keypath, &parking_rwlock_profile, |bio: &String| bio.clone());
        println!("✅ Simple adapter - Bio from parking_lot::RwLock: {}", bio);
    }
    
    println!("\n🎯 Advanced: Working with Nested KeyPaths");
    println!("----------------------------------------");
    
    // Demonstrate composition with nested keypaths using direct access
    let nested_name_keypath = Profile::user_r().to_optional().then(User::name_r().to_optional());
    {
        let guard = parking_rwlock_profile.read();
        if let Some(name) = nested_name_keypath.get(&*guard) {
            println!("✅ Nested name from parking_lot::RwLock: {}", name);
        }
    }
    
    // Demonstrate working with Option fields
    let email_keypath = User::email_fr();
    {
        let guard = parking_mutex_user.lock();
        if let Some(email) = email_keypath.get(&*guard) {
            println!("✅ Email from parking_lot::Mutex: {}", email);
        } else {
            println!("✅ No email in user");
        }
    }
    
    println!("\n💡 Key Takeaways:");
    println!("==================");
    println!("1. Direct access: Use lock guards with keypath.get()/get_mut()");
    println!("2. Adapter functions: Create simple functions that handle locking");
    println!("3. Generic adapters: Use traits to work with multiple lock types");
    println!("4. Composable adapters: Create reusable adapter structs");
    println!("5. parking_lot provides better performance than std::sync primitives");
    println!("6. Universal adapters work with any lock that implements the trait");
}
