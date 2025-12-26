use keypaths_proc::Keypaths;
use rust_key_paths::{KP, PKP};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

/// Shared application state that needs to be synchronized across microservices
#[derive(Keypaths, Clone, Debug)]
#[All]
struct AppState {
    version: String,
    user_count: u32,
    active_connections: u32,
    config: Config,
}

#[derive(Keypaths, Clone, Debug)]
#[All]
struct Config {
    max_users: u32,
    timeout_seconds: u32,
    feature_flags: HashMap<String, bool>,
}

/// Message type for broadcasting updates across microservices
#[derive(Clone, Debug)]
enum UpdateMessage {
    /// Update a field using a keypath
    FieldUpdate {
        keypath_id: String,
        value: String, // Serialized value
    },
    /// Full state sync
    FullSync(AppState),
}

/// Microservice node that maintains local state and can receive updates
struct MicroserviceNode {
    id: String,
    state: Arc<RwLock<AppState>>,
    update_handler: Box<dyn Fn(&AppState, &str) + Send + Sync>,
}

impl MicroserviceNode {
    fn new(id: String, initial_state: AppState) -> Self {
        let state = Arc::new(RwLock::new(initial_state));
        let id_clone = id.clone();
        let update_handler: Box<dyn Fn(&AppState, &str) + Send + Sync> = 
            Box::new(move |_state, msg| {
                println!("    [{}] Received update: {}", id_clone, msg);
            });
        
        Self {
            id,
            state,
            update_handler,
        }
    }
    
    /// Apply an update using a keypath
    fn apply_update<Root, Value>(&self, kp: KP<Root>, value: Value) -> Result<(), String>
    where
        Root: 'static,
        Value: std::fmt::Debug + Clone + 'static,
    {
        let mut state = self.state.write().unwrap();
        
        match kp {
            KP::KeyPath(_k) => {
                // In production: Deserialize value and apply update
                println!("    [{}] Applying KeyPath update", self.id);
                Ok(())
            },
            KP::OptionalKeyPath(_k) => {
                println!("    [{}] Applying OptionalKeyPath update", self.id);
                Ok(())
            },
            KP::WritableKeyPath(_k) => {
                println!("    [{}] Applying WritableKeyPath update", self.id);
                Ok(())
            },
            KP::WritableOptionalKeyPath(_k) => {
                println!("    [{}] Applying WritableOptionalKeyPath update", self.id);
                Ok(())
            },
        }
    }
    
    /// Get current state (read-only)
    fn get_state(&self) -> AppState {
        self.state.read().unwrap().clone()
    }
}

/// Message broker that routes updates to all microservices
struct MessageBroker {
    nodes: Arc<RwLock<Vec<Arc<MicroserviceNode>>>>,
}

impl MessageBroker {
    fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Register a microservice node
    fn register(&self, node: Arc<MicroserviceNode>) {
        self.nodes.write().unwrap().push(node);
    }
    
    /// Broadcast an update to all registered nodes
    fn broadcast_update<Value>(&self, field_name: &str, value: Value) -> Result<(), String>
    where
        Value: std::fmt::Debug + Clone + Send + Sync + 'static,
    {
        let nodes = self.nodes.read().unwrap();
        println!("  Broadcasting update to {} nodes", nodes.len());
        
        // Create KP for each node (in production, you'd serialize the keypath identifier)
        for node in nodes.iter() {
            // Recreate the KP based on field name for each node
            let kp: KP<AppState> = match field_name {
                "version" => AppState::version_r().into(),
                "user_count" => AppState::user_count_r().into(),
                "active_connections" => AppState::active_connections_r().into(),
                _ => return Err(format!("Unknown field: {}", field_name)),
            };
            node.apply_update(kp, value.clone())?;
        }
        
        Ok(())
    }
    
    /// Get all registered node IDs
    fn get_node_ids(&self) -> Vec<String> {
        self.nodes.read().unwrap().iter().map(|n| n.id.clone()).collect()
    }
}

/// Update coordinator that manages state changes
struct UpdateCoordinator {
    broker: Arc<MessageBroker>,
    state_registry: Arc<RwLock<HashMap<String, PKP<AppState>>>>, // Store keypaths by ID
}

