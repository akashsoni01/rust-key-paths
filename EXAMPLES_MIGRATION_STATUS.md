# Examples Migration Status

## Overview

The examples in the `examples/` directory were originally written for the `key-paths-core` (dynamic dispatch) API and need updates to work with the new `rust-keypaths` (static dispatch) API.

## Status

- ✅ **Imports Updated**: All 80+ examples have been updated to use `keypaths-proc` and `rust-keypaths` instead of `key-paths-derive` and `key-paths-core`
- ⚠️ **API Updates Needed**: Many examples still need API changes to work with the new type-based system

## Common Issues

### 1. `KeyPath` vs `OptionalKeyPath` for Chaining

**Problem**: `KeyPath` doesn't have a `then()` method - only `OptionalKeyPath` does.

**Solution**: Use `_fr()` methods (failable readable) instead of `_r()` methods when you need to chain:

```rust
// ❌ Wrong - KeyPath can't chain
let path = Struct::field_r().then(Other::value_r());

// ✅ Correct - Use OptionalKeyPath for chaining
let path = Struct::field_fr().then(Other::value_fr());
```

### 2. `get()` Return Type

**Problem**: `KeyPath::get()` returns `&Value` directly, not `Option<&Value>`.

**Solution**: Remove `if let Some()` patterns for `KeyPath`:

```rust
// ❌ Wrong
if let Some(value) = keypath.get(&instance) {
    // ...
}

// ✅ Correct
let value = keypath.get(&instance);
// use value directly
```

### 3. `get_mut()` Method

**Problem**: `KeyPath` doesn't have `get_mut()` - only `WritableKeyPath` does.

**Solution**: Use `_w()` methods for writable access:

```rust
// ❌ Wrong
let mut_ref = keypath.get_mut(&mut instance);

// ✅ Correct
let writable_kp = Struct::field_w();
let mut_ref = writable_kp.get_mut(&mut instance);
```

### 4. Return Type Annotations

**Problem**: The new API uses `impl Trait` in return types which can't be stored in struct fields.

**Solution**: Use type inference or store the keypath in a variable without explicit type:

```rust
// ❌ Wrong - can't use impl Trait in struct fields
struct MyStruct {
    keypath: KeyPath<Root, Value, impl Fn(&Root) -> &Value>,
}

// ✅ Correct - use type inference
let keypath = KeyPath::new(|r: &Root| &r.field);
// or use a type alias if needed
```

### 5. Missing `WithContainer` Trait

**Problem**: `rust-keypaths` doesn't have a `WithContainer` trait.

**Solution**: Use the `containers` module functions directly:

```rust
// ❌ Wrong
use rust_keypaths::WithContainer;

// ✅ Correct
use rust_keypaths::containers;
let vec_kp = containers::for_vec_index::<String>(0);
```

## Migration Checklist

For each example file:

- [ ] Update imports: `key_paths_derive` → `keypaths_proc`
- [ ] Update imports: `key_paths_core::KeyPaths` → `rust_keypaths::{KeyPath, OptionalKeyPath, ...}`
- [ ] Replace `KeyPaths::readable()` → `KeyPath::new()`
- [ ] Replace `KeyPaths::failable_readable()` → `OptionalKeyPath::new()`
- [ ] Replace `KeyPaths::writable()` → `WritableKeyPath::new()`
- [ ] Replace `KeyPaths::failable_writable()` → `WritableOptionalKeyPath::new()`
- [ ] Replace `.compose()` → `.then()`
- [ ] Fix `get()` calls: Remove `if let Some()` for `KeyPath`, keep for `OptionalKeyPath`
- [ ] Fix chaining: Use `_fr()` methods instead of `_r()` when chaining
- [ ] Fix `get_mut()`: Use `_w()` methods to get `WritableKeyPath`
- [ ] Remove `WithContainer` usage if present
- [ ] Update return type annotations to avoid `impl Trait` in struct fields

## Examples That Work

These examples should work with minimal changes:
- Simple examples using only `KeyPath::new()` and `get()`
- Examples using only `OptionalKeyPath::new()` and `get()`
- Examples that don't chain keypaths

## Examples That Need More Work

These examples need significant refactoring:
- Examples using `KeyPaths` enum in struct fields
- Examples using `WithContainer` trait
- Examples with complex chaining that use `_r()` methods
- Examples using owned keypaths (not supported in new API)

## Testing Strategy

1. Start with simple examples (e.g., `basics.rs`, `keypath_simple.rs`)
2. Fix common patterns in those examples
3. Apply fixes to similar examples
4. Handle complex examples individually

## Next Steps

1. Create a test script to identify which examples compile
2. Fix examples one category at a time (simple → complex)
3. Update documentation to reflect the new API patterns
4. Create new examples showcasing the new API's strengths

