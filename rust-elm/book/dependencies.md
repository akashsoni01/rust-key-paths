# Dependency injection

The [`dependencies`](../src/dependencies.rs) module provides **typed dependency bags** for effects — the rust-elm counterpart to TCA's `DependencyValues` / `DependencyKey`.

Use it when an effect needs:

- **Wall time, monotonic time, UUIDs, or randomness** without hard-coding `Instant::now()` / `SystemTime::now()`
- **Deterministic test doubles** that replay the same sequence across test runs
- **Scoped overrides** via [`Environment`](../src/env.rs) layers (see [Environment overrides](#5-environment-overrides))

For layered runtime wiring (`live` / `test`, `FakeClock`, `MockHttp`), see [`env.rs`](../src/env.rs) and the [`Environment`](#integration-with-environment) section below.

---

## Core types

### `DependencyValues`

A thread-safe map from Rust `TypeId` to `Arc<dyn Any + Send + Sync>`. Each dependency type `D` is stored under `TypeId::of::<D>()`.

| Method | Behavior |
|--------|----------|
| `new()` | Empty bag |
| `live()` | Production bag: real clock, live UUID/RNG, wall `Now` |
| `test()` | Deterministic bag: shared `TestClock` + `TestNow`, seeded UUID/RNG |
| `insert(value)` | Register or replace a dependency |
| `get::<D>()` | `Option<Arc<D>>` — read lock only |
| `require::<D>()` | `Result<Arc<D>, DependencyError>` |
| `contains::<D>()` | Whether `D` is registered |
| `remove::<D>()` | Remove and return `Arc<D>` |
| `get_or_insert_with(f)` | Lazy init with double-checked locking |
| `with(value)` | Builder-style insert, returns `self` |
| `merge_from(overlay)` | Overlay wins on key conflicts (deadlock-safe snapshot) |

**Clone is shallow:** two clones share the same underlying map. Mutations through one clone are visible in the other.

### `DependencyKey`

Marker trait for registering a dependency under a stable key type:

```rust
pub trait DependencyKey: Send + Sync + 'static {
    type Value: Send + Sync + 'static;

    fn live() -> Self::Value;
    fn try_test() -> Result<Self::Value, DependencyError> { /* Err by default */ }
    fn test() -> Self::Value { /* panics if try_test fails */ }
    fn preview() -> Self::Value { Self::live() }
    fn register(values: &DependencyValues, value: Self::Value);
}
```

`Value` must be `Send + Sync + 'static` so it can live in the shared bag and be read from async effect tasks on any thread.

---

## Built-in dependencies

Four keys ship with rust-elm. Each follows the same pattern: a **key struct** (`ClockKey`), a **trait** (`Clock`), a **wrapper dep** (`ClockDep(Arc<dyn Clock>)`), and **live / test** implementations.

| Key | Trait | Live impl | Test impl |
|-----|-------|-----------|-----------|
| `ClockKey` | `Clock` → `Instant` | `RealClock` | `TestClock` (shared via `OnceLock`) |
| `NowKey` | `Now` → `SystemTime` | `LiveNow` | `TestNow` (tracks `TestClock`) |
| `UuidKey` | `UuidGen` → `u128` | `LiveUuidGen` (time + pid + counter) | `SeededUuidGen` (splitmix64) |
| `RngKey` | `DepRng` → `u64` | `LiveRng` (xorshift64) | `SeededRng` (xorshift64, mutex) |

### Clock vs Now

- **`Clock`** — monotonic [`Instant`](https://doc.rust-lang.org/std/time/struct.Instant.html) for debounce/throttle timers and elapsed measurements.
- **`Now`** — [`SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html) for timestamps shown to users or persisted.

In **`DependencyValues::test()`**, both use the same `TestClock` instance so advancing time moves `Clock` and `Now` together.

### Live vs test semantics

| Concern | Live | Test |
|---------|------|------|
| UUID uniqueness | Atomic counter + wall nanos + pid | Fixed seed `1`, monotonic counter |
| RNG | Entropy-seeded xorshift, lock-free atomic | Seed `0xC0FFEE`, mutex-protected state |
| Reproducibility | Non-deterministic across runs | Same first values across fresh `test()` bags |

**Note:** `SeededUuidGen` is **deterministic, not cryptographically random**. Use a real UUID crate in production if you need RFC 4122 compliance.

---

## Use cases

### 1. Read a built-in dependency in an effect

```rust
use rust_elm::dependencies::{ClockDep, DependencyValues};
use rust_elm::Environment;

async fn measure<F, M>(env: &Environment, work: F) -> M
where
    F: FnOnce() -> M,
{
    let clock = env.require::<ClockDep>().unwrap();
    let start = clock.0.now();
    let result = work();
    let elapsed = start.elapsed(); // uses the injected clock in tests
    let _ = elapsed;
    result
}
```

In tests, swap `Environment::test()` and advance the shared `TestClock` via `FakeClock` on the environment.

### 2. Register a custom dependency

```rust
use rust_elm::dependencies::{DependencyKey, DependencyValues};

struct ApiClientKey;

#[derive(Clone)]
struct ApiClient {
    base_url: String,
}

impl DependencyKey for ApiClientKey {
    type Value = ApiClient;

    fn live() -> Self::Value {
        ApiClient { base_url: "https://api.example.com".into() }
    }

    fn try_test() -> Result<Self::Value, rust_elm::DependencyError> {
        Ok(ApiClient { base_url: "http://127.0.0.1:0".into() })
    }
}

let bag = DependencyValues::live().with(ApiClientKey::live());
let client = bag.require::<ApiClient>().unwrap();
```

Override `try_test()` (not just `test()`) so callers can use `try_test()` for graceful fallback without panicking.

### 3. Override one dependency for a single test

```rust
use std::sync::Arc;
use rust_elm::{DependencyValues, SeededUuidGen, UuidDep};

let bag = DependencyValues::test()
    .with(UuidDep(Arc::new(SeededUuidGen::new(42))));

let uuid = bag.require::<UuidDep>().unwrap().0.next();
// Always starts from seed 42, independent of the default test seed.
```

### 4. Merge bags (layer composition)

```rust
let base = DependencyValues::test();
let overlay = DependencyValues::new();
overlay.insert("custom-config".to_string());

base.merge_from(&overlay);
// overlay keys win; built-ins remain unless overlay replaces the same TypeId
```

`merge_from` snapshots the overlay under a read lock before writing to `self`, so concurrent `a.merge_from(&b)` and `b.merge_from(&a)` cannot deadlock.

### 5. Environment overrides

[`Environment`](../src/env.rs) stacks `DependencyValues` layers. Resolution walks **top-down**; the first hit wins.

```rust
use std::sync::Arc;
use rust_elm::{Environment, SeededRng, RngDep};

let env = Environment::test()
    .with(RngDep(Arc::new(SeededRng::new(123))));

let draw = env.require::<RngDep>().unwrap().0.next_u64();
// Repeatable across runs with the same override seed
```

`Environment::get::<D>()` resolves a single key without merging entire bags. `Environment::values()` returns a merged bag (cached until layers change).

### 6. Lazy registration with `get_or_insert_with`

```rust
let bag = DependencyValues::new();
let config = bag.get_or_insert_with(|| expensive_load_config());
// Factory runs at most once, even under concurrent first access
```

### 7. Teardown / swap in long-running tests

```rust
let bag = DependencyValues::test();
assert!(bag.contains::<UuidDep>());

let old = bag.remove::<UuidDep>();
assert!(old.is_some());
assert!(!bag.contains::<UuidDep>());
```

---

## Testing patterns

### Deterministic UUID and RNG

```rust
let bag = DependencyValues::test();
let u1 = bag.require::<UuidDep>().unwrap().0.next();
let u2 = bag.require::<UuidDep>().unwrap().0.next();

let bag2 = DependencyValues::test();
assert_eq!(u1, bag2.require::<UuidDep>().unwrap().0.next()); // same seed
assert_ne!(u1, u2); // monotonic within one bag
```

### Advance time for debounce / throttle tests

```rust
use rust_elm::Environment;

let env = Environment::test();
let clock = env.clock().expect("test env includes FakeClock-compatible clock");
// Or advance the shared TestClock used by ClockDep / NowDep in the bag

let now_before = env.require::<rust_elm::NowDep>().unwrap().0.system_time();
clock.advance(std::time::Duration::from_secs(5));
let now_after = env.require::<rust_elm::NowDep>().unwrap().0.system_time();
assert!(now_after > now_before);
```

### Catch missing test doubles at compile time / early panic

If you add a new `DependencyKey` but forget `try_test()`, calling `MyKey::test()` panics with:

```text
missing test value for dependency `my_crate::MyKey` — override `DependencyKey::try_test`
```

Prefer `MyKey::try_test()` in library code that should fall back gracefully.

---

## Integration with `Environment`

```
┌─────────────────────────────────────────┐
│ Environment                             │
│  layers: [ base, overlay₁, overlay₂ ]   │
│  merged_cache (invalidated on push)     │
└─────────────────────────────────────────┘
         │ get::<D>()  ──► walk layers rev
         │ values()    ──► merge all layers (cached)
         ▼
┌─────────────────────────────────────────┐
│ DependencyValues                        │
│  Arc<RwLock<HashMap<TypeId, Arc<Any>>>> │
└─────────────────────────────────────────┘
```

| Entry point | When to use |
|-------------|-------------|
| `DependencyValues::live()` | Standalone bag, tests without layers |
| `Environment::live()` | Production runtime default |
| `Environment::test()` | Integration / effect tests |
| `env.with(dep)` | Single override layer |
| `env.scoped_with(other)` | `Effect::provide`-style fork |

---

## Implementation notes

- **Read-heavy locking:** `DependencyValues` uses `parking_lot::RwLock`; lookups take a shared guard.
- **TypeId hasher:** custom `TypeIdHasher` avoids SipHash cost on already-distinct keys.
- **Concurrency:** `LiveRng` / `LiveUuidGen` use atomics; `SeededRng` uses a mutex with a single read-modify-write guard to prevent duplicate draws.
- **Shared test clock:** `ClockKey::test()`, `NowKey::test()`, and `DependencyValues::test()` share one `TestClock` via `OnceLock` so isolated key registration still stays in sync.

---

## Related reading

- [`env.rs`](../src/env.rs) — `Environment`, `FakeClock`, `MockHttp`, fiber-local `batch`
- [`effect.rs`](../src/effect.rs) — `Effect::provide` for scoped dependency injection
- [`tests/dependencies_integration.rs`](../tests/dependencies_integration.rs) — end-to-end override tests
- Workspace [`todo0.2.0.md`](../../todo0.2.0.md) — optimization and correctness changelog
