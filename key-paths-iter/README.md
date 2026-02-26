# key-paths-iter

Query builder for iterating over `Vec<Item>` collections accessed via [rust-key-paths](https://crates.io/crates/rust-key-paths) `KpType`.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
rust-key-paths = "2"
key-paths-iter = { path = "../key-paths-iter" }  # or from crates.io when published
```

For parallel iteration and Rayon tuning helpers, enable the `rayon` feature:

```toml
key-paths-iter = { path = "../key-paths-iter", features = ["rayon"] }
```

**Rayon version:** the crate uses **Rayon 1.10** (optional dependency). It is expected to work with Rayon **1.10.x** and newer 1.x; if you need a different Rayon version, use a patch in your workspace or fork. Rayon 1.x is stable and API-compatible across minor updates.

Use with a keypath whose value type is `Vec<Item>`:

```rust
use key_paths_iter::{CollectionQuery, QueryableCollection};
use rust_key_paths::Kp;

let users_kp: rust_key_paths::KpType<'_, Database, Vec<User>> = Kp::new(
    |db: &Database| Some(&db.users),
    |db: &mut Database| Some(&mut db.users),
);

// Chain filters, limit, offset, then execute
let results = users_kp
    .query()
    .filter(|u| u.active)
    .filter(|u| u.age > 26)
    .limit(2)
    .offset(0)
    .execute(&db);

// Or use count / exists / first
let n = users_kp.query().filter(|u| u.active).count(&db);
let any = users_kp.query().filter(|u| u.active).exists(&db);
let first = users_kp.query().filter(|u| u.active).first(&db);
```

The keypath and the root reference share the same lifetime; use a type annotation like `KpType<'_, Root, Vec<Item>>` so the compiler infers the scope correctly.

---

## GPU-scale parallel validation and calculation (`scale_par`)

With the `rayon` feature, the **`scale_par`** module provides types and helpers for GPU-style pipelines: parallel validation, data extraction for transfer, pre/post processing, and adaptive dispatch — all driven by keypaths.

**Benefits of using `scale_par` with keypaths:**

| Benefit | Description |
|--------|-------------|
| **Zero-cost + composable** | Keypaths use `fn` pointers that inline to direct field access; no boxed closures. Compose keypaths (e.g. into nested `reduction_net.nodes`) with no runtime overhead. |
| **Parallel-first** | `par_all`, `par_count_by`, `par_flat_map`, etc. run over the collection at the keypath. Validate large node/pair arrays in parallel before GPU transfer. |
| **GPU data pipeline** | Extract via keypath → validate in parallel → transfer; then write back results via `get_mut` and keypath. Same pattern works for CPU-only “GPU-style” batches. |
| **First-class field access** | Pass keypaths as arguments (e.g. `validate_for_gpu(..., nodes_kp, pairs_kp)`). Generic code can work over any root/collection type. |
| **Adaptive dispatch** | Use keypath queries (`par_count`, `par_count_by`) to choose workgroup size, batch size, or local memory usage from data shape. |
| **No boilerplate** | One keypath per field; validation and calculation functions take keypath references. Works with `#[derive(Kp)]` or manual `Kp::new`. |

**Example:** validate a compute pipeline, extract data for GPU, run parallel buffer scaling, then write back results:

```rust
use key_paths_iter::scale_par::{validate_for_gpu, extract_gpu_data, par_scale_buffers, ...};
use rust_key_paths::Kp;

let nodes_kp = Kp::new(|p: &GpuComputePipeline| Some(&p.reduction_net.nodes), ...);
let pairs_kp = Kp::new(|p: &GpuComputePipeline| Some(&p.reduction_net.active_pairs), ...);

validate_for_gpu(&pipeline, &nodes_kp, &pairs_kp, 1000)?;
let gpu_data = extract_gpu_data(&pipeline, &nodes_kp, &pairs_kp);
par_scale_buffers(&buffers_kp, &mut pipeline, 2.0);
```

Run the full example: `cargo run --example scale_par_gpu_validation` (from the workspace root, with `key-paths-iter` and `rayon` enabled).

### Benchmark (scale_par: parallel vs sequential)

From the **workspace root** (rust-key-paths), run:

```bash
cargo bench --bench scale_par_bench
```

This compares:

- **Buffer scaling:** sequential nested loops vs keypath `par_scale_buffers` (multiple buffer × length sizes).
- **Validation (all non-empty):** sequential `iter().all(...)` vs `par_validate_buffers_non_empty`.
- **Count by predicate:** sequential `filter().count()` vs keypath `par_count_by` (nodes by kind at 5k–100k nodes).

On multi-core machines, parallel validation and `par_count_by` typically show speedups for large collections; buffer scaling may favor sequential for small inputs due to Rayon overhead. Use the benchmark to tune for your workload.

---

## Enabling GPU access (using keypaths with a GPU backend)

This crate does **not** ship a GPU driver or runtime. It gives you **CPU-side** parallel validation, extraction, and preprocessing via keypaths. To run work on an actual GPU, add a GPU backend to your project and use keypaths to prepare and consume data.

