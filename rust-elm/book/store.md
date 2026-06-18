# Store

The `Store` is the **handle** to a running app. It does not own state — it shares the
runtime's `Arc<Mutex<S>>` and the action bus. You dispatch actions through it, read
snapshots from it, scope it to a child, and subscribe to state changes.

All examples below use the shop from [`examples/ecommerce.rs`](../examples/ecommerce.rs):
`ShopState` (root), `CartState` / `CheckoutState` / `WishlistState` (children), and the
`ShopAction` enum tree.

---

## 1. Anatomy

A `Store<S, M>` is a thin, cloneable wrapper over a shared `StoreBackend<S, M>`:

```62:102:rust-elm/src/runtime/store.rs
pub struct Store<S, M> {
    pub(crate) backend: StoreBackend<S, M>,
}

impl<S, M> Clone for Store<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
        }
    }
}

pub(crate) struct StoreBackend<S, M> {
    pub state: Arc<Mutex<S>>,
    pub sender: BusSender<M>,
    state_listeners: Arc<Mutex<Vec<Sender<()>>>>,
    effect_done: Arc<Mutex<VecDeque<Sender<()>>>>,
    in_flight: Arc<AtomicUsize>,
    pub interpreter: Arc<InterpreterState<M>>,
}
```

Every field is an `Arc`, so `store.clone()` is cheap — all clones point at the same
state, bus, and listener list. You can hand a clone to every worker thread / UI surface.

```
            ┌─────────────── one StoreBackend (all Arc) ───────────────┐
 Store ───► │ state: Arc<Mutex<S>>   sender: Bus   listeners: Vec<tx>   │
 Store ───► │           ▲                  │              ▲             │
 Store ───► │           │ reducer thread   │ bus          │ notify      │
            └───────────┼──────────────────┼──────────────┼────────────┘
                        │                  ▼              │
                    locks state      reducer applies   sends () to
                    & mutates        action → state     each listener
```

**Key rule:** the reducer runs on the runtime's dedicated thread, not on your caller's
thread. `send` / `dispatch` only enqueue — they never run `update` inline. This is why
reducers can't re-enter themselves from your call stack (UDF parity).

---

## 2. How do you bind a view with state?

Two halves: **read** state out, **dispatch** actions in.

> **Zero-copy path (recommended):** see [binding.md](./binding.md) for `StateBinding`,
> keypath projection, and `subscribe_changes()` — no full-state clone on the hot path.

### Read: subscribe to snapshots (legacy)

```189:200:rust-elm/src/runtime/store.rs
    pub fn subscribe_state(&self) -> StateSubscriber<S>
    where
        S: Clone + PartialEq,
    {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.backend.state_listeners.lock().push(tx);
        StateSubscriber {
            rx,
            state: self.backend.state.clone(),
            last: Some(Arc::new(self.backend.state.lock().clone())),
        }
    }
```

`last` is the subscriber's **last emitted snapshot**. It is touched in three places,
all inside `StateSubscriber` (`rust-elm/src/runtime/store.rs`):

| Where | What `last` does |
|-------|------------------|
| `subscribe_state` (above) | Seed with the current state so the first `next()` does **not** fire for state that already existed at subscribe time. |
| `next()` | Dedup: after coalescing channel ticks, compare the fresh snapshot against `last` with `PartialEq`; skip if equal (notify without real change). On a real change, `last = Some(snapshot.clone())` — an `Arc` refcount bump, not a deep clone of `S`. |
| `latest()` | Return `self.last.clone()` — the most recent snapshot the subscriber yielded, without waiting on the channel. |

