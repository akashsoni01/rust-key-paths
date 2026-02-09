//! Change tracking and synchronization using rust-key-paths.
//!
//! Demonstrates:
//! 1. Detecting changes between two states using keypaths
//! 2. Serializing changes for transmission
//! 3. Applying changes from remote sources
//!
//! Run with: `cargo run --example change_tracker`

use key_paths_derive::Kp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Kp)]
struct AppState {
    user: User,
    settings: Settings,
    cache: Cache,
}

#[derive(Debug, Clone, Serialize, Deserialize, Kp)]
struct User {
    id: u64,
    name: String,
    online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Kp)]
struct Settings {
    theme: String,
    language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Kp)]
struct Cache {
    last_sync: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldChange {
    path: Vec<String>,
    old_value: String,
    new_value: String,
}

struct ChangeTracker<T: 'static> {
    read_paths: Vec<Box<dyn Fn(&T) -> Option<&String>>>,
    write_paths: Vec<Box<dyn Fn(&mut T) -> Option<&mut String>>>,
    path_names: Vec<Vec<String>>,
}

impl<T> ChangeTracker<T>
where
    T: 'static,
{
    fn new() -> Self {
        Self {
            read_paths: Vec::new(),
            write_paths: Vec::new(),
            path_names: Vec::new(),
        }
    }

    fn add_path<FR, FW>(&mut self, read: FR, write: FW, name: Vec<String>)
    where
        FR: Fn(&T) -> Option<&String> + 'static,
        FW: Fn(&mut T) -> Option<&mut String> + 'static,
    {
        self.read_paths.push(Box::new(read));
        self.write_paths.push(Box::new(write));
        self.path_names.push(name);
    }

    fn detect_changes(&self, old: &T, new: &T) -> Vec<FieldChange> {
        let mut changes = Vec::new();
        for (path, path_name) in self.read_paths.iter().zip(&self.path_names) {
            let old_val = path(old);
            let new_val = path(new);
            if old_val != new_val {
                changes.push(FieldChange {
                    path: path_name.clone(),
                    old_value: old_val.map(|s| s.to_string()).unwrap_or_default(),
                    new_value: new_val.map(|s| s.to_string()).unwrap_or_default(),
                });
            }
        }
        changes
    }

    fn apply_changes(&self, target: &mut T, changes: &[FieldChange]) {
        for change in changes {
            for (path, path_name) in self.write_paths.iter().zip(&self.path_names) {
                if path_name == &change.path {
                    if let Some(field) = path(target) {
                        *field = change.new_value.clone();
                    }
                    break;
                }
            }
        }
    }
}

fn main() {
    println!("=== Change Tracker Demo ===\n");

    let mut local_state = AppState {
        user: User {
            id: 1,
            name: "Akash".to_string(),
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

    let remote_state = AppState {
        user: User {
            id: 1,
            name: "Akash Cooper".to_string(),
            online: true,
        },
        settings: Settings {
            theme: "light".to_string(),
            language: "en".to_string(),
        },
        cache: Cache { last_sync: 1000 },
    };

    println!("Remote state (from server):");
    println!("{:#?}\n", remote_state);

    let mut tracker = ChangeTracker::new();

    // Add paths: closures that use the composed keypaths
    tracker.add_path(
        |s: &AppState| AppState::user().then(User::name()).get(s),
        |s: &mut AppState| AppState::user().then(User::name()).get_mut(s),
        vec!["user".into(), "name".into()],
    );
    tracker.add_path(
        |s: &AppState| AppState::settings().then(Settings::theme()).get(s),
        |s: &mut AppState| AppState::settings().then(Settings::theme()).get_mut(s),
        vec!["settings".into(), "theme".into()],
    );
    tracker.add_path(
        |s: &AppState| AppState::settings().then(Settings::language()).get(s),
        |s: &mut AppState| AppState::settings().then(Settings::language()).get_mut(s),
        vec!["settings".into(), "language".into()],
    );

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

    let json = serde_json::to_string_pretty(&changes).unwrap();
    println!("\n--- Serialized Changes (JSON) ---");
    println!("{}\n", json);

    println!("--- Applying Changes to Local State ---");
    tracker.apply_changes(&mut local_state, &changes);

    println!("Updated local state:");
    println!("{:#?}\n", local_state);

    println!("--- Verification ---");
    let verification_changes = tracker.detect_changes(&local_state, &remote_state);
    if verification_changes.is_empty() {
        println!("✓ Local and remote states are now synchronized!");
    } else {
        println!("✗ States still differ");
    }

    println!("\n=== Bidirectional Sync Demo ===\n");

    {
        let name_kp = AppState::user().then(User::name());
        if let Some(name) = name_kp.get_mut(&mut local_state) {
            *name = "Akash C. Johnson".to_string();
        }
    }
    {
        let lang_kp = AppState::settings().then(Settings::language());
        if let Some(lang) = lang_kp.get_mut(&mut local_state) {
            *lang = "es".to_string();
        }
    }

    println!("Local state after modifications:");
    println!("{:#?}\n", local_state);

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

    println!("\n--- Deserializing Changes from JSON ---");
    let deserialized: Vec<FieldChange> = serde_json::from_str(&outgoing_json).unwrap();
    println!("Deserialized {} changes", deserialized.len());

    let mut server_state = remote_state.clone();
    tracker.apply_changes(&mut server_state, &deserialized);

    println!("\nServer state after applying changes:");
    println!("{:#?}", server_state);

    let final_check = tracker.detect_changes(&local_state, &server_state);
    if final_check.is_empty() {
        println!("\n✓ Full bidirectional sync successful!");
    }
}
