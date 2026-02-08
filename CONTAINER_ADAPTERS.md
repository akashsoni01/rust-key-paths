# Container Adapters for rust-key-paths

This document explains how to use keypaths with smart pointers and container types (`Arc<T>`, `Box<T>`, `Rc<T>`) using the adapter methods.

## Problem Statement

When working with collections of wrapped types (e.g., `Vec<Arc<Product>>`), the standard keypaths don't work directly because they expect the unwrapped type:

```rust
let products: Vec<Arc<Product>> = vec![Arc::new(Product { /* ... */ })];
let price_path = Product::price_r();  // Returns KeyPaths<Product, f64>

// This doesn't work! price_path expects &Product but product is &Arc<Product>
for product in &products {
    price_path.get(product);  // Type mismatch!
}
```

## Solution: Adapter Methods

The `key-paths-core` library provides three adapter methods that create new keypaths for wrapped types:

### 1. `for_arc()` - Arc<T> Adapter

Adapts a `KeyPaths<T, V>` to work with `Arc<T>`:

```rust
pub fn for_arc(self) -> KeyPaths<Arc<Root>, Value>
```

**Usage:**
```rust
let products: Vec<Arc<Product>> = /* ... */;
let price_path = Product::price_r().for_arc();

for product in &products {
    if let Some(&price) = price_path.get(product) {
        println!("Price: ${}", price);
    }
}
```

**Supported KeyPath Types:**
- ✅ Readable
- ✅ FailableReadable  
- ✅ ReadableEnum
- ❌ Writable (Arc is immutable)
- ❌ FailableWritable (Arc is immutable)

### 2. `for_box()` - Box<T> Adapter

Adapts a `KeyPaths<T, V>` to work with `Box<T>`:

```rust
pub fn for_box(self) -> KeyPaths<Box<Root>, Value>
```

**Usage:**
```rust
let users: Vec<Box<User>> = /* ... */;
let name_path = User::name_r().for_box();
let name_path_w = User::name_w().for_box();  // Writable works with Box!

// Read access
for user in &users {
    if let Some(name) = name_path.get(user) {
        println!("Name: {}", name);
    }
}

// Write access
let mut users_mut = users;
if let Some(user) = users_mut.get_mut(0) {
    if let Some(name) = name_path_w.get_mut(user) {
        *name = "New Name".to_string();
    }
}
```

**Supported KeyPath Types:**
- ✅ Readable
- ✅ Writable
- ✅ FailableReadable
- ✅ FailableWritable
- ✅ ReadableEnum
- ✅ WritableEnum

### 3. `for_rc()` - Rc<T> Adapter

Adapts a `KeyPaths<T, V>` to work with `Rc<T>`:

```rust
pub fn for_rc(self) -> KeyPaths<Rc<Root>, Value>
```

**Usage:**
```rust
let products: Vec<Rc<Product>> = /* ... */;
let category_path = Product::category_r().for_rc();

for product in &products {
    if let Some(category) = category_path.get(product) {
        println!("Category: {}", category);
    }
}
```

**Supported KeyPath Types:**
- ✅ Readable
- ✅ FailableReadable
- ✅ ReadableEnum
- ❌ Writable (Rc is immutable)
- ❌ FailableWritable (Rc is immutable)

## Common Patterns

### Pattern 1: Filtering Wrapped Collections

```rust
let products: Vec<Arc<Product>> = /* ... */;
let price_path = Product::price_r().for_arc();
let in_stock_path = Product::in_stock_r().for_arc();

let affordable: Vec<&Arc<Product>> = products
    .iter()
    .filter(|p| {
        price_path.get(p).map_or(false, |&price| price < 100.0)
            && in_stock_path.get(p).map_or(false, |&stock| stock)
    })
    .collect();
```

### Pattern 2: Grouping by Field

```rust
use std::collections::HashMap;

let products: Vec<Arc<Product>> = /* ... */;
let category_path = Product::category_r().for_arc();

let mut by_category: HashMap<String, Vec<Arc<Product>>> = HashMap::new();

for product in products {
    if let Some(category) = category_path.get(&product) {
        by_category
            .entry(category.clone())
            .or_insert_with(Vec::new)
            .push(product);
    }
}
```

### Pattern 3: Shared State with Arc

