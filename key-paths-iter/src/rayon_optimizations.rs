//! Rayon performance tuning: thread pool config, chunk sizing, cache-friendly patterns,
//! profiling helpers, and optimization patterns.
//!
//! Enable the `rayon` feature and use with [query_par] for parallel keypath collection ops.

use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ══════════════════════════════════════════════════════════════════════════
// 1. THREAD POOL CONFIGURATION
// ══════════════════════════════════════════════════════════════════════════

/// Optimal thread pool configurations for different workloads.
pub struct RayonConfig {
    num_threads: usize,
    stack_size: usize,
    thread_name: Option<String>,
    breadth_first: bool,
}

impl RayonConfig {
    /// CPU-bound workload (default — use all cores).
    pub fn cpu_bound() -> Self {
        Self {
            num_threads: num_cpus::get(),
            stack_size: 2 * 1024 * 1024, // 2MB
            thread_name: Some("rayon-cpu".into()),
            breadth_first: false,
        }
    }

    /// I/O-bound workload (oversubscribe for better utilization).
    pub fn io_bound() -> Self {
        Self {
            num_threads: num_cpus::get() * 2,
            stack_size: 1 * 1024 * 1024, // 1MB
            thread_name: Some("rayon-io".into()),
            breadth_first: true,
        }
    }

    /// Memory-intensive workload (fewer threads to reduce memory pressure).
    pub fn memory_intensive() -> Self {
        Self {
            num_threads: (num_cpus::get() / 2).max(1),
            stack_size: 4 * 1024 * 1024, // 4MB
            thread_name: Some("rayon-mem".into()),
            breadth_first: false,
        }
    }

    /// Latency-sensitive workload (fewer threads, breadth-first).
    pub fn latency_sensitive() -> Self {
        Self {
            num_threads: (num_cpus::get() / 2).max(2),
            stack_size: 2 * 1024 * 1024,
            thread_name: Some("rayon-latency".into()),
            breadth_first: true,
        }
    }

    /// Hyperthreading-aware (use physical cores only).
    pub fn physical_cores_only() -> Self {
        Self {
            num_threads: num_cpus::get_physical(),
            stack_size: 2 * 1024 * 1024,
            thread_name: Some("rayon-physical".into()),
            breadth_first: false,
        }
    }