impl UpdateCoordinator {
    fn new(broker: Arc<MessageBroker>) -> Self {
        Self {
            broker,
            state_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a keypath for a specific field
    fn register_keypath(&self, field_name: String, pkp: PKP<AppState>) {
        let field_name_clone = field_name.clone();
        self.state_registry.write().unwrap().insert(field_name, pkp);
        println!("  Registered keypath for field: {}", field_name_clone);
    }
    
    /// Update a field and broadcast to all microservices
    fn update_field(&self, field_name: &str, value: String) -> Result<(), String> {
        println!("\n📡 UpdateCoordinator: Updating field '{}'", field_name);
        
        // Get the registered keypath
        let registry = self.state_registry.read().unwrap();
        if let Some(_pkp) = registry.get(field_name) {
            // In production: Convert PKP to KP and apply the update
            // For this example, we'll simulate the update
            println!("  Found registered keypath for '{}'", field_name);
            
            // Broadcast to all nodes (KP will be created inside broadcast_update)
            self.broker.broadcast_update(field_name, value)?;
            Ok(())
        } else {
            Err(format!("No keypath registered for field: {}", field_name))
        }
    }
}

fn main() {
    println!("=== Microservice Architecture - KeyPath-Based State Synchronization ===\n");
    
    // Initialize shared state
    let initial_state = AppState {
        version: "1.0.0".to_string(),
        user_count: 0,
        active_connections: 0,
        config: Config {
            max_users: 1000,
            timeout_seconds: 30,
            feature_flags: HashMap::new(),
        },
    };
    
    // Create message broker
    let broker = Arc::new(MessageBroker::new());
    
    // Create microservice nodes (simulating different services)
    println!("1. Creating microservice nodes...");
    let node1 = Arc::new(MicroserviceNode::new("api-server".to_string(), initial_state.clone()));
    let node2 = Arc::new(MicroserviceNode::new("auth-server".to_string(), initial_state.clone()));
    let node3 = Arc::new(MicroserviceNode::new("db-server".to_string(), initial_state.clone()));
    let node4 = Arc::new(MicroserviceNode::new("cache-server".to_string(), initial_state.clone()));
    
    // Register nodes with broker
    broker.register(node1);
    broker.register(node2);
    broker.register(node3);
    broker.register(node4);
    
    println!("  Registered nodes: {:?}\n", broker.get_node_ids());
    
    // Create update coordinator
    let coordinator = UpdateCoordinator::new(broker.clone());
    
    // Register keypaths for different fields
    println!("2. Registering keypaths for state fields...");
    coordinator.register_keypath("version".to_string(), AppState::version_r().to_partial().into());
    coordinator.register_keypath("user_count".to_string(), AppState::user_count_r().to_partial().into());
    coordinator.register_keypath("active_connections".to_string(), AppState::active_connections_r().to_partial().into());
    
    // Simulate updates from a primary server
    println!("\n3. Simulating updates from primary server...");
    
    // Update 1: Version change
    thread::sleep(Duration::from_millis(100));
    coordinator.update_field("version", "1.1.0".to_string()).unwrap();
    
    // Update 2: User count change
    thread::sleep(Duration::from_millis(100));
    coordinator.update_field("user_count", "150".to_string()).unwrap();
    
    // Update 3: Active connections change
    thread::sleep(Duration::from_millis(100));
    coordinator.update_field("active_connections", "45".to_string()).unwrap();
    
    // Real-world scenario: Handling struct changes
    println!("\n4. Handling struct schema changes...");
    println!("  Scenario: New field 'maintenance_mode' added to AppState");
    println!("  Solution: Register new keypath, all servers can handle it:");
    
    // In production, this would be:
    // coordinator.register_keypath("maintenance_mode".to_string(), AppState::maintenance_mode_r().to_partial().into());
    println!("    coordinator.register_keypath(\"maintenance_mode\", AppState::maintenance_mode_r().to_partial().into());");
    
    // Demonstrate using keypaths in a real update function
    println!("\n5. Real-world update function example:");
    update_field_safely(&coordinator, "version", "2.0.0".to_string());
    update_field_safely(&coordinator, "user_count", "200".to_string());
    
    println!("\n✅ Microservice synchronization example completed!");
    println!("\n📝 Key Benefits:");
    println!("  - Type-safe field updates using keypaths");
    println!("  - Easy to add new fields without breaking existing code");
    println!("  - Centralized update logic with keypath-based routing");
    println!("  - All microservices receive updates automatically");
}

/// Real-world helper function: Safely update a field using keypath
fn update_field_safely(coordinator: &UpdateCoordinator, field: &str, value: String) {
    match coordinator.update_field(field, value) {
        Ok(()) => println!("  ✅ Successfully updated '{}'", field),
        Err(e) => println!("  ❌ Failed to update '{}': {}", field, e),
    }
}

