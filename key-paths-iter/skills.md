# How to Use key-paths-iter

This file is a quick reference for using the **key-paths-iter** crate: query and parallel iteration over `Vec<Item>` collections accessed via [rust-key-paths](https://crates.io/crates/rust-key-paths) `KpType`.

---

## 1. Setup

**Cargo.toml:**

```toml
[dependencies]
rust-key-paths = "2"
key-paths-iter = { version = "0.1", path = "../key-paths-iter" }  # or from crates.io
```

**With parallel (Rayon) and optional GPU:**

```toml
key-paths-iter = { version = "0.1", path = "../key-paths-iter", features = ["rayon"] }
# Optional: features = ["rayon", "gpu"] for wgpu compute
```

**Requirement:** The keypath’s **value type** must be `Vec<Item>`. Use `KpType<'static, Root, Vec<Item>>` (e.g. from `#[derive(Kp)]`) or build with `Kp::new(...)`.

---

## 2. Query API (sequential)

Use when the keypath points at a `Vec<Item>`.

**With a keypath that has a non-static lifetime** (e.g. `KpType<'_, Root, Vec<Item>>`):

```rust
use key_paths_iter::QueryableCollection;
use rust_key_paths::Kp;

let users_kp: KpType<'_, Database, Vec<User>> = Kp::new(
    |db: &Database| Some(&db.users),
    |db: &mut Database| Some(&mut db.users),
);

// Build query: filter, limit, offset, then execute
let results = users_kp
    .query()
    .filter(|u| u.active)
    .filter(|u| u.age > 26)
    .limit(10)
    .offset(0)
    .execute(&db);

let count = users_kp.query().filter(|u| u.active).count(&db);
let first = users_kp.query().filter(|u| u.active).first(&db);
let any = users_kp.query().filter(|u| u.active).exists(&db);
```

**With a static keypath** (e.g. from `#[derive(Kp)]`):

```rust
use key_paths_iter::QueryableCollectionStatic;

let results = Company::employees()
    .query()
    .filter(|e| e.active)
    .limit(5)
    .execute(&company);
```

**Traits:** `QueryableCollection<'a, Root, Item>` for `KpType<'a, Root, Vec<Item>>`; `QueryableCollectionStatic<Root, Item>` for `KpType<'static, Root, Vec<Item>>`.

---

## 3. Parallel API (feature `rayon`)

Use when you have `KpType<'static, Root, Vec<Item>>` and want parallel collection ops.

**Import:**

```rust
use key_paths_iter::query_par::ParallelCollectionKeyPath;
```

**Main methods:**

| Category        | Methods |
|----------------|---------|
| Map / transform | `par_map`, `par_filter`, `par_filter_map`, `par_flat_map`, `par_map_with_index` |
| Reduce / aggregate | `par_fold`, `par_reduce`, `par_count`, `par_count_by` |
| Search         | `par_find`, `par_find_any`, `par_any`, `par_all`, `par_contains` |
| Min / max      | `par_min`, `par_max`, `par_min_by_key`, `par_max_by_key` |
| Partition / group | `par_partition`, `par_group_by` |
| Ordering       | `par_sort`, `par_sort_by_key` |
| Side effects   | `par_for_each` |

**Examples:**

```rust
let kp = Company::employees();  // KpType<'static, Company, Vec<Employee>>

let salaries = kp.par_map(&company, |e| e.salary);
let active = kp.par_filter(&company, |e| e.active);
let n_active = kp.par_count_by(&company, |e| e.active);
let all_ok = kp.par_all(&company, |e| e.salary > 0);
let total = kp.par_fold(&company, &(|| 0u32), |acc, e| acc + e.salary, |a, b| a + b);
```

---

## 4. scale_par (GPU-style validation & pipelines)

With feature **`rayon`**, the **`scale_par`** module provides:

- **Types:** `ComputeState`, `GpuBuffer`, `GpuComputePipeline`, `InteractionNet`, `NetNode`, `NodeKind`, `RedexPair`, `GpuBufferData`, `GpuDispatchConfig`
- **Validation:** `validate_nodes_parallel`, `validate_active_pairs`, `validate_for_gpu`
- **Extraction:** `extract_gpu_data`, `slice_collection`
- **Pre/post:** `preprocess_sort_pairs`, `count_nodes_by_kind`, `process_gpu_results`, `adaptive_gpu_dispatch`
- **Buffers:** `par_scale_buffers`, `par_validate_buffers_non_empty`, `par_flat_map_buffer_data`

**Pattern:** Build keypaths to your pipeline’s `Vec<NetNode>` and `Vec<(u32,u32)>` (or `Vec<GpuBuffer>`), then call these functions with `&keypath` and `&root` / `&mut root`. Mutable access uses **`get_mut(root)`** from rust_key_paths.

**Example:**

```rust
use key_paths_iter::scale_par::{validate_for_gpu, extract_gpu_data, par_scale_buffers, adaptive_gpu_dispatch};
use rust_key_paths::Kp;

let nodes_kp = Kp::new(
    |p: &GpuComputePipeline| Some(&p.reduction_net.nodes),
    |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.nodes),
);
let pairs_kp = Kp::new(
    |p: &GpuComputePipeline| Some(&p.reduction_net.active_pairs),
    |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.active_pairs),
);

validate_for_gpu(&pipeline, &nodes_kp, &pairs_kp, 1_000_000)?;
let gpu_data = extract_gpu_data(&pipeline, &nodes_kp, &pairs_kp);
let config = adaptive_gpu_dispatch(&pipeline, &nodes_kp, &pairs_kp);
```

**GPU feature:** With **`gpu`** you get `GpuCompute::new()`, `execute_reduction()`, and `run_gpu_reduction_pipeline(...)`. Requires wgpu and a GPU adapter.

---

## 5. Rayon tuning (feature `rayon`)

- **Module:** `key_paths_iter::rayon_optimizations`
- **Helpers:** thread pool presets, chunk sizing, `RayonProfiler::compare_parallel_vs_sequential`, `ChunkSizeOptimizer`, `RayonEnvConfig`, etc.
- **Examples (workspace root):** `rayon_config_example`, `chunk_size_example`, `optimization_guide_example`, and others — run with `cargo run --example <name>`.

---

## 6. Common patterns

1. **Keypath to `Vec<Item>`:** Use `Kp::new(|r| Some(&r.field), |r| Some(&mut r.field))` or `#[derive(Kp)]` and call the generated function (e.g. `Root::field()`).
2. **Lifetime:** For parallel APIs you need `KpType<'static, Root, Vec<Item>>`; the `get_vec_static` helper is used internally.
3. **Query vs par_*:** Use `.query().filter(...).execute(&root)` for sequential; use `kp.par_filter(&root, ...)`, `kp.par_map(&root, ...)`, etc. when the keypath is `'static` and you want parallelism.
4. **Validation pipeline:** Use `par_all` / `par_count_by` over the keypath’s collection for GPU-ready validation; optionally use `scale_par` types and helpers for full GPU pipelines.

---

## 7. Runnable examples (workspace root)

From the **rust-key-paths** workspace root (parent of key-paths-iter):

```bash
cargo run --example key_paths_iter_par_derive   # parallel with derive(Kp)
cargo run --example scale_par_gpu_validation    # scale_par validation + buffers
cargo run --example pain001_pipeline           # PAIN.001 with map, filter, PKp, AKp
cargo bench --bench scale_par_bench             # parallel vs sequential benchmark
```

Ensure `key-paths-iter` is in dev-dependencies with the features you need (e.g. `rayon`).

---

## 8. Summary

| Need                    | Use |
|-------------------------|-----|
| Query/filter/count a `Vec` via keypath | `QueryableCollection` / `QueryableCollectionStatic` and `.query()` |
| Parallel map/filter/fold over that `Vec` | Feature `rayon` + `ParallelCollectionKeyPath` (`par_map`, `par_filter`, `par_all`, etc.) |
| GPU-style validation & buffers        | Feature `rayon` + `scale_par` module |
| Actual GPU compute (wgpu)              | Feature `gpu` + `scale_par::run_gpu_reduction_pipeline` etc. |
| Tuning Rayon                           | Feature `rayon` + `rayon_optimizations` module |

All keypath-based APIs expect a **keypath whose value type is `Vec<Item>`** (or, for scale_par, the specific types like `Vec<NetNode>` / `Vec<GpuBuffer>`). Build keypaths with rust_key_paths’ `Kp::new` or `#[derive(Kp)]`.
