# Rw ecommerce example — read-heavy store architecture

Companion to [`examples/rw_ecommerce.rs`](../examples/rw_ecommerce.rs). Same shop domain as [ecommerce.md](./ecommerce.md); this document focuses on **`RwRuntime`** and a single **`RwStore`** handle.

Run it:

```bash
cargo run -p rust-elm --example rw_ecommerce
```

For reducer composition, subscriptions, and DI, see [ecommerce.md](./ecommerce.md). For mutex stores and `subscribe_changes`, see [store.md](./store.md) and [binding.md](./binding.md).

---

## What this example adds

| Feature | Where |
|---------|--------|
| `RwRuntime` | `Arc<RwLock<ShopState>>` + same bus / Tokio stack as `Runtime` |
| `RwStore` | One cloneable handle — dispatch, `send`, `scope`, subscriptions |
| `RwStore::read_store()` | Read-only view for concurrent `with_read` / `read_binding` |
| `ScopedRwStore` | Cart child dispatch (`cart_scope`) |
| Concurrent reader threads | `spawn_catalog_readers` — each thread calls `store.read_store()` |
| Shared shop logic | [`examples/ecommerce/shop.rs`](../examples/ecommerce/shop.rs) |

The mutex [`ecommerce`](../examples/ecommerce.rs) example keeps panic demos and `subscribe_state` snapshots. This example omits those and highlights **read-heavy** access patterns instead.

---

## One handle pattern

Hold **`RwStore`** everywhere. Derive read access on demand — no separate `runtime.read_store()`:

```rust
let store = runtime.rw_store();

// writes
store.dispatch(ShopAction::Global(GlobalAction::SignIn(user)));
let cart = store.scope(cart_lens(), ShopAction::cart_cp());

// reads (cheap Arc clone of the same backend)
let user = store.read_store().with_read(|s| s.session.user.clone());

// reader threads
let read_store = store.read_store(); // clone onto each thread
```

```mermaid
flowchart TB
    RT["RwRuntime"]
    S["RwStore — clone to any thread"]
    RS["read_store on demand"]
    W["dispatch / send / scope"]
    R["with_read / read_binding"]

    RT -->|rw_store once| S
    S --> W
    S --> RS
    RS --> R
```

`ReadStore` is still a distinct type (read-only API), but you obtain it from **`RwStore::read_store()`**, not as a second top-level runtime handle.

---

## High-level architecture

```mermaid
flowchart TB
    subgraph Producers["Any thread"]
        UI["UI / HTTP handlers"]
        RD["Reader threads (×4)"]
        FX["Tokio effects + subs"]
    end

    subgraph Runtime["RwRuntime"]
        Bus["Action bus (FIFO)"]
        RT["Reducer thread — write lock"]
        Hub["StoreHub (shared)"]
    end

    subgraph State["Arc RwLock ShopState"]
        W["write lock — reducer only"]
        R["read lock — many concurrent"]
    end

    UI -->|"RwStore dispatch"| Bus
    RD -->|"read_store with_read"| R
    FX -->|dispatch_from_effect| Bus
    Bus --> RT
    RT --> W
    W --> State
    R --> State
    RT --> Hub
    Hub -->|notify after drain| RD
```

**Single writer, many readers:** the reducer thread holds `write()` for the duration of `reduce`. Reader threads hold `read()` for short borrows. Writers block readers; readers do not block each other.

---

## Store handles compared

```mermaid
flowchart LR
    subgraph Mutex["Runtime (Mutex)"]
        MS["Store"]
        MS -->|lock| M1["one holder at a time"]
    end

    subgraph Rw["RwRuntime (RwLock)"]
        RS["RwStore"]
        RS -->|dispatch / scope| W["write on reduce"]
        RS -->|read_store| RO["read view"]
        RO -->|with_read| R["concurrent readers"]
    end
```

| Handle | Lock | Use when |
|--------|------|----------|
| `Store` (mutex) | `Mutex` | Simple default; reduce + read serialize |
| `RwStore` | `write()` on reduce | Dispatch, cancel, scoped child writes |
| `ReadStore` (from `store.read_store()`) | `read()` only | Pass to reader threads; analytics loops |
| `ReadStateBinding` | `read()` per `with_read` | Zero-copy field access without cloning `ShopState` |
| `RwStateBinding` | `read()` / `write()` | Local UI that mutates through keypaths (advanced) |

