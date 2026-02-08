# Kp Derive Macro - Implementation Summary

## Overview

The `#[derive(Kp)]` macro has been extended to intelligently handle all wrapper types supported in the rust-keypaths library. It generates keypath methods that automatically unwrap, dereference, or access elements from various container types.

## Supported Types

### Basic Types
- Direct field access for primitives and structs
- Generated method: `field_name() -> KpType<'static, Root, FieldType>`

### Smart Container Handling

#### Option<T>
- Automatically unwraps `Option` to access inner `T`
- Returns `None` if the `Option` is `None`
- Example: `Option<String>` → keypath returns `Option<&String>`

#### Vec<T>
- Accesses the **first element** of the vector
- Returns `None` if vector is empty
- Supports mutable access via `first_mut()`
- Example: `Vec<Person>` → keypath returns `Option<&Person>` (first person)

#### Box<T>
- Automatically dereferences to access inner `T`
- Supports both read and mutable access
- Example: `Box<i32>` → keypath returns `Option<&i32>`

#### Arc<T> / Rc<T>
- Dereferences to access inner `T`
- **Read-only** - mutable access returns `None`
- Example: `Arc<String>` → keypath returns `Option<&String>`

#### HashMap<K, V> / BTreeMap<K, V>
- Returns keypath to the **container itself**
- Allows further operations on the map
- Example: `HashMap<String, i32>` → keypath returns `Option<&HashMap<String, i32>>`

#### HashSet<T> / BTreeSet<T>
- Accesses **any element** via `.iter().next()`
- **Read-only** - no mutable iteration support
- Example: `HashSet<String>` → keypath returns `Option<&String>`

#### VecDeque<T>
- Accesses the **front element**
- Supports mutable access via `front_mut()`
- Example: `VecDeque<i32>` → keypath returns `Option<&i32>`

#### LinkedList<T>
- Accesses the **front element**
- Supports mutable access via `front_mut()`
- Example: `LinkedList<i32>` → keypath returns `Option<&i32>`

#### BinaryHeap<T>
- Accesses the **top element** via `peek()`
- **Read-only** - no mutable peek support
- Example: `BinaryHeap<i32>` → keypath returns `Option<&i32>`

#### Result<T, E>
- Accesses the **Ok value**
- Returns `None` if the result is `Err`
- Supports mutable access
- Example: `Result<String, Error>` → keypath returns `Option<&String>`

#### Mutex<T> / RwLock<T>
- Returns keypath to the **container itself**
- Does not automatically lock - user must lock explicitly
- Supports both `std::sync` and `parking_lot` variants
- Example: `Mutex<i32>` → keypath returns `Option<&Mutex<i32>>`

#### Weak<T>
- Returns keypath to the **container itself**
- **Read-only** - no mutable access
- Example: `Weak<String>` → keypath returns `Option<&Weak<String>>`

### Enum Support

The derive macro now supports enums with different variant types:

#### Unit Variants
```rust
#[derive(Kp)]
enum Status {
    Active,
    Inactive,
}

// Generates:
// Status::active() -> checks if enum matches Active variant
```

#### Single-Field Tuple Variants
```rust
#[derive(Kp)]
enum Data {
    Text(String),
    Number(i32),
}

// Generates:
// Data::text() -> returns Option<&String> if variant is Text
// Data::number() -> returns Option<&i32> if variant is Number
```

Smart unwrapping applies to tuple variants too:
```rust
#[derive(Kp)]
enum OptionalData {
    Value(Option<String>),
}

// Generates:
// OptionalData::value() -> returns Option<&String> (unwraps the inner Option)
```

#### Multi-Field Tuple Variants
```rust
#[derive(Kp)]
enum Complex {
    Pair(i32, String),
}

// Generates:
// Complex::pair() -> returns Option<&Complex> (the variant itself)
```

#### Named Field Variants
```rust
#[derive(Kp)]
enum Point {
    TwoD { x: f64, y: f64 },
}

// Generates:
// Point::two_d() -> returns Option<&Point> (the variant itself)
```

