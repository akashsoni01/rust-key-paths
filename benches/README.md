# KeyPaths Performance Benchmarks

This directory contains comprehensive benchmarks comparing the performance of KeyPaths versus direct nested unwraps.

## Running Benchmarks

### Quick Run
```bash
# Benchmark nested Option access
cargo bench --bench keypath_vs_unwrap

# Benchmark RwLock write operations with deeply nested structures
cargo bench --bench rwlock_write_deeply_nested
```

### Using the Script
```bash
./benches/run_benchmarks.sh
```

## Benchmark Suites

### 1. Read Nested Option (`read_nested_option`)
Compares reading through nested `Option` types:
- **Keypath**: `SomeComplexStruct::scsf_fw().then(...).then(...).get()`
- **Direct**: `instance.scsf.as_ref().and_then(...).and_then(...)`

### 2. Write Nested Option (`write_nested_option`)
Compares writing through nested `Option` types:
- **Keypath**: `keypath.get_mut(&mut instance)`
- **Direct**: Multiple nested `if let Some(...)` statements

### 3. Deep Nested with Enum (`deep_nested_with_enum`)
Compares deep nested access including enum case paths:
- **Keypath**: Includes `SomeEnum::b_w()` and `for_box()` adapter
- **Direct**: Pattern matching on enum variants

### 4. Write Deep Nested with Enum (`write_deep_nested_with_enum`)
Compares writing through deep nested structures with enums:
- **Keypath**: Full composition chain with enum case path
- **Direct**: Nested pattern matching and unwraps

### 5. Keypath Creation (`keypath_creation`)
Measures the overhead of creating composed keypaths:
- Tests the cost of chaining multiple keypaths together

### 6. Keypath Reuse (`keypath_reuse`)
Compares performance when reusing the same keypath vs repeated unwraps:
- **Keypath**: Single keypath reused across 100 instances
- **Direct**: Repeated unwrap chains for each instance

### 7. Composition Overhead (`composition_overhead`)
Compares pre-composed vs on-the-fly composition:
- **Pre-composed**: Keypath created once, reused
- **Composed on-fly**: Keypath created in each iteration

### 8. RwLock Write Deeply Nested (`rwlock_write_deeply_nested`)
**Use Case**: Demonstrates updating deeply nested values inside `Arc<RwLock<T>>` structures.

This benchmark is particularly useful for scenarios where you need to:
- Update nested fields in thread-safe shared data structures
- Avoid manual write guard management and nested unwraps
- Maintain type safety when accessing deeply nested Option fields

**Example Structure**:
```rust
SomeStruct {
    f1: Arc<RwLock<SomeOtherStruct>>  // Thread-safe shared data
        -> SomeOtherStruct {
            f4: DeeplyNestedStruct {
                f1: Option<String>  // Deeply nested field to update
            }
        }
}
```

**Keypath Approach**:
```rust
use keypaths_proc::Keypaths;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Keypaths)]
#[Writable]
struct SomeStruct {
    f1: Arc<RwLock<SomeOtherStruct>>,
}

// Compose keypath: SomeStruct -> Arc<RwLock<...>> -> SomeOtherStruct -> DeeplyNestedStruct -> f1
let keypath = SomeStruct::f1_fw_at(
    SomeOtherStruct::f4_w()
        .then(DeeplyNestedStruct::f1_w())
);

// Use keypath to update value
keypath.get_mut(&instance, |value| {
    *value = Some(String::from("new value"));
});
```

**Traditional Approach**:
```rust
// Manual write guard and nested unwraps
let mut guard = instance.f1.write();
if let Some(ref mut f1) = guard.f4.f1 {
    *f1 = String::from("new value");
}
```

**Benchmark Variants**:
- `rwlock_write_deeply_nested`: Write to `f1` (Option<String>) 3 levels deep
- `rwlock_write_deeply_nested_f2`: Write to `f2` (Option<i32>) 3 levels deep
- `rwlock_write_f3`: Write to `f3` (Option<String>) 2 levels deep
- `rwlock_multiple_writes`: Multiple sequential writes using keypath vs single/multiple write guards

**Benchmark Results**:

### parking_lot::RwLock Benchmarks

| Benchmark | Approach | Mean Time | Comparison |
|-----------|----------|-----------|------------|
| `rwlock_write_deeply_nested` (String, 3 levels) | Keypath | 24.5 ns | 2.5% slower |
| | Write Guard | 23.9 ns | baseline |
| | Write Guard (nested) | 23.8 ns | **0.4% faster** |
| `rwlock_write_deeply_nested_f2` (i32, 3 levels) | Keypath | 8.5 ns | **1.2% faster** ⚡ |
| | Write Guard | 8.6 ns | baseline |
| `rwlock_write_f3` (String, 2 levels) | Keypath | 23.8 ns | **0.4% faster** ⚡ |
| | Write Guard | 23.9 ns | baseline |
| `rwlock_multiple_writes` (sequential) | Keypath | 55.8 ns | 33.5% slower |
| | Write Guard (single) | 41.8 ns | baseline |
| | Write Guard (multiple) | 56.2 ns | 34.4% slower |

### tokio::sync::RwLock Benchmarks

