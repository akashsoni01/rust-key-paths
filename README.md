# 🔑 KeyPaths in Rust

Key paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **KeyPath and Functional Lenses** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

## Supported containers

The `#[derive(Kp)]` macro (from `key-paths-derive`) generates keypath accessors for these wrapper types:

| Container | Access | Notes |
|-----------|--------|-------|
| `Option<T>` | `field()` | Unwraps to inner type |
| `Box<T>` | `field()` | Derefs to inner |
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

## Performance: Kp vs direct unwrap

Benchmark: nested `Option` chains and enum case paths (`cargo bench --bench keypath_vs_unwrap`).

| Scenario | Keypath | Direct unwrap | Overhead |
|----------|---------|---------------|----------|
| Read 3-level Option | ~2.25 ns | ~387 ps | ~5.8x |
| Write 3-level Option | ~854 ps | ~381 ps | ~2.2x |
| Read 5-level Option | ~3.54 ns | ~383 ps | ~9.2x |
| Read 5-level with enum | ~3.52 ns | ~392 ps | ~9x |
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

### 10-level deep Arc&lt;RwLock&gt; read benchmarks

Benchmark: 10 levels of nested `Arc<RwLock<Next>>`, reading leaf `i32`. Run with:
- `cargo bench --features parking_lot --bench ten_level_arc_rwlock`
- `cargo bench --bench ten_level_std_rwlock`
- `cargo bench --features tokio --bench ten_level_tokio_rwlock`

| RwLock implementation | keypath_static | keypath_dynamic | direct_lock |
|-----------------------|----------------|-----------------|-------------|
| **parking_lot**       | ~33 ns         | ~40 ns          | ~40 ns      |
| **std::sync**         | ~97 ns         | ~108 ns         | ~49 ns      |
| **tokio::sync**       | ~1.74 µs       | ~1.61 µs        | ~255 ns     |

Static keypath (chain built once, reused) matches or beats direct lock for sync RwLocks. For tokio, async keypath has higher overhead than direct `.read().await`; direct lock is fastest.

---

## 📜 License

* Mozilla Public License 2.0