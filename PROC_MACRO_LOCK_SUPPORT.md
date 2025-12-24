# Proc Macro Lock Support Improvements

## Summary

Extended the `keypaths-proc` derive macro to properly support keypaths for all types of locks, including both `std::sync` and `parking_lot` lock types. Added comprehensive helper methods for chaining through locks.

## Generated Methods for Lock Fields

For fields like `f1: Arc<Mutex<T>>` or `f1: Arc<RwLock<T>>`, the derive macro now generates:

### Basic Keypaths
| Method | Returns | Description |
|--------|---------|-------------|
| `{field}_r()` | `KeyPath<Struct, Arc<Lock<T>>>` | Readable keypath to the lock field |
| `{field}_w()` | `WritableKeyPath<Struct, Arc<Lock<T>>>` | Writable keypath to the lock field |
| `{field}_o()` | `KeyPath<Struct, Arc<Lock<T>>>` | Owned access keypath |

### Chain Methods for std::sync Locks
| Method | Returns | Description |
|--------|---------|-------------|
| `{field}_fr_at(inner_kp)` | `ArcMutex/RwLockKeyPathChain` | Chains with readable keypath through std::sync lock |
| `{field}_fw_at(inner_kp)` | `ArcMutex/RwLockWritableKeyPathChain` | Chains with writable keypath through std::sync lock |

### Chain Methods for parking_lot Locks (requires `parking_lot` feature)
| Method | Returns | Description |
|--------|---------|-------------|
| `{field}_parking_fr_at(inner_kp)` | `ArcParkingMutex/RwLockKeyPathChain` | Chains with readable keypath through parking_lot lock |
| `{field}_parking_fw_at(inner_kp)` | `ArcParkingMutex/RwLockWritableKeyPathChain` | Chains with writable keypath through parking_lot lock |

## Usage Pattern

### Method 1: Direct Chain Methods (Recommended)

Use the generated `{field}_parking_fr_at()` and `{field}_parking_fw_at()` methods for simplified chaining:

```rust
// Reading through the lock
SomeStruct::f1_parking_fr_at(SomeOtherStruct::name_r())
    .get(&instance, |name| {
        println!("Name: {}", name);
    });

// Writing through the lock  
SomeStruct::f1_parking_fw_at(SomeOtherStruct::name_w())
    .get_mut(&instance, |name| {
        *name = String::from("new_name");
    });
```

### Method 2: Manual Chaining with Library Methods

Use the generated `{field}_r()` method and chain with library methods:

```rust
SomeStruct::f1_r()
    .then_arc_parking_rwlock_at_kp(SomeOtherStruct::f4_r())
    .get(&instance, |inner| {
        // Access deeply nested value
    });
```

## Example

### Before (Manual Keypaths)

```rust
use rust_keypaths::keypath;

let f1_kp = keypath!(|s: &SomeStruct| &s.f1);
let f4_kp = keypath!(|s: &SomeOtherStruct| &s.f4);

f1_kp.then_arc_parking_rwlock_at_kp(f4_kp)
    .get(&instance, |inner| {
        // Access deeply nested value
    });
```

### After (Derive-Generated Keypaths)

```rust
#[derive(Keypaths)]
#[All]  // Generate both readable and writable keypaths
struct SomeStruct {
    f1: Arc<RwLock<SomeOtherStruct>>,
}

#[derive(Keypaths)]
#[All]
struct SomeOtherStruct {
    f4: Arc<RwLock<DeeplyNestedStruct>>,
}

// Use generated methods
SomeStruct::f1_r()
    .then_arc_parking_rwlock_at_kp(SomeOtherStruct::f4_r())
    .get(&instance, |inner| {
        // Access deeply nested value
    });
```

## Benefits

1. **Type-safe**: The derive macro generates correctly-typed keypaths
2. **Less boilerplate**: No need to manually write `keypath!` macros for lock fields
3. **Works with both std and parking_lot**: The same pattern works for both lock implementations
4. **Zero-copy**: All access is done through references, no cloning required
5. **Composable**: Generated keypaths can be chained through multiple lock layers

## Testing

See `examples/parking_lot_nested_chain.rs` for a complete example demonstrating:
- Using derive-generated keypaths for `Arc<parking_lot::RwLock<T>>` fields
- Chaining through multiple lock layers
- Reading and writing nested optional values
- Zero-copy access (proven by panic-on-clone implementations)

Run with:
```bash
cargo run --example parking_lot_nested_chain --features parking_lot
```

## Important Notes

### Struct Attributes

- Use `#[All]` to generate both readable and writable methods
- Use `#[Writable]` to generate only writable methods (no `_r()` methods)
- Use `#[Readable]` to generate only readable methods
- Default (no attribute) generates readable methods

### Clone Limitations

The generated keypaths use `impl Fn` closures which don't implement `Clone`. Therefore:
- Call the generator method (e.g., `SomeStruct::f1_r()`) each time you need the keypath
- Don't try to store the keypath in a variable and `.clone()` it

**✅ Good:**
```rust
SomeStruct::f1_r().then_arc_parking_rwlock_at_kp(...)
SomeStruct::f1_r().then_arc_parking_rwlock_at_kp(...)  // Call again
```

**❌ Bad:**
```rust
let kp = SomeStruct::f1_r();
kp.clone()  // Error: impl Fn doesn't implement Clone
```

