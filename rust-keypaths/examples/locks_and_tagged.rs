// This example demonstrates support for locks and tagged types
// Run with: cargo run --example locks_and_tagged --features "tagged,parking_lot"

use rust_keypaths::{containers, OptionalKeyPath};
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

#[derive(Debug)]
struct Level2 {
    data: Arc<RwLock<Data>>,
    metadata: Option<String>,
}

#[derive(Debug)]
struct Level1 {
    level2: Option<Level2>,
    count: usize,
}

#[derive(Debug)]
struct Root {
    level1: Option<Level1>,
    id: u64,
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

    // ========== Complex Chaining Example ==========
    println!("\n8. Complex Chaining: Root -> Level1 -> Level2 -> Arc<RwLock<Data>>:");
    
    let root = Root {
        level1: Some(Level1 {
            level2: Some(Level2 {
                data: Arc::new(RwLock::new(Data {
                    value: 900,
                    name: "ChainedData".to_string(),
                })),
                metadata: Some("Some metadata".to_string()),
            }),
            count: 42,
        }),
        id: 1,
    };

    // Chain keypaths: Root -> Option<Level1> -> Option<Level2> -> Arc<RwLock<Data>>
    let root_to_level1 = OptionalKeyPath::new(|r: &Root| r.level1.as_ref());
    let level1_to_level2 = OptionalKeyPath::new(|l1: &Level1| l1.level2.as_ref());
    let level2_to_arc_rwlock = OptionalKeyPath::new(|l2: &Level2| Some(&l2.data));
    
    // Chain all the way to Arc<RwLock<Data>>
    let chained_to_arc = root_to_level1
        .then(level1_to_level2)
        .then(level2_to_arc_rwlock);
    
    // Access the Arc<RwLock<Data>> and then lock it
    if let Some(arc_rwlock) = chained_to_arc.get(&root) {
        println!("  Successfully chained to Arc<RwLock<Data>>");
        
        // Now use the helper function to read the data
        if let Some(guard) = containers::read_arc_rwlock(arc_rwlock) {
            println!("  Chained read - value: {}, name: {}", guard.value, guard.name);
        }
        
        // Write access through the chain
        if let Some(mut guard) = containers::write_arc_rwlock(arc_rwlock) {
            guard.value = 1000;
            guard.name = "UpdatedChainedData".to_string();
            println!("  Chained write - updated value: {}, name: {}", guard.value, guard.name);
        }
    }

    // ========== Even More Complex: Accessing nested field through chain ==========
    println!("\n9. Ultra Complex Chaining: Root -> Level1 -> Level2 -> Arc<RwLock<Data>> -> Data.value:");
    
    // Create a new root for this example
    let mut root2 = Root {
        level1: Some(Level1 {
            level2: Some(Level2 {
                data: Arc::new(RwLock::new(Data {
                    value: 2000,
                    name: "UltraChainedData".to_string(),
                })),
                metadata: Some("More metadata".to_string()),
            }),
            count: 100,
        }),
        id: 2,
    };

    // Chain to get Arc<RwLock<Data>>, then access the value field
    // Note: We need to lock first, then access the field
    let root_to_level1_2 = OptionalKeyPath::new(|r: &Root| r.level1.as_ref());
    let level1_to_level2_2 = OptionalKeyPath::new(|l1: &Level1| l1.level2.as_ref());
    let level2_to_arc_rwlock_2 = OptionalKeyPath::new(|l2: &Level2| Some(&l2.data));
    
    let chained_to_arc_2 = root_to_level1_2
        .then(level1_to_level2_2)
        .then(level2_to_arc_rwlock_2);
    
    // Access and modify through the complete chain
    if let Some(arc_rwlock) = chained_to_arc_2.get(&root2) {
        // Read the value field through the chain
        if let Some(guard) = containers::read_arc_rwlock(arc_rwlock) {
            println!("  Ultra chained read - value: {}, name: {}", guard.value, guard.name);
        }
        
        // Modify the value field through the chain
        if let Some(mut guard) = containers::write_arc_rwlock(arc_rwlock) {
            guard.value = 3000;
            println!("  Ultra chained write - updated value: {}", guard.value);
        }
        
        // Verify the change
        if let Some(guard) = containers::read_arc_rwlock(arc_rwlock) {
            println!("  Verification - final value: {}, name: {}", guard.value, guard.name);
        }
    }

    // ========== Chaining with metadata access ==========
    println!("\n10. Chaining to Optional Field: Root -> Level1 -> Level2 -> metadata:");
    
    let root_to_level1_3 = OptionalKeyPath::new(|r: &Root| r.level1.as_ref());
    let level1_to_level2_3 = OptionalKeyPath::new(|l1: &Level1| l1.level2.as_ref());
    let level2_to_metadata = OptionalKeyPath::new(|l2: &Level2| l2.metadata.as_ref());
    
    let chained_to_metadata = root_to_level1_3
        .then(level1_to_level2_3)
        .then(level2_to_metadata);
    
    if let Some(metadata) = chained_to_metadata.get(&root2) {
        println!("  Chained to metadata: {}", metadata);
    }

    println!("\n✅ All lock and tagged type examples completed!");
}

