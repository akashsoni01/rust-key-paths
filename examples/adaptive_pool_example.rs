//! Example: [AdaptiveThreadPool] — select pool by CPU load.
//!
//! Run: `cargo run --example adaptive_pool_example`

use key_paths_iter::rayon_optimizations::AdaptiveThreadPool;
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    println!("=== AdaptiveThreadPool ===\n");

    let pool = AdaptiveThreadPool::new().expect("build adaptive pool");
    let data: Vec<u32> = (0..20_000).collect();

    // Simulate "low load" -> use smaller pool
    pool.adjust_for_load(20.0);
    let p = pool.get_pool();
    let start = Instant::now();
    let _sum: u32 = p.install(|| data.par_iter().copied().sum());
    println!("After adjust_for_load(20%%): pool installed sum in {:?}", start.elapsed());

    // Simulate "high load" -> use full pool (need to wait 5s for adjustment, so we just get_pool again)
    pool.adjust_for_load(90.0);
    let p = pool.get_pool();
    let start = Instant::now();
    let sum: u32 = p.install(|| data.par_iter().copied().sum());
    println!("After adjust_for_load(90%%): pool installed sum = {} in {:?}", sum, start.elapsed());

    println!("\nNote: adjust_for_load only changes pool every 5 seconds.");
    println!("Done.");
}
