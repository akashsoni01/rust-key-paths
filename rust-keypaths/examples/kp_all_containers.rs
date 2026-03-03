//! Example: Kp derive with core container types (Option, Box, Vec, Result).
//!
//! Run with: cargo run -p rust-keypaths --example kp_all_containers
//! (Or from rust-keypaths: cargo run --example kp_all_containers)

use keypaths_proc::Kp;
use std::sync::Arc;

#[derive(Debug, Kp)]
struct AllContainers {
    name: String,
    age: Option<u32>,
    boxed: Box<String>,
    arc: Arc<bool>,
    vec: Vec<String>,
    opt_vec: Option<Vec<String>>,
}

fn main() {
    let value = AllContainers {
        name: "Alice".to_string(),
        age: Some(30),
        boxed: Box::new("boxed".to_string()),
        arc: Arc::new(true),
        vec: vec!["a".into(), "b".into(), "c".into()],
        opt_vec: Some(vec!["one".into(), "two".into()]),
    };

    println!("=== Kp derive – containers ===\n");

    println!("name_r(): {}", AllContainers::name_r().get(&value));
    let _ = AllContainers::age_fr().get(&value).map(|a| println!("age_fr(): {}", a));
    println!("boxed_r(): {}", AllContainers::boxed_r().get(&value));
    println!("arc_r(): {}", AllContainers::arc_r().get(&value));
    println!("vec_r(): {:?}", AllContainers::vec_r().get(&value));
    let _ = AllContainers::vec_fr().get(&value).map(|s| println!("vec_fr(): {}", s));
    let _ = AllContainers::vec_fr_at(1).get(&value).map(|s| println!("vec_fr_at(1): {}", s));
    let _ = AllContainers::opt_vec_fr().get(&value).map(|v| println!("opt_vec_fr(): {}", v));
    let _ = AllContainers::opt_vec_fr_at(1).get(&value).map(|s| println!("opt_vec_fr_at(1): {}", s));

    println!("\n✅ Container keypaths (Kp derive) exercised.");
}
