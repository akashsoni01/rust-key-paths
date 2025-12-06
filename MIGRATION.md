# Migration Guide: From `key-paths-core` to `rust-keypaths`

This guide helps you migrate from the dynamic dispatch `key-paths-core` (v1.6.0) to the static dispatch `rust-keypaths` (v1.0.0) implementation.

## Overview

**rust-keypaths** is a static dispatch, faster alternative to `key-paths-core`. It provides:
- ✅ **Better Performance**: Write operations can be faster than manual unwrapping at deeper nesting levels
- ✅ **Zero Runtime Overhead**: No dynamic dispatch costs
- ✅ **Better Compiler Optimizations**: Static dispatch allows more aggressive inlining
- ✅ **Type Safety**: Full compile-time type checking with zero runtime cost

## When to Migrate

### ✅ Migrate to `rust-keypaths` if you:
- Want the best performance
- Don't need `Send + Sync` bounds
- Are starting a new project
- Want better compiler optimizations
- Don't need dynamic dispatch with trait objects

### ⚠️ Stay with `key-paths-core` v1.6.0 if you:
- Need `Send + Sync` bounds for multithreaded scenarios
- Require dynamic dispatch with trait objects
- Have existing code using the enum-based `KeyPaths` API
- Need compatibility with older versions

## Installation Changes

### Before (key-paths-core)
```toml
[dependencies]
key-paths-core = "1.6.0"
key-paths-derive = "1.1.0"
```

### After (rust-keypaths)
```toml
[dependencies]
rust-keypaths = "1.0.0"
keypaths-proc = "1.0.0"
```

## API Changes

### Core Types

#### Before: Enum-based API
```rust
use key_paths_core::KeyPaths;

// Readable keypath
let kp: KeyPaths<Struct, String> = KeyPaths::readable(|s: &Struct| &s.field);

// Failable readable keypath
let opt_kp: KeyPaths<Struct, String> = KeyPaths::failable_readable(|s: &Struct| s.opt_field.as_ref());

// Writable keypath
let writable_kp: KeyPaths<Struct, String> = KeyPaths::writable(|s: &mut Struct| &mut s.field);

// Failable writable keypath
let fw_kp: KeyPaths<Struct, String> = KeyPaths::failable_writable(|s: &mut Struct| s.opt_field.as_mut());
```

#### After: Type-based API
```rust
use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

// Readable keypath
let kp = KeyPath::new(|s: &Struct| &s.field);

// Failable readable keypath
let opt_kp = OptionalKeyPath::new(|s: &Struct| s.opt_field.as_ref());

// Writable keypath
let writable_kp = WritableKeyPath::new(|s: &mut Struct| &mut s.field);

// Failable writable keypath
let fw_kp = WritableOptionalKeyPath::new(|s: &mut Struct| s.opt_field.as_mut());
```

### Access Methods

#### Before
```rust
use key_paths_core::KeyPaths;

let kp = KeyPaths::readable(|s: &User| &s.name);
let value = kp.get(&user);  // Returns Option<&String>
```

#### After
```rust
use rust_keypaths::KeyPath;

let kp = KeyPath::new(|s: &User| &s.name);
let value = kp.get(&user);  // Returns &String (direct, not Option)
```

### Optional KeyPaths

#### Before
```rust
use key_paths_core::KeyPaths;

let opt_kp = KeyPaths::failable_readable(|s: &User| s.email.as_ref());
let email = opt_kp.get(&user);  // Returns Option<&String>
```

#### After
```rust
use rust_keypaths::OptionalKeyPath;

let opt_kp = OptionalKeyPath::new(|s: &User| s.email.as_ref());
let email = opt_kp.get(&user);  // Returns Option<&String> (same)
```

### Chaining KeyPaths

#### Before
```rust
use key_paths_core::KeyPaths;

let kp1 = KeyPaths::failable_readable(|s: &Root| s.level1.as_ref());
let kp2 = KeyPaths::failable_readable(|l1: &Level1| l1.level2.as_ref());
let chained = kp1.compose(kp2);
```

#### After
```rust
use rust_keypaths::OptionalKeyPath;

let kp1 = OptionalKeyPath::new(|s: &Root| s.level1.as_ref());
let kp2 = OptionalKeyPath::new(|l1: &Level1| l1.level2.as_ref());
let chained = kp1.then(kp2);  // Note: method name changed from compose() to then()
```

