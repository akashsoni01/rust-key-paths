# rust-elm 0.7.0

Elm Architecture for Rust, evolving toward The Composable Architecture (UDF).

## Install

```toml
[dependencies]
rust-elm = { path = "../rust-elm" }
rust-elm = "0.7.0"
rust-key-paths = "3.4.0"
key-paths-derive = "3.3.0"
tokio = { version = "1.38", features = ["rt-multi-thread", "macros"] }
```

### Feature flags

| Feature | Default | Purpose |
|---------|---------|---------|
| `runtime` | yes | Tokio interpreter, `Runtime`, `RwRuntime`, `TeaRuntime`, `Store`, subscriptions |
| `arc-swap` | no | `SwapRuntime` / `SwapStore` — lock-free snapshot reads (`arc-swap` crate) |
| `websocket` | yes | Real `Sub::websocket` via `tokio-tungstenite` (interval stub without it) |
| `serde` | no | `FileStorage`, replay snapshots |

Build without Tokio (pure descriptions only):

```bash
cargo build -p rust-elm --no-default-features
```

## Quick start

```rust
use rust_elm::{Cmd, Program, Runtime, RuntimeConfig, Sub, Environment};

#[derive(Default)]
struct State { count: i32 }

fn init() -> (State, Cmd<i32>) {
    (State::default(), Cmd::none())
}

fn update(state: &mut State, msg: i32) -> Cmd<i32> {
    state.count += msg;
    Cmd::none()
}

fn subscriptions(_: &State) -> Sub<i32> {
    Sub::none()
}

fn main() -> Result<(), rust_elm::RuntimeError> {
    let program = Program::new(init, update, subscriptions);
    let runtime = Runtime::from_program(program, Environment::new(), RuntimeConfig::new(64))?;
    let store = runtime.store();
    store.dispatch(1);
    runtime.shutdown();
    Ok(())
}
```

## Snapshot reads: blocking vs background

`dispatch` / `send` **never wait for a snapshot** — they only enqueue on the action bus (may block if the bus is full).

How you **read** state depends on the store backend:

| API | When snapshot is taken | Blocks caller? |
|-----|------------------------|----------------|
| `Store::state()` / `RwStore::state()` | immediately on **your thread** | **Yes** — locks `Mutex` / `RwLock`, clones `S` |
| `Store::binding()` / `with_read` | immediately on **your thread** | **Yes** — holds lock for the closure |
| `SwapStore::snapshot_store().load()` | immediately on **your thread** | **No lock** — atomic `Arc` load (may be slightly stale) |
| `Store::subscribe_state()` / `subscribe_changes()` | after reduce (+ effect batch for `send`) | **Background notify** — `next()` / `wait_next()` **block** until a ping or push arrives |
| `TeaStore::subscribe_state()` | reducer **pushes** `Arc<S>` after each `update` | **Background push** — `next()` / `wait_next()` **block** on the channel |
| `TeaStore::state()` / `TeaViewStore::load()` | RPC to reducer thread | **Yes** — caller blocks until reducer replies on control channel |

**Rule of thumb:** subscriptions are **push/async relative to reduce** (you wait on a channel). One-off `state()` / `load()` / `with_snapshot()` are **synchronous reads** on the calling thread (Tea RPC blocks until the reducer responds; Mutex/RwLock blocks on the lock).

See [tea_ecommerce.md](book/tea_ecommerce.md) (channel TEA), [swap_ecommerce.md](book/swap_ecommerce.md) (atomic snapshots), and [store.md](book/store.md) (API reference).

## Release notes

### 0.7.0

- **`arc-swap` feature** — optional `SwapRuntime`, `SwapStore`, `SnapshotStore`, snapshot bindings.
- **`swap_ecommerce` example** — lock-free snapshot reads on the shared shop domain.
- **`TeaRuntime`** / **`TeaStore`** — channel-pushed model (true TEA: no shared mutable state).
- **`tea_ecommerce` example** — same shop with channel-delivered snapshots.
- **`RuntimeError`** / **`TeaStoreError`** — fallible runtime bootstrap and tea snapshot reads (no `unwrap` / `expect` in library code).
- **No `unsafe`** — `BusSender` drop counter uses `Arc<AtomicU64>` instead of raw pointers.

### 0.6.0

- **`RwRuntime`** / **`RwStore`** / **`ReadStore`** — read-heavy store API on `RwLock`.
- **`SwapRuntime`** / **`SwapStore`** (`arc-swap` feature) — lock-free snapshot reads via `ArcSwap`.
- **`StoreHub`** — shared dispatch hub for mutex, Rw, and swap backends.

