# 🔑 KeyPaths in Rust

Key paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **KeyPath and Functional Lenses** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-key-paths = "2.9.8"
key-paths-derive = "2.6.2"
```

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
| `std::sync::Mutex<T>`, `std::sync::RwLock<T>` | `field()` | Container (use `LockKp` for lock-through) |
| `Arc<Mutex<T>>`, `Arc<RwLock<T>>` | `field()`, `field_lock()` | Lock-through via `LockKp` |
| `arc_swap::ArcSwap<T>`, `ArcSwapOption<T>`, `ArcSwapWeak<T>` | `field()` | Container access; `ArcSwap` / `ArcSwapOption` also support lock-through keypaths |
| `tokio::sync::Mutex`, `tokio::sync::RwLock` | `field_async()` | Async lock-through (tokio feature) |
| `parking_lot::Mutex`, `parking_lot::RwLock` | `field()`, `field_lock()` | parking_lot feature |

Nested combinations (e.g. `Option<Box<T>>`, `Option<Vec<T>>`, `Vec<Option<T>>`) are supported.

### Which sync wrapper to use

- `Arc<Mutex<T>>`: balanced default for mixed read/write workloads and simple ownership.
- `Arc<RwLock<T>>`: read-heavy access with relatively infrequent writes.
- `arc_swap::ArcSwap<T>`: read-mostly snapshots with cheap lock-free reads and occasional whole-value swaps.
- `arc_swap::ArcSwapOption<T>`: same as `ArcSwap`, but when the value may be temporarily absent.
- `arc_swap::ArcSwapWeak<T>`: keep weak references in the swap cell to avoid extending strong ownership lifetimes.

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

For the optional ArcSwap variant (`sosf` wrapped in `arc_swap::ArcSwap`; adds groups `arc_swap_keypath_read` / `arc_swap_keypath_write`):

```bash
cargo bench --bench box_keypath_bench --features arc_swap_1_9_1
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

---

## 📜 License

* Mozilla Public License 2.0