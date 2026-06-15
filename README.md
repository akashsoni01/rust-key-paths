# 🔑 KeyPaths in Rust

Key paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **KeyPath and Functional Lenses** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-key-paths = "3.3.1"
key-paths-derive = "3.2.0"
# Optional: trait-only contracts (pulled in by rust-key-paths 3.1+)
# key-paths-core = "2.0"
```

`rust-key-paths` does **not** depend on [`rust-elm`](./rust-elm/) — that relationship runs the other way. Add `rust-elm` only if you want the Elm/TCA store:

```toml
[dependencies]
rust-elm = "0.3.0"
rust-key-paths = "3.3.1"
key-paths-derive = "3.2.0"
```

### Latest releases

| Crate | Version | Notes |
|-------|---------|--------|
| [`key-paths-core`](https://crates.io/crates/key-paths-core) | **2.0.4** | `#![no_std]` traits; `RefKpTrait` for HRTB `focus` / `focus_mut` |
| [`rust-key-paths`](https://crates.io/crates/rust-key-paths) | **3.3.1** | `Kp`, locks, HOF; `RefKpTrait` on `Kp` |
| [`key-paths-derive`](https://crates.io/crates/key-paths-derive) | **3.2.0** | `#[derive(Kp)]`, `#[derive(Cp)]` — see [derive README](./key-paths-derive/README.md) |
| [`rust-elm`](https://crates.io/crates/rust-elm) | **0.3.0** | Elm/TCA store — `RuntimeConfig`, `RefKpTrait` scope/store, pain validation example |
| [`rust_identified_vec`](https://crates.io/crates/rust_identified_vec) | **0.1.2** | Identified collections (used by `rust-elm`) |
| [`rust_dependencies`](https://crates.io/crates/rust_dependencies) | **0.1.2** | Typed DI for effects (used by `rust-elm`) |

#### 3.3.1 / 2.0.4 / 0.3.0

- **`RefKpTrait`** in `key-paths-core` — HRTB `focus` / `focus_mut` for local borrows; `rust-key-paths` `Kp` implements it via `get_ref` / `get_mut_ref`.
- **`rust-elm` 0.3.0** — `RuntimeConfig` (bus capacity, Tokio workers, reducer thread name); scope/store use `RefKpTrait` directly (`StateLens` removed); `StoreTask::finish_with_timeout`; pain/ISO 20022 validation example.
- **`key-paths-derive` 3.2.0** — unchanged; still compatible.

#### 3.3.0 / 2.0.3 / 3.2.0

- **`EnumKp` / `EnumValueKpType`** — casepaths (prisms) for enum variants: extract + embed.
- **`EnumKp::get_ref`**, **`EnumKp::then` / `chain`** — compose nested casepaths.
- **`#[derive(Cp)]`** on enums and structs in `key-paths-derive` 3.2.0.
- **`rust-elm` 0.2.0** — separate crate: WebSocket subs, runtime features, ecommerce example (depends on this crate).
- **`rust_identified_vec` 0.1.2**, **`rust_dependencies` 0.1.2** — patch releases.

#### 3.1.1 / 2.0.1 / 3.0.2 (documentation)

- README guides for adapting keypaths via `Readable` / `Writable` (including `#[derive(Kp)]`).
- Confirmed **`key-paths-derive` is not broken by `key-paths-core` 2.x** — upgrade **`rust-key-paths` to 3.1.1** in your app.

#### 3.1.0 / 2.0.0 (initial trait stack)

- **`key-paths-core` 2.0** — trait-only rewrite (`Readable`, `Writable`, `KpTrait::then`, …).
- **`rust-key-paths` 3.1** — depends on core 2.x; re-exports traits on `Kp`.

### Generic APIs with `Readable` / `Writable`

You can accept any keypath (including derived ones) in generic code:

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
    test(&payload, &BigPayload2::emergency_contact());
}
```

See [key-paths-core/README.md](./key-paths-core/README.md) and [key-paths-derive/README.md](./key-paths-derive/README.md) for migration from `key-paths-core` 1.7 and derive compatibility details.

### Basic usage

```rust
use std::sync::Arc;
use key_paths_derive::Kp;

#[derive(Debug, Kp)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
    scfs2: Arc<std::sync::RwLock<SomeOtherStruct>>,
}

#[derive(Debug, Kp)]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Kp)]
enum SomeEnum {
    A(String),
    B(Box<DarkStruct>),
}

#[derive(Debug, Kp)]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Kp)]
struct DarkStruct {
    dsf: Option<String>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(Box::new(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    }))),
                }),
            }),
            scfs2: Arc::new(std::sync::RwLock::new(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(Box::new(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    }))),
                }),
            })),
        }
    }
}
fn main() {
    let mut instance = SomeComplexStruct::new();

    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf())
        .get_mut(&mut instance).map(|x| {
        *x = String::from("🖖🏿🖖🏿🖖🏿🖖🏿");
    });

    println!("instance = {:?}", instance.scsf.unwrap().sosf.unwrap().omse.unwrap());
    // output - instance = B(DarkStruct { dsf: Some("🖖🏿🖖🏿🖖🏿🖖🏿") })
}
```

### Composing keypaths

Chain through nested structures with `then()`:

```rust
#[derive(Kp)]
struct Address { street: String }