### 0.5.0

- **`safe_reducer`** module — `safe_reduce_update`, `safe_reduce_rollback`, `SafeReduceError` (replaces `reduce_panic` / `catch_reduce_panic` / `ReducePanic`).
- **`StateBinding`** / **`subscribe_changes`** — zero-copy store reads and field-level change signals; see [binding.md](book/binding.md).
- Example renamed: `safe_reducer` (was `catch_reduce`).

### 0.3.0

- **`RuntimeConfig`** — `bus_capacity`, `worker_threads`, `thread_name` for `Runtime::from_program`.
- **`RefKpTrait`** — scope/store/optics use core traits directly; `StateLens` / `StateKeypath` removed.
- **`StoreTask::finish_with_timeout`** — configurable wait for store work.
- **`pain` example** — ISO 20022-style validation with fail-fast `Validator`.

### 0.2.0

- WebSocket subscriptions, runtime feature flags, ecommerce example.

## Modules

| Module | Purpose |
|--------|---------|
| `cmd` | Commands returned from `update` |
| `effect` | Pure async effect descriptions (`debounce`, `throttle`, `from_run`, `cancel`) |
| `sub` | Subscription descriptions (runtime-agnostic data) |
| `safe_reducer` | `safe_reduce_update`, `safe_reduce_rollback`, `SafeReduceError` |
| `runtime` | Bus-driven update loop + Tokio interpreter (`runtime` feature) |
| `subscription` | Sub interpreter — tick/stream/websocket (`runtime` feature) |
| `store` | `Store`, `RwStore`, `TeaStore`, `SwapStore` (`arc-swap`), `StoreTask`, scoped stores (`runtime`) |
| `test_store` | `ExhaustiveTestStore` for synchronous effect/action testing |
| `test_runtime` | Sync testing without Tokio |
| `shared` | `Shared<T>`, `Storage`, `InMemoryStorage`, `FileStorage` (serde) |
| `dependencies` | Re-exports [`rust_dependencies`](../rust_dependencies) |
| `env` | `Environment` (`live`/`test`), `FakeClock`, `MockHttp` |
| `optics` | State/action focusing via `rust-key-paths` |
| `reducer` | `Reducer` trait, `CombineReducers`, `CatchReducer`, `RollbackCatchReducer` |
| `scope` | `ScopeReducer`, `IfLetReducer`, `ForEachReducer`, `lift_cmd` |
| `identified` | Re-exports [`rust_identified_vec`](../rust_identified_vec) |
| `replay` | Action log + replay harness |

## Examples (one per feature area)

| Example | Command | Shows |
|---------|---------|-------|
| `safe_reducer` | `cargo run -p rust-elm --example safe_reducer` | Safe update: default vs rollback |
| `subscriptions` | `cargo run -p rust-elm --example subscriptions` | Tick + map_msg subs |
| `scope_for_each` | `cargo run -p rust-elm --example scope_for_each` | Scope + ForEach without Runtime |
| `dependencies` | `cargo run -p rust-elm --example dependencies` | `Environment` live vs test |
| `shared_state` | `cargo run -p rust-elm --example shared_state` | `Shared<T>` observers |
| `ecommerce` | `cargo run -p rust-elm --example ecommerce` | Full shop (mutex `Runtime`) |
| `rw_ecommerce` | `cargo run -p rust-elm --example rw_ecommerce` | Same shop on `RwRuntime` |
| `swap_ecommerce` | `cargo run -p rust-elm --example swap_ecommerce --features arc-swap` | Same shop on `SwapRuntime` |
| `tea_ecommerce` | `cargo run -p rust-elm --example tea_ecommerce` | Same shop on `TeaRuntime` (channel TEA) |

## Quality tooling

```bash
# Clippy (crate only)
cargo clippy -p rust-elm --all-targets --no-deps -- -D warnings

# Miri (identified, shared, scope — no runtime)
./scripts/miri-rust-elm.sh

# Scope / ForEach benchmarks
cargo bench -p rust-elm --bench scope_dispatch
```

## Key paths prelude

```rust
use rust_elm::keypath::{Kp, KpTrait, RefKpTrait, Readable, Writable};
use key_paths_derive::Kp;
```

See [`ROADMAP.md`](ROADMAP.md), [`book/architecture.md`](book/architecture.md), [`book/safe_reducer.md`](book/safe_reducer.md), [`book/ecommerce.md`](book/ecommerce.md), [`book/rw_ecommerce.md`](book/rw_ecommerce.md), [`book/swap_ecommerce.md`](book/swap_ecommerce.md), [`book/tea_ecommerce.md`](book/tea_ecommerce.md).
