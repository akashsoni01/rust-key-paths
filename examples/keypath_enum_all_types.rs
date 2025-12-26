use keypaths_proc::Keypaths;
use rust_key_paths::{KP, PKP, AKP, AMKP, ARKP, OAMKP, OARKP};
use rust_keypaths::OptionalKeyPath;

#[derive(Keypaths)]
#[All]
struct User {
    name: String,
    age: u32,
    metadata: Option<String>,
}

// ========== Real-Life Usage Examples ==========

// Example 1: API endpoint that accepts any keypath type
// Real-world scenario: A REST API that allows clients to specify which field to read
fn api_get_field<Root>(kp: KP<Root>, root: &Root) -> String {
    match kp {
        KP::KeyPath(k) => {
            // In production: Serialize the accessed value to JSON
            format!("Field accessed via KeyPath")
        },
        KP::OptionalKeyPath(k) => {
            // In production: Handle None case gracefully
            format!("Field accessed via OptionalKeyPath")
        },
        KP::WritableKeyPath(_k) => {
            // In production: Return error - this endpoint is read-only
            format!("Writable keypath not allowed in read-only endpoint")
        },
        KP::WritableOptionalKeyPath(_k) => {
            format!("Writable optional keypath not allowed in read-only endpoint")
        },
    }
}

// Example 2: Configuration system that stores keypaths
// Real-world scenario: A settings system where users can configure which fields to track
struct Config<Root> {
    tracked_fields: Vec<PKP<Root>>,  // Store multiple partial keypaths
}

impl<Root> Config<Root> {
    fn new() -> Self {
        Self {
            tracked_fields: Vec::new(),
        }
    }
    
    fn add_tracked_field(&mut self, pkp: PKP<Root>) {
        // In production: Validate the keypath, check for duplicates, etc.
        self.tracked_fields.push(pkp);
    }
    
    fn get_tracked_count(&self) -> usize {
        self.tracked_fields.len()
    }
}

// Example 3: Generic serializer that works with any keypath
// Real-world scenario: A logging/monitoring system that needs to serialize values
fn serialize_field<Root>(pkp: PKP<Root>, root: &Root) -> Option<String> {
    match pkp {
        PKP::PartialKeyPath(k) => {
            // In production: Use serde or similar to serialize the value
            Some(format!("TypeId: {:?}", k.value_type_id()))
        },
        PKP::PartialOptionalKeyPath(k) => {
            Some(format!("Optional TypeId: {:?}", k.value_type_id()))
        },
        PKP::PartialWritableKeyPath(_k) => {
            Some("Writable keypath - read-only serialization".to_string())
        },
        PKP::PartialWritableOptionalKeyPath(_k) => {
            Some("Writable optional keypath - read-only serialization".to_string())
        },
    }
}

// Example 4: Plugin system with fully type-erased keypaths
// Real-world scenario: A plugin architecture where plugins can work with any data type
struct Plugin {
    name: String,
    keypath: AKP,  // Can work with any root and value type
}

impl Plugin {
    fn new(name: String, keypath: AKP) -> Self {
        Self { name, keypath }
    }
    
    fn process(&self) {
        // In production: Use the keypath to access data in a generic way
        match &self.keypath {
            AKP::AnyKeyPath(_k) => println!("Plugin {} uses AnyKeyPath", self.name),
            AKP::AnyWritableKeyPath(_k) => println!("Plugin {} uses AnyWritableKeyPath", self.name),
        }
    }
}

// Example 5: Thread-safe data access with chain keypaths
// Real-world scenario: A multi-threaded application accessing shared state
fn safe_read<Root>(chain: ARKP<Root>, root: &Root) -> Result<String, String> {
    match chain {
        ARKP::ArcRwLockKeyPathChain(_k) => {
            // In production: Use the chain to safely read through Arc<RwLock<T>>
            Ok("Read successful via ArcRwLockKeyPathChain".to_string())
        },
        ARKP::ArcRwLockOptionalKeyPathChain(_k) => {
            Ok("Read successful via ArcRwLockOptionalKeyPathChain".to_string())
        },
    }
}

// Example functions using the enum types for syntactic sugar
fn process_keypath<Root>(kp: KP<Root>) {
    match kp {
        KP::KeyPath(_k) => println!("  Processing KeyPath"),
        KP::OptionalKeyPath(_k) => println!("  Processing OptionalKeyPath"),
        KP::WritableKeyPath(_k) => println!("  Processing WritableKeyPath"),
        KP::WritableOptionalKeyPath(_k) => println!("  Processing WritableOptionalKeyPath"),
    }
}

