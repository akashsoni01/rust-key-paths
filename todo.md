# rust-elm → TCA Parity Roadmap & Cursor Prompt

> Goal: evolve this Elm-inspired Rust crate (`src/`) toward feature parity with
> [pointfreeco/swift-composable-architecture (TCA)](https://github.com/pointfreeco/swift-composable-architecture/tree/main/Sources/ComposableArchitecture),
> staying idiomatic Rust: zero-cost, lock-free, `async`/Tokio, no `Box<dyn Fn>` on hot paths.
>
> **Scope for THIS cycle:** core engine + composition building blocks ONLY.
> **Deferred to next year (do NOT start):** all UI bindings and all *our* annotation/
> proc-macro work (§D). Using the external `rust-key-paths` derive is allowed.

---

## 0. Cursor working prompt (read first)

You are extending a Rust port of The Elm Architecture toward TCA parity. Loop:

1. **Pick the lowest-numbered unchecked task** in a phase whose prerequisites are done.
2. **Read referenced files first.** Reuse existing types: `Cmd`, `Effect`, `Sub`,
   `Runtime`, `InterpretCtx`, `Environment`, `Slot`.
3. **Map TCA concept → Rust idiom** (glossary below). Don't copy Swift 1:1; prefer
   enums + fn pointers + generics + `rust-key-paths` over protocol witnesses / `Box<dyn>`.
4. **TDD**: unit tests in-module + an integration test in `tests/`. Run
   `cargo test` and `cargo test --features serde`. Keep green.
5. **Effects stay pure descriptions** — interpretation only in `runtime.rs`.
6. **Update `ROADMAP.md` + this file** (check the box) when a task fully passes.
7. **Token discipline:** small surgical diffs; don't rewrite whole files.
8. **Stay in scope:** skip §D entirely this cycle.

### TCA → rust-elm glossary

| TCA concept | rust-elm target |
|-------------|-----------------|
| `Reducer` / `Reduce` | `update(&mut S, A) -> Cmd<A>` + new `Reducer` trait |
| `Store` | `Runtime<M, Msg>` (+ new cloneable `Store` handle) |
| `Effect` | `Effect<M>` (`src/effect.rs`) |
| `@Dependency` / `DependencyValues` | `Environment` + typed env stack (`src/env.rs`) |
| `TestStore` (exhaustive) | `TestRuntime` + new `ExhaustiveTestStore` |
| `KeyPath` (state lens) | `rust-key-paths` `Kp` via `Struct::field()` |
| `CasePath` (action prism) | `rust-key-paths` `Kp` via `Enum::variant()` |
| `Scope` / `ifLet` / `forEach` | `Slot` + new combinators using `Kp` |
| `IdentifiedArray` | new `IdentifiedVec` |
| `@Shared` | new `shared.rs` |
| `BindingReducer`, `@ObservableState`, `@Reducer` macros | **§D — deferred** |

---

## 1. Current state (already implemented in `src/`)

- [x] `cmd.rs` — `Cmd::none/batch/map/into_effects`
- [x] `effect.rs` — Task, EnvTask, Batch, MapMsg, Cancellable, Provide, Retry, Timeout, Sequence, Race, Catch
- [x] `sub.rs` — `Sub` tick/stream/websocket-stub/map
- [x] `program.rs` — `Program{init,update,subscriptions}`
- [x] `runtime.rs` — bus-driven update loop, effect interpreter, sub diffing, pinned thread
- [x] `bus.rs` — lock-free MPMC bus + backpressure
- [x] `batch.rs` — fiber-local coalescing
- [x] `interp.rs` — env stack, sequence queue, `normalize`
- [x] `env.rs` — `Environment`, `FakeClock`, `MockHttp`
- [x] `error.rs` — `EffectError`
- [x] `component.rs` — `Slot`, `lift`
- [x] `replay.rs` — `ReplayHarness`, `ReplayLog` (serde)
- [x] `test_runtime.rs` — `TestRuntime`, `assert_effect!`
- [x] `macros.rs` — `total_update!`, `arbitrary_msg!`

---

## 2. Key paths integration (replaces hand-rolled optics)

Use the external crate [`rust-key-paths`](https://github.com/akashsoni01/rust-key-paths)
(`rust-key-paths = "2.3.0"`, `key-paths-derive = "2.3.0"`) for all state/action focusing.
`Kp` unifies lens (struct fields) and prism (enum variants); compose with `.then()`,
read with `.get()`, write with `.get_mut()`.

- [x] **2.1** Add `rust-key-paths` + `key-paths-derive` to `Cargo.toml`. Re-export a
      `keypath` prelude module from `src/lib.rs` (`pub use rust_key_paths::*`).
- [x] **2.2** `src/optics.rs` — thin adapter/alias layer over `Kp` so the rest of the
      crate refers to `StateKey<Whole, Part>` / `ActionCase<Whole, Part>` type aliases.
      No bespoke lens/prism logic — delegate to `Kp`.
- [x] **2.3** Helper: `embed`/`extract` action wrappers built from `Enum::variant()` Kps
      (needed by `Scope`/`forEach` to inject child actions back into parent).
- [x] **2.4** Smoke tests: `then()` composition over nested `Option`/`Box`/enum on a
      sample state; confirm `get`/`get_mut` round-trips. (Laws covered upstream.)

> Note: `#[derive(Kp)]` is an *external* derive — allowed. Writing our own derive is §D.

## 3. Reducer model (TCA `Reducer.swift`, `Reduce`, `CombineReducers`)

Make logic a composable value, not just a fn pointer.

- [x] **3.1** `src/reducer.rs`: `trait Reducer { type State; type Action; fn reduce(&self, &mut State, Action) -> Cmd<Action>; }`.
- [x] **3.2** Blanket impl for `fn(&mut S, A) -> Cmd<A>` so existing `update` fns are reducers.
- [x] **3.3** `Reduce` — wraps a closure for inline reducers.
- [x] **3.4** `CombineReducers` — run reducers in order, merge their `Cmd`s (`Cmd::batch`).
- [x] **3.5** `reducers![a, b, c]` decl macro → `CombineReducers` (decl macro, not proc — allowed).
- [x] **3.6** `Runtime`/`Program` accept any `R: Reducer` (keep fn-pointer path working).
- [x] Tests: combine ordering, blanket impl, cmd merge.

## 4. Scope & composition operators (TCA `Scope`, `ifLet`, `ifCaseLet`, `forEach`)

All built on §2 `Kp` + §3 `Reducer`.

- [ ] **4.1** `Scope` — focus child `State` (state Kp) + child `Action` (action Kp),
      run child reducer, re-embed child `Cmd` into parent action space.
- [ ] **4.2** `ifLet` — run child reducer when an optional child state is `Some`;
      cancel the child's in-flight effects when it becomes `None`.
- [ ] **4.3** `ifCaseLet` — `ifLet` for enum-variant child state.
- [ ] **4.4** `forEach` — run a child reducer per element of `IdentifiedVec` (§5);
      scope effects per id; cancel on element removal.
- [ ] **4.5** `optional` combinator.
- [ ] Tests: state isolation, child-effect auto-cancel on removal (TCA parity).

## 5. Identified collections (TCA `IdentifiedArray`)

- [ ] **5.1** `src/identified.rs`: `IdentifiedVec<Id, T>` — ordered, `O(1)` id lookup,
      stable order; insert/remove/get/reorder.
- [ ] **5.2** `Identifiable` trait (`fn id(&self) -> Id`).
- [ ] **5.3** `serde` round-trip behind the `serde` feature.
- [ ] Tests: ordering + id stability + remove-by-id.

## 6. Effect parity (TCA `Effects/*`)

Extend `src/effect.rs` + interpreter in `runtime.rs`.

- [ ] **6.1** `Effect::cancel(id)` as a first-class effect (today external via `Runtime::cancel`).
- [ ] **6.2** `Effect::cancellable(id, cancel_in_flight: bool)` flag (TCA option).
- [ ] **6.3** `Effect::debounce(id, dur)`.
- [ ] **6.4** `Effect::throttle(id, dur, latest)`.
- [ ] **6.5** `Effect::run` — emitter effect that can `send` *many* actions over time
      (TCA `.run { send in … }`); today `Task` yields exactly one `Msg`.
- [ ] **6.6** Document/alias `Batch` = merge, `Sequence` = concatenate; add `merge`/`concatenate` ctors.
- [ ] **6.7** `Result`→action helper alongside existing `task_try` (TCA `TaskResult`).
- [ ] Tests: debounce coalescing, throttle-latest, cancel-in-flight, multi-send run, cancel-by-id.

## 7. Dependencies (TCA `Dependencies/*`)

Upgrade `env.rs` into a typed dependency container.

- [ ] **7.1** `DependencyValues` — typed map keyed by `TypeId`; `get::<D>()`, `with::<D>()`.
- [ ] **7.2** `DependencyKey` trait with `live` / `test` / `preview` defaults;
      missing `test` value fails loudly.
- [ ] **7.3** Scoped single-dependency override (generalize `Effect::provide`).
- [ ] **7.4** Built-in controllable deps: `Clock` (real + `FakeClock`), `Uuid` (seeded),
      `Now`/date, `Rng` (seedable). Deterministic under test config.
- [ ] Tests: deterministic uuid/clock/rng in tests; live vs test selection.

## 8. Store / Runtime ergonomics (TCA `Store.swift`, `Core.swift`)

- [ ] **8.1** `Store` — cloneable dispatch handle wrapping `bus_sender`.
- [ ] **8.2** `Store::subscribe_state()` → stream of `Arc<State>` snapshots, deduped via `PartialEq`.
- [ ] **8.3** `Store::scope(state_kp, action_kp)` → child `Store` (uses §2).
- [ ] **8.4** `StoreTask` — awaitable handle for in-flight effects (`send().finish()`).
- [ ] **8.5** Re-entrancy guard/docs: `update` never dispatches synchronously into itself.
- [ ] Tests: scoped store routes actions; state stream dedups.

## 9. Test store (TCA `TestStore.swift`)

- [ ] **9.1** `src/test_store.rs`: `ExhaustiveTestStore` — assert state after each `send`;
      require every effect-produced action to be consumed.
- [ ] **9.2** `send(action, |state| {…})` — expected-state mutation closure with diff on mismatch.
- [ ] **9.3** `receive(action)` — await an effect-produced action (with timeout).
- [ ] **9.4** `finish()` — assert no in-flight effects remain.
- [ ] **9.5** Non-exhaustive mode toggle (TCA `exhaustivity = .off`).
- [ ] Tests: clear diffs on failure; long-running effect detection.

## 10. Shared state (TCA `Sharing/*` — state engine only)

- [ ] **10.1** `src/shared.rs`: `Shared<T>` — ref-counted shared value with change notify.
- [ ] **10.2** Persistence trait: `InMemory`, `FileStorage` (serde). (No UI `AppStorage`.)
- [ ] **10.3** Snapshot/restore hook into `replay.rs`.
- [ ] Tests: cross-scope visibility; file persistence round-trip.

## 11. Cross-cutting / quality

- [ ] **11.1** Real WebSocket sub (`tokio-tungstenite`) replacing the `sub.rs` stub.
- [ ] **11.2** Gate Tokio behind a `runtime` feature; keep pure descriptions runtime-agnostic.
- [ ] **11.3** Miri coverage for new code (`identified`, `shared`, scope routing).
- [ ] **11.4** Benchmarks for `scope`/`forEach` dispatch in `benches/`.
- [ ] **11.5** `cargo clippy -- -D warnings` clean; remove dead-code warnings.
- [ ] **11.6** Extend `README.0.1.0.md` per new module; add an `examples/` per feature.

---

## Suggested execution order (this cycle)

```
2 (key paths) → 3 (reducer) → 5 (identified) → 4 (scope/ifLet/forEach)
→ 6 (effects) → 7 (dependencies) → 8 (store) → 9 (test store)
→ 10 (shared) → 11 (polish)
```

Key paths + Reducer are the foundation; every composition operator depends on them.

## Definition of done (per task)

- New/changed code has unit tests + ≥1 `tests/` integration case.
- `cargo test`, `cargo test --features serde`, `cargo clippy -D warnings` pass.
- `ROADMAP.md` and this `todo.md` checkbox updated.
- Public API re-exported in `src/lib.rs`, documented in `README.0.1.0.md`.

---

## §D. DEFERRED — next year (do NOT implement this cycle)

UI integration and *our own* annotation/proc-macro layers:

- [ ] **D.1** Bindings: `BindingAction`, `BindingReducer`, `binding!` (UI two-way binding).
- [ ] **D.2** Navigation/presentation: `StackState`, `StackAction`, `@Presents`, dismiss
      (state pieces may surface earlier via `IdentifiedVec`/`forEach`, but presentation is deferred).
- [ ] **D.3** Observation: `#[derive(ObservableState)]` field-level change tracking.
- [ ] **D.4** `#[reducer]` attribute proc-macro (auto State/Action/Reducer wiring).
- [ ] **D.5** Our own key-path / case-path derive (use external `rust-key-paths` instead).
- [ ] **D.6** Any SwiftUI/UIKit-equivalent view layer.

---

References:
[TCA source tree](https://github.com/pointfreeco/swift-composable-architecture/tree/main/Sources/ComposableArchitecture) ·
[rust-key-paths](https://github.com/akashsoni01/rust-key-paths)
