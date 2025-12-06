use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_keypaths::{OptionalKeyPath, EnumKeyPaths};
// cargo bench --bench deeply_nested

#[derive(Debug, Clone)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
}

#[derive(Debug, Clone)]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Clone)]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Clone)]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

#[derive(Debug, Clone)]
struct DarkStruct {
    dsf: Option<DeeperStruct>,
}

#[derive(Debug, Clone)]
struct DeeperStruct {
    desf: Option<Box<String>>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(DarkStruct {
                        dsf: Some(DeeperStruct {
                            desf: Some(Box::new(String::from("deepest value"))),
                        }),
                    })),
                }),
            }),
        }
    }
}

// Benchmark: Read omsf field (3 levels deep)
fn bench_read_omsf(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_omsf");
    
    let instance = SomeComplexStruct::new();
    
    // Keypath approach
    let scsf_kp = OptionalKeyPath::new(|s: &SomeComplexStruct| s.scsf.as_ref());
    let sosf_kp = OptionalKeyPath::new(|s: &SomeOtherStruct| s.sosf.as_ref());
    let omsf_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omsf.as_ref());
    let chained_kp = scsf_kp.then(sosf_kp).then(omsf_kp);
    
    group.bench_function("keypath", |b| {
        b.iter(|| {
            let result = chained_kp.get(black_box(&instance));
            black_box(result)
        })
    });
    
    // Manual unwrapping approach
    group.bench_function("manual_unwrap", |b| {
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

// Benchmark: Read desf field (7 levels deep with enum)
fn bench_read_desf(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_desf");
    
    let instance = SomeComplexStruct::new();
    
    // Keypath approach - 7 levels: scsf -> sosf -> omse -> enum -> dsf -> desf -> Box<String>
    let scsf_kp = OptionalKeyPath::new(|s: &SomeComplexStruct| s.scsf.as_ref());
    let sosf_kp = OptionalKeyPath::new(|s: &SomeOtherStruct| s.sosf.as_ref());
    let omse_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omse.as_ref());
    let enum_b_kp = EnumKeyPaths::for_variant(|e: &SomeEnum| {
        if let SomeEnum::B(ds) = e {
            Some(ds)
        } else {
            None
        }
    });
    let dsf_kp = OptionalKeyPath::new(|d: &DarkStruct| d.dsf.as_ref());
    let desf_kp = OptionalKeyPath::new(|d: &DeeperStruct| d.desf.as_ref());
    
    let chained_kp = scsf_kp
        .then(sosf_kp)
        .then(omse_kp)
        .then(enum_b_kp)
        .then(dsf_kp)
        .then(desf_kp)
        .for_box();
    
    group.bench_function("keypath", |b| {
        b.iter(|| {
            let result = chained_kp.get(black_box(&instance));
            black_box(result)
        })
    });
    
    // Manual unwrapping approach - 7 levels
    group.bench_function("manual_unwrap", |b| {
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
                .and_then(|ds| ds.dsf.as_ref())
                .and_then(|deeper| deeper.desf.as_ref())
                .map(|boxed| boxed.as_ref());
            black_box(result)
        })
    });
    
    group.finish();
}

// Benchmark: Keypath creation overhead
fn bench_keypath_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("keypath_creation");
    
    group.bench_function("create_chained_keypath", |b| {
        b.iter(|| {
            let scsf_kp = OptionalKeyPath::new(|s: &SomeComplexStruct| s.scsf.as_ref());
            let sosf_kp = OptionalKeyPath::new(|s: &SomeOtherStruct| s.sosf.as_ref());
            let omse_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omse.as_ref());
            let enum_b_kp = EnumKeyPaths::for_variant(|e: &SomeEnum| {
                if let SomeEnum::B(ds) = e {
                    Some(ds)
                } else {
                    None
                }
            });
            let dsf_kp = OptionalKeyPath::new(|d: &DarkStruct| d.dsf.as_ref());
            let desf_kp = OptionalKeyPath::new(|d: &DeeperStruct| d.desf.as_ref());
            
            let chained = scsf_kp
                .then(sosf_kp)
                .then(omse_kp)
                .then(enum_b_kp)
                .then(dsf_kp)
                .then(desf_kp)
                .for_box();
            
            black_box(chained)
        })
    });
    
    group.finish();
}

// Benchmark: Keypath reuse (pre-created vs on-the-fly)
fn bench_keypath_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("keypath_reuse");
    
    let instance = SomeComplexStruct::new();
    
    // Pre-created keypath
    let scsf_kp = OptionalKeyPath::new(|s: &SomeComplexStruct| s.scsf.as_ref());
    let sosf_kp = OptionalKeyPath::new(|s: &SomeOtherStruct| s.sosf.as_ref());
    let omsf_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omsf.as_ref());
    let pre_created = scsf_kp.then(sosf_kp).then(omsf_kp);
    
    group.bench_function("pre_created", |b| {
        b.iter(|| {
            let result = pre_created.get(black_box(&instance));
            black_box(result)
        })
    });
    
    // Created on-the-fly
    group.bench_function("created_on_fly", |b| {
        b.iter(|| {
            let scsf_kp = OptionalKeyPath::new(|s: &SomeComplexStruct| s.scsf.as_ref());
            let sosf_kp = OptionalKeyPath::new(|s: &SomeOtherStruct| s.sosf.as_ref());
            let omsf_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omsf.as_ref());
            let chained = scsf_kp.then(sosf_kp).then(omsf_kp);
            let result = chained.get(black_box(&instance));
            black_box(result)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_read_omsf,
    bench_read_desf,
    bench_keypath_creation,
    bench_keypath_reuse
);
criterion_main!(benches);