fn process_partial_keypath<Root>(pkp: PKP<Root>) {
    match pkp {
        PKP::PartialKeyPath(_k) => println!("  Processing PartialKeyPath"),
        PKP::PartialOptionalKeyPath(_k) => println!("  Processing PartialOptionalKeyPath"),
        PKP::PartialWritableKeyPath(_k) => println!("  Processing PartialWritableKeyPath"),
        PKP::PartialWritableOptionalKeyPath(_k) => println!("  Processing PartialWritableOptionalKeyPath"),
    }
}

fn process_any_keypath(anykp: AKP) {
    match anykp {
        AKP::AnyKeyPath(_k) => println!("  Processing AnyKeyPath"),
        AKP::AnyWritableKeyPath(_k) => println!("  Processing AnyWritableKeyPath"),
    }
}

fn main() {
    println!("=== KeyPath Enum - Syntactic Sugar Example ===\n");

    // Example 1: Using KP enum
    println!("1. KP enum - Basic keypath types:");
    let kp1: KP<User> = User::name_r().into();
    let kp2: KP<User> = User::metadata_r().into();
    process_keypath(kp1);
    process_keypath(kp2);

    // Example 2: Using PKP enum
    println!("\n2. PKP enum - Type-erased keypaths:");
    let pkp1: PKP<User> = User::name_r().to_partial().into();
    let pkp2: PKP<User> = User::metadata_r().to_partial().into();
    process_partial_keypath(pkp1);
    process_partial_keypath(pkp2);

    // Example 3: Using AKP enum
    println!("\n3. AKP enum - Fully type-erased keypaths:");
    // Note: to_any() is available on OptionalKeyPath, not on KeyPath
    // For demonstration, we'll create an OptionalKeyPath first
    let opt_kp1 = OptionalKeyPath::new(|u: &User| Some(&u.name));
    let anykp1: AKP = opt_kp1.to_any().into();
    process_any_keypath(anykp1);

    // Example 4: Chain keypaths (demonstration of enum types)
    println!("\n4. Chain keypath enums:");
    println!("  - AMKP<Root> (ArcMutexKeyPathChain) enum available");
    println!("  - ARKP<Root> (ArcRwLockKeyPathChain) enum available");
    println!("  - OAMKP<Root> (OptionalArcMutexKeyPathChain) enum available");
    println!("  - OARKP<Root> (OptionalArcRwLockKeyPathChain) enum available");
    #[cfg(feature = "parking_lot")]
    {
        println!("  - APMKP<Root> (ArcParkingMutexKeyPathChain) enum available (parking_lot feature)");
        println!("  - APRKP<Root> (ArcParkingRwLockKeyPathChain) enum available (parking_lot feature)");
        println!("  - OAPMKP<Root> (OptionalArcParkingMutexKeyPathChain) enum available");
        println!("  - OAPRKP<Root> (OptionalArcParkingRwLockKeyPathChain) enum available");
    }

    // Example 5: Using with actual data
    println!("\n5. Using keypaths with actual data:");
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        metadata: Some("Developer".to_string()),
    };

    if let KP::KeyPath(kp) = User::name_r().into() {
        if let Some(name_ref) = kp.get_as::<String>(&user) {
            println!("  User name: {}", name_ref);
        }
    }

    if let PKP::PartialKeyPath(pkp) = User::age_r().to_partial().into() {
        if let Some(age_ref) = pkp.get_as::<u32>(&user) {
            println!("  User age: {}", age_ref);
        }
    }

    // ========== Real-Life Usage Demonstrations ==========
    println!("\n6. Real-Life Usage Examples:");
    
    // Example: Configuration system
    let mut config: Config<User> = Config::new();
    config.add_tracked_field(User::name_r().to_partial().into());
    config.add_tracked_field(User::age_r().to_partial().into());
    println!("  Configuration tracks {} fields", config.get_tracked_count());
    
    // Example: API endpoint simulation
    let api_result = api_get_field(User::name_r().into(), &user);
    println!("  API result: {}", api_result);
    
    // Example: Plugin system
    let opt_kp2 = OptionalKeyPath::new(|u: &User| Some(&u.name));
    let plugin = Plugin::new("NameExtractor".to_string(), opt_kp2.to_any().into());
    plugin.process();
    
    // Example: Serialization
    if let Some(serialized) = serialize_field(User::name_r().to_partial().into(), &user) {
        println!("  Serialized: {}", serialized);
    }

    println!("\n✅ All enum examples completed successfully!");
}
