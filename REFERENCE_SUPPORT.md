# Reference Support in rust-key-paths

This document explains how to use keypaths with collections of references (`Vec<&T>`, `HashMap<K, &V>`, etc.) instead of owned values.

## Problem Statement

When working with collections of references (e.g., `Vec<&Product>` from `HashMap::values()`), the standard keypath methods like `.get()` expect the owned type `T`, not `&T`. This created a type mismatch:

```rust
let products: Vec<&Product> = hashmap.values().collect();
let name_path = Product::name_r();  // Returns KeyPaths<Product, String>

// This doesn't work! product_ref is &&Product but get() expects &Product
for product_ref in &products {
    name_path.get(product_ref);  // Type mismatch!
}
```

## Solution: `get_ref()` and `get_mut_ref()`

The `key-paths-core` library now provides two new methods that work with reference types:

### `get_ref()`

For immutable access when you have a reference to a reference:

```rust
pub fn get_ref<'a, 'b>(&'a self, root: &'b &Root) -> Option<&'b Value> 
where
    'a: 'b
```

**Usage:**
```rust
let products: Vec<&Product> = hashmap.values().collect();
let name_path = Product::name_r();

for product_ref in &products {  // product_ref is &&Product
    if let Some(name) = name_path.get_ref(product_ref) {
        println!("Product: {}", name);
    }
}
```

### `get_mut_ref()`

For mutable access when you have a mutable reference to a mutable reference:

```rust
pub fn get_mut_ref<'a, 'b>(&'a self, root: &'b mut &mut Root) -> Option<&'b mut Value> 
where
    'a: 'b
```

**Usage:**
```rust
let mut products_mut: Vec<&mut Product> = vec![];
let name_path = Product::name_w();

for product_ref in &mut products_mut {  // product_ref is &&mut Product
    if let Some(name) = name_path.get_mut_ref(product_ref) {
        *name = format!("New {}", name);
    }
}
```

## Common Use Cases

### 1. Working with HashMap Values

```rust
use std::collections::HashMap;

let map: HashMap<u32, Product> = /* ... */;

// Collect references without cloning
let products: Vec<&Product> = map.values().collect();
let price_path = Product::price_r();

// Filter using keypaths
let affordable: Vec<&&Product> = products
    .iter()
    .filter(|p| price_path.get_ref(p).map_or(false, |&price| price < 100.0))
    .collect();
```

### 2. Avoiding Unnecessary Cloning

**❌ Without `get_ref` (requires cloning):**
```rust
let expensive: Vec<Product> = products
    .iter()
    .filter(|p| p.price > 100.0)
    .cloned()  // Creates copies!
    .collect();
```

**✓ With `get_ref` (no cloning):**
```rust
let expensive: Vec<&Product> = products
    .iter()
    .filter(|p| price_path.get_ref(p).map_or(false, |&price| price > 100.0))
    .collect();
```

### 3. Lock-Aware Patterns with Arc<RwLock<HashMap>>

```rust
use std::sync::Arc;
use parking_lot::RwLock;

let shared: Arc<RwLock<HashMap<u32, Product>>> = /* ... */;

// Read lock and collect references
let products: Vec<&Product> = shared.read().values().collect();

// Query without cloning the data
let results: Vec<&&Product> = products
    .iter()
    .filter(|p| category_path.get_ref(p).map_or(false, |c| c == "Electronics"))
    .collect();
```

### 4. Grouped Data Processing

```rust
let mut by_category: HashMap<String, Vec<&Product>> = HashMap::new();

for product_ref in &products {
    if let Some(category) = category_path.get_ref(product_ref) {
        by_category
            .entry(category.clone())
            .or_insert_with(Vec::new)
            .push(*product_ref);
    }
}
```

## Key Differences

| Scenario | Method to Use | Example |
|----------|---------------|---------|
| Owned data: `&Product` | `.get()` | `name_path.get(&product)` |
| Reference: `&&Product` | `.get_ref()` | `name_path.get_ref(&product_ref)` |
| Mutable owned: `&mut Product` | `.get_mut()` | `name_path.get_mut(&mut product)` |
| Mutable ref: `&&mut Product` | `.get_mut_ref()` | `name_path.get_mut_ref(&mut product_ref)` |

## Examples

### Complete Working Example

See [`examples/reference_keypaths.rs`](examples/reference_keypaths.rs) for a comprehensive demonstration including:

