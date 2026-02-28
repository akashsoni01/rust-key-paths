# 🔑 KeyPaths in Rust

Key paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **KeyPath and Functional Lenses** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

## Starter Guide

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-key-paths = "2.0.6"
key-paths-derive = "2.0.6"
```

### Basic usage

```rust
use std::sync::Arc;
use key_paths_derive::Kp;

#[derive(Debug, Kp)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
    scfs2: Arc<std::sync::RwLock<SomeOtherStruct>>,
}

#[derive(Debug, Kp)]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Kp)]
enum SomeEnum {
    A(String),
    B(Box<DarkStruct>),
}

#[derive(Debug, Kp)]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Kp)]
struct DarkStruct {
    dsf: Option<String>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(Box::new(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    }))),
                }),
            }),
            scfs2: Arc::new(std::sync::RwLock::new(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(Box::new(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    }))),
                }),
            })),
        }
    }
}
fn main() {
    let mut instance = SomeComplexStruct::new();

    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf())
        .get_mut(&mut instance).map(|x| {
        *x = String::from("🖖🏿🖖🏿🖖🏿🖖🏿");
    });

    println!("instance = {:?}", instance.scsf.unwrap().sosf.unwrap().omse.unwrap());
    // output - instance = B(DarkStruct { dsf: Some("🖖🏿🖖🏿🖖🏿🖖🏿") })
}
```

### Composing keypaths

Chain through nested structures with `then()`:

```rust
#[derive(Kp)]
struct Address { street: String }

#[derive(Kp)]
struct Person { address: Box<Address> }

let street_kp = Person::address().then(Address::street());
let street = street_kp.get(&person);  // Option<&String>
```

### Partial and Any keypaths

Use `#[derive(Pkp, Akp)]` (requires `Kp`) to get type-erased keypath collections:

- **PKp** – `partial_kps()` returns `Vec<PKp<Self>>`; value type erased, root known
- **AKp** – `any_kps()` returns `Vec<AKp>`; both root and value type-erased for heterogeneous collections

Filter by `value_type_id()` / `root_type_id()` and read with `get_as()`. For writes, dispatch to the typed `Kp` (e.g. `Person::name()`) based on TypeId.

See examples: `pkp_akp_filter_typeid`, `pkp_akp_read_write_convert`.

### GPU / wgpu (key-paths-iter, optional)