The same subscriber pattern exists on [`Shared`](rust-elm/src/infra/shared.rs) as
`SharedSubscriber.last: Option<T>` (plain `T`, not `Arc`). Initialization,
dedup-in-`next()`, and `latest()` work the same way; only the snapshot type differs.
See [§5](#5-how-does-subscribe_state-work-and-why-clone-last) for the `next()` loop in
full.

A view loop binds like this (from the ecommerce demo):

```rust
let store: Store<ShopState, ShopAction> = runtime.store();
let mut sub = store.subscribe_state();

// user interaction → dispatch
store.dispatch(ShopAction::Catalog(CatalogAction::SetQuery("hoodie".into())));

// state change → re-render
if let Some(snapshot) = sub.wait_next(Duration::from_secs(2)) {
    // snapshot: Arc<ShopState> — render it
    if let Some(detail) = snapshot.catalog.detail.as_ref() {
        render(detail);
    }
}
```

- `next()` — non-blocking poll; returns `Some(Arc<S>)` only if state changed.
- `wait_next(timeout)` — blocks up to timeout (polls every 5 ms internally).
- `latest()` — last snapshot you saw, no waiting.

The cycle is: **dispatch → reducer mutates state → backend notifies listeners →
subscriber yields a new `Arc<S>` → view renders.**

### Write: dispatch / send

| Call | Behaviour |
|------|-----------|
| `store.dispatch(a)` | fire-and-forget |
| `store.send(a)` | returns `StoreTask`; `.finish()` waits until that action's effects complete |

---

## 3. How do you bind a view with state — no clone / no copy?

Be aware of what actually clones today:

- `store.state()` clones the **whole** `S` (`self.backend.state.lock().clone()`).
- `subscribe_state()` clones once at creation, and `next()` clones once per change.

`Store` does **not** hand out a `&S` reference, because the canonical state lives behind
a `Mutex` owned by the reducer thread; lending a borrow across that boundary isn't sound.
So "zero clone" needs one of these strategies:

**a) Lock the runtime's state directly (true zero-clone read).**
`Runtime.state` is public, so a reader on the same process can take the guard:

```rust
let guard = runtime.state.lock();   // parking_lot::MutexGuard<ShopState>
render(&guard.cart);                 // &CartState — no clone
// drop(guard) ASAP — you are blocking the reducer thread while held
```

This borrows `&S` with no clone/copy, but it **contends with the reducer**. Hold it for
the shortest possible time (read the fields you need, drop the guard).

**b) Make `Clone` shallow by putting big payloads behind `Arc`.**
If `ShopState` fields are `Arc<…>`, then the `clone()` inside `state()` /
`subscribe_state()` only bumps refcounts — no deep copy. The snapshot you get is
`Arc<ShopState>`, and sharing it to a view is another refcount bump:

```rust
let snap: Arc<ShopState> = sub.next().unwrap(); // cloning Arc<S> = refcount++
view.bind(snap);                                 // share, don't deep-copy
```

This is the recommended pattern for big state: the subscriber already yields `Arc<S>`, so
**downstream** binding is clone-free; only the inner `S` construction matters, and `Arc`
fields keep that shallow.

**c) Scope so you observe only a small child.** A `ScopedStateSubscriber` clones the
parent to diff, then focuses the child — see §6. The child you bind is small.

> Not available today: a `&S` / `Arc<S>`-returning accessor on `Store` itself, and a
> subscriber that yields without any clone. The dedup logic (§5) requires owning a
> snapshot to compare against. If you need fully borrow-based binding, read via (a) or
> structure state with `Arc`/`Shared` (see `infra::shared::Shared`).

---

## 4. Why are state listeners a `Vec`, and why behind a `Mutex`?

```81:81:rust-elm/src/runtime/store.rs
    state_listeners: Arc<Mutex<Vec<Sender<()>>>>,
```

**Why a `Vec` (not a single entry):** a store can have *many* simultaneous observers —
the main view, a debug panel, a logger, several `ScopedStore` subscribers, etc. Each
`subscribe_state()` pushes its **own** channel sender, so every subscriber gets its own
independent notification stream. One entry could only ever feed one observer.

**Why a `Mutex`:** the listener list is touched from different threads at once —
subscribers are added from arbitrary caller threads (`push(tx)` in `subscribe_state`),
while the **reducer thread** iterates the list to notify after each reduce:

```142:146:rust-elm/src/runtime/store.rs
    pub fn notify_state(&self) {
        self.state_listeners
            .lock()
            .retain(|tx| tx.send(()).is_ok());
    }
```