### Tuple Struct Support

```rust
#[derive(Kp)]
struct Coords(f64, f64, Option<f64>);

// Generates:
// Coords::f0() -> access first field (f64)
// Coords::f1() -> access second field (f64)
// Coords::f2() -> access third field, unwrapping Option
```

## Usage Examples

### Basic Struct
```rust
#[derive(Kp)]
struct Person {
    name: String,
    age: i32,
}

let person = Person { name: "Alice".to_string(), age: 30 };
let name_kp = Person::name();
let name = name_kp.get(&person); // Some(&"Alice")
```

### With Option
```rust
#[derive(Kp)]
struct User {
    email: Option<String>,
}

let user = User { email: Some("user@example.com".to_string()) };
let email_kp = User::email();
let email = email_kp.get(&user); // Some(&"user@example.com")
```

### With Vec
```rust
#[derive(Kp)]
struct Company {
    employees: Vec<Person>,
}

let company = Company {
    employees: vec![
        Person { name: "Alice".to_string(), age: 30 },
        Person { name: "Bob".to_string(), age: 25 },
    ],
};

let employees_kp = Company::employees();
let first_employee = employees_kp.get(&company); // Some(&Person { name: "Alice", age: 30 })
```

### Mutable Access
```rust
let mut person = Person { name: "Bob".to_string(), age: 25 };
let age_kp = Person::age();

age_kp.get_mut(&mut person).map(|age| *age = 26);
assert_eq!(person.age, 26);
```

## Implementation Details

### Type Introspection
The macro uses `extract_wrapper_inner_type()` to detect wrapper types by examining the `syn::Type` structure. It identifies:
- Single-parameter generics (Option, Box, Vec, etc.)
- Two-parameter generics (HashMap, BTreeMap)
- Nested combinations (Option<Box<T>>, Arc<Mutex<T>>, etc.)
- Synchronization primitive variants (std::sync vs parking_lot)

### Code Generation Strategy
For each field:
1. **Analyze the field type** to determine wrapper kind
2. **Generate appropriate getter** that handles unwrapping/dereferencing
3. **Generate appropriate mutable getter** (or return `None` if not supported)
4. **Return `KpType<'static, Root, Value>`** where `Value` is the unwrapped type

### Naming Convention
- Struct named fields: use field name directly (`Person::name()`)
- Tuple struct fields: use `f0()`, `f1()`, `f2()`, etc.
- Enum variants: convert to snake_case (`MyVariant` → `my_variant()`)

## Testing

Comprehensive test coverage includes:
- ✅ Basic field access (5 tests)
- ✅ Wrapper type handling (18 tests)
  - Option (with Some and None)
  - Vec, Box, Arc, Rc
  - Result (with Ok and Err)
  - HashMap, BTreeMap, HashSet, BTreeSet
  - VecDeque, LinkedList, BinaryHeap
  - Mutex, RwLock
- ✅ Enum variants (unit, single-field, multi-field, named)
- ✅ Tuple structs
- ✅ Mutable access
- ✅ Type signatures

All 23 tests pass successfully.

## Comparison with Reference Implementation

The `Kp` derive macro implementation closely mirrors the reference `Keypath` derive from `keypaths-proc/src/lib.rs` (lines 4818-5353), with these key differences:

1. **Return Type**: Uses `KpType` instead of `OptionalKeyPath`
2. **API**: Single method per field instead of separate `_r()`, `_w()`, `_fr()`, `_fw()` methods
3. **Simplicity**: Combined read/write access in one keypath
4. **Composability**: Designed for keypath composition via `.then()` method

## Future Enhancements

Potential improvements:
- Support for nested wrapper combinations (Option<Vec<T>>, Arc<Mutex<T>>)
- Custom unwrapping strategies via attributes
- Index-based access for Vec/arrays (`#[kp(index)]`)
- Named field access for enum variants with named fields
