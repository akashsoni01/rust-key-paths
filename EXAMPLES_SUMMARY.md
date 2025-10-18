# Examples Summary - rust-key-paths

This document provides an overview of all the examples in this repository, organized by functionality and use case.

## 📚 Example Categories

### 1. **Form Processing & Validation**

#### `examples/user_form.rs`
**Purpose:** Basic form validation and processing using keypaths

**Features:**
- Type-safe form field definitions
- Custom validators per field
- Generic form processing
- Nested field updates (e.g., `settings.theme`)

**Run:** `cargo run --example user_form`

**Key Concepts:**
- FormField struct with keypaths
- Validator functions
- Generic `process_form()` function

---

#### `examples/form_binding.rs`
**Purpose:** Advanced UI field binding without hardcoded access patterns

**Features:**
- Two-way data binding (read & write)
- Multiple field types (String, bool, u32)
- Field-level validation
- Update fields by name
- Display current form state

**Run:** `cargo run --example form_binding`

**Key Concepts:**
- FormBinding system
- Type-specific field collections
- Validation before writes
- Field lookup by name

---

### 2. **State Management & Synchronization**

#### `examples/change_tracker.rs`
**Purpose:** Track and synchronize changes between states

**Features:**
- Detect changes between old and new state
- Serialize changes to JSON
- Apply changes from remote sources
- Bidirectional synchronization

**Run:** `cargo run --example change_tracker`

**Key Concepts:**
- ChangeTracker with read/write paths
- FieldChange serialization
- Deserialization and application
- State verification

---

#### `examples/undo_redo.rs`
**Purpose:** Implement undo/redo for deeply nested data structures

**Features:**
- Command pattern for changes
- Full undo/redo stack
- Change history display
- Multiple field type support
- State verification

**Run:** `cargo run --example undo_redo`

**Key Concepts:**
- ChangeCommand<T, F>
- UndoStack management
- Command descriptions
- Redo stack truncation

---

### 3. **Query Building & Data Access**

#### `examples/query_builder.rs`
**Purpose:** Basic dynamic query builder

**Features:**
- Filter collections using keypaths
- Chain multiple `where_()` predicates
- Count and execute queries
- Mutable query results

**Run:** `cargo run --example query_builder`

**Key Concepts:**
- Query<T> builder
- Filter predicates
- execute() and execute_mut()

---

#### `examples/advanced_query_builder.rs`
**Purpose:** Full-featured SQL-like query system

**Features:**
- SELECT (projection)
- ORDER BY (ascending/descending)
- GROUP BY with aggregations
- LIMIT and pagination
- Aggregates (count, sum, avg, min, max)
- EXISTS queries
- Complex multi-stage queries

**Run:** `cargo run --example advanced_query_builder`

**Key Concepts:**
- LazyQuery<T>
- Aggregation functions
- Float-specific sorting/aggregation
- Group-then-aggregate pattern

---

#### `examples/join_query_builder.rs`
**Purpose:** SQL-like JOIN operations between collections

**Features:**
- INNER JOIN
- LEFT JOIN
- Filtered joins (JOIN ... WHERE)
- Three-way joins
- Aggregated joins
- Self-joins

**Run:** `cargo run --example join_query_builder`

**Key Concepts:**
- JoinQuery<L, R>
- Hash-based indexing for O(n) joins
- Generic mapper functions
- Multi-table queries

---

### 4. **Container Support**

#### `examples/container_adapters.rs`
**Purpose:** Working with smart pointers (Arc, Box, Rc)

**Features:**
- `.for_arc()` adapter for `Vec<Arc<T>>`
- `.for_box()` adapter for `Vec<Box<T>>`
- `.for_rc()` adapter for `Vec<Rc<T>>`
- Filtering wrapped types
- Mutable access with Box
- Shared state patterns

**Run:** `cargo run --example container_adapters`

