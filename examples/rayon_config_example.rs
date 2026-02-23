//! Example: [RayonConfig] presets and custom thread pools.
//!
//! Run: `cargo run --example rayon_config_example`

use key_paths_iter::rayon_optimizations::{OptimizationGuide, RayonConfig};
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    println!("=== RayonConfig & OptimizationGuide ===\n");
    let n = num_cpus::get();
    println!("Detected {} logical cores\n", n);

    // 1. RayonConfig presets
    println!("1. RayonConfig presets (build only, no work):");
    let _pool_cpu = RayonConfig::cpu_bound().build().expect("cpu_bound");
    println!("   - cpu_bound()      -> {} threads", n);

    let _pool_io = RayonConfig::io_bound().build().expect("io_bound");
    println!("   - io_bound()       -> {} threads", n * 2);

    let _pool_mem = RayonConfig::memory_intensive().build().expect("memory_intensive");
    println!("   - memory_intensive() -> {} threads", (n / 2).max(1));

    let _pool_lat = RayonConfig::latency_sensitive().build().expect("latency_sensitive");
    println!("   - latency_sensitive() -> {} threads", (n / 2).max(2));

    let phys = num_cpus::get_physical();
    let _pool_phys = RayonConfig::physical_cores_only().build().expect("physical_cores_only");
    println!("   - physical_cores_only() -> {} threads\n", phys);

    // 2. OptimizationGuide presets (run work on data_pipeline only to avoid stack size variance)
    println!("2. OptimizationGuide — run parallel sum using data_pipeline pool:");
    let data: Vec<u32> = (0..50_000).collect();
    let pool = OptimizationGuide::data_pipeline();
    let start = Instant::now();
    let sum: u32 = pool.install(|| data.par_iter().copied().sum());
    println!("   data_pipeline(): sum = {}, elapsed = {:?}", sum, start.elapsed());
    println!("   (Other presets: web_server, scientific_computing, real_time, machine_learning)");

    println!("\nDone.");
}
