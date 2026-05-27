# key-paths-derive

Proc-macro `#[derive(Kp)]` (and related derives) that generate `rust_key_paths::Kp` accessors on your structs and enums.

## Release notes

### 3.0.2

- README: compatibility with `key-paths-core` 2.x / `rust-key-paths` 3.1.x and generic `Readable` examples.

### 3.0.1

- Proc-macro `#[derive(Kp)]` for `rust-key-paths` 3.x.

## Compatibility with `key-paths-core` 2.x / `rust-key-paths` 3.1.x

| Crate | Relationship to this proc-macro |
|-------|----------------------------------|
| **key-paths-derive** | No dependency on `key-paths-core` (only `syn` / `quote` / `proc-macro2`). |
| **Generated code** | Expands to `rust_key_paths::Kp`, `SyncKp`, etc.—not `key_paths_core` types directly. |
| **Your app** | Depend on **`rust-key-paths` ≥ 3.1.1** (pulls in `key-paths-core` 2.x). Pin **`key-paths-derive` = "3.0.2"**. |

**`key-paths-core` 2.x** does **not** change what the derive macro emits. **`rust-key-paths` 3.1+** re-exports `Readable`, `Writable`, `KpTrait` so you can write generic APIs over any keypath—including paths from `#[derive(Kp)]`.

```toml
[dependencies]
rust-key-paths = "3.1.1"
key-paths-derive = "3.0.2"
```

## Generic APIs over derived keypaths

Derived field methods return `Kp<…>` which implements `Readable` / `Writable` from `key_paths_core` (re-exported as `rust_key_paths::Readable`, etc.). Pass a derived path anywhere you bound on those traits:

```rust
use key_paths_derive::Kp;
use rust_key_paths::Readable;

#[derive(Kp)]
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

fn main() {
    let payload = BigPayload2 {
        emergency_contact: Some("555-0100".into()),
    };
    // `Option<String>` field: use the generated accessor (name matches field convention).
    test(&payload, BigPayload2::emergency_contact());
}
```

Use `Readable::get(&path, root)` if you prefer explicit trait syntax. For write paths, bound on `Writable<&'p mut BigPayload2, &'p mut String>` and call `set`.

**Note:** `#[derive(Kp)]` on `Option<T>` fields typically navigates to `&T` when the option is `Some`. Adjust the `Readable` value type in your bound to match the field you generated (e.g. `&'p str` vs `&'p String`).

## Chaining (unchanged)

```rust
SomeComplexStruct::scsf()
    .then(SomeOtherStruct::sosf())
    .get(&root);
```

See the [repository README](../README.md) and [key-paths-core README](../key-paths-core/README.md) for trait-level integration and migration from `key-paths-core` 1.x.

## License

Mozilla Public License 2.0
