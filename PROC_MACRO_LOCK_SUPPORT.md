# Proc Macro Lock Support

## ⚠️ IMPORTANT: Lock Type Defaults

**`Mutex` and `RwLock` default to `parking_lot` types!**

- If you write `Arc<Mutex<T>>` or `Arc<RwLock<T>>`, it will be treated as **parking_lot**
- To use **std::sync**, you MUST use the full path: `Arc<std::sync::Mutex<T>>` or `Arc<std::sync::RwLock<T>>`

```rust
use parking_lot::RwLock;  // OK - will use parking_lot methods
use std::sync::RwLock;    // Must use: std::sync::RwLock<T> in the field type

#[derive(Keypaths)]
struct Example {
    // These default to parking_lot:
    data1: Arc<RwLock<Inner>>,           // parking_lot (default)
    data2: Arc<Mutex<Inner>>,            // parking_lot (default)
    
    // These use std::sync:
    data3: Arc<std::sync::RwLock<Inner>>,  // std::sync (explicit)
    data4: Arc<std::sync::Mutex<Inner>>,   // std::sync (explicit)
}
```

## Generated Methods

### For parking_lot (Default)

For fields like `f1: Arc<RwLock<T>>` (without `std::sync::` prefix):

| Method | Returns | Description |
|--------|---------|-------------|
| `{field}_r()` | `KeyPath<Struct, Arc<RwLock<T>>>` | Readable keypath to the lock field |
| `{field}_w()` | `WritableKeyPath<Struct, Arc<RwLock<T>>>` | Writable keypath to the lock field |
| `{field}_fr_at(inner_kp)` | Chain | Chains through parking_lot lock for reading |
| `{field}_fw_at(inner_kp)` | Chain | Chains through parking_lot lock for writing |

### For std::sync (Explicit)

For fields like `f1: Arc<std::sync::RwLock<T>>`:

| Method | Returns | Description |
|--------|---------|-------------|
| `{field}_r()` | `KeyPath<Struct, Arc<RwLock<T>>>` | Readable keypath to the lock field |
| `{field}_w()` | `WritableKeyPath<Struct, Arc<RwLock<T>>>` | Writable keypath to the lock field |
| `{field}_fr_at(inner_kp)` | Chain | Chains through std::sync lock for reading |
| `{field}_fw_at(inner_kp)` | Chain | Chains through std::sync lock for writing |

## Usage Examples

### parking_lot (Default)

```rust
use std::sync::Arc;
use parking_lot::RwLock;
use keypaths_proc::Keypaths;

#[derive(Keypaths)]
#[All]
struct Container {
    data: Arc<RwLock<Inner>>,  // parking_lot (default)
}

#[derive(Keypaths)]
#[All]
struct Inner {
    value: String,
}

fn main() {
    let instance = Container {
        data: Arc::new(RwLock::new(Inner { value: "hello".into() })),
    };

    // Use the generated _fr_at() method (parking_lot)
    Container::data_fr_at(Inner::value_r())
        .get(&instance, |value| {
            println!("Value: {}", value);
        });

    // Or chain manually
    Container::data_r()
        .chain_arc_parking_rwlock_at_kp(Inner::value_r())
        .get(&instance, |value| {
            println!("Value: {}", value);
        });
}
```

### std::sync (Explicit)

```rust
use std::sync::{Arc, RwLock};
use keypaths_proc::Keypaths;

#[derive(Keypaths)]
#[All]
struct Container {
    // MUST use full path for std::sync
    data: Arc<std::sync::RwLock<Inner>>,
}

#[derive(Keypaths)]
#[All]
struct Inner {
    value: String,
}

fn main() {
    let instance = Container {
        data: Arc::new(RwLock::new(Inner { value: "hello".into() })),
    };

    // Use the generated _fr_at() method (std::sync)
    Container::data_fr_at(Inner::value_r())
        .get(&instance, |value| {
            println!("Value: {}", value);
        });

    // Or chain manually
    Container::data_r()
        .chain_arc_rwlock_at_kp(Inner::value_r())
        .get(&instance, |value| {
            println!("Value: {}", value);
        });
}
```

## Why This Design?

1. **parking_lot is faster** - No lock poisoning, better performance
2. **Simpler common case** - Most users will use parking_lot
3. **Explicit std::sync** - Forces awareness when using std::sync
4. **Compile-time safety** - The macro detects `std::sync::` in the path and generates appropriate code

## Testing

Run the parking_lot example:
```bash
cargo run --example parking_lot_nested_chain --features parking_lot
```

This demonstrates chaining through multiple nested `Arc<parking_lot::RwLock<T>>` layers with zero-copy access.
