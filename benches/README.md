# KeyPaths Performance Benchmarks

This directory contains comprehensive benchmarks comparing the performance of KeyPaths versus direct nested unwraps.

## Running Benchmarks

### Quick Run
```bash
cargo bench --bench keypath_vs_unwrap
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
- **Keypath**: Includes `SomeEnum::b_case_w()` and `for_box()` adapter
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

## Viewing Results

After running benchmarks, view the HTML reports:

```bash
# Open the main report directory
open target/criterion/keypath_vs_unwrap/read_nested_option/report/index.html
```

Or navigate to `target/criterion/keypath_vs_unwrap/` and open any `report/index.html` file in your browser.

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

