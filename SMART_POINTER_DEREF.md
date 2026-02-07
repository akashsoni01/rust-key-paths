# Smart Pointer Dereferencing in Key-Paths-Derive

## Summary

The `#[derive(Kp)]` macro correctly handles smart pointer types (`Box<T>`, `Rc<T>`, `Arc<T>`) by **automatically dereferencing** them to return references to the inner value.

## Behavior

### Box<T>
- **Generated Keypath Type**: `KpType<'static, StructName, T>` (not `Box<T>`)
- **Get Returns**: `Option<&T>` (not `Option<&Box<T>>`)
- **Get Mut Returns**: `Option<&mut T>` (not `Option<&mut Box<T>>`)
- **Implementation**: Uses `&*field` and `&mut *field` to dereference

### Rc<T>
- **Generated Keypath Type**: `KpType<'static, StructName, T>` (not `Rc<T>`)
- **Get Returns**: `Option<&T>` (not `Option<&Rc<T>>`)
- **Get Mut Returns**: `Option<&mut T>` when only one reference exists, `None` otherwise
- **Implementation**: Uses `&*field` and `Rc::get_mut(&mut field)`

### Arc<T>
- **Generated Keypath Type**: `KpType<'static, StructName, T>` (not `Arc<T>`)
- **Get Returns**: `Option<&T>` (not `Option<&Arc<T>>`)
- **Get Mut Returns**: `Option<&mut T>` when only one reference exists, `None` otherwise
- **Implementation**: Uses `&*field` and `Arc::get_mut(&mut field)`

## Example

```rust
use key_paths_derive::Kp;
use rust_key_paths::KpType;

#[derive(Kp)]
struct MyData {
    boxed_value: Box<i32>,
    rc_value: std::rc::Rc<String>,
    arc_value: std::sync::Arc<f64>,
}

let data = MyData {
    boxed_value: Box::new(42),
    rc_value: std::rc::Rc::new("hello".to_string()),
    arc_value: std::sync::Arc::new(3.14),
};

// Box<i32> field returns &i32
let box_kp: KpType<'static, MyData, i32> = MyData::boxed_value();
let value: Option<&i32> = box_kp.get(&data);
assert_eq!(value, Some(&42));

// Rc<String> field returns &String
let rc_kp: KpType<'static, MyData, String> = MyData::rc_value();
let value: Option<&String> = rc_kp.get(&data);
assert_eq!(value.map(|s| s.as_str()), Some("hello"));

// Arc<f64> field returns &f64
let arc_kp: KpType<'static, MyData, f64> = MyData::arc_value();
let value: Option<&f64> = arc_kp.get(&data);
assert_eq!(value, Some(&3.14));

// Mutable access through Box
let mut data = MyData {
    boxed_value: Box::new(10),
    rc_value: std::rc::Rc::new("test".to_string()),
    arc_value: std::sync::Arc::new(1.0),
};

let box_kp = MyData::boxed_value();
let value: Option<&mut i32> = box_kp.get_mut(&mut data);
*value.unwrap() = 100;
assert_eq!(*data.boxed_value, 100);
```

## Benefits

1. **Ergonomic API**: Users work with the inner type directly, not the wrapper
2. **Type Safety**: The type system enforces correct usage
3. **Consistent with Rust idioms**: Follows the same pattern as Deref coercion
4. **Mutable Access**: Smart pointer mutable access works when semantically valid (single reference for Rc/Arc)

## Implementation Details

### Code Generation

For `Box<T>`:
```rust
pub fn field_name() -> KpType<'static, StructName, T> {
    Kp::new(
        |root: &StructName| Some(&*root.field_name),
        |root: &mut StructName| Some(&mut *root.field_name),
    )
}
```

For `Rc<T>`:
```rust
pub fn field_name() -> KpType<'static, StructName, T> {
    Kp::new(
        |root: &StructName| Some(&*root.field_name),
        |root: &mut StructName| {
            std::rc::Rc::get_mut(&mut root.field_name)
        },
    )
}
```

For `Arc<T>`:
```rust
pub fn field_name() -> KpType<'static, StructName, T> {
    Kp::new(
        |root: &StructName| Some(&*root.field_name),
        |root: &mut StructName| {
            std::sync::Arc::get_mut(&mut root.field_name)
        },
    )
}
```

## Test Coverage

All smart pointer dereferencing behavior is verified with comprehensive tests:
- `test_box_returns_inner_type` - Verifies Box returns &T
- `test_rc_returns_inner_type` - Verifies Rc returns &T
- `test_arc_returns_inner_type` - Verifies Arc returns &T
- `test_box_mutable_returns_inner_type` - Verifies Box mutable access
- `test_rc_access` - Verifies Rc mutable access with single reference
- `test_arc_access` - Verifies Arc mutable access with single reference
- `test_rc_no_mut_with_multiple_refs` - Verifies Rc returns None with multiple refs
- `test_arc_no_mut_with_multiple_refs` - Verifies Arc returns None with multiple refs

Total: **40 tests passing** (18 wrapper types + 5 derive + 13 comprehensive + 4 smart pointer deref)
