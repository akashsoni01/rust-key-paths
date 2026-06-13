# rust_dependencies

Typed dependency injection for Rust

Register values by type, swap live vs test implementations, and override individual deps without global state.

## Install

```toml
[dependencies]
rust_dependencies = "0.1.0"
```

## Quick start

```rust
use rust_dependencies::{DependencyKey, DependencyValues, UuidDep};

let bag = DependencyValues::test();
let uuid = bag.require::<UuidDep>().unwrap().0.next();
assert_ne!(uuid, 0);
```

## Built-in dependencies

| Key | Purpose |
|-----|---------|
| `ClockKey` | Monotonic `Instant` (debounce, elapsed time) |
| `NowKey` | Wall `SystemTime` (timestamps) |
| `UuidKey` | Unique `u128` identifiers |
| `RngKey` | `u64` random draws |

Each key provides `live()` (production) and `test()` (deterministic) via [`DependencyKey`].

## Documentation

- In-depth guide: [`book/dependencies.md`](book/dependencies.md)
- API docs: `cargo doc --open`

## Used by

[`rust-elm`](../rust-elm) re-exports this crate for effect environments and runtime wiring.

## License

MPL-2.0 — see [LICENSE](LICENSE).
