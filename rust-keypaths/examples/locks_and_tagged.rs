// This example demonstrates support for locks and tagged types
// Run with: cargo run --example locks_and_tagged --features "tagged,parking_lot"

use rust_keypaths::containers;
use std::sync::{Arc, Mutex, RwLock};

#[cfg(feature = "tagged")]
use tagged_core::Tagged;

#[cfg(feature = "parking_lot")]
use parking_lot::{Mutex as ParkingMutex, RwLock as ParkingRwLock};

#[derive(Debug)]
struct Data {
    value: i32,
    name: String,
}

fn main() {
    println!("=== Locks and Tagged Types Example ===\n");

    // ========== Mutex Examples ==========
    println!("1. Mutex Examples:");
    let mutex_data = Mutex::new(Data {
        value: 42,
        name: "MutexData".to_string(),
    });

    // Use helper function to lock and access
    if let Some(guard) = containers::lock_mutex(&mutex_data) {
        println!("  Mutex value: {}, name: {}", guard.value, guard.name);
    }

    // ========== RwLock Examples ==========
    println!("\n2. RwLock Examples:");
    let rwlock_data = RwLock::new(Data {
        value: 100,
        name: "RwLockData".to_string(),
    });

    // Read access
    if let Some(guard) = containers::read_rwlock(&rwlock_data) {
        println!("  RwLock (read) value: {}, name: {}", guard.value, guard.name);
    }

    // Write access
    if let Some(mut guard) = containers::write_rwlock(&rwlock_data) {
        guard.value = 200;
        println!("  RwLock (write) updated value to: {}", guard.value);
    }

    // ========== Arc<Mutex<T>> Examples ==========
    println!("\n3. Arc<Mutex<T>> Examples:");
    let arc_mutex_data = Arc::new(Mutex::new(Data {
        value: 300,
        name: "ArcMutexData".to_string(),
    }));

    if let Some(guard) = containers::lock_arc_mutex(&arc_mutex_data) {
        println!("  Arc<Mutex> value: {}, name: {}", guard.value, guard.name);
    }

    // ========== Arc<RwLock<T>> Examples ==========
    println!("\n4. Arc<RwLock<T>> Examples:");
    let arc_rwlock_data = Arc::new(RwLock::new(Data {
        value: 400,
        name: "ArcRwLockData".to_string(),
    }));

    if let Some(guard) = containers::read_arc_rwlock(&arc_rwlock_data) {
        println!("  Arc<RwLock> (read) value: {}, name: {}", guard.value, guard.name);
    }

    // ========== Weak References ==========
    println!("\n5. Weak Reference Examples:");
    let arc_data = Arc::new(Data {
        value: 500,
        name: "ArcData".to_string(),
    });
    let weak_data = Arc::downgrade(&arc_data);

    if let Some(upgraded) = containers::upgrade_weak(&weak_data) {
        println!("  Weak upgraded to Arc, value: {}", upgraded.value);
    }

    // ========== Tagged Types ==========
    #[cfg(feature = "tagged")]
    {
        println!("\n6. Tagged Types Examples:");
        
        // Note: Tagged types require Deref/DerefMut implementation
        // This example shows the API, but actual usage depends on how Tagged is implemented
        println!("  Tagged type support is available via containers::for_tagged()");
        println!("  Usage: containers::for_tagged::<Tag, T>() where Tagged<Tag, T>: Deref<Target = T>");
    }

    // ========== Parking Lot (if enabled) ==========
    #[cfg(feature = "parking_lot")]
    {
        println!("\n7. Parking Lot Examples:");
        
        let parking_mutex = ParkingMutex::new(Data {
            value: 600,
            name: "ParkingMutexData".to_string(),
        });

        let guard = containers::lock_parking_mutex(&parking_mutex);
        println!("  Parking Mutex value: {}, name: {}", guard.value, guard.name);
        drop(guard);

        let parking_rwlock = ParkingRwLock::new(Data {
            value: 700,
            name: "ParkingRwLockData".to_string(),
        });

        let read_guard = containers::read_parking_rwlock(&parking_rwlock);
        println!("  Parking RwLock (read) value: {}, name: {}", read_guard.value, read_guard.name);
        drop(read_guard);

        let mut write_guard = containers::write_parking_rwlock(&parking_rwlock);
        write_guard.value = 800;
        println!("  Parking RwLock (write) updated value to: {}", write_guard.value);
    }

    println!("\n✅ All lock and tagged type examples completed!");
}

