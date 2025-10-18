use key_paths_derive::Keypaths;
use key_paths_core::WithContainer;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Keypaths, Clone, Debug)]
struct SomeOtherStruct {
    value: String,
    count: u32,
}

#[derive(Keypaths, Clone, Debug)]
struct SomeStruct {
    field1: Arc<RwLock<SomeOtherStruct>>,
    field2: Arc<Mutex<SomeOtherStruct>>,
}

fn main() {
    println!("🔧 Arc<Sync> Derive Macro Support Example");
    println!("=========================================");

    // Create test data
    let some_struct = SomeStruct {
        field1: Arc::new(RwLock::new(SomeOtherStruct {
            value: "Hello from RwLock".to_string(),
            count: 42,
        })),
        field2: Arc::new(Mutex::new(SomeOtherStruct {
            value: "Hello from Mutex".to_string(),
            count: 24,
        })),
    };

    println!("\n🎯 Testing Arc<RwLock<T>> Field Access");
    println!("-------------------------------------");

    // Test Arc<RwLock<T>> field access
    let field1_path = SomeStruct::field1_r();
    if let Some(field1_ref) = field1_path.get_ref(&&some_struct) {
        println!("✅ Arc<RwLock<SomeOtherStruct>> field accessible: {:?}", field1_ref);
    }

    // Test Arc<Mutex<T>> field access
    let field2_path = SomeStruct::field2_r();
    if let Some(field2_ref) = field2_path.get_ref(&&some_struct) {
        println!("✅ Arc<Mutex<SomeOtherStruct>> field accessible: {:?}", field2_ref);
    }

    println!("\n🎯 Testing with WithContainer Trait");
    println!("----------------------------------");

    // Test with WithContainer trait for no-clone access
    let value_path = SomeOtherStruct::value_r();
    let count_path = SomeOtherStruct::count_r();

    // Access through Arc<RwLock<T>> - we need to get the field first, then use with_rwlock
    if let Some(arc_rwlock_field) = field1_path.get_ref(&&some_struct) {
        value_path.clone().with_rwlock(arc_rwlock_field, |value| {
            println!("✅ Value from Arc<RwLock<SomeOtherStruct>>: {}", value);
        });
        count_path.clone().with_rwlock(arc_rwlock_field, |count| {
            println!("✅ Count from Arc<RwLock<SomeOtherStruct>>: {}", count);
        });
    }

    // Access through Arc<Mutex<T>> - we need to get the field first, then use with_mutex
    if let Some(arc_mutex_field) = field2_path.get_ref(&&some_struct) {
        value_path.with_mutex(arc_mutex_field, |value| {
            println!("✅ Value from Arc<Mutex<SomeOtherStruct>>: {}", value);
        });
        count_path.with_mutex(arc_mutex_field, |count| {
            println!("✅ Count from Arc<Mutex<SomeOtherStruct>>: {}", count);
        });
    }

    println!("\n🎯 Testing with Aggregator Functions");
    println!("-----------------------------------");

    // Test with aggregator functions (requires parking_lot feature)
    #[cfg(feature = "parking_lot")]
    {
        let value_arc_rwlock_path = value_path.for_arc_rwlock();
        let count_arc_rwlock_path = count_path.for_arc_rwlock();
        let value_arc_mutex_path = value_path.for_arc_mutex();
        let count_arc_mutex_path = count_path.for_arc_mutex();

        if let Some(value) = value_arc_rwlock_path.get_failable_owned(some_struct.field1.clone()) {
            println!("✅ Value from Arc<RwLock<SomeOtherStruct>> (aggregator): {}", value);
        }

        if let Some(count) = count_arc_rwlock_path.get_failable_owned(some_struct.field1.clone()) {
            println!("✅ Count from Arc<RwLock<SomeOtherStruct>> (aggregator): {}", count);
        }

        if let Some(value) = value_arc_mutex_path.get_failable_owned(some_struct.field2.clone()) {
            println!("✅ Value from Arc<Mutex<SomeOtherStruct>> (aggregator): {}", value);
        }

        if let Some(count) = count_arc_mutex_path.get_failable_owned(some_struct.field2.clone()) {
            println!("✅ Count from Arc<Mutex<SomeOtherStruct>> (aggregator): {}", count);
        }
    }

    #[cfg(not(feature = "parking_lot"))]
    {
        println!("⚠️  Parking lot feature not enabled - aggregator functions not available");
        println!("   Enable with: cargo run --example arc_sync_derive_example --features parking_lot");
    }

    println!("\n💡 Key Takeaways");
    println!("================");
    println!("1. Derive macro now supports Arc<RwLock<T>> and Arc<Mutex<T>> fields");
    println!("2. Generated methods provide container-level access (field1_r(), field2_r())");
    println!("3. Use WithContainer trait for no-clone access to inner values");
    println!("4. Use aggregator functions (with parking_lot feature) for clone-based access");
    println!("5. Arc<Mutex<T>> and Arc<RwLock<T>> don't support writable access (Arc is immutable)");
    println!("6. Direct access to inner types requires proper lock handling");
}