### Writable KeyPaths

#### Before
```rust
use key_paths_core::KeyPaths;

let mut user = User { name: "Alice".to_string() };
let kp = KeyPaths::writable(|s: &mut User| &mut s.name);
if let Some(name_ref) = kp.get_mut(&mut user) {
    *name_ref = "Bob".to_string();
}
```

#### After
```rust
use rust_keypaths::WritableKeyPath;

let mut user = User { name: "Alice".to_string() };
let kp = WritableKeyPath::new(|s: &mut User| &mut s.name);
let name_ref = kp.get_mut(&mut user);  // Returns &mut String directly (not Option)
*name_ref = "Bob".to_string();
```

### Container Unwrapping

#### Before
```rust
use key_paths_core::KeyPaths;

let kp = KeyPaths::failable_readable(|s: &Container| s.boxed.as_ref());
let unwrapped = kp.for_box::<String>();  // Required explicit type parameter
```

#### After
```rust
use rust_keypaths::OptionalKeyPath;

let kp = OptionalKeyPath::new(|s: &Container| s.boxed.as_ref());
let unwrapped = kp.for_box();  // Type automatically inferred!
```

### Enum Variant Extraction

#### Before
```rust
use key_paths_core::KeyPaths;

let enum_kp = KeyPaths::readable_enum(|e: &MyEnum| {
    match e {
        MyEnum::Variant(v) => Some(v),
        _ => None,
    }
});
```

#### After
```rust
use rust_keypaths::EnumKeyPaths;

let enum_kp = EnumKeyPaths::for_variant(|e: &MyEnum| {
    if let MyEnum::Variant(v) = e {
        Some(v)
    } else {
        None
    }
});
```

### Proc Macros

#### Before
```rust
use key_paths_derive::Keypaths;

#[derive(Keypaths)]
struct User {
    name: String,
    email: Option<String>,
}

// Usage
let name_kp = User::name_r();  // Returns KeyPaths<User, String>
let email_kp = User::email_fr();  // Returns KeyPaths<User, String>
```

#### After
```rust
use keypaths_proc::Keypaths;

#[derive(Keypaths)]
struct User {
    name: String,
    email: Option<String>,
}

// Usage
let name_kp = User::name_r();  // Returns KeyPath<User, String, ...>
let email_kp = User::email_fr();  // Returns OptionalKeyPath<User, String, ...>
```

## Breaking Changes

### 1. Type System
- **Before**: Single `KeyPaths<Root, Value>` enum type
- **After**: Separate types: `KeyPath`, `OptionalKeyPath`, `WritableKeyPath`, `WritableOptionalKeyPath`

### 2. Method Names
- `compose()` → `then()` for chaining
- `get()` on `KeyPath` returns `&Value` (not `Option<&Value>`)
- `get_mut()` on `WritableKeyPath` returns `&mut Value` (not `Option<&mut Value>`)

### 3. Owned KeyPaths
- **Before**: `KeyPaths::owned()` and `KeyPaths::failable_owned()` supported
- **After**: Owned keypaths not supported (use readable/writable instead)

### 4. Send + Sync Bounds
- **Before**: `KeyPaths` enum implements `Send + Sync`
- **After**: `rust-keypaths` types do not implement `Send + Sync` (by design for better performance)

### 5. Type Parameters
- Container unwrapping methods (`for_box()`, `for_arc()`, `for_rc()`) no longer require explicit type parameters - types are automatically inferred

## Migration Steps

### Step 1: Update Dependencies
```toml
# Remove
key-paths-core = "1.6.0"
key-paths-derive = "1.1.0"

# Add
rust-keypaths = "1.0.0"
keypaths-proc = "1.0.0"
```

### Step 2: Update Imports
```rust
// Before
use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

// After
use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::Keypaths;
```

### Step 3: Update KeyPath Creation
```rust
// Before
let kp = KeyPaths::readable(|s: &Struct| &s.field);

// After
let kp = KeyPath::new(|s: &Struct| &s.field);
```

### Step 4: Update Chaining
```rust
// Before
let chained = kp1.compose(kp2);

// After
let chained = kp1.then(kp2);
```

### Step 5: Update Access Patterns
```rust
// Before - KeyPath returns Option
let kp = KeyPaths::readable(|s: &Struct| &s.field);
if let Some(value) = kp.get(&instance) {
    // ...
}

// After - KeyPath returns direct reference
let kp = KeyPath::new(|s: &Struct| &s.field);
let value = kp.get(&instance);  // Direct access, no Option
```

