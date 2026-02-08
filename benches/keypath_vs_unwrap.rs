use criterion::{black_box, criterion_group, criterion_main, Criterion};
use key_paths_derive::Kp;
use parking_lot::RwLock;
use std::sync::Arc;

// Structs renamed for better readability - Level1 is root, Level2, Level3, etc. indicate nesting depth
#[derive(Debug, Kp)]
struct Level1Struct {
    level1_field: Option<Level2Struct>,
    level1_field2: Arc<RwLock<Level2Struct>>,
}

#[derive(Debug, Kp)]
struct Level2Struct {
    level2_field: Option<Level3Struct>,
}

#[derive(Debug, Kp)]
enum Level3Enum {
    A(String),
    B(Box<Level3EnumStruct>),
}

#[derive(Debug, Kp)]
struct Level3Struct {
    level3_field: Option<String>,
    level3_enum_field: Option<Level3Enum>,
    level3_deep_field: Option<Level4Struct>, // For 5-level deep nesting without enum
}

#[derive(Debug, Kp)]
struct Level3EnumStruct {
    level3_enum_struct_field: Option<String>,
}

// Additional structs for 5-level deep nesting without enum
#[derive(Debug, Kp)]
struct Level4Struct {
    level4_field: Option<Level5Struct>,
}

#[derive(Debug, Kp)]
struct Level5Struct {
    level5_field: Option<String>,
}

impl Level1Struct {
    fn new() -> Self {
        Self {
            level1_field: Some(Level2Struct {
                level2_field: Some(Level3Struct {
                    level3_field: Some(String::from("level 3 value")),
                    level3_enum_field: Some(Level3Enum::B(Box::new(Level3EnumStruct {
                        level3_enum_struct_field: Some(String::from("level 3 enum struct field")),
                    }))),
                    level3_deep_field: Some(Level4Struct {
                        level4_field: Some(Level5Struct {
                            level5_field: Some(String::from("level 5 value")),
                        }),
                    }),
                }),
            }),
            level1_field2: Arc::new(RwLock::new(Level2Struct {
                level2_field: Some(Level3Struct {
                    level3_field: Some(String::from("level 3 value")),
                    level3_enum_field: Some(Level3Enum::B(Box::new(Level3EnumStruct {
                        level3_enum_struct_field: Some(String::from("level 3 enum struct field")),
                    }))),
                    level3_deep_field: Some(Level4Struct {
                        level4_field: Some(Level5Struct {
                            level5_field: Some(String::from("level 5 value")),
                        }),
                    }),
                }),
            })),
        }
    }
}

// Benchmark: Read access through nested Option chain (3 levels)
fn bench_read_nested_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_nested_option");

    let instance = Level1Struct::new();
    let kp = Level1Struct::level1_field()
        .then(Level2Struct::level2_field())
        .then(Level3Struct::level3_field());

    // Keypath approach: Level1 -> Level2 -> Level3
    group.bench_function("keypath", |b| {
        b.iter(|| {
            let result = kp.get(black_box(&instance));
            black_box(result)
        })
    });

    // Direct unwrap approach
    group.bench_function("direct_unwrap", |b| {
        b.iter(|| {
            let result = instance
                .level1_field
                .as_ref()
                .and_then(|l2| l2.level2_field.as_ref())
                .and_then(|l3| l3.level3_field.as_ref());
            black_box(result)
        })
    });

    group.finish();
}

// Benchmark: Write access through nested Option chain (3 levels)
fn bench_write_nested_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_nested_option");

    group.bench_function("keypath", |b| {
        let mut instance = Level1Struct::new();
        b.iter(|| {
            let keypath = Level1Struct::level1_field()
                .then(Level2Struct::level2_field())
                .then(Level3Struct::level3_field());
            let result = keypath.get_mut(black_box(&mut instance));
            black_box(result.is_some())
        })
    });

    // Direct unwrap approach
    group.bench_function("direct_unwrap", |b| {
        let mut instance = Level1Struct::new();
        b.iter(|| {
            let result = instance
                .level1_field
                .as_mut()
                .and_then(|l2| l2.level2_field.as_mut())
                .and_then(|l3| l3.level3_field.as_mut());
            // Use the result without returning the reference
            black_box(result.is_some())
        })
    });

    group.finish();
}

// Deep nested read without enum (5 levels deep - matching enum depth)
fn bench_deep_nested_without_enum(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_nested_without_enum");

    let instance = Level1Struct::new();

    let keypath = Level1Struct::level1_field()
        .then(Level2Struct::level2_field())
        .then(Level3Struct::level3_deep_field())
        .then(Level4Struct::level4_field())
        .then(Level5Struct::level5_field());

    group.bench_function("keypath", |b| {
        b.iter(|| {
            let result = keypath.get(black_box(&instance));
            black_box(result)
        })
    });

    // Direct unwrap approach - 5 levels deep
    group.bench_function("direct_unwrap", |b| {
        b.iter(|| {
            let result = instance
                .level1_field
                .as_ref()
                .and_then(|l2| l2.level2_field.as_ref())
                .and_then(|l3| l3.level3_deep_field.as_ref())
                .and_then(|l4| l4.level4_field.as_ref())
                .and_then(|l5| l5.level5_field.as_ref());
            black_box(result)
        })
    });

    group.finish();
}

