# key-paths-derive

Proc-macro derives that generate `rust_key_paths` accessors on your types:

| Derive | Purpose |
|--------|---------|
| **`Kp`** | Keypaths for struct fields and enum variant **extractors** |
| **`Cp`** | Casepaths (prisms) for enum variants — **extract + embed** |
| **`Pkp`** | `partial_kps()` — type-erased partial keypaths (requires `Kp`) |
| **`Akp`** | `any_kps()` — fully type-erased keypaths (requires `Kp`) |

## Release notes

### Unreleased

- **`#[derive(Cp)]`** — casepath (prism) accessors for enum variants. Each variant gets a `variant_cp()` method returning a ready-to-use `EnumKpType` or `EnumValueKpType` (extract + embed). See [Casepaths (`Cp`)](#casepaths-cp) below.
- Supports **unit**, **single-field tuple**, **single-field named**, **multi-field tuple**, and **multi-field named** variants. Multi-field payloads are extracted **by value** (clone) as tuples; field types must implement `Clone`.
- Coexists with `#[derive(Kp)]` on the same enum (`child()` = extract-only keypath, `child_cp()` = full casepath).

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

## Casepaths (`Cp`)

[`Kp`](https://docs.rs/rust-key-paths/latest/rust_key_paths/struct.Kp.html) keypaths focus **struct fields** and **extract** enum variant payloads. **Casepaths** (inspired by [Swift CasePaths](https://github.com/pointfreeco/swift-case-paths)) extend that model to enums: they **extract and embed** a variant — the prism you need when scoping reducers, routing actions, or lifting child values into a parent enum.

Use `#[derive(Cp)]` on enums (often together with `#[derive(Kp)]`):

```rust
use key_paths_derive::{Cp, Kp};

#[derive(Clone, Kp, Cp)]
enum Action {
    Child(ChildAction),                    // single-field tuple
    Auth { token: String },                // single-field named
    Card(String, String),                  // multi-field tuple
    Wallet { id: String, balance: u32 },   // multi-field named
    Tick,                                  // unit
}

#[derive(Clone, Copy, Kp, Cp)]
enum ChildAction {
    Inc,
}
```

### Generated accessors

| Variant shape | Method | Return type | Extraction |
|---------------|--------|-------------|------------|
| Unit `Tick` | `tick_cp()` | `EnumKpType<'static, Self, ()>` | by reference |
| Single tuple `Child(T)` | `child_cp()` | `EnumKpType<'static, Self, T>` | by reference |
| Single named `Auth { token: T }` | `auth_cp()` | `EnumKpType<'static, Self, T>` | by reference |
| Multi tuple `Card(A, B)` | `card_cp()` | `EnumValueKpType<'static, Self, (A, B)>` | by value (clone) |
| Multi named `Wallet { id, bal }` | `wallet_cp()` | `EnumValueKpType<'static, Self, (Id, Bal)>` | by value (clone) |

Methods use a `_cp` suffix so they do not clash with `#[derive(Kp)]` variant accessors (`child()` vs `child_cp()`).

### Usage

```rust
// Single-payload: reference extraction + embed (zero-copy reads)
let kp = Action::child_cp();
let action = kp.embed(ChildAction::Inc);
assert_eq!(kp.get_ref(&action), Some(&ChildAction::Inc));

// Multi-field: owned tuple payload (fields must be Clone)
let card = Action::card_cp();
let payment = card.embed(("4242".into(), "123".into()));
assert_eq!(card.get(&payment), Some(("4242".into(), "123".into())));
```

**Why two casepath flavors?** Rust stores multi-field enum variants as separate fields, not one contiguous tuple, so you cannot borrow `Card(String, String)` as `&(String, String)`. Like Swift's case paths, multi-field variants surface the payload as an **owned tuple** via `EnumValueKpType`. Single-payload variants stay reference-based via `EnumKpType`.

### Manual construction (without derive)

If you already have `#[derive(Kp)]` extractors, pair them with the variant constructor:

```rust
// From derived variant accessor + constructor
Action::child().with_embed(Action::Child)

// Or explicit factory (same as what Cp generates for single-field variants)
use rust_key_paths::variant_of;
variant_of(
    |a: &Action| Action::child().get_ref(a),
    |a: &mut Action| Action::child().get_mut_ref(a),
    Action::Child,
)
```

See the [repository README](../README.md#casepaths-enum-prisms) for `EnumKp`, `variant_of`, and built-in `Option`/`Result` casepaths.

## Chaining (unchanged)

```rust
SomeComplexStruct::scsf()
    .then(SomeOtherStruct::sosf())
    .get(&root);
```

See the [repository README](../README.md) and [key-paths-core README](../key-paths-core/README.md) for trait-level integration and migration from `key-paths-core` 1.x.

## License

Mozilla Public License 2.0