- Working with `Vec<&T>`
- HashMap reference values
- Filtering without cloning
- Nested references
- Performance comparisons

Run with:
```bash
cargo run --example reference_keypaths
```

### Test Suite

See [`key-paths-core/examples/reference_test.rs`](key-paths-core/examples/reference_test.rs) for detailed test cases covering:

- Basic `get_ref` functionality
- Nested references
- Failable keypaths with references
- Mutable reference access
- Value correctness verification

Run with:
```bash
cd key-paths-core && cargo run --example reference_test
```

## Performance Benefits

Using `get_ref()` with reference collections provides:

1. **Zero-cost Abstraction**: Compiles to direct field access
2. **No Cloning**: Work with borrowed data directly
3. **Memory Efficiency**: Avoid duplicate allocations
4. **Type Safety**: Compiler-verified reference handling

### Benchmark Example

```rust
// Without get_ref: O(n) clones
let filtered: Vec<Product> = products
    .iter()
    .filter(|p| p.price < 100.0)
    .cloned()  // Allocates n new objects
    .collect();

// With get_ref: Zero clones
let filtered: Vec<&Product> = products
    .iter()
    .filter(|p| price_path.get_ref(p).map_or(false, |&p| p < 100.0))
    .collect();  // Only allocates Vec of pointers
```

## Lifetime Semantics

The lifetime bound `'a: 'b` in `get_ref` means:
- `'a`: Lifetime of the keypath itself
- `'b`: Lifetime of the data being accessed
- Constraint: The keypath must live at least as long as the borrowed data

This ensures:
- No dangling references
- Safe access patterns
- Correct borrow checker validation

## Integration with Query Builders

The reference support integrates seamlessly with query builders:

```rust
use rust_queries_core::*;

let products: Vec<&Product> = hashmap.values().collect();

// Query builder with references
let results = products.lazy_query()
    .where_(|p| price_path.get_ref(&p).map_or(false, |&price| price < 100.0))
    .order_by(|p| category_path.get_ref(&p))
    .collect();
```

## Migration Guide

If you have existing code using cloning, here's how to migrate:

### Before (with cloning)
```rust
let data: Vec<SomeStruct> = hashmap.values().cloned().collect();
data.lazy_query()
    .where_(SomeStruct::field_r(), |x| /* ... */)
    .collect()
```

### After (with references)
```rust
let data: Vec<&SomeStruct> = hashmap.values().collect();
data.iter()
    .filter(|item| {
        SomeStruct::field_r().get_ref(item).map_or(false, |val| /* ... */)
    })
    .collect()
```

## Best Practices

1. **Use `get_ref()` when working with borrowed data**
   - HashMap values
   - Shared state (Arc/RwLock)
   - Temporary collections

2. **Use `get()` with owned data**
   - Direct struct instances
   - Vec<T> (owned)
   - Local variables

3. **Prefer references when possible**
   - Reduces memory allocation
   - Better cache locality
   - Faster for large structs

4. **Consider ownership needs**
   - Need to modify? Use owned or mutable references
   - Read-only queries? Use immutable references
   - Need to return data? Consider owned types

## Troubleshooting

### Error: "expected `T`, found `&T`"

**Problem:** Mixing `get()` with reference types

**Solution:** Use `get_ref()` instead:
```rust
// ❌ Wrong
name_path.get(&&product)

// ✓ Correct
name_path.get_ref(&product_ref)
```

### Error: "lifetime may not live long enough"

**Problem:** Keypath lifetime doesn't outlive the data

**Solution:** Ensure keypath is created before or with the same scope as data:
```rust
let name_path = Product::name_r();  // Create first
let products: Vec<&Product> = map.values().collect();
// Now use with get_ref
```

### Error: "cannot borrow as mutable"

**Problem:** Using `get_mut_ref()` with immutable references

**Solution:** Use `get_ref()` for immutable access or ensure you have mutable references:
```rust
// For reading only
name_path.get_ref(&product_ref)

// For writing (need &mut)
let mut product_mut_ref = &mut product;
name_path.get_mut_ref(&mut product_mut_ref)
```

## Future Enhancements

Potential future additions:

1. **Automatic deref coercion** for common patterns
2. **Iterator adapters** for seamless reference handling  
3. **Query builder integration** with automatic reference detection
4. **Macro support** for generating reference-aware methods

## Contributing

Found a bug or have a suggestion? Please file an issue or submit a PR!

## License

Same as rust-key-paths: MPL-2.0

