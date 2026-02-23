# key-paths-iter

Query builder for iterating over `Vec<Item>` collections accessed via [rust-key-paths](https://crates.io/crates/rust-key-paths) `KpType`.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
rust-key-paths = "2"
key-paths-iter = { path = "../key-paths-iter" }  # or from crates.io when published
```

Use with a keypath whose value type is `Vec<Item>`:

```rust
use key_paths_iter::{CollectionQuery, QueryableCollection};
use rust_key_paths::Kp;

let users_kp: rust_key_paths::KpType<'_, Database, Vec<User>> = Kp::new(
    |db: &Database| Some(&db.users),
    |db: &mut Database| Some(&mut db.users),
);

// Chain filters, limit, offset, then execute
let results = users_kp
    .query()
    .filter(|u| u.active)
    .filter(|u| u.age > 26)
    .limit(2)
    .offset(0)
    .execute(&db);

// Or use count / exists / first
let n = users_kp.query().filter(|u| u.active).count(&db);
let any = users_kp.query().filter(|u| u.active).exists(&db);
let first = users_kp.query().filter(|u| u.active).first(&db);
```

The keypath and the root reference share the same lifetime; use a type annotation like `KpType<'_, Root, Vec<Item>>` so the compiler infers the scope correctly.