**Key Concepts:**
- Smart pointer adapters
- Zero-cost abstraction
- Immutable vs mutable containers

---

#### `examples/reference_keypaths.rs`
**Purpose:** Working with collections of references

**Features:**
- `.get_ref()` for `Vec<&T>`
- HashMap value references
- Nested references
- Performance comparison (owned vs references)
- Avoid cloning

**Run:** `cargo run --example reference_keypaths`

**Key Concepts:**
- Reference keypath access
- get_ref() and get_mut_ref()
- Zero-copy querying

---

#### `key-paths-core/examples/container_adapter_test.rs`
**Purpose:** Comprehensive test suite for container adapters

**Features:**
- 12 comprehensive tests
- Arc, Box, and Rc coverage
- Readable and writable paths
- Failable paths
- Value correctness verification

**Run:** `cd key-paths-core && cargo run --example container_adapter_test`

---

#### `key-paths-core/examples/reference_test.rs`
**Purpose:** Test suite for reference support

**Features:**
- 8 comprehensive tests
- get_ref() verification
- get_mut_ref() verification  
- Nested references
- Performance demonstration

**Run:** `cd key-paths-core && cargo run --example reference_test`

---

## 📊 Example Matrix

| Example | Forms | Queries | Joins | Undo/Redo | Sync | Containers | References |
|---------|-------|---------|-------|-----------|------|------------|------------|
| user_form | ✅ | - | - | - | - | - | - |
| form_binding | ✅ | - | - | - | - | - | - |
| change_tracker | - | - | - | - | ✅ | - | - |
| undo_redo | - | - | - | ✅ | - | - | - |
| query_builder | - | ✅ | - | - | - | - | - |
| advanced_query_builder | - | ✅ | - | - | - | - | - |
| join_query_builder | - | ✅ | ✅ | - | - | - | - |
| container_adapters | - | ✅ | - | - | - | ✅ | - |
| reference_keypaths | - | ✅ | - | - | - | - | ✅ |

---

## 🎯 Use Case Guide

### "I need to validate form inputs"
→ [`user_form.rs`](examples/user_form.rs) or [`form_binding.rs`](examples/form_binding.rs)

### "I need to track changes for synchronization"
→ [`change_tracker.rs`](examples/change_tracker.rs)

### "I need undo/redo functionality"
→ [`undo_redo.rs`](examples/undo_redo.rs)

### "I need to query in-memory data"
→ [`query_builder.rs`](examples/query_builder.rs) or [`advanced_query_builder.rs`](examples/advanced_query_builder.rs)

### "I need to join multiple collections"
→ [`join_query_builder.rs`](examples/join_query_builder.rs)

### "I have Vec<Arc<T>> or other smart pointers"
→ [`container_adapters.rs`](examples/container_adapters.rs)

### "I have Vec<&T> from HashMap.values()"
→ [`reference_keypaths.rs`](examples/reference_keypaths.rs)

---

## 🚀 Quick Start

### Run All Examples

```bash
# Form examples
cargo run --example user_form
cargo run --example form_binding

# State management
cargo run --example change_tracker
cargo run --example undo_redo

# Queries
cargo run --example query_builder
cargo run --example advanced_query_builder
cargo run --example join_query_builder

# Containers
cargo run --example container_adapters
cargo run --example reference_keypaths

# Tests
cd key-paths-core
cargo run --example container_adapter_test
cargo run --example reference_test
```

### Example Complexity Levels

**Beginner:**
1. `user_form.rs` - Simple form validation
2. `reference_keypaths.rs` - Reference support basics

**Intermediate:**
3. `query_builder.rs` - Basic queries
4. `change_tracker.rs` - State synchronization
5. `container_adapters.rs` - Smart pointers

**Advanced:**
6. `form_binding.rs` - Full binding system
7. `advanced_query_builder.rs` - SQL-like queries
8. `join_query_builder.rs` - Multi-table joins
9. `undo_redo.rs` - Command pattern

