//! Example demonstrating deeply nested enums with Arc<RwLock<T>> case paths
//! 
//! Run with: cargo run --example deeply_nested_enum_arc_rwlock

use keypaths_proc::{Casepaths, Keypaths, WritableKeypaths};
use std::sync::{Arc, RwLock};

// ========== DATA STRUCTURES ==========

/// Top-level application state
#[derive(Debug, Keypaths, WritableKeypaths)]
struct AppState {
    current_mode: AppMode,
}

/// Application mode - can be in different states
#[derive(Debug, Casepaths)]
#[All]
enum AppMode {
    /// Idle mode - no active session
    Idle,
    /// Loading mode - fetching data
    Loading(String),
    /// Active session with thread-safe state
    Active(Arc<RwLock<Session>>),
}

/// A user session that can be accessed from multiple threads
#[derive(Debug, Keypaths, WritableKeypaths)]
struct Session {
    user_name: String,
    user_email: Option<String>,
    theme: Theme,
    task_progress: Option<f32>,
}

/// Theme selection - nested enum
#[derive(Debug, Clone, Casepaths)]
#[All]
enum Theme {
    Light,
    Dark,
    Custom(CustomTheme),
}

/// Custom theme configuration
#[derive(Debug, Clone, Keypaths, WritableKeypaths)]
struct CustomTheme {
    primary_color: String,
    font_size: u32,
}

// ========== HELPER CONSTRUCTORS ==========

impl AppState {
    fn new_active() -> Self {
        Self {
            current_mode: AppMode::Active(Arc::new(RwLock::new(Session {
                user_name: "Alice".to_string(),
                user_email: Some("alice@example.com".to_string()),
                theme: Theme::Custom(CustomTheme {
                    primary_color: "#3498db".to_string(),
                    font_size: 14,
                }),
                task_progress: Some(0.75),
            }))),
        }
    }
    
    fn new_idle() -> Self {
        Self {
            current_mode: AppMode::Idle,
        }
    }
}

fn main() {
    println!("=== Deeply Nested Enum with Arc<RwLock<T>> Example ===\n");
    let app_state = AppState::new_active();
    
    // ========== READING ==========
    // Use _fr() (failable readable) with chain_arc_rwlock or chain_arc_rwlock_optional
    
    // Read non-optional field through the chain
    println!("Reading user_name (non-optional field):");
    let kp = AppState::current_mode_fr()
        .then(AppMode::active_case_r())
        .chain_arc_rwlock(Session::user_name_r());  // Use _r() for non-optional
    kp.get(&app_state, |value| {
        println!("  ✓ user_name = {:?}", value);
    });
    
    // Read optional field through the chain
    println!("\nReading user_email (optional field):");
    let kp = AppState::current_mode_fr()
        .then(AppMode::active_case_r())
        .chain_arc_rwlock_optional(Session::user_email_fr());  // Use _fr() for optional
    kp.get(&app_state, |value| {
        println!("  ✓ user_email = {:?}", value);
    });
    
    // ========== WRITING (with full chain syntax!) ==========
    // Use _w() or _fw() (writable) with chain_arc_rwlock_writable or chain_arc_rwlock_writable_optional
    
    // Write to non-optional field through the full chain
    println!("\nWriting to user_name (using chain_arc_rwlock_writable):");
    let kp = AppState::current_mode_fr()
        .then(AppMode::active_case_r())
        .chain_arc_rwlock_writable(Session::user_name_w());  // Use _w() for non-optional writable
    kp.get_mut(&app_state, |value| {
        *value = "Alice (Updated via chain!)".to_string();
        println!("  ✓ Updated user_name");
    });
    
    // Write to optional field through the full chain
    println!("\nWriting to user_email (using chain_arc_rwlock_writable_optional):");
    let kp = AppState::current_mode_fr()
        .then(AppMode::active_case_r())
        .chain_arc_rwlock_writable_optional(Session::user_email_fw());  // Use _fw() for optional writable
    kp.get_mut(&app_state, |value| {
        *value = "updated@example.com".to_string();
        println!("  ✓ Updated user_email");
    });
    
    // Verify the writes
    println!("\nVerifying writes:");
    let kp = AppState::current_mode_fr()
        .then(AppMode::active_case_r())
        .chain_arc_rwlock(Session::user_name_r());
    kp.get(&app_state, |value| {
        println!("  ✓ user_name = {:?}", value);
    });
    
    let kp = AppState::current_mode_fr()
        .then(AppMode::active_case_r())
        .chain_arc_rwlock_optional(Session::user_email_fr());
    kp.get(&app_state, |value| {
        println!("  ✓ user_email = {:?}", value);
    });
    
    // ========== IDLE STATE (Non-matching variant) ==========
    println!("\nTesting with Idle state (non-matching variant):");
    let idle_state = AppState::new_idle();
    let kp = AppState::current_mode_fr()
        .then(AppMode::active_case_r())
        .chain_arc_rwlock(Session::user_name_r());
    let result = kp.get(&idle_state, |_| ());
    if result.is_none() {
        println!("  ✓ Correctly returned None - enum is Idle, not Active");
    }
    
    println!("\n=== Example completed ===");
    println!("\nSyntax summary:");
    println!("  READING non-optional:  .chain_arc_rwlock(Session::field_r())");
    println!("  READING optional:      .chain_arc_rwlock_optional(Session::field_fr())");
    println!("  WRITING non-optional:  .chain_arc_rwlock_writable(Session::field_w())");
    println!("  WRITING optional:      .chain_arc_rwlock_writable_optional(Session::field_fw())");
}
