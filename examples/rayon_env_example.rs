//! Example: [RayonEnvConfig] — configure_env, load_from_file, save_to_file.
//!
//! Run: `cargo run --example rayon_env_example`

use key_paths_iter::rayon_optimizations::RayonEnvConfig;
use std::env;
use std::fs;

fn main() {
    println!("=== RayonEnvConfig ===\n");

    // Show current env (if set)
    let threads = env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "<not set>".into());
    let stack = env::var("RAYON_STACK_SIZE").unwrap_or_else(|_| "<not set>".into());
    println!("Before: RAYON_NUM_THREADS = {}, RAYON_STACK_SIZE = {}", threads, stack);

    // Set defaults for this process
    RayonEnvConfig::configure_env();
    let threads = env::var("RAYON_NUM_THREADS").unwrap();
    let stack = env::var("RAYON_STACK_SIZE").unwrap();
    println!("After configure_env(): RAYON_NUM_THREADS = {}, RAYON_STACK_SIZE = {}\n", threads, stack);

    // Save to file
    let path = "rayon_example.conf";
    RayonEnvConfig::save_to_file(path).expect("save config");
    println!("Saved config to {}", path);
    let content = fs::read_to_string(path).expect("read");
    println!("Content:\n{}", content);

    // Load from file (overwrites env for this process)
    unsafe {
        env::remove_var("RAYON_NUM_THREADS");
        env::remove_var("RAYON_STACK_SIZE");
    }
    RayonEnvConfig::load_from_file(path).expect("load config");
    println!("After load_from_file: RAYON_NUM_THREADS = {:?}", env::var("RAYON_NUM_THREADS"));
    fs::remove_file(path).ok();
    println!("\nDone.");
}