// Deep nested read with enum (5 levels deep)
fn bench_deep_nested_with_enum(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_nested_with_enum");

    let instance = Level1Struct::new();

    let keypath = Level1Struct::level1_field()
        .then(Level2Struct::level2_field())
        .then(Level3Struct::level3_enum_field())
        .then(Level3Enum::b())
        .then(Level3EnumStruct::level3_enum_struct_field());

    group.bench_function("keypath", |b| {
        b.iter(|| {
            let result = keypath.get(black_box(&instance));
            black_box(result)
        })
    });

    // Direct unwrap approach
    group.bench_function("direct_unwrap", |b| {
        b.iter(|| {
            let result = instance
                .level1_field
                .as_ref()
                .and_then(|l2| l2.level2_field.as_ref())
                .and_then(|l3| l3.level3_enum_field.as_ref())
                .and_then(|e| match e {
                    Level3Enum::B(ds) => Some(ds),
                    _ => None,
                })
                .and_then(|ds| ds.level3_enum_struct_field.as_ref());
            black_box(result)
        })
    });

    group.finish();
}
// Benchmark: Write access with enum case path (5 levels deep)
fn bench_write_deep_nested_with_enum(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_deep_nested_with_enum");

    group.bench_function("keypath", |b| {
        let mut instance = Level1Struct::new();
        b.iter(|| {
            let keypath = Level1Struct::level1_field()
                .then(Level2Struct::level2_field())
                .then(Level3Struct::level3_enum_field())
                .then(Level3Enum::b())
                .then(Level3EnumStruct::level3_enum_struct_field());
            let result = keypath.get_mut(black_box(&mut instance));
            black_box(result.is_some())
        })
    });

    // Direct unwrap approach
    group.bench_function("direct_unwrap", |b| {
        let mut instance = Level1Struct::new();
        b.iter(|| {
            let result = instance
                .level1_field
                .as_mut()
                .and_then(|l2| l2.level2_field.as_mut())
                .and_then(|l3| l3.level3_enum_field.as_mut())
                .and_then(|e| match e {
                    Level3Enum::B(ds) => Some(ds),
                    _ => None,
                })
                .and_then(|ds| ds.level3_enum_struct_field.as_mut());
            // Use the result without returning the reference
            black_box(result.is_some())
        })
    });

    group.finish();
}

// Benchmark: Keypath creation overhead
fn bench_keypath_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("keypath_creation");

    group.bench_function("create_complex_keypath", |b| {
        let instance = Level1Struct::new();
        b.iter(|| {
            let keypath = Level1Struct::level1_field()
                .then(Level2Struct::level2_field())
                .then(Level3Struct::level3_enum_field())
                .then(Level3Enum::b())
                .then(Level3EnumStruct::level3_enum_struct_field());
            black_box(keypath.get(black_box(&instance)).is_some())
        })
    });

    group.finish();
}

// Benchmark: Multiple accesses with same keypath (reuse)
fn bench_keypath_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("keypath_reuse");

    let mut instances: Vec<_> = (0..100).map(|_| Level1Struct::new()).collect();

    group.bench_function("keypath_reused", |b| {
        b.iter(|| {
            let keypath = Level1Struct::level1_field()
                .then(Level2Struct::level2_field())
                .then(Level3Struct::level3_field());
            let mut sum = 0;
            for instance in &mut instances {
                if let Some(value) = keypath.get_mut(instance) {
                    sum += value.len();
                }
            }
            black_box(sum)
        })
    });

    group.bench_function("direct_unwrap_repeated", |b| {
        b.iter(|| {
            let mut sum = 0;
            for instance in &instances {
                if let Some(l2) = instance.level1_field.as_ref() {
                    if let Some(l3) = l2.level2_field.as_ref() {
                        if let Some(l3_field) = l3.level3_field.as_ref() {
                            sum += l3_field.len();
                        }
                    }
                }
            }
            black_box(sum)
        })
    });

    group.finish();
}

// Benchmark: Composition overhead
fn bench_composition_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("composition_overhead");

    let mut instance = Level1Struct::new();

    group.bench_function("pre_composed", |b| {
        b.iter(|| {
            let pre_composed = Level1Struct::level1_field()
                .then(Level2Struct::level2_field())
                .then(Level3Struct::level3_field());
            let result = pre_composed.get_mut(black_box(&mut instance));
            black_box(result.is_some())
        })
    });

    // Composed on-the-fly
    group.bench_function("composed_on_fly", |b| {
        b.iter(|| {
            let keypath = Level1Struct::level1_field()
                .then(Level2Struct::level2_field())
                .then(Level3Struct::level3_field());
            let result = keypath.get(black_box(&instance)).map(|s| s.len());
            black_box(result)
        })
    });

    group.finish();
}

