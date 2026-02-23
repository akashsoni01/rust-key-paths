//! Example: [PerformanceMonitor] — record_task, average_task_time, throughput_per_second.
//!
//! Run: `cargo run --example performance_monitor_example`

use key_paths_iter::rayon_optimizations::PerformanceMonitor;
use std::time::Duration;

fn main() {
    println!("=== PerformanceMonitor ===\n");

    let monitor = PerformanceMonitor::new();

    // Simulate 100 tasks, each ~50μs
    for _ in 0..100 {
        monitor.record_task(Duration::from_micros(50));
    }

    let avg = monitor.average_task_time();
    println!("After 100 tasks of 50μs each:");
    println!("  average_task_time() = {:?}", avg);
    println!("  throughput_per_second(1) = {:.0}", monitor.throughput_per_second(1));

    // Add more tasks
    for _ in 0..900 {
        monitor.record_task(Duration::from_micros(50));
    }
    println!("\nAfter 1000 total tasks:");
    println!("  average_task_time() = {:?}", monitor.average_task_time());
    println!("  throughput_per_second(1) = {:.0}", monitor.throughput_per_second(1));
    println!("\nDone.");
}
