#[cfg(feature = "tagged_core")]
#[cfg(feature = "tagged_core")]
use chrono::{DateTime, Utc};
#[cfg(feature = "tagged_core")]
use keypaths_proc::Kp;
#[cfg(feature = "tagged_core")]
use tagged_core::Tagged;
#[cfg(feature = "tagged_core")]
use uuid::Uuid;

#[cfg(feature = "tagged_core")]
#[derive(Debug, Clone, Kp)]
struct SomeStruct {
    id: Tagged<Uuid, ()>,
    time_id: Tagged<DateTime<Utc>, ()>,
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
    println!("=== Tagged Test Struct Example ===\n");

    // Create a test instance
    let test_struct = SomeStruct::new(Uuid::new_v4(), Utc::now());

    println!("Created struct: {:?}", test_struct);

    // Test direct keypath access to Tagged fields
    println!("\n1. Direct access to Tagged fields:");
    if let Some(id) = SomeStruct::id_r().to_optional().get(&test_struct) {
        println!("   ID: {}", id);
    }

    if let Some(time) = SomeStruct::time_id_r().to_optional().get(&test_struct) {
        println!("   Time: {}", time);
    }

    // Test using with_tagged for direct access (this doesn't make sense here since the field is already Tagged)
    // Instead, let's demonstrate with_tagged by wrapping the entire struct in Tagged
    println!("\n2. Using with_tagged on Tagged<SomeStruct, ()>:");
    let tagged_struct: Tagged<SomeStruct, ()> = Tagged::new(test_struct.clone());

    // Now we can use for_tagged to adapt the keypath to work with Tagged<SomeStruct, ()>
    // let id_path = SomeStruct::id_r().to_optional().for_tagged::<()>();
    // if let Some(id) = id_path.get(&tagged_struct) {
    //     println!("   ID from Tagged<SomeStruct>: {}", id);
    // }

    // Test using with_tagged for no-clone access
    // println!("\n3. Using with_tagged for no-clone access:");
    // SomeStruct::id_r().with_tagged(&tagged_struct, |id| {
    //     println!("   ID: {}", id);
    // });

    // SomeStruct::time_id_r().with_tagged(&tagged_struct, |time| {
    //     println!("   Time: {}", time);
    // });
    //
    // // Test composition with Tagged wrapper
    // println!("\n4. Testing composition with Tagged wrapper:");
    // let id_string_path = SomeStruct::id_r().for_tagged::<()>();
    // if let Some(id) = id_string_path.get(&tagged_struct) {
    //     println!("   ID as string: {}", id.to_string());
    // }
    //
    // // Test with Option<Tagged<T>>
    // println!("\n5. Testing with Option<Tagged<T>>:");
    // let maybe_struct: Option<Tagged<SomeStruct, ()>> = Some(Tagged::new(test_struct.clone()));
    // let option_id_path = SomeStruct::id_r().for_tagged::<()>().for_option();
    //
    // if let Some(id) = option_id_path.get(&maybe_struct) {
    //     println!("   Optional ID: {}", id);
    // }

    // Test with Vec<Tagged<T>>
    println!("\n6. Testing with Vec<Tagged<T>>:");
    let structs: Vec<Tagged<SomeStruct, ()>> = vec![
        Tagged::new(SomeStruct::new(Uuid::new_v4(), Utc::now())),
        Tagged::new(SomeStruct::new(Uuid::new_v4(), Utc::now())),
    ];

    // let id_path = SomeStruct::id_r();
    // for (i, tagged_struct) in structs.iter().enumerate() {
    //     id_path.with_tagged(tagged_struct, |id| {
    //         println!("   Struct {} ID: {}", i + 1, id);
    //     });
    // }

    println!("\n✅ Tagged test struct example completed!");
}

#[cfg(not(feature = "tagged_core"))]
fn main() {
    println!("⚠️  Tagged support requires the 'tagged_core' feature");
    println!("   Enable with: cargo run --example tagged_test_struct --features tagged_core");
}
