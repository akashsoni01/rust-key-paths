# 🔑 KeyPaths in Rust

Key paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **KeyPath and Functional Lenses** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

| Operation | Keypath | Direct Locks | Overhead |
|-----------|---------|--------------|----------|
| **Read**  | ~241 ns | ~117 ns      | ~2.1x    |
| **Write** | ~239 ns | ~114 ns      | ~2.1x    |

The keypath approach builds the chain each iteration and traverses through `LockKp.then().then().then_async().then()`; direct locks use `sync_mutex.lock()` then `tokio_mutex.lock().await`. The keypath overhead reflects chain construction plus composed traversal vs. manual lock nesting. Hot-path functions are annotated with `#[inline]` for improved performance.

---

## 📜 License

* Mozilla Public License 2.0