# key-paths-macros

Ergonomic `keypath!` macros for [rust-key-paths]. Use with [key-paths-derive] to build keypaths with dot notation.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
rust-key-paths = "2"
key-paths-derive = "2"
key-paths-macros = "1"
```

Derive `Kp` on your types, then use the macros:

```rust
use key_paths_derive::Kp;
use key_paths_macros::{keypath, get, get_mut, set};

#[derive(Kp)]
struct User { name: String, age: u32 }

// Single field
keypath!(User.name).get(&user);
keypath!(User.name).get_mut(&mut user);

// Nested: alternate Type.field for each segment
keypath!(App.user.User.name).get(&app);
keypath!{ SomeComplexStruct.scsf.SomeOtherStruct.sosf.OneMoreStruct.omse.SomeEnum.b.DarkStruct.dsf }.get(&instance);
keypath!{ SomeComplexStruct.scsf.SomeOtherStruct.sosf.OneMoreStruct.omse.SomeEnum.b.DarkStruct.dsf }.get_mut(&mut instance);

// Shorthand macros
get!(&user => User.name);
get_mut!(&mut user => User.name);
set!(&mut user => (User.name) = "Alice".to_string());  // path in parens for set!
```

## Syntax

- **Single segment:** `keypath!(Type.field)` or `keypath!{ Type.field }`
- **Multiple segments:** `keypath!(Type1.f1.Type2.f2.Type3.f3)` — each segment is `Type.field` so the macro can expand to `Type1::f1().then(Type2::f2()).then(Type3::f3())`
- **`set!`** requires the path in parentheses: `set!(root => (Type.field) = value)` so the macro can separate path from value

## License

Mozilla Public License 2.0

[rust-key-paths]: https://docs.rs/rust-key-paths
[key-paths-derive]: https://docs.rs/key-paths-derive