The `Mutex` makes add-vs-notify races safe. `retain(... send().is_ok())` doubles as
cleanup: when a subscriber is dropped its receiver disconnects, `send` fails, and the dead
sender is pruned automatically. The notification payload is just `()` — a "state changed"
tick; the subscriber then reads the real snapshot from the shared `Arc<Mutex<S>>`.

---

## 5. How does `subscribe_state` work, and why clone `last`?

The subscriber holds the receiver, a handle to shared state, and the **last snapshot it
emitted**:

```264:302:rust-elm/src/runtime/store.rs
impl<S: PartialEq + Clone> StateSubscriber<S> {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Arc<S>> {
        loop {
            match self.rx.try_recv() {
                Ok(()) => {
                    while self.rx.try_recv().is_ok() {}
                    let snapshot = Arc::new(self.state.lock().clone());
                    if self
                        .last
                        .as_ref()
                        .is_some_and(|prev| prev.as_ref() == snapshot.as_ref())
                    {
                        continue;
                    }
                    self.last = Some(snapshot.clone());
                    return Some(snapshot);
                }
                Err(crossbeam_channel::TryRecvError::Empty) => return None,
                Err(crossbeam_channel::TryRecvError::Disconnected) => return None,
            }
        }
    }
```

Step by step:

1. **Coalesce** — `while self.rx.try_recv().is_ok() {}` drains all pending ticks, so a
   burst of N reduces becomes at most **one** render.
