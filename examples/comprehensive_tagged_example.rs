#[cfg(feature = "tagged_core")]
use tagged_core::Tagged;
#[cfg(feature = "tagged_core")]
use keypaths_proc::Keypaths;
#[cfg(feature = "tagged_core")]

#[cfg(feature = "tagged_core")]
use chrono::{DateTime, Utc};
#[cfg(feature = "tagged_core")]
use uuid::Uuid;

// Define tag types for type safety
#[cfg(feature = "tagged_core")]
struct UserIdTag;
#[cfg(feature = "tagged_core")]
struct TimestampTag;

#[cfg(feature = "tagged_core")]
#[derive(Debug, Clone, Keypaths)]
struct SomeStruct {
    id: Tagged<Uuid, UserIdTag>,
    time_id: Tagged<DateTime<Utc>, TimestampTag>,
}

#[cfg(feature = "tagged_core")]
impl SomeStruct {
    fn new(id: Uuid, time: DateTime<Utc>) -> Self {
        Self {
            id: Tagged::new(id),
            time_id: Tagged::new(time),
        }
    }
}

#[cfg(feature = "tagged_core")]
fn main() {
    println!("=== Comprehensive Tagged Example ===\n");
    
    // Create test instances
    let struct1 = SomeStruct::new(Uuid::new_v4(), Utc::now());
    let struct2 = SomeStruct::new(Uuid::new_v4(), Utc::now());
    
    println!("Created structs:");
    println!("  Struct 1: {:?}", struct1);
    println!("  Struct 2: {:?}", struct2);
    
    // 1. Direct access to Tagged fields (most common use case)
    println!("\n1. Direct access to Tagged fields:");
    if let Some(id) = SomeStruct::id_r().get(&struct1) {
        println!("   Struct 1 ID: {}", id);
    }
    
    if let Some(time) = SomeStruct::time_id_r().get(&struct1) {
        println!("   Struct 1 Time: {}", time);
    }
    
    // 2. Working with collections of Tagged structs
    println!("\n2. Working with Vec<SomeStruct> containing Tagged fields:");
    let structs = vec![struct1.clone(), struct2.clone()];
    
    for (i, s) in structs.iter().enumerate() {
        if let Some(id) = SomeStruct::id_r().get(&s) {
            println!("   Struct {} ID: {}", i + 1, id);
        }
    }
    
    // 3. Using for_tagged when the entire struct is wrapped in Tagged
    println!("\n3. Using for_tagged when struct is wrapped in Tagged:");
    let tagged_struct: Tagged<SomeStruct, ()> = Tagged::new(struct1.clone());
    
    let id_path = SomeStruct::id_r().for_tagged::<()>();
    let time_path = SomeStruct::time_id_r().for_tagged::<()>();
    
    if let Some(id) = id_path.get(&tagged_struct) {
        println!("   Wrapped ID: {}", id);
    }
    
    if let Some(time) = time_path.get(&tagged_struct) {
        println!("   Wrapped Time: {}", time);
    }
    
    // 4. Using with_tagged for no-clone access
    println!("\n4. Using with_tagged for no-clone access:");
    SomeStruct::id_r().with_tagged(&tagged_struct, |id| {
        println!("   ID via with_tagged: {}", id);
    });
    
    SomeStruct::time_id_r().with_tagged(&tagged_struct, |time| {
        println!("   Time via with_tagged: {}", time);
    });
    
    // 5. Composition with other containers
    println!("\n5. Composition with Option<Tagged<SomeStruct>>:");
    let maybe_struct: Option<Tagged<SomeStruct, ()>> = Some(Tagged::new(struct2.clone()));
    let option_id_path = SomeStruct::id_r().for_tagged::<()>().for_option();
    
    if let Some(id) = option_id_path.get(&maybe_struct) {
        println!("   Optional wrapped ID: {}", id);
    }
    
    // 6. Working with Vec<Tagged<SomeStruct>>
    println!("\n6. Working with Vec<Tagged<SomeStruct>>:");
    let tagged_structs: Vec<Tagged<SomeStruct, ()>> = vec![
        Tagged::new(SomeStruct::new(Uuid::new_v4(), Utc::now())),
        Tagged::new(SomeStruct::new(Uuid::new_v4(), Utc::now())),
    ];
    
    let id_path = SomeStruct::id_r();
    for (i, tagged_struct) in tagged_structs.iter().enumerate() {
        id_path.clone().with_tagged(tagged_struct, |id| {
            println!("   Tagged Struct {} ID: {}", i + 1, id);
        });
    }
    
    // 7. Demonstrating type safety with different tag types
    println!("\n7. Type safety with different tag types:");
    let user_id: Tagged<Uuid, UserIdTag> = Tagged::new(Uuid::new_v4());
    let timestamp: Tagged<DateTime<Utc>, TimestampTag> = Tagged::new(Utc::now());
    
    // These are different types even though they contain the same inner type
    println!("   User ID: {}", *user_id);
    println!("   Timestamp: {}", *timestamp);
    
    // 8. Complex composition example
    println!("\n8. Complex composition example:");
    let complex_struct = SomeStruct::new(Uuid::new_v4(), Utc::now());
    let wrapped_complex: Tagged<SomeStruct, ()> = Tagged::new(complex_struct);
    let maybe_wrapped: Option<Tagged<SomeStruct, ()>> = Some(wrapped_complex);
    
    // Chain multiple adapters
    let complex_path = SomeStruct::id_r()
        .for_tagged::<()>()  // Adapt to work with Tagged<SomeStruct, ()>
        .for_option();       // Then adapt to work with Option<Tagged<...>>
    
    if let Some(id) = complex_path.get(&maybe_wrapped) {
        println!("   Complex composition ID: {}", id);
    }
    
    println!("\n✅ Comprehensive tagged example completed!");
}

#[cfg(not(feature = "tagged_core"))]
fn main() {
    println!("⚠️  Tagged support requires the 'tagged_core' feature");
    println!("   Enable with: cargo run --example comprehensive_tagged_example --features tagged_core");
}