---

## Threading model (shop scenario)

```mermaid
sequenceDiagram
    participant Main as main thread
    participant S as RwStore
    participant Bus as Bus
    participant RT as Reducer thread
    participant Lock as RwLock
    participant R1 as catalog-reader-0

    par Reader threads start
        R1->>S: read_store
        R1->>S: read_binding project catalog
        R1->>Lock: read lock — query borrow
        Lock-->>R1: CatalogState ref
    end

    Main->>S: dispatch SetQuery hoodie
    S->>Bus: send_blocking
    Bus->>RT: recv
    RT->>Lock: write lock
    RT->>RT: shop_reducer
    RT->>Lock: unlock
    Note over R1: next read sees new query

    Main->>S: read_store with_read detail
    S->>Lock: read lock
    Lock-->>Main: borrow — no ShopState clone
```

`StoreHub` (listeners, effect-done queue, in-flight counter) is **shared** between mutex and Rw backends — only the state lock type changes.

---

## Reader worker pattern (from the example)

```rust
fn spawn_catalog_readers(store: RwStore<ShopState, ShopAction>, ...) {
    thread::spawn(move || {
        let read_store = store.read_store();
        let binding = read_store.read_binding().project(ShopState::catalog());
        loop {
            if let Some(q) = binding.with_read(|c| c.query.clone()) {
                // only clone the small field, not ShopState
            }
        }
    });
}

let store = runtime.rw_store();
spawn_catalog_readers(store.clone(), ...);
store.dispatch(...);
```

```mermaid
flowchart TB
    S["RwStore clone"]
    RS["read_store per thread"]
    B["ReadStateBinding"]
    P["ReadProjectedBinding"]
    T1["reader thread 1"]
    T2["reader thread 2"]

    S --> RS
    RS --> B
    B --> P
    P --> T1
    P --> T2
```

Clone `RwStore` onto the spawner; each reader calls `read_store()` once — cheap `Arc` clones of `RwLock` + `StoreHub`.

---

## When to choose Rw vs Mutex

| Scenario | Recommendation |
|----------|----------------|
| Single UI thread + occasional `state()` | `Runtime` / `Store` — simpler |
| Many threads reading catalog/cart while reducer runs | `RwRuntime` + `RwStore::read_store()` |
| Hot path needs zero-copy field reads | `read_binding()` + keypaths |
| Write-heavy, few readers | Mutex is fine (less RwLock writer overhead) |
| Huge state, frequent full snapshots | Avoid `state()` on both; prefer bindings / `subscribe_changes` |

RwLock helps when **read contention** on `Mutex` would serialize unrelated readers. Reduce throughput is still **one action at a time** on the reducer thread.

---

## Scoped dispatch

```rust
let store = runtime.rw_store();
let cart_scope = store.scope(cart_lens(), ShopAction::cart_cp());
cart_scope.dispatch(CartAction::Line(0, CartLineAction::Inc));
```

Same optics as `ScopedStore`; child actions lift to `ShopAction` on the bus. Subscriptions and effects use the shared `StoreHub` — see [ecommerce.md](./ecommerce.md#subscriptions-in-a-child-scope).

---

## Related files

| File | Role |
|------|------|
| [`examples/rw_ecommerce.rs`](../examples/rw_ecommerce.rs) | Single `RwStore` + concurrent readers |
| [`examples/ecommerce.rs`](../examples/ecommerce.rs) | Mutex runtime + panic / subscription demos |
| [`examples/ecommerce/shop.rs`](../examples/ecommerce/shop.rs) | Shared state, reducers, subscriptions |
| [`examples/ecommerce/deps.rs`](../examples/ecommerce/deps.rs) | HTTP + date + WebSocket DI |
| [`src/runtime/rw_engine.rs`](../src/runtime/rw_engine.rs) | `RwRuntime` bootstrap |
| [`src/runtime/rw_store.rs`](../src/runtime/rw_store.rs) | `RwStore`, `ReadStore`, subscribers |
| [`store.md`](./store.md) | Mutex store API + Rw quick reference |
| [`ecommerce.md`](./ecommerce.md) | Full shop composition guide |