| Benchmark | Approach | Mean Time | Comparison |
|-----------|----------|-----------|------------|
| `tokio_rwlock_read_deeply_nested` (String, 3 levels) | Keypath | 104.8 ns | 0.2% slower |
| | Read Guard | 104.6 ns | baseline |
| `tokio_rwlock_write_deeply_nested` (String, 3 levels) | Keypath | 124.8 ns | 0.6% slower |
| | Write Guard | 124.1 ns | baseline |
| `tokio_rwlock_write_deeply_nested_f2` (i32, 3 levels) | Keypath | 103.8 ns | **1.2% faster** ⚡ |
| | Write Guard | 105.0 ns | baseline |
| `tokio_rwlock_read_f3` (String, 2 levels) | Keypath | 103.3 ns | 0.1% slower |
| | Read Guard | 103.2 ns | baseline |
| `tokio_rwlock_write_f3` (String, 2 levels) | Keypath | 125.7 ns | 0.9% slower |
| | Write Guard | 124.6 ns | baseline |

**Key Findings**:

**parking_lot::RwLock:**
- ✅ For single write operations, keypath approach is **essentially identical** (0-2.5% overhead) to manual write guards
- ✅ Simple field writes can be **1.2% faster** with keypaths
- ⚠️ For multiple sequential writes, using a single write guard is more efficient (33% faster) than creating multiple keypaths
- ✅ Keypath approach performs similarly to multiple write guards when doing multiple operations

**tokio::sync::RwLock:**
- ✅ For async read operations, keypath approach shows **essentially identical performance** (0-0.2% overhead)
- ✅ For async write operations, keypath approach shows **essentially identical performance** (0-1% overhead)
- ✅ Simple async field operations can be **1.2% faster** with keypaths
- ✅ Async operations maintain similar performance characteristics to synchronous operations

**Overall:**
- ✅ The performance difference is negligible (sub-nanosecond) for most use cases
- ✅ Keypath approach provides significant benefits in type safety, composability, and maintainability with minimal performance cost
- ✅ Both synchronous (`parking_lot`) and asynchronous (`tokio`) primitives show excellent performance with keypaths

**Benefits of Keypath Approach**:
1. **Type Safety**: Compile-time verification of the access path
2. **Composability**: Easy to build complex nested access paths
3. **Reusability**: Create keypath once, use many times
4. **Readability**: Clear, declarative code that shows the exact path
5. **Maintainability**: Changes to structure automatically caught at compile time

**When to Use**:
- Thread-safe shared data structures with deep nesting
- Frequent updates to nested fields in concurrent applications
- Complex data access patterns that benefit from composition
- Code that needs to be self-documenting about data access paths

## Viewing Results

After running benchmarks, view the HTML reports:

```bash
# Open the main report directory
open target/criterion/keypath_vs_unwrap/read_nested_option/report/index.html
```

Or navigate to `target/criterion/keypath_vs_unwrap/` and open any `report/index.html` file in your browser.

For RwLock benchmarks:
```bash
open target/criterion/rwlock_write_deeply_nested/rwlock_write_deeply_nested/report/index.html
```

## Expected Findings

### Keypaths Advantages
- **Type Safety**: Compile-time guarantees
- **Reusability**: Create once, use many times
- **Composability**: Easy to build complex access paths
- **Maintainability**: Clear, declarative code

### Performance Characteristics (After Optimizations)

**Read Operations:**
- **Overhead**: Only 1.43x (43% slower) - **44% improvement from previous 2.45x!**
- **Absolute difference**: ~170 ps (0.17 ns) - negligible
- **Optimizations**: Direct `match` composition + Rc migration

**Write Operations:**
- **Overhead**: 10.8x slower - **17% improvement from previous 13.1x**
- **Absolute difference**: ~3.8 ns - still small
- **Optimizations**: Direct `match` composition + Rc migration

**Reuse Performance:**
- **98.3x faster** when keypaths are reused - this is the primary benefit!
- Pre-composed keypaths are 390x faster than on-the-fly composition

**Key Optimizations Applied:**
- ✅ Phase 1: Direct `match` instead of `and_then` (eliminated closure overhead)
- ✅ Phase 3: Aggressive inlining with `#[inline(always)]`
- ✅ Rc Migration: Replaced `Arc` with `Rc` (removed `Send + Sync`)

See [`BENCHMARK_SUMMARY.md`](BENCHMARK_SUMMARY.md) for detailed results and analysis.

## Interpreting Results

The benchmarks use Criterion.rs which provides:
- **Mean time**: Average execution time
- **Throughput**: Operations per second
- **Comparison**: Direct comparison between keypath and unwrap approaches
- **Statistical significance**: Confidence intervals and p-values

Look for:
- **Slower**: Keypath approach is slower (expected for creation)
- **Faster**: Keypath approach is faster (possible with reuse)
- **Similar**: Performance is equivalent (ideal for zero-cost abstraction)

## Notes

- Benchmarks run in release mode with optimizations
- Results may vary based on CPU architecture and compiler optimizations
- The `black_box` function prevents compiler optimizations that would skew results
- Multiple iterations ensure statistical significance