```rust
use std::sync::Arc;

// Common pattern: Vec<Arc<T>> for shared ownership
let shared_data: Vec<Arc<Product>> = vec![
    Arc::new(Product { /* ... */ }),
];

// Clone Arcs (cheap - just increments reference count)
let thread1_data = shared_data.clone();
let thread2_data = shared_data.clone();

// Both threads can query using adapted keypaths
let price_path = Product::price_r().for_arc();

// Thread 1
let expensive = thread1_data
    .iter()
    .filter(|p| price_path.get(p).map_or(false, |&p| p > 100.0))
    .collect::<Vec<_>>();

// Thread 2 sees the same data
let total: f64 = thread2_data
    .iter()
    .filter_map(|p| price_path.get(p).copied())
    .sum();
```

### Pattern 4: Mutable Access with Box

```rust
let mut users: Vec<Box<User>> = /* ... */;
let age_path = User::age_w().for_box();

// Increment everyone's age
for user in &mut users {
    if let Some(age) = age_path.get_mut(user) {
        *age += 1;
    }
}
```

### Pattern 5: HashMap Values

```rust
use std::collections::HashMap;
use std::sync::Arc;

let map: HashMap<u32, Arc<Product>> = /* ... */;

// Collect Arc references
let products: Vec<Arc<Product>> = map.values().cloned().collect();

// Query with adapted keypath
let price_path = Product::price_r().for_arc();
let results: Vec<&Arc<Product>> = products
    .iter()
    .filter(|p| price_path.get(p).map_or(false, |&price| price < 200.0))
    .collect();
```

## Adapter Chaining

Adapters can be chained for nested containers (though this is uncommon):

```rust
// Vec<Arc<Box<Product>>> (unusual but possible)
let nested: Vec<Arc<Box<Product>>> = /* ... */;

// Chain adapters: Product -> Box<Product> -> Arc<Box<Product>>
let price_path = Product::price_r()
    .for_box()
    .for_arc();

for item in &nested {
    if let Some(&price) = price_path.get(item) {
        println!("Price: ${}", price);
    }
}
```

## Composition with Adapters

Adapted keypaths work with composition:

```rust
#[derive(Kp)]
struct Order {
    product: Product,
    quantity: u32,
}

let orders: Vec<Arc<Order>> = /* ... */;

// Compose adapted keypaths
let product_price_path = Order::product_r()
    .then(Product::price_r())
    .for_arc();

for order in &orders {
    if let Some(&price) = product_price_path.get(order) {
        println!("Product price: ${}", price);
    }
}
```

## Performance Characteristics

### Zero-Cost Abstraction

The adapter methods create new keypaths that dereference the wrapper at access time. The Rust compiler optimizes this to direct field access:

```rust
// This:
let price = price_path_arc.get(&product);

// Compiles to the same code as:
let price = &(**product).price;
```

### Memory Overhead

- **Arc/Rc Cloning**: O(1) - Just increments reference count
- **Box Moving**: O(1) - Moves pointer, not data
- **Adapter Creation**: O(1) - Creates new keypath wrapper

### Reference Counting

With `Arc` and `Rc`, be aware of reference counting implications:

```rust
let products: Vec<Arc<Product>> = /* ... */;

// Cloning products doesn't clone Product data
let clone = products.clone();  // Fast: just increments refcounts

// But keeping references prevents deallocation
let forever = products[0].clone();  // Product won't be freed while `forever` exists
```

## Common Pitfalls & Solutions

### Pitfall 1: Trying to Mutate Through Arc/Rc

**Problem:**
```rust
let data: Vec<Arc<Product>> = /* ... */;
let price_path = Product::price_w().for_arc();  // Panics!
```

**Solution:** Use `Box` for mutable access, or use interior mutability (`Mutex`, `RwLock`):
```rust
// Option 1: Use Box instead
let data: Vec<Box<Product>> = /* ... */;
let price_path = Product::price_w().for_box();  // Works!

// Option 2: Use Arc with interior mutability
use parking_lot::RwLock;
let data: Vec<Arc<RwLock<Product>>> = /* ... */;
// Access through RwLock
```

### Pitfall 2: Forgetting to Adapt

**Problem:**
```rust
let products: Vec<Arc<Product>> = /* ... */;
let price_path = Product::price_r();  // Not adapted!

for product in &products {
    price_path.get(product);  // Type error!
}
```

**Solution:** Always use the adapter:
```rust
let price_path = Product::price_r().for_arc();  // ✓ Adapted
```

### Pitfall 3: Adapter Order Matters

