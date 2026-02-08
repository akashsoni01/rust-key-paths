# 🔑 KeyPaths in Rust

Key paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **KeyPath and Functional Lenses** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

## Performance: Kp vs direct unwrap

Benchmark: nested `Option` chains and enum case paths (`cargo bench --bench keypath_vs_unwrap`).

| Scenario | Keypath | Direct unwrap | Overhead |
|----------|---------|---------------|----------|
| Read 3-level Option | ~2.25 ns | ~387 ps | ~5.8x |
| Write 3-level Option | ~854 ps | ~381 ps | ~2.2x |
| Read 5-level Option | ~3.54 ns | ~383 ps | ~9.2x |
| Read 5-level with enum | ~3.52 ns | ~392 ps | ~9x |
| 100× reuse (3-level) | ~36.6 ns | ~36.7 ns | ~1x |

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

---

## 📜 License

* Mozilla Public License 2.0