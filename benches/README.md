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

### Performance Characteristics
- **Creation Overhead**: Small cost when creating keypaths
- **Access Overhead**: Minimal runtime overhead (typically < 5%)
- **Reuse Benefit**: Significant advantage when reusing keypaths
- **Composition**: Pre-composed keypaths perform better than on-the-fly

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

