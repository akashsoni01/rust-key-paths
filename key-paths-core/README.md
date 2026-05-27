# key-paths-core

**Trait-only** keypath contracts for Rust. No proc-macros, no locks, no async runtime — only the interfaces your library can implement so callers share a common read/write surface.

Use this crate when you want to build a custom keypath library (or integrate keypaths into an existing framework) without pulling in `rust-key-paths`.

For a full reference implementation (derive macros, `Kp`, sync/async locks, composition), see [`rust-key-paths`](https://github.com/codefonsi/rust-key-paths) in the same repository.

## Release notes

### 2.0.1

- README: generic `Readable` / `Writable` integration guide and `#[derive(Kp)]` compatibility notes.

### 2.0.0

- **Trait-only surface** — `Readable`, `Writable`, `KeyPath`, `KpTrait` (with `then`), `AccessorTrait`, `KeyPathValueTarget`.
- **`#![no_std]`** — no proc-macros, locks, or async in this crate.
- **Breaking vs 1.7** — the old `KeyPaths` enum and container helpers are removed; stay on 1.7 or use `rust-key-paths` 3.x.

**`key-paths-derive` is unaffected as a crate** (it does not depend on `key-paths-core`). Generated `#[derive(Kp)]` code still targets `rust-key-paths` ≥ 3.1, which implements these traits on `Kp`. See [key-paths-derive README](../key-paths-derive/README.md).

## Traits

| Trait | Role |
|-------|------|
| [`Readable<Root, Value>`](https://docs.rs/key-paths-core/latest/key_paths_core/trait.Readable.html) | Getter path: `root` → optional `Value` |
| [`Writable<MutRoot, MutValue>`](https://docs.rs/key-paths-core/latest/key_paths_core/trait.Writable.html) | Setter path: `mut root` → optional mutable `Value` |
| [`KeyPath<Root, Value, MutRoot, MutValue>`](https://docs.rs/key-paths-core/latest/key_paths_core/trait.KeyPath.html) | Marker: both read and write |
| [`KpTrait<R, V, Root, Value, MutRoot, MutValue>`](https://docs.rs/key-paths-core/latest/key_paths_core/trait.KpTrait.html) | Above + `TypeId` helpers + [`then`](https://docs.rs/key-paths-core/latest/key_paths_core/trait.KpTrait.html#tymethod.then) |
| [`KeyPathValueTarget`](https://docs.rs/key-paths-core/latest/key_paths_core/trait.KeyPathValueTarget.html) | Maps `&T` / `&mut T` → `T` for generic chaining |
| [`AccessorTrait`](https://docs.rs/key-paths-core/latest/key_paths_core/trait.AccessorTrait.html) | Optional-root and `or_else` helpers (default methods) |

## Architecture: adapting keypaths in your crate

### 1. Pick root and value types

- **Root** — what the caller passes into `get` (often `&MyStruct` or `&mut MyStruct`).
- **Value** — what the keypath returns on success (often `&Field` or `&mut Field`).
- **MutRoot** / **MutValue** — usually the same shapes as Root/Value for mutation paths.

Keep logical types `R` and `V` in mind when you implement [`KpTrait`] (for `TypeId` and documentation).

### 2. Implement `Readable` and `Writable` or use key-paths-derive macro

```rust
use key_paths_core::{Readable, Writable};

pub struct NameKp;

impl Readable<&Person, &str> for NameKp {
    fn get(&self, root: &Person) -> Option<&str> {
        Some(&root.name)
    }
}

impl Writable<&mut Person, &mut str> for NameKp {
    fn set(&self, root: &mut Person) -> Option<&mut str> {
        Some(&mut root.name)
    }
}
```

### 2b. Generic functions over any keypath (including `#[derive(Kp)]`)

Bound on `Readable` / `Writable` instead of a concrete `Kp` type. This works with hand-written keypaths and with paths from `key-paths-derive` (via `rust-key-paths`, which implements the traits on `Kp`):

```rust
use key_paths_core::Readable;
// With derive: use key_paths_derive::Kp; use rust_key_paths::Readable;

struct BigPayload2 {
    emergency_contact: Option<String>,
}

fn test<'p, G>(payload: &'p BigPayload2, g: G)
where
    G: Readable<&'p BigPayload2, &'p String>,
{
    if let Some(emg_contact) = g.get(payload) {
        println!("there value = {:?}", emg_contact);
    } else {
        println!("not there");
    }
}

// Call site (after #[derive(Kp)] on BigPayload2):
// test(&payload, BigPayload2::emergency_contact());
```

- **`Root`** / **`Value`** in the trait are the types you pass to `get` and receive on success (often `&Struct` and `&Field`).
- Use **`Writable<&'p mut Root, &'p mut Value>`** for mutation helpers the same way.

### 3. Optional: `AccessorTrait`

Blanket-style ergonomics without new methods on your type:

```rust
use key_paths_core::{AccessorTrait, Readable, Writable};

impl AccessorTrait<&Person, &str, &mut Person, &mut str> for NameKp {}
```

### 4. Composition with [`KpTrait::then`]

Implement `then` on your keypath type; return type `Out` is inferred at the call site (see `rust-key-paths` `Kp::then` for a reference impl).

Lock traversal and async chaining stay in `rust-key-paths` (`SyncKp`, `AsyncLockKp`, `ChainExt`).

### 5. Async keypaths

Define async methods on your type (e.g. `async fn get(&self, root: Root) -> Option<Value>`), and optionally a **blocking** adapter that implements `Readable`/`Writable` for background threads (see `rust-key-paths` `block_async` pattern). The core traits stay synchronous so they stay runtime-agnostic.

### 6. Lock / shared state

Do not fake `&mut T` through `Arc` or lock-free snapshots. Either:

- return owned snapshots / guards from your own API, or
- use `rust-key-paths` lock keypaths (`SyncKp`, `AsyncLockKp`) that encode the right semantics.

## `Cargo.toml`

```toml
[dependencies]
key-paths-core = "2.0"
```

No default features. `#![no_std]` with `alloc` not required.

## Migration from 1.x

Version 2.0 is a **breaking** rewrite: the old dynamic `KeyPaths` enum and container helpers were removed in favor of these traits. Existing code that used `key_paths_core::KeyPaths` should stay on 1.x or migrate to `rust-key-paths` 3.x.

## Related crates

| Crate | Purpose |
|-------|---------|
| `rust-key-paths` | Reference `Kp`, derive, sync/async locks, `ChainExt`, HOF helpers |
| `key-paths-derive` | `#[derive(Kp)]` proc-macro (depends on `rust-key-paths`, not this crate) |
