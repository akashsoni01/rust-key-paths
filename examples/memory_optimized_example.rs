//! Example: [MemoryOptimizedConfig] — L1/L2/L3 and cache-line aligned chunk sizes.
//!
//! Run: `cargo run --example memory_optimized_example`

use key_paths_iter::rayon_optimizations::MemoryOptimizedConfig;
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    println!("=== MemoryOptimizedConfig ===\n");

    // Slice of u64 (8 bytes)
    let data_u64: Vec<u64> = (0..100_000).collect();
    let l1_u64 = MemoryOptimizedConfig::l1_cache_friendly(&data_u64);
    let l2_u64 = MemoryOptimizedConfig::l2_cache_friendly(&data_u64);
    let l3_u64 = MemoryOptimizedConfig::l3_cache_friendly(&data_u64);
    let line_u64 = MemoryOptimizedConfig::cache_line_aligned_chunks::<u64>();
    println!("For u64 (size {}):", std::mem::size_of::<u64>());
    println!("  l1_cache_friendly()         = {} items", l1_u64);
    println!("  l2_cache_friendly()         = {} items", l2_u64);
    println!("  l3_cache_friendly()         = {} items", l3_u64);
    println!("  cache_line_aligned_chunks()  = {} items\n", line_u64);

    // Slice of u8
    let data_u8: Vec<u8> = (0..10_000_u32).map(|i| i as u8).collect();
    let l1_u8 = MemoryOptimizedConfig::l1_cache_friendly(&data_u8);
    println!("For u8 (size {}): l1_cache_friendly() = {} items\n", std::mem::size_of::<u8>(), l1_u8);

    // Use L2-friendly chunks for a parallel sum
    let chunk = MemoryOptimizedConfig::l2_cache_friendly(&data_u64);
    let start = Instant::now();
    let sum: u64 = data_u64
        .par_chunks(chunk)
        .map(|c| c.iter().copied().sum::<u64>())
        .sum();
    println!("par_chunks(l2_cache_friendly) sum = {}, elapsed = {:?}", sum, start.elapsed());
    println!("\nDone.");
}
