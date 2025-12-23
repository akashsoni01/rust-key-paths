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
    let mut app_state = AppState::new_active();
    let kp = AppState::current_mode_fr().then(AppMode::active_case_r()).chain_arc_rwlock_optional(Session::user_name_fr());
    kp.get(&app_state, |value| {
        println!("value = {:?}", value);
    });

    // let kp = AppState::current_mode_fr().then(AppMode::active_case_r()).chain_arc_rwlock_optional(Session::user_name_fw());
    // kp.get(&mut app_state, |value| {
    //     println!("value = {:?}", value);
    // });



}
