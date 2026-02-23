//! Example: [ChunkSizeOptimizer] — uniform, variable, expensive, cheap, auto_detect.
//!
//! Run: `cargo run --example chunk_size_example`

use key_paths_iter::rayon_optimizations::ChunkSizeOptimizer;
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    println!("=== ChunkSizeOptimizer ===\n");
    let n = num_cpus::get();
    let size: usize = 100_000;

    let u = ChunkSizeOptimizer::uniform(size, n);
    let v = ChunkSizeOptimizer::variable(size, n);
    let e = ChunkSizeOptimizer::expensive(size, n);
    let c = ChunkSizeOptimizer::cheap(size, n);
    println!("Collection size = {}, num_threads = {}", size, n);
    println!("  uniform()   chunk_size = {}", u);
    println!("  variable()  chunk_size = {}", v);
    println!("  expensive() chunk_size = {}", e);
    println!("  cheap()     chunk_size = {}\n", c);

    // Auto-detect from cheap work
    let data: Vec<u64> = (0..10_000).collect();
    let chunk = ChunkSizeOptimizer::auto_detect(&data, 500, |&x| {
        let _ = x + 1;
    });
    println!("auto_detect (cheap work, sample 500): chunk_size = {}", chunk);

    // Use a chosen chunk size in par_chunks
    let chunk_size = ChunkSizeOptimizer::uniform(data.len(), n);
    let start = Instant::now();
    let sum: u64 = data
        .par_chunks(chunk_size)
        .map(|c| c.iter().copied().sum::<u64>())
        .sum();
    println!(
        "par_chunks({}) then sum: result = {}, elapsed = {:?}",
        chunk_size,
        sum,
        start.elapsed()
    );
    println!("\nDone.");
}