    /// Build the thread pool.
    pub fn build(self) -> Result<ThreadPool, rayon::ThreadPoolBuildError> {
        let mut builder = ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .stack_size(self.stack_size);

        if let Some(name) = self.thread_name {
            builder = builder.thread_name(move |i| format!("{}-{}", name, i));
        }

        if self.breadth_first {
            #[allow(deprecated)]
            {
                builder = builder.breadth_first();
            }
        }

        builder.build()
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 2. ADAPTIVE THREAD POOL
// ══════════════════════════════════════════════════════════════════════════

/// Thread pool that can be adjusted based on load.
pub struct AdaptiveThreadPool {
    pools: Vec<Arc<ThreadPool>>,
    current_load: AtomicUsize,
    last_adjustment: Mutex<Instant>,
}

impl AdaptiveThreadPool {
    /// Create pools with 25%, 50%, 75%, and 100% of cores.
    pub fn new() -> Result<Self, rayon::ThreadPoolBuildError> {
        let max_cores = num_cpus::get();
        let mut pools = Vec::new();

        for threads in [
            (max_cores / 4).max(1),
            (max_cores / 2).max(1),
            ((max_cores * 3) / 4).max(1),
            max_cores,
        ] {
            let pool = ThreadPoolBuilder::new().num_threads(threads).build()?;
            pools.push(Arc::new(pool));
        }

        Ok(Self {
            pools,
            current_load: AtomicUsize::new(3),
            last_adjustment: Mutex::new(Instant::now()),
        })
    }

    /// Get the pool selected for current load.
    pub fn get_pool(&self) -> Arc<ThreadPool> {
        let idx = self.current_load.load(Ordering::Relaxed).min(self.pools.len() - 1);
        Arc::clone(&self.pools[idx])
    }

    /// Adjust pool based on CPU utilization (call periodically). Only adjusts every 5 seconds.
    pub fn adjust_for_load(&self, cpu_usage_percent: f32) {
        let mut last = self.last_adjustment.lock().unwrap();
        if last.elapsed().as_secs() < 5 {
            return;
        }
        let new_idx = match cpu_usage_percent {
            x if x < 25.0 => 0,
            x if x < 50.0 => 1,
            x if x < 75.0 => 2,
            _ => 3,
        };
        self.current_load.store(new_idx, Ordering::Relaxed);
        *last = Instant::now();
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 3. CHUNK SIZE TUNING
// ══════════════════════════════════════════════════════════════════════════

/// Optimal chunk sizes for work-stealing.
pub struct ChunkSizeOptimizer;

impl ChunkSizeOptimizer {
    /// Uniform work (each item takes similar time). Target ~8 chunks per thread.
    pub fn uniform(collection_size: usize, num_threads: usize) -> usize {
        let target_chunks = num_threads * 8;
        (collection_size / target_chunks).max(1)
    }

    /// Variable work (items take different times). Smaller chunks.
    pub fn variable(collection_size: usize, num_threads: usize) -> usize {
        let target_chunks = num_threads * 16;
        (collection_size / target_chunks).max(1)
    }

    /// Expensive per-item work. Very small chunks.
    pub fn expensive(collection_size: usize, num_threads: usize) -> usize {
        let target_chunks = num_threads * 32;
        (collection_size / target_chunks).max(1)
    }

    /// Cheap per-item work. Larger chunks to reduce overhead.
    pub fn cheap(collection_size: usize, num_threads: usize) -> usize {
        let target_chunks = num_threads * 2;
        (collection_size / target_chunks).max(100)
    }

    /// Auto-detect chunk size from sample timing (nanos per item).
    pub fn auto_detect<T, F>(items: &[T], sample_size: usize, work_fn: F) -> usize
    where
        F: Fn(&T),
    {
        if items.is_empty() {
            return 1;
        }
        let n = sample_size.min(items.len());
        let start = Instant::now();
        for item in items.iter().take(n) {
            work_fn(item);
        }
        let time_per_item = start.elapsed().as_nanos() as f64 / n as f64;
        let num_threads = num_cpus::get();
        match time_per_item {
            x if x < 1_000.0 => Self::cheap(items.len(), num_threads),
            x if x < 100_000.0 => Self::uniform(items.len(), num_threads),
            x if x < 1_000_000.0 => Self::variable(items.len(), num_threads),
            _ => Self::expensive(items.len(), num_threads),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 4. MEMORY / CACHE-FRIENDLY CHUNKS
// ══════════════════════════════════════════════════════════════════════════

/// Cache-friendly chunk sizes by cache level.
pub struct MemoryOptimizedConfig;

impl MemoryOptimizedConfig {
    /// L1 (~32KB per core).
    pub fn l1_cache_friendly<T>(_items: &[T]) -> usize {
        const L1: usize = 32 * 1024;
        let item_size = std::mem::size_of::<T>().max(1);
        (L1 / item_size).max(1)
    }

    /// L2 (~256KB per core).
    pub fn l2_cache_friendly<T>(_items: &[T]) -> usize {
        const L2: usize = 256 * 1024;
        let item_size = std::mem::size_of::<T>().max(1);
        (L2 / item_size).max(1)
    }

    /// L3 (~8MB shared, divided by cores).
    pub fn l3_cache_friendly<T>(_items: &[T]) -> usize {
        const L3: usize = 8 * 1024 * 1024;
        let item_size = std::mem::size_of::<T>().max(1);
        let per_core = L3 / num_cpus::get();
        (per_core / item_size).max(1)
    }

    /// Chunk size aligned to cache line (64 bytes) to reduce false sharing.
    pub fn cache_line_aligned_chunks<T>() -> usize {
        const LINE: usize = 64;
        let item_size = std::mem::size_of::<T>().max(1);
        (LINE / item_size).max(1)
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 5. PROFILING & BENCHMARKING
// ══════════════════════════════════════════════════════════════════════════

/// Helpers to profile thread counts and chunk sizes.
pub struct RayonProfiler {
    _private: (),
}

impl RayonProfiler {
    /// Profile different thread counts; returns (num_threads, avg_duration).
    pub fn profile_thread_counts<F>(work: F, iterations: usize) -> Vec<(usize, Duration)>
    where
        F: Fn() + Sync,
    {
        let max_threads = num_cpus::get();
        let mut results = Vec::new();

        for threads in [1, 2, 4, 8, max_threads / 2, max_threads] {
            if threads == 0 || threads > max_threads {
                continue;
            }
            let pool = ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            let mut total = Duration::ZERO;
            for _ in 0..iterations {
                let start = Instant::now();
                pool.install(|| work());
                total += start.elapsed();
            }
            results.push((threads, total / iterations as u32));
        }
        results
    }

    /// Compare sequential vs parallel; returns (seq_avg, par_avg, speedup).
    pub fn compare_parallel_vs_sequential<F, G>(
        sequential: F,
        parallel: G,
        iterations: usize,
    ) -> (Duration, Duration, f64)
    where
        F: Fn(),
        G: Fn(),
    {
        sequential();
        parallel();

        let mut seq_total = Duration::ZERO;
        for _ in 0..iterations {
            let start = Instant::now();
            sequential();
            seq_total += start.elapsed();
        }
        let seq_avg = seq_total / iterations as u32;

        let mut par_total = Duration::ZERO;
        for _ in 0..iterations {
            let start = Instant::now();
            parallel();
            par_total += start.elapsed();
        }
        let par_avg = par_total / iterations as u32;

        let speedup = if par_avg.as_nanos() == 0 {
            0.0
        } else {
            seq_avg.as_secs_f64() / par_avg.as_secs_f64()
        };
        (seq_avg, par_avg, speedup)
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 6. OPTIMIZATION PATTERNS
// ══════════════════════════════════════════════════════════════════════════

/// Proven patterns: small-collection cutoff, efficient sum, local accumulation.
pub struct RayonPatterns;

impl RayonPatterns {
    /// Use parallel only when length >= min_len; otherwise sequential.
    pub fn small_collection_optimization<T, F>(items: &[T], min_len: usize, f: F)
    where
        T: Sync,
        F: Fn(&T) + Sync,
    {
        if items.len() < min_len {
            items.iter().for_each(|item| f(item));
        } else {
            items.par_iter().for_each(|item| f(item));
        }
    }

    /// Efficient parallel sum using fold + reduce (no intermediate collect).
    pub fn efficient_sum<T>(items: &[T]) -> T
    where
        T: Send + Sync + Copy + std::ops::Add<Output = T> + Default,
    {
        items
            .par_iter()
            .copied()
            .fold(T::default, |a, b| a + b)
            .reduce(T::default, |a, b| a + b)
    }

    /// Reduce lock contention: per-chunk accumulation, then combine (no shared mutex).
    pub fn reduce_lock_contention<T, F, U>(items: &[T], chunk_size: usize, work: F) -> Vec<U>
    where
        T: Sync,
        F: Fn(&T) -> U + Sync + Send,
        U: Send,
    {
        items
            .par_chunks(chunk_size)
            .flat_map(|chunk| chunk.iter().map(&work).collect::<Vec<_>>())
            .collect()
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 7. ENVIRONMENT CONFIG
// ══════════════════════════════════════════════════════════════════════════

/// Read/set Rayon-related env vars (e.g. RAYON_NUM_THREADS).
pub struct RayonEnvConfig;

impl RayonEnvConfig {
    /// Set default env vars for this process.
    pub fn configure_env() {
        std::env::set_var("RAYON_NUM_THREADS", num_cpus::get().to_string());
        std::env::set_var("RAYON_STACK_SIZE", (2 * 1024 * 1024).to_string());
    }

    /// Load key=value lines from a file into env.
    pub fn load_from_file(path: &str) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                std::env::set_var(key.trim(), value.trim());
            }
        }
        Ok(())
    }

    /// Write current suggested config to a file.
    pub fn save_to_file(path: &str) -> std::io::Result<()> {
        let config = format!(
            "RAYON_NUM_THREADS={}\nRAYON_STACK_SIZE={}\n",
            num_cpus::get(),
            2 * 1024 * 1024
        );
        std::fs::write(path, config)
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 8. PRESET POOLS
// ══════════════════════════════════════════════════════════════════════════

/// Preset thread pools for common scenarios.
pub struct OptimizationGuide;

impl OptimizationGuide {
    /// CPU-bound (ETL, batch).
    pub fn data_pipeline() -> ThreadPool {
        RayonConfig::cpu_bound().build().expect("thread pool")
    }

    /// I/O-bound (many concurrent tasks).
    pub fn web_server() -> ThreadPool {
        RayonConfig::io_bound().build().expect("thread pool")
    }

    /// Memory-heavy (large allocations).
    pub fn scientific_computing() -> ThreadPool {
        RayonConfig::memory_intensive().build().expect("thread pool")
    }

    /// Low latency (games, trading).
    pub fn real_time() -> ThreadPool {
        RayonConfig::latency_sensitive().build().expect("thread pool")
    }

    /// Physical cores only (e.g. training).
    pub fn machine_learning() -> ThreadPool {
        RayonConfig::physical_cores_only().build().expect("thread pool")
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 9. RUNTIME MONITORING
// ══════════════════════════════════════════════════════════════════════════

/// Simple throughput / average task time monitor.
pub struct PerformanceMonitor {
    tasks_completed: Arc<AtomicU64>,
    total_time_nanos: Arc<AtomicU64>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            tasks_completed: Arc::new(AtomicU64::new(0)),
            total_time_nanos: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_task(&self, duration: Duration) {
        self.tasks_completed.fetch_add(1, Ordering::Relaxed);
        self.total_time_nanos
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    pub fn average_task_time(&self) -> Duration {
        let total = self.total_time_nanos.load(Ordering::Relaxed);
        let count = self.tasks_completed.load(Ordering::Relaxed);
        if count == 0 {
            return Duration::ZERO;
        }
        Duration::from_nanos(total / count)
    }

    pub fn throughput_per_second(&self, window_secs: u64) -> f64 {
        let count = self.tasks_completed.load(Ordering::Relaxed);
        count as f64 / window_secs as f64
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_size_optimization() {
        let num_threads = num_cpus::get();
        let uniform = ChunkSizeOptimizer::uniform(10_000, num_threads);
        assert!(uniform > 0);
        let variable = ChunkSizeOptimizer::variable(10_000, num_threads);
        assert!(variable <= uniform || num_threads == 1);
    }

    #[test]
    fn test_auto_detect_chunk_size() {
        let data: Vec<u32> = (0..1000).collect();
        let chunk_size = ChunkSizeOptimizer::auto_detect(&data, 100, |&x| {
            let _ = x * 2;
        });
        assert!(chunk_size > 0);
    }

    #[test]
    fn test_adaptive_pool() {
        let pool = AdaptiveThreadPool::new().unwrap();
        pool.adjust_for_load(20.0);
        let _p1 = pool.get_pool();
        pool.adjust_for_load(90.0);
        let _p2 = pool.get_pool();
    }

    #[test]
    fn test_compare_parallel_vs_sequential() {
        let data: Vec<u32> = (0..5000).collect();
        let (seq, par, speedup) = RayonProfiler::compare_parallel_vs_sequential(
            || {
                let _: u32 = data.iter().map(|&x| x * 2).sum();
            },
            || {
                let _: u32 = data.par_iter().map(|&x| x * 2).sum();
            },
            5,
        );
        assert!(seq.as_nanos() > 0);
        assert!(par.as_nanos() > 0);
        let _ = speedup;
    }

    #[test]
    fn test_memory_optimized_chunks() {
        let data: Vec<u64> = (0..10000).collect();
        let l1 = MemoryOptimizedConfig::l1_cache_friendly(&data);
        let l2 = MemoryOptimizedConfig::l2_cache_friendly(&data);
        let l3 = MemoryOptimizedConfig::l3_cache_friendly(&data);
        assert!(l1 > 0 && l2 > 0 && l3 > 0);
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        for _ in 0..100 {
            monitor.record_task(Duration::from_micros(50));
        }
        let avg = monitor.average_task_time();
        assert!(avg.as_micros() > 0);
    }

    #[test]
    fn test_efficient_sum() {
        let data: Vec<u32> = (0..1000).collect();
        let sum = RayonPatterns::efficient_sum(&data);
        assert_eq!(sum, (0..1000).sum());
    }
}
