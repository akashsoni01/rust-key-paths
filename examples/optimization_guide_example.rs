//! Example: [OptimizationGuide] preset pools — data_pipeline, web_server, etc.
//!
//! Run: `cargo run --example optimization_guide_example`

use key_paths_iter::rayon_optimizations::OptimizationGuide;
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    println!("=== OptimizationGuide preset pools ===\n");
    let data: Vec<u32> = (0..10_000).collect();

    // Run presets that use 2MB stack and depth-first to avoid overflow in debug builds
    for (name, pool) in [
        ("data_pipeline", OptimizationGuide::data_pipeline()),
        ("scientific_computing", OptimizationGuide::scientific_computing()),
        ("machine_learning", OptimizationGuide::machine_learning()),
    ] {
        let start = Instant::now();
        let sum: u32 = pool.install(|| data.par_iter().copied().sum());
        println!("{}: sum = {}, elapsed = {:?}", name, sum, start.elapsed());
    }
    println!("(Also available: web_server, real_time — see OptimizationGuide and RayonConfig)");
    println!("\nDone.");
}
