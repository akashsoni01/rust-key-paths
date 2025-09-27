# Edge Cases Review for Rust Key-Paths Library

## Current Status

### ✅ Working Cases
- Basic types: `String`, `i32`, `bool`
- Basic containers: `Option<T>`, `Vec<T>`, `Box<T>`
- Smart pointers: `Rc<T>`, `Arc<T>`

### ❌ Issues Found

#### 1. Type Mismatch Issues
- **Problem**: Macro generates `KeyPaths<Struct, str>` instead of `KeyPaths<Struct, String>`
- **Cause**: Incorrect type variable usage in macro generation
- **Impact**: Affects all container types with `String` inner type

#### 2. BTreeMap Generic Constraints
- **Problem**: `error[E0277]: the trait bound 'String: Borrow<K>' is not satisfied`
- **Cause**: Macro generates incorrect generic constraints for BTreeMap key access
- **Impact**: BTreeMap and BTreeSet keypath generation fails

#### 3. Sized Trait Issues
- **Problem**: `error[E0277]: the size for values of type 'str' cannot be known at compilation time`
- **Cause**: Macro tries to use `str` instead of `String` in failable keypaths
- **Impact**: All failable keypaths with `String` inner type fail

#### 4. Nested Container Issues
- **Problem**: `Box<Option<T>>` generates wrong return types
- **Cause**: Macro not correctly handling nested container combinations
- **Impact**: All nested combinations fail

### 🔧 Container Types Support Status

| Container Type | Status | Issues |
|----------------|--------|---------|
| `Option<T>` | ✅ Working | None |
| `Vec<T>` | ✅ Working | None |
| `Box<T>` | ✅ Working | None |
| `Rc<T>` | ✅ Working | None |
| `Arc<T>` | ✅ Working | None |
| `HashSet<T>` | ❌ Failing | Type mismatch (`str` vs `String`) |
| `BTreeSet<T>` | ❌ Failing | Type mismatch (`str` vs `String`) |
| `VecDeque<T>` | ❌ Failing | Type mismatch (`str` vs `String`) |
| `LinkedList<T>` | ❌ Failing | Type mismatch (`str` vs `String`) |
| `BinaryHeap<T>` | ❌ Failing | Type mismatch (`str` vs `String`) |
| `HashMap<K, V>` | ❌ Failing | Type mismatch (`str` vs `String`) |
| `BTreeMap<K, V>` | ❌ Failing | Generic constraint issues |

### 🔧 Nested Combinations Status

| Combination | Status | Issues |
|-------------|--------|---------|
| `Option<Box<T>>` | ❌ Failing | Type mismatch |
| `Box<Option<T>>` | ❌ Failing | Type mismatch |
| `Option<Vec<T>>` | ❌ Failing | Type mismatch |
| `Vec<Option<T>>` | ❌ Failing | Type mismatch |
| `Rc<Option<T>>` | ❌ Failing | Type mismatch |
| `Arc<Option<T>>` | ❌ Failing | Type mismatch |

## Root Cause Analysis

The main issues stem from:

1. **Incorrect Type Variable Usage**: The macro is using wrong type variables in return types
2. **Generic Constraint Issues**: BTreeMap key access requires proper generic constraints
3. **Nested Type Handling**: The recursive type extraction is not working correctly for nested combinations

## Recommended Fixes

### 1. Fix Type Variable Usage
- Ensure `#ty` refers to the full field type (e.g., `Box<Option<String>>`)
- Ensure `#inner_ty` refers to the extracted inner type (e.g., `String`)
- Fix return types in all macro cases

### 2. Fix BTreeMap Constraints
- Add proper generic constraints for BTreeMap key access
- Use correct key types in generated methods

### 3. Fix Nested Combinations
- Debug the `extract_wrapper_inner_type` function
- Ensure nested combinations are correctly detected and handled

### 4. Add Comprehensive Tests
- Create tests for all container types
- Create tests for all nested combinations
- Add error case tests

## Test Coverage Needed

### Basic Container Tests
- [ ] `HashSet<T>` - failable access to elements
- [ ] `BTreeSet<T>` - failable access to elements  
- [ ] `VecDeque<T>` - indexed access
- [ ] `LinkedList<T>` - indexed access
- [ ] `BinaryHeap<T>` - peek access
- [ ] `HashMap<K, V>` - key-based access
- [ ] `BTreeMap<K, V>` - key-based access

### Nested Combination Tests
- [ ] `Option<Box<T>>` - all method types
- [ ] `Box<Option<T>>` - all method types
- [ ] `Option<Vec<T>>` - all method types
- [ ] `Vec<Option<T>>` - all method types
- [ ] `Rc<Option<T>>` - all method types
- [ ] `Arc<Option<T>>` - all method types

### Edge Case Tests
- [ ] Empty containers
- [ ] Single element containers
- [ ] Large containers
- [ ] Complex nested structures
- [ ] Error conditions
- [ ] Composition between different keypath types

## Conclusion

The library has good foundation support for basic containers but needs fixes for:
1. Type variable usage in macro generation
2. Generic constraints for map types
3. Nested container combinations
4. Comprehensive test coverage

Once these issues are resolved, the library will provide comprehensive support for all Rust standard library container types and their combinations.
