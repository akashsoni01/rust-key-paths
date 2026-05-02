use criterion::{black_box, criterion_group, criterion_main, Criterion};
use key_paths_derive::Kp;
use rust_key_paths::ChainExt;
use std::sync::Arc;

#[derive(Debug, Kp, Clone)]
struct SomeComplexStruct {
    scsf: Box<SomeOtherStruct>,
}

#[derive(Debug, Kp, Clone)]
struct SomeOtherStruct {
    sosf: Arc<arc_swap::ArcSwap<OneMoreStruct>>,
}

#[derive(Debug, Kp, Clone)]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

#[derive(Debug, Kp, Clone)]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Kp, Clone)]
struct DarkStruct {
    dsf: String,
}

fn init_fixture() -> SomeComplexStruct {
    let inner = OneMoreStruct {
        omsf: "omsf_value".to_string(),
        omse: SomeEnum::B(DarkStruct {
            dsf: "dark_value".to_string(),
        }),
    };
    SomeComplexStruct {
        scsf: Box::new(SomeOtherStruct {
            sosf: Arc::new(arc_swap::ArcSwap::from_pointee(inner)),
        }),
    }
}

fn read_keypath(instance: &SomeComplexStruct) -> Option<String> {
    SomeComplexStruct::scsf()
        .then_lock(
            SomeOtherStruct::sosf()
                .then(OneMoreStruct::omse())
                .then(SomeEnum::b())
                .then(DarkStruct::dsf()),
        )
        .get(instance)
        .map(|s| s.clone())
}

fn read_load_full(instance: &SomeComplexStruct) -> Option<String> {
    let inner = instance.scsf.sosf.load_full();
    match &inner.omse {
        SomeEnum::B(ds) => Some(ds.dsf.clone()),
        SomeEnum::A(_) => None,
    }
}

fn bench_arcswap_keypath(c: &mut Criterion) {
    let mut g = c.benchmark_group("arcswap_keypath_read");
    let instance = init_fixture();
    g.bench_function("keypath_then_lock", |b| {
        b.iter(|| black_box(read_keypath(black_box(&instance))))
    });
    g.bench_function("load_full_manual", |b| {
        b.iter(|| black_box(read_load_full(black_box(&instance))))
    });
    g.finish();
}

criterion_group!(benches, bench_arcswap_keypath);
criterion_main!(benches);
