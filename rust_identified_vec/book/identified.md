# Identified collections

The [`rust_identified_vec`](https://crates.io/crates/rust_identified_vec) crate provides **ordered, id-indexed collections** — the state-engine counterpart.

Use an `IdentifiedVec` when you need:

- Stable **O(1) lookup** by id
- **Deterministic iteration order** (insertion order, user-reorderable)
- **Per-element reducers** via [`ForEachReducer`](../../rust-elm/src/scope.rs) in `rust-elm` without losing sync between list UI and state

---

## Core types

### `Identifiable`

Every element must expose a stable identifier:

```rust
pub trait Identifiable {
    type Id: Copy + Eq + Hash;
    fn id(&self) -> Self::Id;
}
```

**Invariant:** the id must not change while the element lives in an `IdentifiedVec`. Changing the id through `get_mut` leaves the internal `HashMap` stale — lookups by the old id still hit the slot, but `is_consistent()` returns false and lookups by the new id fail.

Prefer [`IdentifiedVec::update`](#update) for field changes, and [`insert`](#insert) when an element genuinely moves to a new id.

### `IdentifiedVec<Id, T>`

Two backing structures kept in sync:

| Structure | Role |
|-----------|------|
| `Vec<T>` | Insertion / display order |
| `HashMap<Id, usize>` | Id → index for O(1) `get` / `remove` |

Equality compares **only** the ordered `Vec` (`PartialEq` ignores the index map).

---

## API reference

| Method | Behavior |
|--------|----------|
| `new()` / `with_capacity(n)` | Empty collection |
| `insert(item)` | Append, or **replace in place** if id exists (order unchanged). Returns `Some(old)` on replace. |
| `get(id)` / `get_mut(id)` | O(1) lookup by id |
| `update(id, f)` | Mutate fields in place; closure must not change the id |
| `remove(id)` | Remove by id; shifts later elements and repairs the index |
| `reorder(from, to)` | Move element between indices; rebuilds index. Returns `false` if out of bounds or `from == to` |
| `index_of(id)` | Position in display order |
| `contains(id)` | Whether id is present |
| `ids()` | Iterator of ids in display order |
| `iter()` / `iter_mut()` / `&vec` iteration | Walk elements in order |
| `is_consistent()` | Debug helper: index matches items |

---

## Use cases

### 1. Todo list with per-row reducers

Each todo row has its own id and nested state. Parent state holds an `IdentifiedVec`:

```rust
use rust_identified_vec::{Identifiable, IdentifiedVec};

#[derive(Clone, PartialEq, Eq)]
struct Todo {
    id: u64,
    title: String,
    done: bool,
}

impl Identifiable for Todo {
    type Id = u64;
    fn id(&self) -> u64 { self.id }
}

struct AppState {
    todos: IdentifiedVec<u64, Todo>,
}

enum Action {
    Todo(u64, TodoAction),
    AddTodo(Todo),
}

// ForEachReducer (rust-elm) routes TodoAction to the matching row by id.
```

When the user toggles row `3`, only that row's reducer runs — siblings are untouched.

### 2. Chat threads keyed by conversation id

Insert or refresh a thread when messages arrive:

```rust
vec.insert(Thread { id: conv_id, last_message: msg, unread: 1 });
// Same conv_id → updates in place, row stays at same scroll position
```

Replacing in place avoids reorder flicker when polling updates an existing thread.

### 3. Drag-and-drop reorder

UI drag from index 2 → 0:

```rust
vec.reorder(2, 0);
// ids() now reflects new visual order; index_of(id) updated
```

Use `ids()` to drive list rendering; use `get(id)` when handling a tap on a specific row.

### 4. Removing a dismissed sheet / row

```rust
if let Some(removed) = vec.remove(row_id) {
    // emit cancel effect for removed.id in ForEachReducer
}
```

Removing the middle element shifts indices; the implementation repairs the map automatically.

### 5. Persistence (serde feature)

Serialized as a plain JSON array of items (order preserved):

```json
[
  { "id": 1, "value": 10 },
  { "id": 2, "value": 20 }
]
```

On deserialize, each item is `insert`ed in order. **Duplicate ids in the payload:** last occurrence wins (same as calling `insert` repeatedly).

Enable with the `serde` feature:

```toml
rust_identified_vec = { version = "0.1.1", features = ["serde"] }
```

### 6. Safe in-place edits

```rust
// Good — value fields only
vec.update(todo_id, |todo| todo.done = true);

// Bad — breaks index
if let Some(t) = vec.get_mut(todo_id) {
    t.id = new_id; // stale index; use insert with new item instead
}
```

---

## Edge cases (tested)

| Scenario | Expected behavior |
|----------|-------------------|
| Empty vec | `get` / `remove` → `None`; `reorder` → `false` |
| Insert duplicate id | Replace at same index; `insert` returns previous element |
| Remove first / last / middle | Index repaired for shifted elements |
| `reorder` out of bounds | No-op, returns `false` |
| `reorder(from, from)` | No-op |
| Mutate id via `get_mut` | `is_consistent()` → false; stale key still resolves |
| Serde empty `[]` | Empty vec |
| Serde duplicate ids | Last item wins; order from first occurrence of each surviving id |
| `PartialEq` | Compares ordered items only, not internal map |

---

## Integration with rust-elm

`ForEachReducer` in `rust-elm/src/scope.rs` takes:

```rust
get_vec: fn(&mut Parent) -> &mut IdentifiedVec<Id, Child>,
embed: fn(Id, ChildAction) -> ParentAction,
extract: fn(ParentAction) -> Option<(Id, ChildAction)>,
```

When a child is removed from the vec, the reducer can emit `Effect::cancel` for that row's in-flight work — mirroring `onDelete` cancellation.

See `rust-elm/tests/identified_scope_integration.rs` for end-to-end scope + identified vec tests.

---

## When *not* to use `IdentifiedVec`

- **Order doesn't matter, only membership** → `HashMap<Id, T>` is simpler
- **Ids change frequently** → re-key with `remove` + `insert`, or store id only in the map key
- **Very large collections with mostly sequential access** → plain `Vec` may be enough

---

## Related

- [`src/lib.rs`](../src/lib.rs) — implementation
- [`rust-elm` scope module](../../rust-elm/src/scope.rs) — `ForEachReducer`
- [`rust-elm` integration tests](../../rust-elm/tests/identified_scope_integration.rs)
