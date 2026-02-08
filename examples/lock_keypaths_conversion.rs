use keypaths_proc::Kp;
use rust_keypaths::KeyPath;
use std::sync::{Arc, RwLock};

#[derive(Kp)]
#[All]
struct SomeStruct {
    data: String,
}

#[derive(Kp)]
#[All]
struct Container {
    rwlock_data: Arc<std::sync::RwLock<SomeStruct>>,
}

fn main() {
    let container = Container {
        rwlock_data: Arc::new(RwLock::new(SomeStruct {
            data: "test".to_string(),
        })),
    };

    // Using the new to_arc_rwlock_kp() method directly on keypaths
    println!("=== Using to_arc_rwlock_kp() method ===");

    // Convert a normal keypath to a lock keypath using the method
    Container::rwlock_data_r()
        .to_arc_rwlock_kp()
        .chain_arc_rwlock_at_kp(SomeStruct::data_r())
        .get(&container, |value| {
            println!("✅ Read value via to_arc_rwlock_kp(): {}", value);
        });

    // Direct usage (without conversion method) - still works
    println!("\n=== Direct usage (without conversion) ===");
    Container::rwlock_data_r()
        .chain_arc_rwlock_at_kp(SomeStruct::data_r())
        .get(&container, |value| {
            println!("✅ Read value directly: {}", value);
        });

    println!("\n✅ to_arc_rwlock_kp() method is working correctly!");
}
