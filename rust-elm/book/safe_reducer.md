# Safe reducer updates

Catch panics during `update` / `reduce` without crashing the runtime thread. The default
policy **keeps partial state** (no rollback). Opt in to checkpoint rollback when you need
it.

For store dispatch and composition context, see [store.md](./store.md) and
[architecture.md](./architecture.md).

---

## 1. API overview

| Symbol | Role |
|--------|------|
| **`safe_reduce_update`** | Run one reduce inside `catch_unwind`; state **not** reverted on panic |
| **`safe_reduce_rollback`** | Same, but swaps `state` with `checkpoint` on panic |
| **`SafeReduceError`** | Panic payload (`message()` if `&str` / `String`) |
| **`CatchReducer`** | Composable wrapper using `safe_reduce_update` + custom `recover` cmd |
| **`RollbackCatchReducer`** | Composable wrapper using `safe_reduce_rollback` |

Module: `rust_elm::safe_reducer` (re-exported at crate root).

---

## 2. Runtime default

[`Runtime`](../../src/runtime/engine.rs) always reduces through `safe_reduce_update`:

```rust
match safe_reduce_update(&mut *guard, |s, a| update(s, a), msg) {
    Ok(cmd) => { /* interpret cmd */ }
    Err(_) => { /* drop cmd for this turn */ }
}
```

A reducer panic does **not** kill the reducer thread. Partial mutations before the panic
remain in state.

---

## 3. Standalone usage

```rust
use rust_elm::{safe_reduce_update, safe_reduce_rollback, SafeReduceError, Cmd};

fn risky(state: &mut App, action: Action) -> Cmd<Action> {
    state.count += 1;
    panic!("bug");
}

// Default — partial mutation kept
let mut state = App::default();
let _ = safe_reduce_update(&mut state, risky, Action::Tick);

// Opt-in rollback
let mut state = App::default();
let mut checkpoint = state.clone();
let _ = safe_reduce_rollback(&mut state, &mut checkpoint, risky, Action::Tick);
```

---

## 4. Composable reducers

### `CatchReducer` — recover with a command

```rust
use rust_elm::{CatchReducer, Reduce, SafeReduceError, Cmd};

let safe = CatchReducer::new(
    Reduce::new(my_reducer),
    |err: SafeReduceError| {
        eprintln!("reduce failed: {:?}", err.message());
        Cmd::none()
    },
);
```

### `RollbackCatchReducer` — revert state on panic

```rust
use rust_elm::{RollbackCatchReducer, Reduce, SafeReduceError, Cmd};

let committed = initial_state.clone();
let safe = RollbackCatchReducer::new(
    Reduce::new(my_reducer),
    |_: SafeReduceError| Cmd::none(),
    &committed,
);
```

Rollback costs a `Clone` of `S` per reduce (checkpoint maintenance).

---

## 5. Default vs rollback

| | **`safe_reduce_update`** | **`safe_reduce_rollback`** |
|---|--------------------------|----------------------------|
| State on panic | Keeps partial writes | Restores last committed checkpoint |
| Clone cost | None | `S: Clone` per successful reduce |
| Runtime | **Yes** (built in) | No — opt-in via `RollbackCatchReducer` |
| Shop demo | **Yes** (`CatchReducer`) | See example below |

---

## 6. Example

```bash
cargo run -p rust-elm --example safe_reducer
```

[`examples/safe_reducer.rs`](../examples/safe_reducer.rs) demonstrates all four paths:
`safe_reduce_update`, `CatchReducer`, `safe_reduce_rollback`, `RollbackCatchReducer`.

---

## 7. Practices

1. Treat reducers as **pure** — avoid `unwrap` and I/O in `reduce`.
2. Use **`Result` in effects**; map failures with `Effect::catch` (panics are a last resort).
3. Wrap composed reducer trees in **`CatchReducer`** at the root (Runtime already uses `safe_reduce_update`).
4. Use **`RollbackCatchReducer`** only when reverting state is worth the clone cost.
5. Read `SafeReduceError::message()` in `recover` for logging/telemetry actions.

---

## Related

| Document | Topic |
|----------|-------|
| [architecture.md](./architecture.md) | Panic layers, module map |
| [store.md](./store.md) | `CatchReducer` vs `ScopeReducer` |
| [ecommerce.md](./ecommerce.md) | `TriggerPanic` demo in the shop |