### 1. System and driver requirements

| Backend | Typical use | What you need |
|--------|-------------|----------------|
| **Vulkan** | Cross-platform (Windows, Linux, macOS) | Vulkan SDK, GPU drivers with Vulkan support |
| **Metal** | macOS, iOS | Xcode / Metal (usually already installed on Mac) |
| **CUDA** | NVIDIA GPUs only | NVIDIA drivers, CUDA Toolkit, `nvcc` in path for Rust CUDA builds |
| **ROCm** | AMD GPUs (Linux) | ROCm stack, compatible AMD GPU |

- **Windows:** Install [Vulkan SDK](https://vulkan.lunarg.com/) and/or [CUDA Toolkit](https://developer.nvidia.com/cuda-downloads) (NVIDIA). GPU drivers from your vendor must be up to date.
- **macOS:** Metal is built in; for Vulkan you can use MoltenVK (often bundled by wgpu).
- **Linux:** Install Vulkan (e.g. `vulkan-tools`, `libvulkan-dev`), or NVIDIA drivers + CUDA for NVIDIA, or ROCm for AMD.

### 2. Rust crates that enable GPU

- **[wgpu](https://crates.io/crates/wgpu)** — Cross-platform (Vulkan / Metal / DX12 / WebGPU). Good default for “run on GPU” without tying to one vendor. Use keypaths to build buffers (e.g. `extract_gpu_data`, `par_flat_map_buffer_data`) and validate before `queue.write_buffer` / dispatch.
- **[cudarc](https://crates.io/crates/cudarc)** / **[rust-cuda](https://github.com/Rust-GPU/Rust-CUDA)** — NVIDIA CUDA from Rust. Use keypaths to prepare host data, then copy to device and launch kernels.
- **[vulkano](https://crates.io/crates/vulkano)** — Vulkan bindings. Keypaths can feed validated/extracted data into Vulkan buffers and compute dispatches.

### 3. How to “enable GPU” in your project

1. **Add a GPU dependency** to your `Cargo.toml`, e.g. `wgpu` or `cudarc`.
2. **Use keypaths + `scale_par` on the CPU** to validate and prepare data (e.g. `validate_for_gpu`, `extract_gpu_data`, `par_scale_buffers`).
3. **Transfer to the GPU** using the backend’s API (e.g. `wgpu::Queue::write_buffer` with data produced via keypath helpers).
4. **Run your compute or render work** on the device; then read back results and, if needed, write them back into your structures via keypaths (e.g. `process_gpu_results`).

Example pattern (pseudo-code):

```text
let data = extract_gpu_data(&pipeline, &nodes_kp, &pairs_kp);  // keypath-based
validate_for_gpu(&pipeline, &nodes_kp, &pairs_kp, MAX_PORT)?;    // parallel validation
queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&data.nodes));  // wgpu/CUDA/...
// ... dispatch ...
// read back and write into pipeline via keypath
process_gpu_results(&mut pipeline, &nodes_kp, gpu_results);
```

No feature flag in **key-paths-iter** is required for GPU: enable the **`rayon`** feature for parallel CPU prep; the GPU itself is enabled by your choice of backend and system drivers.

---

## Rayon performance tuning

With the `rayon` feature, the crate exposes a **Rayon optimization** module: thread pool presets, chunk sizing, cache-friendly patterns, profiling helpers, and workload-specific guides. Use these with parallel keypath collection ops (e.g. `query_par`).

**Examples** (from the workspace root): `rayon_config_example`, `adaptive_pool_example`, `chunk_size_example`, `memory_optimized_example`, `rayon_profiler_example`, `rayon_patterns_example`, `rayon_env_example`, `optimization_guide_example`, `performance_monitor_example`. Run with `cargo run --example <name>` (requires `key-paths-iter` with `rayon` in dev-dependencies).

### Performance benefits of parallel (`par`)

- **Throughput:** On multi-core machines, parallel iteration spreads work across cores, so total time can drop by roughly a factor of the number of cores (for CPU-bound work with good load balance).
- **When you gain the most:** Large collections (e.g. &gt; 10k items), CPU-heavy per-item work (math, encoding, parsing), and batch operations (map, filter, count, sort, fold). Typical speedups are **~2–8×** on 2–8 cores when the workload is uniform and not memory-bound.
- **When `par` may not help (or can hurt):** Very small collections (overhead dominates), very cheap per-item work (&lt; ~1 μs), or when the bottleneck is memory bandwidth or a single shared resource. Use `RayonProfiler::compare_parallel_vs_sequential` to measure.

### Where you can use `par`

**In this crate (keypath collections)** — use the `query_par` module and the `ParallelCollectionKeyPath` trait on `KpType<'static, Root, Vec<Item>>` (e.g. from `#[derive(Kp)]`):

| Category | Methods |
|----------|---------|
| **Map / transform** | `par_map`, `par_filter`, `par_filter_map`, `par_flat_map`, `par_map_with_index` |
| **Reduce / aggregate** | `par_fold`, `par_reduce`, `par_count`, `par_count_by` |
| **Search** | `par_find`, `par_find_any`, `par_any`, `par_all`, `par_contains` |
| **Min / max** | `par_min`, `par_max`, `par_min_by_key`, `par_max_by_key` |
| **Partition / group** | `par_partition`, `par_group_by` |
| **Ordering** | `par_sort`, `par_sort_by_key` |
| **Side effects** | `par_for_each` |

Example: `employees_kp.par_map(&company, |e| e.salary)`, `employees_kp.par_count_by(&company, |e| e.active)`.

**With raw slices and Rayon** — on any `&[T]` or `Vec<T>` you can use Rayon’s `par_iter()`, `par_chunks()`, `par_chunks_mut()`, and the rest of the `rayon::prelude` API. The `rayon_optimizations` helpers (chunk sizing, pool config, profiling) work with both keypath-based and raw-slice parallel code.

### Thread count rules of thumb

- **CPU-bound:** use all cores → `RAYON_NUM_THREADS = num_cpus::get()`
- **I/O-bound:** oversubscribe 2× → `RAYON_NUM_THREADS = num_cpus::get() * 2`
- **Memory-intensive:** use half → `RAYON_NUM_THREADS = num_cpus::get() / 2`
- **Latency-sensitive:** physical cores only → `RAYON_NUM_THREADS = num_cpus::get_physical()`

### Chunk size formulas

- **Uniform work:** ~8 chunks per thread → `chunk_size = total_items / (num_threads * 8)`
- **Variable work:** ~16 chunks per thread → `chunk_size = total_items / (num_threads * 16)`
- **Expensive work:** ~32 chunks per thread → `chunk_size = total_items / (num_threads * 32)`
- **Cheap work:** ~2 chunks per thread → `chunk_size = total_items / (num_threads * 2)`

Helpers: `ChunkSizeOptimizer::uniform`, `variable`, `expensive`, `cheap`, and `auto_detect(items, sample_size, work_fn)`.

### When to use parallel

Use parallel iteration when:

- `items.len() > 1000` **and**
- cost per item is non-trivial (e.g. &gt; ~1 μs).

Otherwise prefer sequential to avoid overhead. Use `RayonPatterns::small_collection_optimization(items, min_len, f)` to switch automatically.

### Cache-friendly chunk sizes

- **L1 (~32 KB):** `chunk_size = 32KB / sizeof(T)` → `MemoryOptimizedConfig::l1_cache_friendly`
- **L2 (~256 KB):** `chunk_size = 256KB / sizeof(T)` → `MemoryOptimizedConfig::l2_cache_friendly`
- **L3 (~8 MB shared):** `chunk_size = (8MB / num_threads) / sizeof(T)` → `MemoryOptimizedConfig::l3_cache_friendly`

### Anti-patterns to avoid

- **Multiple collects:** avoid `let a = data.par_iter().map(...).collect(); let b = a.par_iter().filter(...).collect();`. Prefer chaining: `data.par_iter().map(...).filter(...).collect()`.
- **Shared mutex:** avoid a single `Mutex<Vec<_>>` with `par_iter().for_each(|x| results.lock().unwrap().push(...))`. Prefer local accumulation then combine, e.g. `par_chunks(...).map(|chunk| ...).collect()` or fold/reduce. See `RayonPatterns::reduce_lock_contention`.

### Configuration file

Create `rayon.conf`:

```bash
RAYON_NUM_THREADS=16
RAYON_STACK_SIZE=2097152
```

Load in code:

```rust
key_paths_iter::rayon_optimizations::RayonEnvConfig::load_from_file("rayon.conf")?;
```

Save current suggested config: `RayonEnvConfig::save_to_file("rayon.conf")?`.

### Quick benchmark (parallel vs sequential)

```rust
use std::time::Instant;

let start = Instant::now();
data.par_iter().for_each(|x| expensive_work(x));
println!("Parallel: {:?}", start.elapsed());

let start = Instant::now();
data.iter().for_each(|x| expensive_work(x));
println!("Sequential: {:?}", start.elapsed());
```

Or use `RayonProfiler::compare_parallel_vs_sequential(sequential_fn, parallel_fn, iterations)` for averaged timings and speedup.

### Optimal settings by workload

| Workload       | Threads      | Stack size | Breadth-first | Chunk size   |
|----------------|-------------|------------|---------------|--------------|
| CPU-bound      | All cores   | 2 MB       | No            | Medium (8×)  |
| I/O-bound      | 2× cores    | 1 MB       | Yes           | Small (16×)  |
| Memory-heavy   | Half cores  | 4 MB       | No            | Large (2×)   |
| Latency        | Physical only | 2 MB     | Yes           | Very small (32×) |
| Real-time      | Half cores  | 2 MB       | Yes           | Adaptive     |

Preset pools: `OptimizationGuide::data_pipeline()`, `web_server()`, `scientific_computing()`, `real_time()`, `machine_learning()`. Config builder: `RayonConfig::cpu_bound()`, `io_bound()`, `memory_intensive()`, `latency_sensitive()`, `physical_cores_only()`, then `.build()`.
