# rust_identified_vec

Ordered, id-indexed vector with **O(1) lookup** by stable id — a standalone Rust library inspired by [TCA's `IdentifiedArray`](https://github.com/pointfreeco/swift-composable-architecture).

Zero dependencies by default. Optional `serde` feature for JSON round-trip.

## Install

```toml
[dependencies]
rust_identified_vec = "0.1.0"

# with persistence
rust_identified_vec = { version = "0.1.0", features = ["serde"] }
```

## Quick start

```rust
use rust_identified_vec::{Identifiable, IdentifiedVec};

#[derive(Clone, PartialEq, Eq, Debug)]
struct Todo {
    id: u64,
    title: String,
    done: bool,
}

impl Identifiable for Todo {
    type Id = u64;
    fn id(&self) -> u64 {
        self.id
    }
}

let mut todos = IdentifiedVec::new();
todos.insert(Todo { id: 1, title: "Buy milk".into(), done: false });
todos.update(1, |t| t.done = true);
assert_eq!(todos.get(1).unwrap().done, true);
```

## Features

| Feature | Description |
|---------|-------------|
| `serde` | Serialize/deserialize as a plain JSON array (order preserved) |

## Documentation

- In-depth guide: [`book/identified.md`](book/identified.md)
- API docs: `cargo doc --open`

## Used by

[`rust-elm`](../rust-elm) re-exports this crate for `ForEachReducer` and list state.

## License

MPL-2.0 — see [LICENSE](LICENSE).
