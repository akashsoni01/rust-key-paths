# rust-elm: lock-free state reads (`no lock`) — LLM checklist

Goal: **state snapshot reads never take `Mutex` / `RwLock`** on the hot path. Writes stay serial on one reducer thread (copy-on-write via `ArcSwap`). Docs, examples, and diagrams must match the three store backends.

**Scope:** `rust-elm/src`, `rust-elm/book/*.md`, `rust-elm/examples/*`, root `README.md` cross-links.

**Out of scope (keep as-is):** interpreter registries, subscription handle maps, `StoreHub` listener queues, `Environment` test doubles, `Shared<T>`, effect type-erasure registries — these are cold-path metadata, not state reads.

---

## Phase 0 — Baseline (run before every phase)

- [ ] `cd rust-elm && cargo test --features runtime --lib`
- [ ] `cd rust-elm && cargo test --features "runtime,arc-swap" --lib`
- [ ] `cd rust-elm && cargo check --examples`
- [ ] `cd rust-elm && cargo check --examples --features arc-swap`
- [ ] Grep inventory: `rg 'parking_lot::(Mutex|RwLock)|\.lock\(\)|\.read\(\)|\.write\(\)' rust-elm/src/runtime` — classify each hit as **state read**, **state write**, or **metadata** (document in a comment block at bottom of this file when done).

---

## Phase 1 — Source: lock-free read path (mostly done; verify & fill gaps)

### 1.1 ArcSwap backend (`arc-swap` feature)

Files: `src/runtime/swap_engine.rs`, `src/runtime/swap_store.rs`, `src/runtime/state_access.rs`, `src/runtime/binding.rs` (`SnapshotStateBinding`).

- [x] `SwapRuntime` — reducer clones snapshot, `store(Arc::new(next))`
- [x] `SwapStore` / `SnapshotStore` — `load()` / `with_snapshot()` without locks
- [x] `ScopedSwapStore`, subscribers, `StoreHub` shared with Rw/Mutex backends
- [ ] Add `tests/swap_store_integration.rs` (mirror `store_integration.rs` + concurrent reader threads)
- [ ] Add Miri smoke path: `scripts/miri-rust-elm.sh` includes `--features arc-swap` swap test
- [ ] Bench: extend `benches/hashmap_dispatch.rs` with `SwapRuntime` read-heavy scenario (optional)

**Acceptance:** `cargo test --features "runtime,arc-swap"` green; concurrent readers never call `Mutex`/`RwLock` on `S`.

### 1.2 Unify duplicated engine code (optional refactor)

Files: `engine.rs`, `rw_engine.rs`, `swap_engine.rs`.

- [ ] Extract shared reducer-loop body into `runtime/engine_loop.rs` (bus recv, effect spawn, sub sync)
- [ ] Parameterize state mutation: `with_write` (Mutex/Rw) vs clone-and-swap (ArcSwap)
- [ ] Keep public types unchanged: `Runtime`, `RwRuntime`, `SwapRuntime`

**Acceptance:** no behavior change; line count drops; all existing tests pass.

### 1.3 Binding & test-support parity

Files: `src/runtime/binding.rs`, `src/testing/test_support.rs`, `src/lib.rs` re-exports.

- [ ] Document `SnapshotProjectedBinding` in rustdoc (same patterns as `RwProjectedBinding`)
- [ ] Add `scoped_subscribe_state` / `store_state` helpers for `SwapStore` if missing
- [ ] Wire `SnapshotStateBinding` into `ExhaustiveTestStore` only if needed (likely N/A)

**Acceptance:** `binding.md` examples compile; swap bindings listed in `lib.rs` module docs.

### 1.4 Locks that stay (document why)

| Location | Lock | Reason |
|----------|------|--------|
| `StoreHub::state_listeners`, `effect_done` | `Mutex` | listener registration; not on read hot path |
| `InterpreterState::*` | `Mutex` | debounce/cancel maps; Tokio-only |
| `SubscriptionHandles` | `Mutex` | sub lifecycle |
| `RollbackCatchReducer::checkpoint` | `Mutex` | panic path only |
| `effect.rs` registries | `std::Mutex` | type-erasure; init-time registration |

- [x] Add short “Locks by design” subsection to `book/architecture.md` (table above)

---

## Phase 2 — Docs (`.md` files)

### 2.1 `book/architecture.md` (primary diagram fix)

