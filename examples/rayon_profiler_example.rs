//! Example: [RayonProfiler] — compare parallel vs sequential, profile thread counts.
//!
//! Run: `cargo run --example rayon_profiler_example`

use key_paths_iter::rayon_optimizations::RayonProfiler;
use rayon::prelude::*;

fn main() {
    println!("=== RayonProfiler ===\n");

    let data: Vec<u64> = (0..100_000).collect();
    let iterations = 5;

    // Compare parallel vs sequential
    let (seq, par, speedup) = RayonProfiler::compare_parallel_vs_sequential(
        || {
            let _: u64 = data.iter().copied().sum();
        },
        || {
            let _: u64 = data.par_iter().copied().sum();
        },
        iterations,
    );
    println!("compare_parallel_vs_sequential ({} iterations):", iterations);
    println!("  sequential avg = {:?}", seq);
    println!("  parallel   avg = {:?}", par);
    println!("  speedup    = {:.2}x\n", speedup);

    // Profile different thread counts (light work)
    let results = RayonProfiler::profile_thread_counts(
        || {
            let _: u64 = data.par_iter().copied().sum();
        },
        3,
    );
    println!("profile_thread_counts (3 iterations each):");
    for (threads, avg) in results {
        println!("  {} threads -> {:?}", threads, avg);
    }
    println!("\nDone.");
}