#[derive(Kp)]
struct Person { address: Box<Address> }

let street_kp = Person::address().then(Address::street());
let street = street_kp.get(&person);  // Option<&String>
```

### Casepaths (enum prisms)

Keypaths focus **struct fields**. **Casepaths** focus **enum variants** and support both **extraction** (read the payload when the enum matches) and **embedding** (wrap a payload in the variant).

#### Types

| Type | Role |
|------|------|
| [`EnumKp`](https://docs.rs/rust-key-paths/latest/rust_key_paths/struct.EnumKp.html) | Generic casepath (extractor `Kp` + embedder) |
| [`EnumKpType<'a, E, V>`](https://docs.rs/rust-key-paths/latest/rust_key_paths/type.EnumKpType.html) | Common alias: reference-shaped extract, `fn` embedder. Single-payload variants. |
| [`EnumValueKpType<'a, E, P>`](https://docs.rs/rust-key-paths/latest/rust_key_paths/type.EnumValueKpType.html) | Multi-field variants: payload `P` is extracted **by value** (clone), typically a tuple. |

#### Derive (`#[derive(Cp)]`)

The fastest path — no manual `variant_of` wiring:

```rust
use key_paths_derive::{Cp, Kp};

#[derive(Clone, Kp, Cp)]
enum Action {
    Child(ChildAction),
    Card(String, String),
    Tick,
}

let child = Action::child_cp();  // EnumKpType<'static, Action, ChildAction>
let embedded = child.embed(ChildAction::Inc);
assert_eq!(child.get_ref(&embedded), Some(&ChildAction::Inc));

let card = Action::card_cp();    // EnumValueKpType<'static, Action, (String, String)>
let payment = card.embed(("4242".into(), "123".into()));
assert_eq!(card.get(&payment), Some(("4242".into(), "123".into())));
```

Full variant matrix: [key-paths-derive/README.md — Casepaths (`Cp`)](./key-paths-derive/README.md#casepaths-cp).

#### Factory helpers

```rust
use rust_key_paths::{enum_ok, enum_err, enum_some, variant_of, EnumKp, Kp};

// Option / Result built-ins
let some_kp = enum_some::<String>();
assert_eq!(some_kp.get(&Some("x".into())), Some(&"x".to_string()));

// Custom enum variant (same as #[derive(Cp)] for single-field variants)
enum Payment { Cash(u32), Card(String) }
let cash_kp = variant_of(
    |p: &Payment| match p { Payment::Cash(n) => Some(n), _ => None },
    |p: &mut Payment| match p { Payment::Cash(n) => Some(n), _ => None },
    |n: u32| Payment::Cash(n),
);
assert_eq!(cash_kp.embed(10), Payment::Cash(10));

// Pair #[derive(Kp)] variant accessor with constructor
// Action::child().with_embed(Action::Child)
```

#### `Kp` vs casepath

| | `#[derive(Kp)]` on enum | `#[derive(Cp)]` / `EnumKp` |
|--|-------------------------|----------------------------|
| Struct fields | ✅ `field()` lens | — |
| Enum variant extract | ✅ `variant()` | ✅ via extractor |
| Enum variant embed | ❌ | ✅ `embed(payload)` |
| Typical use | Chaining `.then()` through enums | Scoping actions, prisms, routing |

#### Four-level nested actions

Each level is a single-payload variant. Compose casepaths with **`.then()`** / **`.chain()`** — one chain for both extract and embed (mirroring Swift CasePaths `append`):

```rust
#[derive(Clone, Copy, Kp, Cp)]
enum RootAction { App(AppAction) }
#[derive(Clone, Copy, Kp, Cp)]
enum AppAction { Panel(PanelAction) }
#[derive(Clone, Copy, Kp, Cp)]
enum PanelAction { Widget(WidgetAction) }
#[derive(Clone, Copy, Kp, Cp)]
enum WidgetAction { Tap, Submit }

let to_widget = RootAction::app_cp()
    .then(AppAction::panel_cp())
    .chain(PanelAction::widget_cp());

to_widget.get_ref(&root);              // extract leaf
to_widget.embed(WidgetAction::Submit); // embed leaf → RootAction
```

In [`rust-elm`](./rust-elm/), each scope/reducer layer still uses **one** casepath per step (`Action::child_cp()`); four levels means four nested scopes. For ad-hoc routing without scopes, use the composed chain above. See [`examples/casepath.rs`](./examples/casepath.rs).

### Partial and Any keypaths

Use `#[derive(Pkp, Akp)]` (requires `Kp`) to get type-erased keypath collections:

- **PKp** – `partial_kps()` returns `Vec<PKp<Self>>`; value type erased, root known
- **AKp** – `any_kps()` returns `Vec<AKp>`; both root and value type-erased for heterogeneous collections

Filter by `value_type_id()` / `root_type_id()` and read with `get_as()`. For writes, dispatch to the typed `Kp` (e.g. `Person::name()`) based on TypeId.

See examples: `pkp_akp_filter_typeid`, `pkp_akp_read_write_convert`.

### Features

| Feature | Description |
|---------|-------------|
| `parking_lot` | Use `parking_lot::Mutex` / `RwLock` instead of `std::sync` |
| `tokio` | Async lock support (`tokio::sync::Mutex`, `RwLock`) |
| `arcswap` | [`arc-swap`](https://docs.rs/arc-swap) (`Arc<ArcSwap<T>>`, `Arc<ArcSwapOption<T>>`) via [`SyncKp`](https://docs.rs/rust-key-paths/latest/rust_key_paths/sync_kp/struct.SyncKp.html) |
| `pin_project` | Enable `#[pin]` field support for pin-project compatibility |

### More examples

```bash

```

---

## Supported containers

The `#[derive(Kp)]` macro (from `key-paths-derive`) generates keypath accessors for these wrapper types:

| Container | Access | Notes |
|-----------|--------|-------|
| `Option<T>` | `field()` | Unwraps to inner type |
| `Box<T>` | `field()` | Derefs to inner |
| `Pin<T>`, `Pin<Box<T>>` | `field()`, `field_inner()` | Container + inner (when `T: Unpin`) |
| `Rc<T>`, `Arc<T>` | `field()` | Derefs; mut when unique ref |
| `Vec<T>` | `field()`, `field_at(i)` | Container + index access |
| `HashMap<K,V>`, `BTreeMap<K,V>` | `field_at(k)` | Key-based access |
| `HashSet<T>`, `BTreeSet<T>` | `field()` | Container identity |
| `VecDeque<T>`, `LinkedList<T>`, `BinaryHeap<T>` | `field()`, `field_at(i)` | Index where applicable |
| `Result<T,E>` | `field()` | Unwraps `Ok` |
| `Cow<'_, T>` | `field()` | `as_ref` / `to_mut` |
| `Option<Cow<'_, T>>` | `field()` | Optional Cow unwrap |
| `std::sync::Mutex<T>`, `std::sync::RwLock<T>` | `field()` | Container (use `SyncKp` for lock-through) |
| `Arc<Mutex<T>>`, `Arc<RwLock<T>>` | `field()`, `field_kp()` / `field()` as `SyncKp` | Lock-through via `SyncKp` |
| `Arc<arcswap::ArcSwap<T>>`, `Arc<arcswap::ArcSwapOption<T>>` | `field_kp()` / `field()` as `SyncKp` | arcswap feature; use `arcswap` dependency key (see below) |
| `tokio::sync::Mutex`, `tokio::sync::RwLock` | `field_async()` | Async lock-through (tokio feature) |
| `parking_lot::Mutex`, `parking_lot::RwLock` | `field()`, `field_lock()` | parking_lot feature |

Nested combinations (e.g. `Option<Box<T>>`, `Option<Vec<T>>`, `Vec<Option<T>>`) are supported.

### `arcswap` (optional): atomically swappable `Arc`

Enable **`arcswap`** on `rust-key-paths` and add the same dependency key in your crate so generated paths resolve:

```toml
[dependencies]
rust-key-paths = { version = "3.2.0", features = ["arcswap"] }
arcswap = { package = "arc-swap", version = "1.9" }
```

**When to use [`ArcSwap`](https://docs.rs/arc-swap/latest/arc_swap/struct.ArcSwap.html) instead of `RwLock<Arc<T>>`:** you reload or publish whole snapshots (`store` / `swap` / `rcu`) and many threads read the current snapshot most of the time. Reads use `load()` (default strategy: low-latency, lock-free snapshots) instead of contending on a reader–writer lock. Prefer a **`static`** or **`LazyLock`** holding an `ArcSwap` when a single global pointer is enough; wrap in **`Arc<ArcSwap<T>>`** when the swap container is created at runtime and shared across threads (the outer `Arc` is only for sharing the container; hot-path reads touch the inner atomic pointer, not the `Arc` refcount).

**When not to:** you need a true in-place `&mut T` through a lock for arbitrary mutation of `T` inside the guard. `ArcSwap` stores an `Arc<T>`; updates replace the pointer. Use `store` / `rcu` at the call site for writes.

**Chaining:** compose the full lock path on the first `SyncKp` with `.then(…)` / `.then_sync(…)` (see `examples/box_keypath_arcswap.rs`). Import [`ChainExt`](https://docs.rs/rust-key-paths/latest/rust_key_paths/trait.ChainExt.html) for `Kp::then_sync`. Nested `then_sync` from the crate root can infer a `'static` root in some compositions; if you hit that, call an inner `SyncKp` from a `&` to the inner struct (same example).

### pin_project `#[pin]` fields (optional feature)

When using [pin-project](https://docs.rs/pin-project), mark pinned fields with `#[pin]`. The derive generates:

| `#[pin]` field type | Access | Notes |
|---------------------|--------|-------|
| Plain (e.g. `i32`) | `field()`, `field_pinned()` | Pinned projection via `this.project()` |
| `Future` | `field()`, `field_pinned()`, `field_await()` | Poll through `Pin<&mut Self>` |
| `Box<dyn Future<Output=T>>` | `field()`, `field_pinned()`, `field_await()` | Same for boxed futures |

Enable with `pin_project` feature and add `#[pin_project]` to your struct:

```rust
#[pin_project]
#[derive(Kp)]
struct WithPinnedFuture {
    fair: bool,
    #[pin]
    fut: Pin<Box<dyn Future<Output = String> + Send>>,
}
```

Examples: `pin_project_example`, `pin_project_fair_race` (FairRaceFuture use case).

## Performance: `box_keypath` benchmark

Benchmark file: `benches/box_keypath_bench.rs`

Run with:

```bash
cargo bench --bench box_keypath_bench
```

### Read path (`scsf -> sosf -> omse -> B -> dsf`)

| Variant | Time (approx) |
|---------|---------------|
| keypath | 996.46-997.18 ps |
| unwrap | 944.10-946.59 ps |
| as_ref().map | 996.31-997.39 ps |
| `?` operator | 996.33-997.24 ps |

### Write path (`scsf -> sosf -> omse -> B -> dsf`)

| Variant | Time (approx) |
|---------|---------------|
| keypath | 147.44-149.09 ns |
| unwrap | 143.13-145.02 ns |
| as_ref().map | 141.04-142.65 ns |
| `?` operator | 141.41-150.25 ns |

These numbers are from Criterion's reported confidence ranges on this machine. In this benchmark, keypaths are very close to direct traversal for reads and only slightly slower for writes.

### Keypath size

From `examples/box_keypath.rs`, the composed keypath prints:

```text
size of kp = 0
```

So this composed `kp` is zero-sized (no captured runtime state).

## Performance: `arcswap_keypath` benchmark

Benchmark file: `benches/arcswap_keypath_bench.rs` (requires `--features arcswap`).

```bash
cargo bench --bench arcswap_keypath_bench --features arcswap
```

### Read path (`scsf` → `ArcSwap` load → `omse` → `B` → `dsf`)

Compared to `load_full()` plus a manual `match` on the loaded `OneMoreStruct` (which clones the inner `Arc` on every read), the composed keypath uses `load()` under the hood and stays on the snapshot for the rest of the chain.

| Variant | Time (approx, `--quick` run on one machine) |
|---------|-----------------------------------------------|
| `keypath_then_sync` | 37.8–38.7 ns |
| `load_full_manual` | 115–119 ns |

Your numbers will vary by CPU and optimization level; treat this as a sanity check that keypath traversal stays in the same ballpark as a small manual load, while **`load_full` + clone** is heavier by design when you need an owned `Arc`.

---

### Skills guide (AI assistants & tooling)

Usage-focused patterns—versions/features, `#[derive(Kp)]`, chaining, sync/async locks, deep nesting, migrating from `Option` / Java-style getter chains—live in **[`Skills.md`](./Skills.md)** at the **repository root**.

**Paths to reference:** `Skills.md` (repo root), or `./Skills.md` from this README’s directory. Point Cursor **Rules**, **Agent Skills**, or similar project instructions at that file so assistants can resolve the path and include it in context.

---

## 📜 License

* Mozilla Public License 2.0