- [x] Replace Mutex-only “System overview” with **three-backend** diagram (Store / RwStore / SwapStore)
- [x] Add “Choosing a store backend” table (read path, write path, `S: Clone` requirement, feature flag)
- [x] Update action lifecycle sequence: branch for `ArcSwap` (load → clone → reduce → store) vs lock path
- [x] Update threading table: add `SnapshotStore::load()` row
- [x] Extend module map mindmap: `swap_engine`, `swap_store`, `StoreHub`
- [x] Cross-link: [swap_ecommerce.md](./rust-elm/book/swap_ecommerce.md), [rw_ecommerce.md](./rust-elm/book/rw_ecommerce.md)

**Acceptance:** no diagram references `State (Mutex)` as the only model; mermaid renders in GitHub preview.

### 2.2 `book/store.md`

- [x] Fix “Quick reference” table — three columns (`Store` | `RwStore` | `SwapStore`), one row per task
- [ ] Update §1 anatomy snippet: note `StoreHub` extraction (Mutex/Rw/Swap share hub)
- [ ] Add § “SwapStore (`arc-swap` feature)” with `snapshot_store()`, `load()`, trade-offs
- [ ] Throughput section: mention SwapStore read path avoids read-side lock contention

### 2.3 `book/binding.md`

- [x] Add § “ArcSwap (`SwapStore`)” mirroring RwLock section:
  - `store.snapshot_store().with_snapshot(f)`
  - `snapshot_binding().project(kp)` → `SnapshotProjectedBinding`
- [x] Link to [swap_ecommerce.md](./rust-elm/book/swap_ecommerce.md)

### 2.4 Other book files

- [ ] `book/README.md` — throughput note: SwapStore for read-heavy fan-out
- [ ] `book/ecommerce.md` — store-backend picker at top (default vs rw vs swap examples)
- [ ] `README.0.1.0.md` — feature matrix accurate; `arc-swap` not default
- [x] `ROADMAP.md` — point here; mark Phase 11 polish items

### 2.5 Root `README.md`

- [ ] Crates table mentions `SwapRuntime` + link to `swap_ecommerce.md`

---

## Phase 3 — Examples

| Example | Runtime | Status |
|---------|---------|--------|
| `ecommerce.rs` | `Runtime` (Mutex) | OK — baseline |
| `rw_ecommerce.rs` | `RwRuntime` | OK |
| `swap_ecommerce.rs` | `SwapRuntime` | OK — needs `arc-swap` |
| `calculator.rs` | none (pure reduce) | **fix** — remove broken Runtime/ShopState refs |
| `throughput.rs` | `Runtime` | consider optional SwapRuntime bench flag |
| `safe_reducer.rs` | varies | verify still compiles |

Tasks:

- [x] Fix `examples/calculator.rs` — pure `CatchReducer` + `Reduce` only (no `Runtime`)
- [x] `cargo check --examples` and `cargo check --examples --features arc-swap` green
- [ ] Each example header comment: correct `cargo run` line + link to matching book doc
- [ ] Optional: `throughput.rs --features arc-swap` reports SwapStore read TPS

---

## Phase 4 — Architecture diagrams (checklist)

Ensure these mermaid diagrams exist and are consistent:

| Diagram | File | Shows |
|---------|------|-------|
| Three store backends | `architecture.md` | Mutex / RwLock / ArcSwap |
| Swap high-level | `swap_ecommerce.md` | already present — verify paths |
| Action lifecycle (swap branch) | `architecture.md` | load → clone → reduce → atomic store |
| Store flavor comparison | `swap_ecommerce.md` | already present |
| Module mindmap | `architecture.md` | include swap modules |

- [ ] All `[`../examples/...`]` and `[`./foo.md`]` links resolve from repo root and from `book/`
- [ ] Version strings in docs say `0.7` / `0.7.0` consistently

---

## Phase 5 — Future (not required for “no lock” reads)

- [ ] Make `arc-swap` a default feature? **Decision:** only if `S: Clone` bound is acceptable for all users — likely stay optional.
- [ ] Deprecate `Runtime` + `Mutex<S>`? **Decision:** keep for simple apps / no Clone state.
- [ ] Lock-free `StoreHub` listeners (`crossbeam` or intrusive list) — micro-optimization.

---

## Done when

1. Concurrent state reads use `SnapshotStore::load()` / `with_snapshot()` — **zero** `Mutex`/`RwLock` on `S`.
2. `cargo check --examples --features arc-swap` passes.
3. `book/architecture.md` documents all three backends with updated diagrams.
4. `book/store.md` quick-reference table has three columns.
5. `examples/calculator.rs` runs without compile errors.
6. This checklist: all Phase 0–4 boxes checked.

---

## Lock inventory (fill during Phase 0)

<!-- LLM: paste rg output summary here after classification -->