### Step 6: Update Container Unwrapping
```rust
// Before
let unwrapped = kp.for_box::<String>();

// After
let unwrapped = kp.for_box();  // Type inferred automatically
```

## Performance Improvements

After migration, you can expect:

- **Write Operations**: Can be 2-7% faster than manual unwrapping at deeper nesting levels
- **Read Operations**: ~2-3x overhead vs manual unwrapping, but absolute time is still sub-nanosecond
- **Better Inlining**: Compiler can optimize more aggressively
- **Zero Dynamic Dispatch**: No runtime overhead from trait objects

See [BENCHMARK_REPORT.md](rust-keypaths/benches/BENCHMARK_REPORT.md) for detailed performance analysis.

## Common Patterns

### Pattern 1: Simple Field Access
```rust
// Before
let kp = KeyPaths::readable(|s: &User| &s.name);
let name = kp.get(&user).unwrap();

// After
let kp = KeyPath::new(|s: &User| &s.name);
let name = kp.get(&user);  // No unwrap needed
```

### Pattern 2: Optional Field Access
```rust
// Before
let kp = KeyPaths::failable_readable(|s: &User| s.email.as_ref());
if let Some(email) = kp.get(&user) {
    // ...
}

// After
let kp = OptionalKeyPath::new(|s: &User| s.email.as_ref());
if let Some(email) = kp.get(&user) {
    // ...
}
```

### Pattern 3: Deep Nesting
```rust
// Before
let kp1 = KeyPaths::failable_readable(|r: &Root| r.level1.as_ref());
let kp2 = KeyPaths::failable_readable(|l1: &Level1| l1.level2.as_ref());
let kp3 = KeyPaths::failable_readable(|l2: &Level2| l2.value.as_ref());
let chained = kp1.compose(kp2).compose(kp3);

// After
let kp1 = OptionalKeyPath::new(|r: &Root| r.level1.as_ref());
let kp2 = OptionalKeyPath::new(|l1: &Level1| l1.level2.as_ref());
let kp3 = OptionalKeyPath::new(|l2: &Level2| l2.value.as_ref());
let chained = kp1.then(kp2).then(kp3);
```

### Pattern 4: Mutable Access
```rust
// Before
let mut user = User { name: "Alice".to_string() };
let kp = KeyPaths::writable(|s: &mut User| &mut s.name);
if let Some(name_ref) = kp.get_mut(&mut user) {
    *name_ref = "Bob".to_string();
}

// After
let mut user = User { name: "Alice".to_string() };
let kp = WritableKeyPath::new(|s: &mut User| &mut s.name);
let name_ref = kp.get_mut(&mut user);  // Direct access
*name_ref = "Bob".to_string();
```

## Troubleshooting

### Error: "trait bound `Send + Sync` is not satisfied"
**Solution**: If you need `Send + Sync`, stay with `key-paths-core` v1.6.0. `rust-keypaths` intentionally doesn't implement these traits for better performance.

### Error: "method `compose` not found"
**Solution**: Use `then()` instead of `compose()`.

### Error: "expected `Option`, found `&String`"
**Solution**: `KeyPath::get()` returns `&Value` directly, not `Option<&Value>`. Use `OptionalKeyPath` if you need `Option`.

### Error: "type annotations needed" for `for_box()`
**Solution**: Remove the type parameter - `for_box()` now infers types automatically.

## Need Help?

- Check the [rust-keypaths README](rust-keypaths/README.md) for detailed API documentation
- See [examples](rust-keypaths/examples/) for comprehensive usage examples
- Review [benchmark results](rust-keypaths/benches/BENCHMARK_REPORT.md) for performance analysis

## Summary

Migrating from `key-paths-core` to `rust-keypaths` provides:
- ✅ Better performance (especially for write operations)
- ✅ Zero runtime overhead
- ✅ Better compiler optimizations
- ✅ Automatic type inference
- ⚠️ No `Send + Sync` support (by design)
- ⚠️ No owned keypaths (use readable/writable instead)

For most use cases, `rust-keypaths` is the recommended choice. Only use `key-paths-core` v1.6.0 if you specifically need `Send + Sync` bounds or dynamic dispatch.

