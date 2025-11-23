use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use parking_lot::RwLock;
use key_paths_derive::{Casepaths, Keypaths};

// Same structs as in basics_casepath.rs
#[derive(Debug, Clone, Keypaths)]
#[All]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
    scfs2: Arc<RwLock<SomeOtherStruct>>,
}

#[derive(Debug, Clone, Keypaths)]
#[All]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Clone, Casepaths)]
enum SomeEnum {
    A(String),
    B(Box<DarkStruct>),
}

#[derive(Debug, Clone, Keypaths)]
#[All]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Clone, Keypaths)]
#[All]
struct DarkStruct {
    dsf: Option<String>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(Box::new(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    }))),
                }),
            }),
            scfs2: Arc::new(
                RwLock::new(
                    SomeOtherStruct {
                        sosf: Some(OneMoreStruct {
                            omsf: Some(String::from("no value for now")),
                            omse: Some(SomeEnum::B(Box::new(DarkStruct {
                                dsf: Some(String::from("dark field")),
                            }))),
                        }),
                    }
                )
            ),
        }
    }
}

// Benchmark: Read access through nested Option chain
fn bench_read_nested_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_nested_option");
    
    let instance = SomeComplexStruct::new();
    
    // Keypath approach
    let keypath = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omsf_fw());
    
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
                .scsf
                .as_ref()
                .and_then(|s| s.sosf.as_ref())
                .and_then(|o| o.omsf.as_ref());
            black_box(result)
        })
    });
    
    group.finish();
}

// Benchmark: Write access through nested Option chain
fn bench_write_nested_option(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_nested_option");
    
    // Keypath approach
    let keypath = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omsf_fw());
    
    group.bench_function("keypath", |b| {
        b.iter(|| {
            let mut instance = SomeComplexStruct::new();
            if let Some(value) = keypath.get_mut(&mut instance) {
                *value = black_box(String::from("updated"));
            }
            black_box(instance)
        })
    });
    
    // Direct unwrap approach
    group.bench_function("direct_unwrap", |b| {
        b.iter(|| {
            let mut instance = SomeComplexStruct::new();
            if let Some(sos) = instance.scsf.as_mut() {
                if let Some(oms) = sos.sosf.as_mut() {
                    if let Some(omsf) = oms.omsf.as_mut() {
                        *omsf = black_box(String::from("updated"));
                    }
                }
            }
            black_box(instance)
        })
    });
    
    group.finish();
}

// Benchmark: Deep nested access with enum case path
fn bench_deep_nested_with_enum(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_nested_with_enum");
    
    let instance = SomeComplexStruct::new();
    
    // Keypath approach
    let keypath = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw().for_box());
    
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
                .scsf
                .as_ref()
                .and_then(|s| s.sosf.as_ref())
                .and_then(|o| o.omse.as_ref())
                .and_then(|e| match e {
                    SomeEnum::B(ds) => Some(ds),
                    _ => None,
                })
                .and_then(|ds| ds.dsf.as_ref());
            black_box(result)
        })
    });
    
    group.finish();
}

// Benchmark: Write access with enum case path
fn bench_write_deep_nested_with_enum(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_deep_nested_with_enum");
    
    // Keypath approach
    let keypath = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw().for_box());
    
    group.bench_function("keypath", |b| {
        b.iter(|| {
            let mut instance = SomeComplexStruct::new();
            if let Some(value) = keypath.get_mut(&mut instance) {
                *value = black_box(String::from("updated"));
            }
            black_box(instance)
        })
    });
    
    // Direct unwrap approach
    group.bench_function("direct_unwrap", |b| {
        b.iter(|| {
            let mut instance = SomeComplexStruct::new();
            if let Some(sos) = instance.scsf.as_mut() {
                if let Some(oms) = sos.sosf.as_mut() {
                    if let Some(SomeEnum::B(ds)) = oms.omse.as_mut() {
                        if let Some(dsf) = ds.dsf.as_mut() {
                            *dsf = black_box(String::from("updated"));
                        }
                    }
                }
            }
            black_box(instance)
        })
    });
    
    group.finish();
}

// Benchmark: Keypath creation overhead
fn bench_keypath_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("keypath_creation");
    
    group.bench_function("create_complex_keypath", |b| {
        b.iter(|| {
            let keypath = SomeComplexStruct::scsf_fw()
                .then(SomeOtherStruct::sosf_fw())
                .then(OneMoreStruct::omse_fw())
                .then(SomeEnum::b_case_w())
                .then(DarkStruct::dsf_fw().for_box());
            black_box(keypath)
        })
    });
    
    group.finish();
}

// Benchmark: Multiple accesses with same keypath (reuse)
fn bench_keypath_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("keypath_reuse");
    
    let keypath = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omsf_fw());
    
    let instances: Vec<_> = (0..100).map(|_| SomeComplexStruct::new()).collect();
    
    group.bench_function("keypath_reused", |b| {
        b.iter(|| {
            let mut sum = 0;
            for instance in &instances {
                if let Some(value) = keypath.get(instance) {
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
                if let Some(sos) = instance.scsf.as_ref() {
                    if let Some(oms) = sos.sosf.as_ref() {
                        if let Some(omsf) = oms.omsf.as_ref() {
                            sum += omsf.len();
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
    
    let instance = SomeComplexStruct::new();
    
    // Pre-composed keypath
    let pre_composed = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omsf_fw());
    
    group.bench_function("pre_composed", |b| {
        b.iter(|| {
            let result = pre_composed.get(black_box(&instance));
            black_box(result)
        })
    });
    
    // Composed on-the-fly
    group.bench_function("composed_on_fly", |b| {
        b.iter(|| {
            let keypath = SomeComplexStruct::scsf_fw()
                .then(SomeOtherStruct::sosf_fw())
                .then(OneMoreStruct::omsf_fw());
            let result = keypath.get(black_box(&instance)).map(|s| s.len());
            black_box(result)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_read_nested_option,
    bench_write_nested_option,
    bench_deep_nested_with_enum,
    bench_write_deep_nested_with_enum,
    bench_keypath_creation,
    bench_keypath_reuse,
    bench_composition_overhead
);
criterion_main!(benches);