**Problem:**
```rust
// Wrong order: composing THEN adapting
let path = Order::product_r()
    .for_arc()  // Too early!
    .then(Product::price_r());  // Type mismatch
```

**Solution:** Compose first, adapt last:
```rust
// Correct: compose THEN adapt
let path = Order::product_r()
    .then(Product::price_r())
    .for_arc();  // ✓ Correct
```

## Integration with Query Builders

If you're using a query builder library, adapted keypaths work seamlessly:

```rust
let products: Vec<Arc<Product>> = /* ... */;

// Create adapted keypaths once
let price_path = Product::price_r().for_arc();
let category_path = Product::category_r().for_arc();

// Use in queries
let results = products
    .iter()
    .filter(|p| {
        price_path.get(p).map_or(false, |&price| price > 100.0)
            && category_path.get(p).map_or(false, |cat| cat == "Electronics")
    })
    .collect();
```

## Best Practices

1. **Create Adapted Keypaths Once**
   ```rust
   // Good: Create once, use many times
   let price_path = Product::price_r().for_arc();
   for product in &products {
       price_path.get(product);
   }
   
   // Bad: Creating adapter in loop
   for product in &products {
       Product::price_r().for_arc().get(product);  // Wasteful
   }
   ```

2. **Choose the Right Container**
   - `Arc<T>`: Shared ownership, thread-safe, immutable
   - `Rc<T>`: Shared ownership, single-threaded, immutable
   - `Box<T>`: Unique ownership, mutable

3. **Prefer Arc for Shared Data**
   ```rust
   // Good: Arc for sharing between threads/owners
   let data: Vec<Arc<Product>> = /* ... */;
   let clone = data.clone();  // Cheap
   
   // Bad: Box requires actual cloning
   let data: Vec<Box<Product>> = /* ... */;
   let clone = data.clone();  // Expensive if Product is large
   ```

4. **Use get_ref() for Reference Collections**
   ```rust
   // If you have Vec<&Arc<T>>, combine with get_ref:
   let refs: Vec<&Arc<Product>> = /* ... */;
   let price_path = Product::price_r().for_arc();
   
   for product_ref in &refs {
       price_path.get_ref(product_ref);
   }
   ```

## Examples

See [`examples/container_adapters.rs`](examples/container_adapters.rs) for comprehensive examples including:

- Vec<Arc<T>> with filtering
- Vec<Box<T>> with mutable access
- Vec<Rc<T>> with grouping
- Shared state patterns
- Complex filtering scenarios
- Mixed container types

Run with:
```bash
cargo run --example container_adapters
```

## API Reference

### for_arc()

```rust
pub fn for_arc(self) -> KeyPaths<Arc<Root>, Value>
where
    Root: 'static,
    Value: 'static
```

Adapts this keypath to work with `Arc<Root>`.

**Panics:** If called on a Writable or FailableWritable keypath (Arc is immutable).

### for_box()

```rust
pub fn for_box(self) -> KeyPaths<Box<Root>, Value>
where
    Root: 'static,
    Value: 'static
```

Adapts this keypath to work with `Box<Root>`. Supports both readable and writable access.

### for_rc()

```rust
pub fn for_rc(self) -> KeyPaths<Rc<Root>, Value>
where
    Root: 'static,
    Value: 'static
```

Adapts this keypath to work with `Rc<Root>`.

**Panics:** If called on a Writable or FailableWritable keypath (Rc is immutable).

## Future Enhancements

Potential future additions:

1. **Generic Deref Adapter**: Single `.for_deref()` method that works with any `Deref` type
2. **Nested Container Support**: Automatic multi-level dereferencing
3. **Interior Mutability Support**: `.for_arc_mutex()`, `.for_arc_rwlock()`
4. **Macro Helpers**: `arc_path!(Product::price_r)` shorthand

## Troubleshooting

### Error: "Cannot create writable keypath for Arc"

**Problem:** Trying to use `.for_arc()` with a writable keypath

**Solution:** Arc is immutable. Use `Box` or `Rc<RefCell<T>>` for mutation

### Error: "expected Arc<T>, found T"

**Problem:** Forgot to adapt the keypath

**Solution:** Add `.for_arc()` after creating the keypath

### Error: "method `for_arc` not found"

**Problem:** Using an older version of key-paths-core

**Solution:** Update to key-paths-core >= 1.0.2

## Contributing

Found a bug or have a suggestion? Please file an issue or submit a PR!

## License

Same as rust-key-paths: MPL-2.0