The [key-paths-iter](https://github.com/codefonsi/rust-key-paths) crate can run **numeric** keypaths (e.g. `f32`) on the GPU via wgpu. Two styles:

- **AKp runner** (`wgpu` module): `IntoNumericAKp` from Kp, `AKpTier::Numeric` / `Arbitrary`, `AKpRunner`. Examples: `kp_pkp_wgpu`, `akp_wgpu_runner`.
- **Kp-only** (`kp_gpu` module): no AKp/PKp — `.map_gpu(wgsl)`, `.par_gpu(wgsl, roots, ctx)`, `GpuKpRunner`. Example: `kp_gpu_example`.

Run benchmarks: `cargo bench --bench akp_cpu_bench`. Typical results (MacBook Air M1):

| Roots   | Serial (CPU) | Parallel CPU (Rayon) | Parallel GPU (wgpu) |
|--------|---------------|----------------------|---------------------|
| 1,000  | ~35 µs        | ~86 µs               | ~1.6 ms             |
| 10,000 | ~350 µs       | ~425 µs              | ~1.9 ms             |
| 50,000 | ~1.8 ms       | ~1.8 ms              | ~3.7 ms             |
| 100,000| ~3.7 ms       | ~3.7 ms              | ~5.5 ms             |

For this lightweight transform, CPU wins; GPU pays off for larger batches or heavier per-element math.

### Features

| Feature | Description |
|---------|-------------|
| `parking_lot` | Use `parking_lot::Mutex` / `RwLock` instead of `std::sync` |
| `tokio` | Async lock support (`tokio::sync::Mutex`, `RwLock`) |
| `pin_project` | Enable `#[pin]` field support for pin-project compatibility |

### More examples

```bash
cargo run --example kp_derive_showcase
cargo run --example pkp_akp_filter_typeid
cargo run --example pkp_akp_read_write_convert
# Kp/Pkp + wgpu (key-paths-iter with gpu feature)
cargo run --example kp_pkp_wgpu
cargo run --example akp_wgpu_runner
cargo run --example kp_gpu_example
# Box and Pin support
cargo run --example box_and_pin_example
# pin_project #[pin] fields
cargo run --example pin_project_example --features pin_project
cargo run --example pin_project_fair_race --features "pin_project,tokio"
# Deadlock prevention (parallel execution)
cargo run --example deadlock_prevention_sync --features parking_lot
cargo run --example deadlock_prevention_async --features tokio
```

---

## Supported containers

The `#[derive(Kp)]` macro (from `key-paths-derive`) generates keypath accessors for these wrapper types:

| Container | Access | Notes |
|-----------|--------|-------|
| `Option<T>` | `field()` | Unwraps to inner type |
| `Box<T>` | `field()` | Derefs to inner |
| `Pin<T>`, `Pin<Box<T>>` | `field()`, `field_inner()` | Container + inner (when `T: Unpin`) |
| `Rc<T>`, `Arc<T>` | `field()` | Derefs; mut when unique ref |
| `Vec<T>` | `field()`, `field_at(i)` | Container + index access |
| `HashMap<K,V>`, `BTreeMap<K,V>` | `field_at(k)` | Key-based access |
| `HashSet<T>`, `BTreeSet<T>` | `field()` | Container identity |
| `VecDeque<T>`, `LinkedList<T>`, `BinaryHeap<T>` | `field()`, `field_at(i)` | Index where applicable |
| `Result<T,E>` | `field()` | Unwraps `Ok` |
| `Cow<'_, T>` | `field()` | `as_ref` / `to_mut` |
| `Option<Cow<'_, T>>` | `field()` | Optional Cow unwrap |
| `std::sync::Mutex<T>`, `std::sync::RwLock<T>` | `field()` | Container (use `LockKp` for lock-through) |
| `Arc<Mutex<T>>`, `Arc<RwLock<T>>` | `field()`, `field_lock()` | Lock-through via `LockKp` |
| `tokio::sync::Mutex`, `tokio::sync::RwLock` | `field_async()` | Async lock-through (tokio feature) |
| `parking_lot::Mutex`, `parking_lot::RwLock` | `field()`, `field_lock()` | parking_lot feature |

Nested combinations (e.g. `Option<Box<T>>`, `Option<Vec<T>>`, `Vec<Option<T>>`) are supported.

### pin_project `#[pin]` fields (optional feature)

When using [pin-project](https://docs.rs/pin-project), mark pinned fields with `#[pin]`. The derive generates:

| `#[pin]` field type | Access | Notes |
|---------------------|--------|-------|
| Plain (e.g. `i32`) | `field()`, `field_pinned()` | Pinned projection via `this.project()` |
| `Future` | `field()`, `field_pinned()`, `field_await()` | Poll through `Pin<&mut Self>` |
| `Box<dyn Future<Output=T>>` | `field()`, `field_pinned()`, `field_await()` | Same for boxed futures |

Enable with `pin_project` feature and add `#[pin_project]` to your struct:

```rust
#[pin_project]
#[derive(Kp)]
struct WithPinnedFuture {
    fair: bool,
    #[pin]
    fut: Pin<Box<dyn Future<Output = String> + Send>>,
}
```

Examples: `pin_project_example`, `pin_project_fair_race` (FairRaceFuture use case).

## Performance: Kp vs direct unwrap

Benchmark: nested `Option` chains and enum case paths (`cargo bench --bench keypath_vs_unwrap`).

| Scenario | Keypath | Direct unwrap | Overhead |
|----------|---------|---------------|----------|
| 100× reuse (3-level) | ~36.6 ns | ~36.7 ns | ~1x |
| 100× reuse (5-level) | ~52.3 ns | ~52.5 ns | ~1x |

Access overhead comes from closure indirection in the composed chain. **Reusing a keypath** (build once, use many times) matches direct unwrap; building the chain each time adds ~1–2 ns.

### Would static keypaths help?

Yes. Static/const keypaths would:
- Remove creation cost entirely (no closure chain construction per use)
- Allow the compiler to inline the full traversal
- Likely close the gap to near-zero overhead vs manual unwrap

Currently, `Kp::then()` composes via closures that capture the previous step, so each access goes through a chain of function calls. A static keypath could flatten this to direct field offsets.

---

## Performance: LockKp (Arc&lt;Mutex&gt;, Arc&lt;RwLock&gt;)

| Operation | Keypath | Direct Locks | Overhead |
|-----------|---------|--------------|----------|
| **Read**  | ~241 ns | ~117 ns      | ~2.1x    |
| **Write** | ~239 ns | ~114 ns      | ~2.1x    |

The keypath approach builds the chain each iteration and traverses through `LockKp.then().then().then_async().then()`; direct locks use `sync_mutex.lock()` then `tokio_mutex.lock().await`. Hot-path functions are annotated with `#[inline]` for improved performance.

### 10-level deep Arc&lt;RwLock&gt; benchmarks (leaf: f64)

Benchmark: 10 levels of nested `Arc<RwLock<Next>>`, reading/writing leaf `f64`. Run with:
- `cargo bench --features parking_lot --bench ten_level_arc_rwlock`
- `cargo bench --bench ten_level_std_rwlock`
- `cargo bench --features tokio --bench ten_level_tokio_rwlock`

**Incr** (write: leaf += 0.25):

| RwLock implementation | keypath_static | keypath_dynamic | direct_lock |
|-----------------------|----------------|-----------------|-------------|
| **parking_lot**       | ~34 ns         | ~41 ns          | ~39 ns      |
| **std::sync**         | ~46 ns         | ~54 ns          | ~46 ns      |
| **tokio::sync**       | ~1.79 µs       | ~1.78 µs        | ~278 ns     |

Static keypath (chain built once, reused) matches or beats direct lock for sync RwLocks. For tokio, async keypath has higher overhead than direct `.read().await`/`.write().await`; direct lock is fastest.

---

## 📜 License

* Mozilla Public License 2.0
