//! Example: [RayonPatterns] — small_collection_optimization, efficient_sum, reduce_lock_contention.
//!
//! Run: `cargo run --example rayon_patterns_example`

use key_paths_iter::rayon_optimizations::RayonPatterns;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    println!("=== RayonPatterns ===\n");

    // 1. small_collection_optimization: sequential below min_len, parallel above
    let small: Vec<u32> = (0..100).collect();
    let count = AtomicUsize::new(0);
    RayonPatterns::small_collection_optimization(&small, 500, |_| {
        count.fetch_add(1, Ordering::Relaxed);
    });
    println!("small_collection_optimization(small, min_len=500): count = {}", count.load(Ordering::Relaxed));

    let large: Vec<u32> = (0..5_000).collect();
    count.store(0, Ordering::Relaxed);
    RayonPatterns::small_collection_optimization(&large, 500, |_| {
        count.fetch_add(1, Ordering::Relaxed);
    });
    println!("small_collection_optimization(large, min_len=500): count = {}\n", count.load(Ordering::Relaxed));

    // 2. efficient_sum (fold + reduce, no intermediate collect)
    let data: Vec<u32> = (0..10_000).collect();
    let sum = RayonPatterns::efficient_sum(&data);
    let expected: u32 = (0..10_000).sum();
    println!("efficient_sum(0..10000) = {} (expected {})\n", sum, expected);

    // 3. reduce_lock_contention: per-chunk results, no shared mutex
    let items: Vec<u32> = (0..1_000).collect();
    let strings = RayonPatterns::reduce_lock_contention(&items, 100, |&x| format!("{}", x * 2));
    println!("reduce_lock_contention(chunk_size=100): {} results", strings.len());
    println!("  first few: {:?}", &strings[..3.min(strings.len())]);
    println!("\nDone.");
}