---

## 💡 Common Patterns

### Pattern 1: Form Validation
```rust
struct FormField<T, F> {
    path: KeyPaths<T, F>,
    validator: fn(&F) -> Result<(), String>,
}
```

### Pattern 2: Change Tracking
```rust
struct FieldChange {
    path: Vec<String>,
    old_value: String,
    new_value: String,
}
```

### Pattern 3: Query Building
```rust
Query::new(&data)
    .where_(field_path, predicate)
    .order_by(sort_path)
    .limit(10)
```

### Pattern 4: Container Adapters
```rust
let items: Vec<Arc<T>> = /* ... */;
let path = T::field_r().for_arc();

items.iter().filter(|item| {
    path.get(item).map_or(false, |val| /* predicate */)
})
```

---

## 📖 Learning Path

1. **Start with:** `user_form.rs` - Learn basic keypath usage
2. **Then try:** `query_builder.rs` - Understand filtering
3. **Next:** `container_adapters.rs` - Learn adapters
4. **Reference:** `reference_keypaths.rs` - Optimize with references
5. **Advanced:** `join_query_builder.rs` - Multi-collection queries
6. **Expert:** `undo_redo.rs` - Complex state management

---

## 🏆 Example Statistics

- **Total Examples:** 11 (9 in examples/, 2 in key-paths-core/examples/)
- **Lines of Code:** ~3,500+ across all examples
- **Use Cases Covered:** 20+
- **Container Types:** 5 (Arc, Box, Rc, &T, &mut T)
- **Query Operations:** 15+ (where, select, order, group, join, etc.)

---

## 🔧 Development Tools

### Expand Macros
```bash
cargo expand --example user_form
```

### Check a Specific Example
```bash
cargo check --example advanced_query_builder
```

### Run with Output Filter
```bash
cargo run --example join_query_builder 2>&1 | grep "Join"
```

---

## 📝 Contributing Examples

When adding new examples:

1. Add descriptive header comment with:
   - Purpose
   - Features demonstrated
   - Run command

2. Include diverse scenarios:
   - Happy path
   - Edge cases
   - Error handling

3. Use clear, descriptive names:
   - Variables: `user_orders` not `uo`
   - Functions: `create_profile_form` not `make_form`

4. Add assertions where appropriate

5. Run and verify:
   ```bash
   cargo run --example your_example
   cargo fmt --all
   ```

---

## 🎯 Quick Reference

| Need | Example | Method |
|------|---------|---------|
| Filter data | `query_builder.rs` | `where_()` |
| Sort data | `advanced_query_builder.rs` | `order_by()` |
| Join tables | `join_query_builder.rs` | `inner_join()` |
| Track changes | `change_tracker.rs` | `detect_changes()` |
| Undo/redo | `undo_redo.rs` | `undo()`, `redo()` |
| Validate forms | `form_binding.rs` | `validate_all()` |
| Use Arc<T> | `container_adapters.rs` | `.for_arc()` |
| Use &T | `reference_keypaths.rs` | `.get_ref()` |

---

## 📚 Related Documentation

- [`CONTAINER_ADAPTERS.md`](CONTAINER_ADAPTERS.md) - Complete container adapter guide
- [`REFERENCE_SUPPORT.md`](REFERENCE_SUPPORT.md) - Reference support documentation
- [`README.md`](README.md) - Main project documentation
- [`IMPLEMENTATION_SUMMARY.md`](IMPLEMENTATION_SUMMARY.md) - Implementation details

---

## 🎉 Summary

This collection of examples demonstrates the full power of rust-key-paths for:

- ✅ Type-safe data access
- ✅ Generic form processing
- ✅ State management patterns
- ✅ In-memory query engines
- ✅ Change tracking systems
- ✅ Smart pointer support
- ✅ Zero-copy operations

All examples are production-ready patterns that can be adapted for real-world applications!

