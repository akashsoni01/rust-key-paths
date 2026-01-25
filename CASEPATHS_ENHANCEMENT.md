# Casepaths Macro Enhancement

## Overview

The `Casepaths` derive macro has been updated to work with the new `rust-keypaths` API, providing automatic generation of enum variant keypaths.

## Generated Methods

For each enum variant, the macro generates:

### Single-Field Variants
```rust
enum MyEnum {
    Variant(InnerType),
}

// Generated methods:
MyEnum::variant_fr() -> OptionalKeyPath<MyEnum, InnerType, ...>
MyEnum::variant_fw() -> WritableOptionalKeyPath<MyEnum, InnerType, ...>
```

### Multi-Field Tuple Variants
```rust
enum MyEnum {
    Variant(T1, T2, T3),
}

// Generated methods:
MyEnum::variant_fr() -> OptionalKeyPath<MyEnum, (T1, T2, T3), ...>
MyEnum::variant_fw() -> WritableOptionalKeyPath<MyEnum, (T1, T2, T3), ...>
```

### Named Field Variants
```rust
enum MyEnum {
    Variant { field1: T1, field2: T2 },
}

// Generated methods:
MyEnum::variant_fr() -> OptionalKeyPath<MyEnum, (T1, T2), ...>
MyEnum::variant_fw() -> WritableOptionalKeyPath<MyEnum, (T1, T2), ...>
```

### Unit Variants
```rust
enum MyEnum {
    Variant,
}

// Generated methods:
MyEnum::variant_fr() -> OptionalKeyPath<MyEnum, (), ...>
```

## Attribute Support

The macro supports the same attributes as `Keypaths`:

- `#[Readable]` - Generate only readable methods (`_fr()`)
- `#[Writable]` - Generate only writable methods (`_fw()`)
- `#[All]` - Generate both readable and writable methods (default)

### Example

```rust
#[derive(Casepaths)]
#[Writable]  // Generate only writable methods
enum MyEnum {
    A(String),
    B(Box<InnerStruct>),
}

// Usage:
let path = MyEnum::b_fw()  // Returns WritableOptionalKeyPath
    .for_box()                   // Unwrap Box<InnerStruct>
    .then(InnerStruct::field_fw());
```

## Key Features

1. **Type Safety**: Returns `OptionalKeyPath`/`WritableOptionalKeyPath` since variant extraction may fail
2. **Container Support**: Works seamlessly with `Box<T>`, `Arc<T>`, `Rc<T>` via `.for_box()`, `.for_arc()`, `.for_rc()`
3. **Chaining**: Can be chained with `.then()` for nested access
4. **Attribute-Based Control**: Use `#[Readable]`, `#[Writable]`, or `#[All]` to control which methods are generated

## Migration from Old API

### Old API (key-paths-core)
```rust
#[derive(Casepaths)]
enum MyEnum {
    Variant(InnerType),
}

// Generated methods returned KeyPaths enum
let path = MyEnum::variant_r();
```

### New API (rust-keypaths)
```rust
#[derive(Casepaths)]
#[Writable]
enum MyEnum {
    Variant(InnerType),
}

// Generated methods return specific types
let path = MyEnum::variant_fw();  // Returns WritableOptionalKeyPath
```

## Example: Deep Nesting with Box

```rust
#[derive(Kp)]
#[Writable]
struct Outer {
    inner: Option<MyEnum>,
}

#[derive(Casepaths)]
#[Writable]
enum MyEnum {
    B(Box<InnerStruct>),
}

#[derive(Kp)]
#[Writable]
struct InnerStruct {
    field: Option<String>,
}

// Chain through Option -> Enum variant -> Box -> Option
let path = Outer::inner_fw()
    .then(MyEnum::b_fw())  // Extract variant
    .for_box()                   // Unwrap Box
    .then(InnerStruct::field_fw());

// Use it
if let Some(value) = path.get_mut(&mut instance) {
    *value = "new value".to_string();
}
```

## Implementation Details

- **Variant Extraction**: Uses pattern matching to safely extract variant values
- **Type Inference**: Automatically handles all variant field types
- **Error Handling**: Returns `None` if the enum is not the expected variant
- **Zero-Cost**: Compiles to direct pattern matching, no runtime overhead

