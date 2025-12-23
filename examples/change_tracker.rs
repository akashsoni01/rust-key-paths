// Demonstrates using rust-key-paths for change tracking and synchronization
// This example shows how to:
// 1. Track changes between two states using keypaths
// 2. Serialize changes for network transmission
// 3. Apply changes from remote sources
// 4. Build a generic change detection system
// cargo run --example change_tracker

use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Keypaths)]
struct AppState {
    user: User,
    settings: Settings,
    cache: Cache,
}

#[derive(Debug, Clone, Serialize, Deserialize, Keypaths)]
struct User {
    id: u64,
    name: String,
    online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Keypaths)]
struct Settings {
    theme: String,
    language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Keypaths)]
struct Cache {
    last_sync: u64,
}

// Path identifier for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldChange {
    path: Vec<String>, // e.g., ["user", "name"]
    old_value: String,
    new_value: String,
}

// Track which paths changed
// Note: We need both readable and writable keypaths because:
// - Readable paths (_r) work with immutable references for comparison
// - Writable paths (_w) work with mutable references for updates
struct ChangeTracker<T: 'static> {
    read_paths: Vec<KeyPaths<T, String>>,  // For reading/comparing
    write_paths: Vec<KeyPaths<T, String>>, // For writing changes
    path_names: Vec<Vec<String>>,          // Human-readable path identifiers
}

impl<T> ChangeTracker<T> {
    fn new() -> Self {
        Self {
            read_paths: Vec::new(),
            write_paths: Vec::new(),
            path_names: Vec::new(),
        }
    }

    fn add_path(
        &mut self,
        read_path: KeyPaths<T, String>,
        write_path: KeyPaths<T, String>,
        name: Vec<String>,
    ) {
        self.read_paths.push(read_path);
        self.write_paths.push(write_path);
        self.path_names.push(name);
    }

    fn detect_changes(&self, old: &T, new: &T) -> Vec<FieldChange> {
        let mut changes = Vec::new();

        for (path, path_name) in self.read_paths.iter().zip(&self.path_names) {
            let old_val = path.get(old);
            let new_val = path.get(new);

            if old_val != new_val {
                changes.push(FieldChange {
                    path: path_name.clone(),
                    old_value: old_val.map(|s| s.clone()).unwrap_or_default(),
                    new_value: new_val.map(|s| s.clone()).unwrap_or_default(),
                });
            }
        }

        changes
    }

    fn apply_changes(&self, target: &mut T, changes: &[FieldChange]) {
        for change in changes {
            for (path, path_name) in self.write_paths.iter().zip(&self.path_names) {
                if path_name == &change.path {
                    if let Some(field) = path.get_mut(target) {
                        *field = change.new_value.clone();
                    }
                    break;
                }
            }
        }
    }
}

// Usage: Real-time sync
fn main() {
    println!("=== Change Tracker Demo ===\n");

    // Initial local state
    let mut local_state = AppState {
        user: User {
            id: 1,
            name: "Alice".to_string(),
            online: true,
        },
        settings: Settings {
            theme: "dark".to_string(),
            language: "en".to_string(),
        },
        cache: Cache { last_sync: 1000 },
    };

    println!("Initial local state:");
    println!("{:#?}\n", local_state);

    // Simulated remote state (as if from server)
    let remote_state = AppState {
        user: User {
            id: 1,
            name: "Alice Cooper".to_string(), // Name changed
            online: true,
        },
        settings: Settings {
            theme: "light".to_string(), // Theme changed
            language: "en".to_string(),
        },
        cache: Cache { last_sync: 1000 },
    };

    println!("Remote state (from server):");
    println!("{:#?}\n", remote_state);

    // Create tracker with keypaths for fields we want to monitor
    let mut tracker = ChangeTracker::new();

    // Add paths to track (need both readable for comparison and writable for updates)
    tracker.add_path(
        AppState::user_r().then(User::name_r()),
        AppState::user_w().then(User::name_w()),
        vec!["user".into(), "name".into()],
    );

    tracker.add_path(
        AppState::settings_r().then(Settings::theme_r()),
        AppState::settings_w().then(Settings::theme_w()),
        vec!["settings".into(), "theme".into()],
    );

    tracker.add_path(
        AppState::settings_r().then(Settings::language_r()),
        AppState::settings_w().then(Settings::language_w()),
        vec!["settings".into(), "language".into()],
    );

    // Detect what changed on server
    println!("--- Detecting Changes ---");
    let changes = tracker.detect_changes(&local_state, &remote_state);

    if changes.is_empty() {
        println!("No changes detected.");
    } else {
        println!("Detected {} change(s):", changes.len());
        for change in &changes {
            println!(
                "  • {:?}: '{}' -> '{}'",
                change.path, change.old_value, change.new_value
            );
        }
    }

    // Serialize changes for network transmission
    let json = serde_json::to_string_pretty(&changes).unwrap();
    println!("\n--- Serialized Changes (JSON) ---");
    println!("{}\n", json);

    // Apply changes from server to local state
    println!("--- Applying Changes to Local State ---");
    tracker.apply_changes(&mut local_state, &changes);

    println!("Updated local state:");
    println!("{:#?}\n", local_state);

    // Verify synchronization
    println!("--- Verification ---");
    let verification_changes = tracker.detect_changes(&local_state, &remote_state);
    if verification_changes.is_empty() {
        println!("✓ Local and remote states are now synchronized!");
    } else {
        println!("✗ States still differ:");
        for change in verification_changes {
            println!(
                "  • {:?}: '{}' -> '{}'",
                change.path, change.old_value, change.new_value
            );
        }
    }

    // Demonstrate bidirectional sync
    println!("\n=== Bidirectional Sync Demo ===\n");

    // Make local changes
    println!("Making local changes...");
    if let Some(name) = AppState::user_w()
        .then(User::name_w())
        .get_mut(&mut local_state)
    {
        *name = "Alice C. Johnson".to_string();
    }

    if let Some(language) = AppState::settings_w()
        .then(Settings::language_w())
        .get_mut(&mut local_state)
    {
        *language = "es".to_string();
    }

    println!("Local state after modifications:");
    println!("{:#?}\n", local_state);

    // Detect changes to send to server
    let outgoing_changes = tracker.detect_changes(&remote_state, &local_state);
    println!("Changes to send to server:");
    for change in &outgoing_changes {
        println!(
            "  • {:?}: '{}' -> '{}'",
            change.path, change.old_value, change.new_value
        );
    }

    let outgoing_json = serde_json::to_string_pretty(&outgoing_changes).unwrap();
    println!("\nOutgoing JSON:");
    println!("{}", outgoing_json);

    // Demonstrate deserializing and applying changes
    println!("\n--- Deserializing Changes from JSON ---");
    let deserialized_changes: Vec<FieldChange> =
        serde_json::from_str(&outgoing_json).unwrap();
    println!("Successfully deserialized {} changes", deserialized_changes.len());

    // Apply to a new state (simulating server receiving changes)
    let mut server_state = remote_state.clone();
    tracker.apply_changes(&mut server_state, &deserialized_changes);
    
    println!("\nServer state after applying changes:");
    println!("{:#?}", server_state);

    // Final verification
    let final_check = tracker.detect_changes(&local_state, &server_state);
    if final_check.is_empty() {
        println!("\n✓ Full bidirectional sync successful!");
    }
}

