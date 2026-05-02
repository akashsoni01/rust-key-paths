# rust-key-paths Skills (Usage Only)

This guide is for LLMs and app developers who want to **use** this library in their own code.
It intentionally avoids contributor/maintenance instructions and does not describe changing library internals.

Examples below use **neutral placeholders** only: types `S1`, `S2`, … and fields `f1`, `f2`, … — not prescriptive names for your domain.

## 0) Recommended versions and feature flags (crates.io)

**Prefer the latest published versions** on [crates.io](https://crates.io): `rust-key-paths` and `key-paths-derive` (see their pages for the current numbers). In `Cargo.toml` you can pin a minimum line like `version = "2"` and run `cargo update` to pick up compatible releases, or set an explicit latest pair after checking crates.io.

**`rust-key-paths` features** (enable only what you use):

| Feature | Purpose |
|--------|---------|
| *(default)* | No optional deps; `std` sync locks, etc. as in the main crate API. |
| `parking_lot` | `parking_lot::Mutex` / `RwLock` in keypath / `SyncKp` support. |
| `arc-swap` | `arc_swap` + `Arc<ArcSwap<…>>` / `Arc<ArcSwapOption<…>>` style paths via `SyncKp`. |
| `tokio` | Async lock keypaths (`AsyncLockKp`, tokio `Mutex` / `RwLock`). |
| `pin_project` | Pin / projection-related keypath support that needs `pin-project`. |
| `tagged_core` | Integration with the `tagged-core` crate for tagged newtypes. |

**`key-paths-derive`** is a normal proc-macro dependency; it does not mirror those feature flags. Your app should enable the same `rust-key-paths` features that match the field types you put in `#[derive(Kp)]` structs (for example `parking_lot` on `rust-key-paths` if you use `parking_lot` locks in keypaths).

**Example: typical app `Cargo.toml` (crates.io):**

```toml
[dependencies]
rust-key-paths = { version = "3.0.0", features = ["parking_lot", "arc-swap", "tokio"] }
key-paths-derive = { version = "3.0.0" }
```

Trim `features` to what you need (fewer features = faster builds).

**Local checkout / path deps** (optional, for developing against a git clone):

```toml
[dependencies]
rust-key-paths = { path = "…", features = ["parking_lot"] }
key-paths-derive = { path = "…" }
```

## 1) Quick Setup

Add dependencies in your app crate (see **§0** for versions and features). Minimal:

```toml
[dependencies]
rust-key-paths = "3"
key-paths-derive = "3"
```

Import commonly used items:

```rust
use rust_key_paths::prelude::*;
use key_paths_derive::Kp;
```

## 2) Derive and Initialize Keypaths

Use `#[derive(Kp)]` on your structs/enums, then call generated field methods (here: `f1()`, `f2()`, … matching fields `f1`, `f2`, …).

```rust
#[derive(Kp)]
struct S2 {
    f1: String,
}

#[derive(Kp)]
struct S1 {
    f1: String,
    f2: S2,
}

let kp_a = S1::f1();
let kp_b = S1::f2().then(S2::f1());
```

### 2.1) Deep nesting and multiple files

The derive macro runs **per type**: every struct or enum you want to step through with generated accessors needs its own `#[derive(Kp)]` on that type’s definition. Nesting depth does not change that rule—you add one derive per layer, then **chain** them.

**Rules of thumb**

1. **One derive per type in the path**  
   If the path is `S1 → S2 → S3 → S4`, then `S1`, `S2`, `S3`, and `S4` should each have `#[derive(Kp)]` (unless you only ever call manual `Kp::new` for some layer).

2. **Compose with `.then(...)` across types**  
   The chain uses each type’s generated methods:

   ```rust
   let kp = S1::f1()
       .then(S2::f1())
       .then(S3::f1())
       .then(S4::f1());
   ```

3. **Multiple files = multiple modules**  
   Splitting types across files is normal. Put `#[derive(Kp)]` on each struct/enum in whatever module file defines it. **In the file where you build or call the chain, you must bring every type into scope** (`use crate::…::S2`, `use crate::…::S3`, …): generated keypath methods are ordinary associated functions (`S1::f1`, `S2::f1`, …), so Rust only resolves them if each struct is imported (or fully qualified). LLMs should emit those `use` lines in the same module as the working code—missing imports are a common cause of “method not found” errors.

   Example layout:

   ```text
   src/
     lib.rs
     model/
       s1.rs      // #[derive(Kp)] struct S1 { f1: S2, ... }
       s2.rs      // #[derive(Kp)] pub struct S2 { f1: S3, ... }
       s3.rs      // #[derive(Kp)] pub struct S3 { f1: String }
   ```

   In `s1.rs` you only need the types in scope to write `S1::f1().then(S2::f1()).then(S3::f1())`; the chain can live in `lib.rs` or a dedicated module that imports `model::*`.

4. **Visibility**  
   Generated methods follow normal Rust visibility. For accessors from another module, fields usually need to be at least visible to that code (`pub`, `pub(crate)`, or same-module `pub(super)` as appropriate). If a field is private to another module, you cannot build a keypath into it from outside—either widen visibility or expose a small API module that owns the chain.

5. **Same crate vs workspace crates**  
   Inside **one** crate, derive works across all modules and files with no extra setup. If types live in **another** crate in your workspace, that crate must list `key-paths-derive` and `rust-key-paths` (with matching features) in **its** `Cargo.toml`, and you put `#[derive(Kp)]` there. Your app crate then depends on that library and chains using the **public** types and paths you expose.

6. **Avoid circular `use` traps**  
   Keep types in a clear DAG if possible. Keypath composition usually imports child types from deeper modules; cyclic modules make imports awkward—same as plain Rust, not specific to `Kp`.

## 3) Chaining Rules

- Use `.then(...)` to compose regular keypaths.
- Keep each segment typed from the previous value to the next field.
- Build once, reuse many times.

```rust
let kp = S1::f2().then(S2::f1());
```

### 3.1) Recognize Option-chain patterns → replace with keypaths

When you see **deep navigation** through `Option`, it is often the same logical path every time: root → field → field → … → leaf. That repeats across:

- long `.and_then` chains,
- stacked `if let` / `let … && let …` (Rust 1.65+),
- repeated `?` on `Option` inside a function.

Those styles are easy to get wrong (wrong intermediate name, drift between call sites) and hard to **reuse** or **test** as one unit.

**Prefer the same shape expressed as a composed keypath**: one chain of `.then(...)`, stored in a variable or helper, then `.get` / `.get_mut` on the root. That path is **type-checked**, **composable**, and you can pass it around like any other value.

Legacy patterns (illustrative; **homogeneous** nesting: `S1.f1: Option<S2>`, `S2.f1: Option<S3>`, …, leaf `Option<String>`; adjust names in real code):

```rust
// Deep navigation with Option chaining
let v = root
    .f1
    .as_ref()
    .and_then(|x| x.f1.as_ref())
    .and_then(|x| x.f1.as_ref())
    .and_then(|x| x.f1.as_ref());

// Rust — combined let chains (single condition)
let v = if let Some(s2) = root.f1.as_ref()
    && let Some(s3) = s2.f1.as_ref()
    && let Some(s4) = s3.f1.as_ref()
    && let Some(out) = s4.f1.as_ref()
{
    Some(out)
} else {
    None
};

// `?` on Option (inside a function returning Option)
fn read_leaf(root: &S1) -> Option<&String> {
    root.f1.as_ref()?
        .f1.as_ref()?
        .f1.as_ref()?
        .f1.as_ref()
}
```

*(If one hop is through a collection — index / first element — insert that step in both legacy style and keypath; the exact accessor comes from your derive or a manual `Kp`.)*

**Keypath replacement** (same navigation; import every type used in the chain in this file):

```rust
// One composed path; share across tests and call sites.
let kp = S1::f1()
    .then(S2::f1())
    .then(S3::f1())
    .then(S4::f1());

kp.get(&root).map(|x| {
    assert_eq!(x, &"…".to_string());
});
```

You can also bind the result without extra logic:

```rust
let v = kp.get(&root);
```

**Notes for LLMs and authors**

- **Imports**: bring `S1`, `S2`, … into scope so each `Sn::f1()` resolves.
- **Collections / indexing**: names like `f1_at(0)` are **examples** — use the actual generated methods for your `Vec`, slice, or custom container, or one manual `Kp::new` hop for that step.
- **Reuse**: assign `let kp = …` once (or `fn leaf_kp() -> …`) and call `kp.get(&root)` everywhere instead of duplicating Option chains.

### 3.2) Java (and similar) null-check + getter chains → keypaths

In **Java**, **C#**, **Kotlin** (nullable style), and similar languages, the same problem appears as:

- a long `if` with `!= null` / `isEmpty()` / `get(0)` guards, and then
- **the same** getter chain **repeated** in the `if` body to read the final value.

That is the same *navigation path* as the Rust `Option` examples above: root → … → leaf. **LLMs should map that repeated getter graph to one composed keypath** instead of re-deriving it in prose.

**Java-style pattern (recognize this; getters are placeholders, e.g. field `f1` → `getF1()`):**

```java
// Same anti-pattern: long null guards, then the same getter chain repeated in the body.
String out = null;
if (root.getF1() != null
    && root.getF1().getF1() != null
    && root.getF1().getF1().getF1() != null
    && root.getF1().getF1().getF1().getF1() != null) {

    out = root.getF1().getF1().getF1().getF1();
}
```

*(Longer graphs — collections, extra layers — look the same: more `&&` lines and a longer body chain.)*

**Rust keypath equivalent** (import all types in the chain; method names follow your `#[derive(Kp)]` output):

```rust
S1::f1()
    .then(S2::f1())
    .then(S3::f1())
    .then(S4::f1())
    .get(&root)
    .map(|x| {
        assert_eq!(x, &"…".to_string());
    });
```

- **Java** uses `null` and reference types; **Rust** often uses `Option` fields—`get` on a keypath still yields `Option<&T>` at the leaf, so you do not need to mirror every `!= null` by hand.
- For **lists / indices**, add one keypath segment that matches your derive (or manual `Kp`); the Rust method name is not fixed here.

### 3.3) Complex models: `then` vs `then_sync` vs `then_async`

When a struct mixes **plain fields**, **sync locks** (`Mutex` / `RwLock` in `Arc` or not, `parking_lot`, `Option<…>` around locks), and **async locks** (Tokio), the right **chaining combinator** depends on what the *previous* segment returns and what the *next* segment is.

| You have | Next segment is | Use |
|----------|-------------------|-----|
| `Kp` (plain path) | another `Kp` | `.then(S_next::f1())` |
| `Kp` | a **`SyncKp`** (lock-through accessor from derive) | `.then_sync(S_next::f2())` — requires **`ChainExt`** in scope |
| `Kp` | an **`AsyncLockKp`** (Tokio field accessor) | `.then_async(S_next::f3())` — requires **`ChainExt`** + **`tokio`** feature on `rust-key-paths` |
| **`SyncKp`** | inner plain path on the guarded value | `.then(S_inner::f1())` — provided on `SyncKp` directly |
| **`SyncKp`** | another nested **`SyncKp`** (second lock level) | `.then_sync(S_inner::f2())` — stacks sync lock traversal |
| **`SyncKp`** | **`AsyncLockKp`** | `.then_async(...)` when bridging sync container to async inner |

Import `ChainExt` wherever you call **`then_sync`** or **`then_async`** on a plain `Kp`:

```rust
use rust_key_paths::ChainExt;
```

Enable crate features that match your field types (see **§0**): e.g. `parking_lot`, `tokio`, `arc-swap`.

**Illustrative struct shape** (many combinations are supported by `#[derive(Kp)]`; adjust field types to your app):

```rust
#[derive(Debug, Kp)]
struct S1 {
    f1: String,
    f2: Option<Box<S2>>,
    f3: Arc<std::sync::Mutex<S2>>,
    f4: Arc<std::sync::RwLock<S2>>,
    // … Option<Mutex<…>>, Mutex<Option<…>>, Arc<Mutex<Option<…>>>, parking_lot, arc-swap, …
    // #[cfg(feature = "tokio")] Arc<tokio::sync::Mutex<S2>>, …
}
```

**Example:** start from a **sync lock** field accessor (`SyncKp`), chain **plain** inner keypaths with `.then`, then read or mutate the leaf:

```rust
use rust_key_paths::prelude::*;
use key_paths_derive::Kp;

// … `instance: S1`, import `S2`…`S5` for `.then` …

S1::f3()
    .then(S2::f1())
    .then(S3::f1())
    .then(S4::f1())
    .then(S5::f1())
    .get_mut(&mut instance)
    .map(|x| {
        *x = String::from("…");
    });
```

For **two (or more) lock layers** in a row, use **`.then_sync`** between the corresponding `SyncKp` segments. For **Tokio**-backed fields, use **`.then_async`**, then use the async keypath’s **`.get` / `.get_mut` with `.await`** as required by the API.

Details for each wrapper kind follow your **§0** feature set and the generated method names for that type.

## 4) Lock Types (SyncKp)

For lock wrappers, generated methods return `SyncKp` directly (single accessor name).

- `S1::f1()` returns `SyncKp` when `f1` is a supported lock-backed field type.
- Do not expect extra lock-only accessor suffixes beyond what derive emits.

Typical pattern:

```rust
#[derive(Kp)]
struct S1 {
    // f1: Arc<std::sync::Mutex<i32>>
}

// let kp = S1::f1(); // SyncKp
```

For multi-level composition rules (`then` / `then_sync` / `then_async`), see **§3.3**.

## 5) Async Lock Types

Enable **`tokio`** on `rust-key-paths` (see **§0**). Chain into Tokio lock fields with **`then_async`** from **`ChainExt`** when continuing from a plain `Kp`; see **§3.3**. Use `.await` on the async keypath accessors as documented for `AsyncLockKp`.

## 6) Add New Keypaths in App Code

You can add keypaths in your application without changing this library:

1. Add a field to your own type.
2. Keep `#[derive(Kp)]` on that type.
3. Use the new generated accessor and compose with `.then(...)`.

When derive is not possible (external types or special shape), create keypaths manually with `Kp::new(get, set)` in your app code.

## 7) Reuse Patterns

Prefer reusable helpers over rebuilding paths inline:

```rust
#[derive(Kp)]
struct S1 {
    f1: String,
}

fn your_fn() {
    let reusable_kp = S1::f1();
    // reusable_kp.get(&instance);
    // reusable_kp.get_mut(&mut instance);
}
```

Recommended reuse style:

- Create small helper functions that return typed keypaths.
- Keep keypaths in a module near the types they traverse.
- Reuse composed keypaths for reads/writes instead of recreating chains at call sites.

## 8) LLM Prompting Tips

When asking another LLM to generate usage code, provide:

- root type name (or placeholder like `S1`)
- target field path (`f1` → `f2` → …) and each intermediate type (`S2`, `S3`, …) if deeply nested
- which files or modules own each type, if the tree is split across the codebase
- whether locks are sync/async
- desired output (`get`, `get_mut`, set/update flow)
- if migrating from **Java/C#/Kotlin**-style null-check + getter chains, paste that pattern so the model can mirror it as one `.then` chain
- for **locks / Tokio** in the same struct, say which fields are `SyncKp` vs `AsyncLockKp` so the model picks `.then` vs `.then_sync` vs `.then_async` (see **§3.3**) and includes `use rust_key_paths::ChainExt`

Example prompt:

`Generate rust-key-paths code for S1 -> f2 -> S2 -> f1 (derive Kp), with a reusable helper function.`