2. **Snapshot** — clone the current state into an `Arc<S>` once.
3. **Dedup** — compare against `last` with `PartialEq`. If equal, skip (`continue`) — the
   state was notified but is identical (e.g. an action that didn't change anything).
4. **Remember** — store `last = Some(snapshot.clone())` and return the snapshot.

**Why clone `last`:** dedup needs to compare the *next* snapshot against the *previous*
one, so the subscriber must **own** the previous value. Since the emitted value is an
`Arc<S>`, `snapshot.clone()` is just a refcount bump — one `Arc` goes back to the caller,
an identical `Arc` is retained in `last`. It's not a deep copy of `S`.

**Why `last` starts at the current state** (in `subscribe_state`): so a brand-new
subscriber doesn't immediately fire for the state that already existed when it subscribed —
`next()` only yields on the *first real change* after subscription.

The same three touch points exist on `SharedSubscriber` in
`rust-elm/src/infra/shared.rs` (`last: Option<T>` instead of `Option<Arc<S>>`); see the
table in [§2](#2-how-do-you-bind-a-view-with-state).

---

## 6. How does `scope` work, and in what order?

There are **two** different "scopes". Don't confuse them.

### 6a. `Store::scope` — a child *handle* (runtime side)

```203:223:rust-elm/src/runtime/store.rs
    pub fn scope<CS, CM, AK, SK>(
        &self,
        state_kp: SK,
        action_kp: AK,
    ) -> ScopedStore<S, M, CS, CM, AK, SK>
    where
        CS: Clone + PartialEq + Send + Sync + 'static,
        CM: Clone + Send + 'static,
        S: 'static,
        M: 'static,
        AK: Casepath<M, CM> + Clone + Send + Sync + 'static,
        SK: RefKpTrait<S, CS> + Clone,
    {
        ScopedStore {
            store: self.clone(),
            state_kp,
            action_kp,
            _marker: PhantomData,
        }
    }
```

A `ScopedStore` **shares the same backend** (it holds `self.clone()`) plus two optics:

- `state_kp: RefKpTrait<S, CS>` — focus parent state → child state (read side).
- `action_kp: Casepath<M, CM>` — wrap child action → parent action (write side).

From the shop:

```rust
let cart_store = store.scope(cart_lens(), ShopAction::cart_cp());
cart_store.dispatch(CartAction::Line(0, CartLineAction::Inc));
```

When you dispatch on the child, it **wraps** the child action into a parent action and
sends it on the *same* bus:

```343:349:rust-elm/src/runtime/store.rs
    pub fn send(&self, action: CM) -> StoreTask {
        self.store.send(self.action_kp.wrap(action))
    }

    pub fn dispatch(&self, action: CM) {
        self.store.dispatch(self.action_kp.wrap(action));
    }
```

So `cart_store.dispatch(CartAction::Line(0, Inc))` is exactly
`store.dispatch(ShopAction::Cart(CartAction::Line(0, Inc)))` — both lines in the example do
the same thing. Reading focuses the child out of a parent snapshot:

```351:353:rust-elm/src/runtime/store.rs
    pub fn child_state(&self) -> Option<CS> {
        self.state_kp.focus(&self.store.state()).cloned()
    }
```

**Order for `Store::scope`:**
1. `action_kp.wrap(child_action)` → parent action
2. parent action → bus → reducer thread
3. root reducer runs (see 6b for *its* order), state mutates under the `Mutex`
4. listeners notified; a `ScopedStateSubscriber` focuses child state back out

### 6b. `ScopeReducer` — a child *reducer* (reduce side, ordering matters)

`ScopeReducer` is the composition piece used inside the root reducer:

```52:62:rust-elm/src/compose/scope.rs
    fn reduce(&self, state: &mut PS, action: PA) -> Cmd<PA> {
        let Some(child_action) = self.action_kp.extract(&action) else {
            return Cmd::none();
        };
        let Some(child) = self.state_kp.focus_mut(state) else {
            return Cmd::none();
        };
        let cmd = self.child.reduce(child, child_action);
        lift_cmd(cmd, self.action_kp.clone(), self.cancel_id)
    }
```

Order inside one reduce:
1. **Extract** — does this parent action belong to my case? If not, `Cmd::none()` (no-op).
2. **Focus** — get `&mut child` out of parent state. If absent, `Cmd::none()`.
3. **Delegate** — run the child reducer on child state.
4. **Lift** — map the child's `Cmd` back into the parent action space and tag effects with
   `cancel_id` (so the child's in-flight effects can be cancelled on dismiss/removal).

In the shop, the root `shop_reducer` is a `CombineReducers` tuple; its **slot order** is
the execution order for a single action:

```431:455:rust-elm/examples/ecommerce.rs
fn shop_reducer() -> impl Reducer<State = ShopState, Action = ShopAction> {
    CatchReducer::new(
        CombineReducers((
            Reduce::new(subscription_metrics_reducer),
            Reduce::new(detail_cart_bridge_reducer),
            Reduce::new(global_reducer),
            ScopeReducer::new(
                ShopState::catalog(),
                ShopAction::catalog_cp(),
                CATALOG_CANCEL,
                Reduce::new(catalog_reducer),
            ),
            ScopeReducer::new(
                ShopState::cart(),
                ShopAction::cart_cp(),
                CART_CANCEL,
                Reduce::new(cart_reducer),
            ),
            checkout_and_wishlist_reducer(),
        )),
        ...
```

`CombineReducers` runs each slot **in tuple order**, cloning the action for each and
batching the resulting commands:

```71:74:rust-elm/src/compose/reducer.rs
            fn reduce(&self, state: &mut S, action: A) -> Cmd<A> {
                let ($($R,)+) = &self.0;
                Cmd::batch([$( $R.reduce(state, action.clone()) ),+])
            }
```

So a `ShopAction::Cart(...)` flows: metrics (no-op) → bridge (no-op) → global (no-op) →
catalog scope (extract fails, no-op) → **cart scope (matches → runs `cart_reducer`)** →
checkout/wishlist (no-op). Only the matching scope mutates; the rest extract-fail cheaply.

---

## 7. Does sending an action through the store update the state in the mutex?

**Yes — asynchronously.** `send` only enqueues:

```165:176:rust-elm/src/runtime/store.rs
    pub fn send(&self, action: M) -> StoreTask {
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.backend.effect_done.lock().push_back(tx);
        self.backend.begin_store_work();
        let _ = self.backend.sender.send_blocking(action);
        StoreTask { rx }
    }

    /// Fire-and-forget dispatch.
    pub fn dispatch(&self, action: M) {
        let _ = self.send(action);
    }
```

The action goes onto the bus; the **reducer thread** picks it up, locks
`Arc<Mutex<S>>`, runs the reducer (mutating the state in place), then unlocks and notifies
listeners. So after `dispatch` returns, the mutation has been *scheduled*, not necessarily
*applied* yet. To observe it deterministically:

- `store.send(a).finish()` — wait for that action's effects to complete, or
- `sub.wait_next(timeout)` — wait for the next state change, or
- (tests) sleep / `ExhaustiveTestStore`.

This is why the ecommerce demo sleeps between dispatch and `store.state()`.

---

## 8. Does sending through `let cs = store.scope(...)` update the state in the mutex?

**Yes — same mechanism, same mutex.** A `ScopedStore` holds a clone of the *same*
backend, so `cs.dispatch(child)` just wraps and forwards onto the *same* bus
(`self.store.dispatch(self.action_kp.wrap(action))`). There is no separate state — the
root `Arc<Mutex<ShopState>>` is the only state. These two are identical in effect:

```rust
let cart_store = store.scope(cart_lens(), ShopAction::cart_cp());
cart_store.dispatch(CartAction::Line(0, CartLineAction::Inc)); // wraps → ShopAction::Cart(..)
store.dispatch(ShopAction::Cart(CartAction::Line(0, CartLineAction::Inc))); // same thing
```

Both reach the reducer thread, both mutate `cart` inside the one root mutex.

---

## 9. How to subscribe without clone?

Today there is **no fully clone-free subscriber** — dedup needs an owned snapshot to
compare (`prev.as_ref() == snapshot.as_ref()`). Practical ways to minimize/avoid copies:

1. **Yield `Arc<S>`, bind by `Arc`.** `next()` already returns `Arc<S>`; passing it to a
   view is a refcount bump, not a copy. Keep heavy fields in the state behind `Arc` so the
   one internal `clone()` is shallow.
2. **Scope to a small child** so the value you actually bind is tiny:

```355:365:rust-elm/src/runtime/store.rs
    pub fn subscribe_state(&self) -> ScopedStateSubscriber<S, CS, SK>
    where
        S: Clone + PartialEq,
        SK: Clone,
    {
        ScopedStateSubscriber {
            inner: self.store.subscribe_state(),
            state_kp: self.state_kp.clone(),
            _marker: PhantomData,
        }
    }
```

   (The parent is still cloned to diff, but you only hand the focused `CS` to the view.)
3. **Read via lock for a true `&S`** when you don't need change-notifications:
   `let g = runtime.state.lock(); render(&g.cart);` — borrow, no clone, but hold briefly.
4. **Use `infra::shared::Shared<T>`** for values you want to observe by shared handle
   rather than by snapshot.

> If you want a genuinely zero-clone *push* subscription, that would be a new API (e.g. a
> listener that receives `Arc<S>` the reducer already built, skipping per-subscriber
> dedup). Not present today — call it out if you need it.

---

## 10. `Reducer` vs `ScopeReducer` vs `CatchReducer`

All three implement the same trait — they differ in *what they wrap*:

```5:10:rust-elm/src/compose/reducer.rs
pub trait Reducer {
    type State;
    type Action;

    fn reduce(&self, state: &mut Self::State, action: Self::Action) -> Cmd<Self::Action>;
}
```

### `Reducer` (and `Reduce` / fn pointers / `CombineReducers`)

The base abstraction: given `&mut state` and an `action`, mutate state and return a `Cmd`
(effects to run). Plain `fn(&mut S, A) -> Cmd<A>` is a `Reducer` via a blanket impl;
`Reduce::new(f)` wraps one; `CombineReducers((..))` runs several **siblings of the same
`State`/`Action`** in order and batches their commands. Use it for your actual update
logic — e.g. `global_reducer`, `cart_reducer` in the shop.

### `ScopeReducer` — a *structural* adapter (different state/action)

`ScopeReducer<…>` lets a child reducer with `State = CS, Action = CA` participate in a
parent whose `State = PS, Action = PA`. It **extracts** the child action (casepath),
**focuses** the child state (keypath), runs the child, and **lifts** the result back —
including cancel-id tagging for the child's effects (see §6b). It changes *which slice of
state/action* a reducer sees; it does **not** change semantics or catch anything.
Relatives: `IfLetReducer` (optional child + dismiss/cancel), `ForEachReducer`
(per-`IdentifiedVec`-element), `OptionalReducer`, `IfCaseLetReducer`.

### `CatchReducer` — a *safety* adapter (same state/action)

`CatchReducer<R, F>` wraps any reducer and runs it inside [`safe_reduce_update`](./safe_reducer.md). On panic
it calls your `recover` with a [`SafeReduceError`](./safe_reducer.md) to produce a fallback `Cmd` instead of unwinding the reducer
thread:

```100:114:rust-elm/src/compose/reducer.rs
impl<R, F, S, A> Reducer for CatchReducer<R, F>
where
    R: Reducer<State = S, Action = A>,
    F: Fn(SafeReduceError) -> Cmd<A>,
{
    type State = S;
    type Action = A;

    fn reduce(&self, state: &mut S, action: A) -> Cmd<A> {
        match safe_reduce_update(state, |s, a| self.inner.reduce(s, a), action) {
            Ok(cmd) => cmd,
            Err(err) => (self.recover)(err),
        }
    }
}
```

It keeps the **same** `State`/`Action` (it's a transparent wrapper), and by default does
**not** revert partial mutations made before the panic (use [`RollbackCatchReducer`](./safe_reducer.md) /
[`safe_reduce_rollback`](./safe_reducer.md) for checkpoint rollback). In the shop it wraps the entire `CombineReducers` tuple so any
reducer panic is contained and the app keeps running.

### Side-by-side

| Aspect | `Reducer` / `CombineReducers` | `ScopeReducer` | `CatchReducer` |
|--------|-------------------------------|----------------|----------------|
| Purpose | the actual update logic | focus a child slice | contain panics |
| Changes State/Action type? | no | **yes** (`PS/PA` ↔ `CS/CA`) | no |
| Uses optics? | no | yes (keypath + casepath) | no |
| Wraps another reducer? | combines siblings | wraps a child reducer | wraps one reducer |
| On panic | propagates (unwinds) | propagates | **caught** → `recover` cmd |
| Effects | as returned | child effects lifted + cancel-tagged | inner's, or recover's |
| Shop usage | `global_reducer`, `cart_reducer` | `catalog`/`cart` scopes, `ForEach` wishlists | root wrapper |

**Typical composition** (outermost → innermost): `CatchReducer` → `CombineReducers` →
`ScopeReducer`/`ForEachReducer`/`IfLetReducer` → your leaf `Reduce`/fn.

---

## Quick reference

| Task | `Store` (mutex) | `RwStore` (RwLock) | `SwapStore` (`arc-swap`) | `TeaStore` (channel) |
|------|-----------------|---------------------|---------------------------|------------------------|
| Get handle | `runtime.store()` | `runtime.rw_store()` | `runtime.swap_store()` | `runtime.tea_store()` |
| Fire-and-forget | `store.dispatch(action)` | same | same | same |
| Dispatch + await effects | `store.send(action).finish()` | same | same | same |
| Read latest model | `store.state()` (lock) | `store.state()` | `snapshot_store().load()` | `subscribe_state().latest()` or `view_store().load()` |
| Live view / readers | `subscribe_state()` ping+lock | same | push via ArcSwap load | **`subscribe_state()` push `Arc<S>`** |
| Concurrent reader threads | mutex serializes | `read_store()` | `snapshot_store()` | `view_store().subscribe_state()` |
| Child handle | `store.scope(kp, cp)` | `ScopedRwStore` | `ScopedSwapStore` | `ScopedTeaStore` |
| State bound | `S: Send` | `S: Send + Sync` | `S: Send + Sync + Clone` | `S: Send + Sync + Clone` |

RwLock guide: [rw_ecommerce.md](./rw_ecommerce.md). Arc-swap: [swap_ecommerce.md](./swap_ecommerce.md). Channel TEA: [tea_ecommerce.md](./tea_ecommerce.md). Safe reduce: [safe_reducer.md](./safe_reducer.md).
