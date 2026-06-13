# rust-elm 0.1.0

Elm Architecture for Rust, evolving toward The Composable Architecture (UDF).

## Install

```toml
[dependencies]
rust-elm = { path = "../rust-elm" }
rust-key-paths = "3.2.0"
key-paths-derive = "3.1.0"
tokio = { version = "1.38", features = ["rt-multi-thread", "macros"] }
```

## Quick start

```rust
use rust_elm::{Cmd, Program, Runtime, Sub, Environment};

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

fn main() {
    let program = Program::new(init, update, subscriptions);
    let runtime = Runtime::from_program(program, Environment::new(), 64);
    let store = runtime.store();
    store.dispatch(1);
    runtime.shutdown();
}

// Or with composable reducers:
// let program = ReducerProgram::new(reducers![update_a, update_b], init, subscriptions);
// let runtime = Runtime::from_reducer_program(program, Environment::new(), 64);
//
// Full shop demo: cargo run -p rust-elm --example ecommerce
// Architecture: book/architecture.md
```

## Modules

| Module | Purpose |
|--------|---------|
| `cmd` | Commands returned from `update` |
| `effect` | Pure async effect descriptions (`debounce`, `throttle`, `from_run`, `cancel`) |
| `sub` | Subscription descriptions |
| `runtime` | Bus-driven update loop + interpreter |
| `store` | `Store`, `StoreTask`, `ScopedStore`, state subscription |
| `test_store` | `ExhaustiveTestStore` for synchronous effect/action testing |
| `shared` | `Shared<T>`, `Storage`, `InMemoryStorage`, `FileStorage` (serde) |
| `dependencies` | Re-exports [`rust_dependencies`](../rust_dependencies) — see [`book/dependencies.md`](../rust_dependencies/book/dependencies.md) |
| `env` | `Environment` (`live`/`test`), `FakeClock`, `MockHttp` |
| `optics` | State/action focusing via `rust-key-paths` |
| `test_runtime` | Sync testing without Tokio |
| `reducer` | `Reducer` trait, `CombineReducers`, `reducers!` |
| `identified` | Re-exports [`rust_identified_vec`](../rust_identified_vec) — see [`book/identified.md`](../rust_identified_vec/book/identified.md) |
| `scope` | `ScopeReducer`, `IfLetReducer`, `ForEachReducer`, `lift_cmd` |
| `replay` | Action log + replay harness; `snapshot`/`restore` for state checkpoints |

## Key paths prelude

```rust
use rust_elm::keypath::{Kp, KpType, Readable, Writable};
use key_paths_derive::Kp;
```

See [`ROADMAP.md`](ROADMAP.md), [`book/architecture.md`](book/architecture.md), and workspace [`todo.md`](../todo.md).
