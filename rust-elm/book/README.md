# rust-elm book

| Document | Description |
|----------|-------------|
| [architecture.md](./architecture.md) | Runtime, threading, effects, composition, actors comparison, panic strategy |
| [store.md](./store.md) | Store dispatch, scoping, legacy snapshot subscription (`subscribe_state`) |
| [binding.md](./binding.md) | Zero-copy bindings, keypath projection, field change signals (`subscribe_changes`) |
| [ecommerce.md](./ecommerce.md) | Shop example: scoping, subs, locks, panic demo, production notes, cross-thread store |
| [pain.md](./pain.md) | ISO 20022 PAIN.001 payload state + keypath field validation |
| [validation.md](./validation.md) | Reusable keypath validation framework (`Rule`, `Validator`, `Validate`) |
| [../examples/ecommerce.rs](../examples/ecommerce.rs) | Full compositional shop example (4-level actions, scopes, ifLet, forEach) |
| [../examples/catch_reduce.rs](../examples/catch_reduce.rs) | Panic handling: default vs rollback |
| [../README.0.1.0.md](../README.0.1.0.md) | Install and quick start |

---

## Throughput estimates

Numbers below are from **release** builds on Apple Silicon (M-series), reducer = `HashMap<u64,u64>::insert` + `Cmd::none()`, no subscriptions, no effects. Reproduce with:

```bash
cargo run -p rust-elm --example throughput --release
cargo bench -p rust-elm --bench hashmap_dispatch -- --noplot
```

| Workload | Estimated TPS | Notes |
|----------|----------------:|-------|
| Raw `HashMap::insert` (in-process loop) | **~30M** | Upper bound — no bus, no mutex, no notify |
| `Store::dispatch` → reduce → insert (1 thread) | **~2.2M** | End-to-end UDF path; ~460 ns/action |
| 50 threads × 1k = **50k burst** (bus ≥ 16k) | **~1.8M** | Same ceiling — reducer is single-threaded |
| 50k burst (bus = 4k) | **~460k** | Producers block on full bus (`send_blocking`) |

**Key takeaway:** parallel dispatch does **not** multiply reduce throughput. Many threads enqueue actions; **one reducer thread** applies them. Extra threads add queue depth and producer blocking, not more inserts/sec.

The insert itself is ~50 ns; the remaining ~410 ns per action is bus send/recv, `parking_lot` lock, listener notify, and `StoreTask` bookkeeping.

---

## 50k parallel requests — recommended config

Use this when many threads (HTTP handlers, workers, UI surfaces) each call `store.dispatch` and the reducer mostly updates a `HashMap`.

### Runtime / program

```rust
fn init() -> (State, Cmd<Action>) {
    (
        State {
            map: HashMap::with_capacity(50_000), // pre-size for burst
            ..Default::default()
        },
        Cmd::none(),
    )
}

// Bus capacity: third argument — hold the burst without blocking producers.
let runtime = Runtime::from_program(program, Environment::new(), 65_536);
// or: Runtime::from_reducer_program(program, env, 65_536)
```

| Knob | Recommended | Why |
|------|-------------|-----|
| **`bus_capacity`** | **65_536** (min **16_384** for 50k) | `Store::dispatch` uses `send_blocking`; a small bus (e.g. 64–4096) forces producers to wait while the reducer drains. Size ≥ expected burst so all 50k enqueues complete quickly. |
| **Reducer** | `Cmd::none()` for hot path | Effects run on Tokio after reduce; avoid spawning work per insert. |
| **Subscriptions** | `Sub::none()` or narrow gates | Sub sync runs after every reduce; disable during ingest bursts. |
| **State shape** | Flat `HashMap` in root or scoped child | Deep trees + `ForEachReducer` add ns–µs per action; benchmarks in `benches/scope_dispatch.rs`. |
| **Preallocate** | `HashMap::with_capacity(N)` in `init` | Avoids rehash during burst. |

### Store usage

| API | When |
|-----|------|
| `store.dispatch(action)` | Fire-and-forget from worker threads (`store.clone()`) |
| `runtime.dispatch(action)` | Same bus, slightly less overhead (no `StoreTask` queue) — wrap `Runtime` in `Arc` if workers need it |
| `runtime.sender().send_blocking(action)` | Lowest-level enqueue; use when you own the sender |
| `store.send(action).finish()` | Wait for effects — **avoid** on hot path (extra channel + in-flight counter) |
| `store.state()` | **Avoid** during burst — clones entire state |
| `store.binding()` | Zero-copy read/write under lock — prefer over `state()` |
| `store.subscribe_changes()` | UI / readers — field-level signals, no full-state clone |
| `store.subscribe_state()` | Legacy — coalesced `Arc<S>` snapshots (clones `S`) |

Clone `Store` onto each worker thread (`store.clone()` is cheap — shared `Arc` backend).

### Effects and Tokio

- Default Tokio (`rt-multi-thread`) is fine; the reducer thread is separate from worker threads.
- Batch side effects: return one `Effect::task` per N inserts from the reducer, or use a dedicated “flush” action, instead of `Cmd::single` per row.
- For parallel I/O (HTTP, DB), use `Effect::Batch` — effects run concurrently; **actions** still serialize on the reducer thread.

### Bus backpressure

`BusSender::send` (non-blocking) drops when full and increments `bus.dropped_count()`. Production ingress should use `send_blocking` (what `Store` uses) or retry on `send` — monitor `runtime.bus.dropped_count()` if you switch to try-send.

### When 50k parallel is the wrong model

| Need | rust-elm approach |
|------|-------------------|
| 50k **concurrent reduces** | Not supported — one reducer thread by design |
| 50k **ingest** with serial consistency | **Good fit** — ~25–30 ms to drain 50k at ~1.8M TPS |
| 50k **independent entities** | Shard: one `Runtime` per partition, or actors / raw async |
| Sub-millisecond p99 per action under load | Queue latency grows with burst size — bound with rate limiting upstream |

### Checklist for a 50k burst

1. `bus_capacity` ≥ 65_536  
2. `HashMap::with_capacity(50_000)` in initial state  
3. Reducer returns `Cmd::none()` on insert  
4. No subscriptions during ingest (or state-gated subs)  
5. Workers use `store.dispatch`, not `send().finish()`  
6. No `state()` polls until drain completes  
7. Run release: `cargo run --example throughput --release` on your hardware to validate

---

## Related benchmarks

| File | Measures |
|------|----------|
| [`benches/hashmap_dispatch.rs`](../benches/hashmap_dispatch.rs) | Raw insert vs single-thread `dispatch` |
| [`benches/scope_dispatch.rs`](../benches/scope_dispatch.rs) | Scoped / forEach reducer overhead (~2–10 ns) |
| [`examples/throughput.rs`](../examples/throughput.rs) | Printable TPS for 50k parallel scenarios |