// 10-level deep struct definitions
#[derive(Debug, Clone, Kp)]
struct TenLevel1Struct {
    level1_field: Option<TenLevel2Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel2Struct {
    level2_field: Option<TenLevel3Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel3Struct {
    level3_field: Option<TenLevel4Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel4Struct {
    level4_field: Option<TenLevel5Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel5Struct {
    level5_field: Option<TenLevel6Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel6Struct {
    level6_field: Option<TenLevel7Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel7Struct {
    level7_field: Option<TenLevel8Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel8Struct {
    level8_field: Option<TenLevel9Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel9Struct {
    level9_field: Option<TenLevel10Struct>,
}

#[derive(Debug, Clone, Kp)]
struct TenLevel10Struct {
    level10_field: Option<String>,
}

impl TenLevel1Struct {
    fn new() -> Self {
        Self {
            level1_field: Some(TenLevel2Struct {
                level2_field: Some(TenLevel3Struct {
                    level3_field: Some(TenLevel4Struct {
                        level4_field: Some(TenLevel5Struct {
                            level5_field: Some(TenLevel6Struct {
                                level6_field: Some(TenLevel7Struct {
                                    level7_field: Some(TenLevel8Struct {
                                        level8_field: Some(TenLevel9Struct {
                                            level9_field: Some(TenLevel10Struct {
                                                level10_field: Some(String::from("level 10 value")),
                                            }),
                                        }),
                                    }),
                                }),
                            }),
                        }),
                    }),
                }),
            }),
        }
    }
}

// Benchmark: 10-level deep read and write operations
fn bench_ten_level(c: &mut Criterion) {
    let mut group = c.benchmark_group("ten_level");

    // Read benchmark
    let instance = TenLevel1Struct::new();
    group.bench_function("read", |b| {
        b.iter(|| {
            let read_kp = TenLevel1Struct::level1_field()
                .then(TenLevel2Struct::level2_field())
                .then(TenLevel3Struct::level3_field())
                .then(TenLevel4Struct::level4_field())
                .then(TenLevel5Struct::level5_field())
                .then(TenLevel6Struct::level6_field())
                .then(TenLevel7Struct::level7_field())
                .then(TenLevel8Struct::level8_field())
                .then(TenLevel9Struct::level9_field())
                .then(TenLevel10Struct::level10_field());
            let result = read_kp.get(black_box(&instance));
            black_box(result.is_some())
        })
    });

    // Write benchmark
    let mut instance_mut = TenLevel1Struct::new();

    group.bench_function("write", |b| {
        b.iter(|| {
            let write_kp = TenLevel1Struct::level1_field()
                .then(TenLevel2Struct::level2_field())
                .then(TenLevel3Struct::level3_field())
                .then(TenLevel4Struct::level4_field())
                .then(TenLevel5Struct::level5_field())
                .then(TenLevel6Struct::level6_field())
                .then(TenLevel7Struct::level7_field())
                .then(TenLevel8Struct::level8_field())
                .then(TenLevel9Struct::level9_field())
                .then(TenLevel10Struct::level10_field());
            if let Some(value) = write_kp.get_mut(black_box(&mut instance_mut)) {
                *value = String::from("updated value");
            }
            black_box(())
        })
    });

    // Traditional approach for comparison (read)
    group.bench_function("read_traditional", |b| {
        b.iter(|| {
            let result = instance
                .level1_field
                .as_ref()
                .and_then(|l2| l2.level2_field.as_ref())
                .and_then(|l3| l3.level3_field.as_ref())
                .and_then(|l4| l4.level4_field.as_ref())
                .and_then(|l5| l5.level5_field.as_ref())
                .and_then(|l6| l6.level6_field.as_ref())
                .and_then(|l7| l7.level7_field.as_ref())
                .and_then(|l8| l8.level8_field.as_ref())
                .and_then(|l9| l9.level9_field.as_ref())
                .and_then(|l10| l10.level10_field.as_ref());
            black_box(result.is_some())
        })
    });

    // Traditional approach for comparison (write)
    group.bench_function("write_traditional", |b| {
        b.iter(|| {
            if let Some(l2) = instance_mut.level1_field.as_mut() {
                if let Some(l3) = l2.level2_field.as_mut() {
                    if let Some(l4) = l3.level3_field.as_mut() {
                        if let Some(l5) = l4.level4_field.as_mut() {
                            if let Some(l6) = l5.level5_field.as_mut() {
                                if let Some(l7) = l6.level6_field.as_mut() {
                                    if let Some(l8) = l7.level7_field.as_mut() {
                                        if let Some(l9) = l8.level8_field.as_mut() {
                                            if let Some(l10) = l9.level9_field.as_mut() {
                                                if let Some(value) = l10.level10_field.as_mut() {
                                                    *value = String::from("updated value");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            black_box(())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_read_nested_option,
    bench_write_nested_option,
    bench_deep_nested_without_enum,
    bench_deep_nested_with_enum,
    bench_write_deep_nested_with_enum,
    bench_keypath_creation,
    bench_keypath_reuse,
    bench_composition_overhead,
    bench_ten_level
);
criterion_main!(benches);
