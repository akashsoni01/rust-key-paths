use criterion::{Criterion, black_box, criterion_group, criterion_main};
use keypaths_proc::Kp;
use parking_lot::RwLock;
use std::sync::Arc;
#[cfg(feature = "tokio")]
use tokio::runtime::Runtime;
#[cfg(feature = "tokio")]
use tokio::sync::RwLock as TokioRwLock;

// Struct definitions matching the user's example
#[derive(Kp)]
#[Writable]
struct SomeStruct {
    f1: Arc<RwLock<SomeOtherStruct>>,
}

#[derive(Kp)]
#[Writable]
struct SomeOtherStruct {
    f3: Option<String>,
    f4: DeeplyNestedStruct,
}

#[derive(Kp)]
#[Writable]
struct DeeplyNestedStruct {
    f1: Option<String>,
    f2: Option<i32>,
}

impl SomeStruct {
    fn new() -> Self {
        Self {
            f1: Arc::new(RwLock::new(SomeOtherStruct {
                f3: Some(String::from("value")),
                f4: DeeplyNestedStruct {
                    f1: Some(String::from("value")),
                    f2: Some(12),
                },
            })),
        }
    }
}

// Benchmark: Write access through Arc<RwLock<...>> with deeply nested keypath
fn bench_rwlock_write_deeply_nested_keypath(c: &mut Criterion) {
    let mut group = c.benchmark_group("rwlock_write_deeply_nested");

    // Keypath approach: SomeStruct -> Arc<RwLock<SomeOtherStruct>> -> SomeOtherStruct -> DeeplyNestedStruct -> f1
    group.bench_function("keypath", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let keypath =
                SomeStruct::f1_fw_at(SomeOtherStruct::f4_w().then(DeeplyNestedStruct::f1_w()));
            keypath.get_mut(black_box(&instance), |value| {
                *value = Some(String::from("new value"));
            });
            black_box(())
        })
    });

    // Traditional approach: Manual write guard and nested unwraps
    group.bench_function("write_guard", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let mut guard = instance.f1.write();
            if let Some(f4) = guard.f4.f1.as_mut() {
                *f4 = String::from("new value");
            }
            black_box(())
        })
    });

    // Alternative traditional approach: More explicit nested unwraps
    group.bench_function("write_guard_nested", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let mut guard = instance.f1.write();
            if let Some(ref mut f4) = guard.f4.f1 {
                *f4 = String::from("new value");
            }
            black_box(())
        })
    });

    group.finish();
}

// Benchmark: Write access to f2 (Option<i32>) in deeply nested structure
fn bench_rwlock_write_deeply_nested_f2(c: &mut Criterion) {
    let mut group = c.benchmark_group("rwlock_write_deeply_nested_f2");

    // Keypath approach: SomeStruct -> Arc<RwLock<SomeOtherStruct>> -> SomeOtherStruct -> DeeplyNestedStruct -> f2
    group.bench_function("keypath", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let keypath =
                SomeStruct::f1_fw_at(SomeOtherStruct::f4_w().then(DeeplyNestedStruct::f2_w()));
            keypath.get_mut(black_box(&instance), |value| {
                *value = Some(42);
            });
            black_box(())
        })
    });

    // Traditional approach
    group.bench_function("write_guard", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let mut guard = instance.f1.write();
            if let Some(ref mut f2) = guard.f4.f2 {
                *f2 = 42;
            }
            black_box(())
        })
    });

    group.finish();
}

// Benchmark: Write access to f3 (Option<String>) in SomeOtherStruct
fn bench_rwlock_write_f3(c: &mut Criterion) {
    let mut group = c.benchmark_group("rwlock_write_f3");

    // Keypath approach: SomeStruct -> Arc<RwLock<SomeOtherStruct>> -> f3
    group.bench_function("keypath", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let keypath = SomeStruct::f1_fw_at(SomeOtherStruct::f3_w());
            keypath.get_mut(black_box(&instance), |value| {
                *value = Some(String::from("updated f3"));
            });
            black_box(())
        })
    });

    // Traditional approach
    group.bench_function("write_guard", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let mut guard = instance.f1.write();
            if let Some(ref mut f3) = guard.f3 {
                *f3 = String::from("updated f3");
            }
            black_box(())
        })
    });

    group.finish();
}

// Benchmark: Multiple sequential writes using keypath vs write guard
fn bench_rwlock_multiple_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("rwlock_multiple_writes");

    // Keypath approach: Create keypaths in each iteration
    group.bench_function("keypath", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let keypath_f3 = SomeStruct::f1_fw_at(SomeOtherStruct::f3_w());
            keypath_f3.get_mut(black_box(&instance), |value| {
                *value = Some(String::from("updated f3"));
            });
            let keypath_f1 =
                SomeStruct::f1_fw_at(SomeOtherStruct::f4_w().then(DeeplyNestedStruct::f1_w()));
            keypath_f1.get_mut(black_box(&instance), |value| {
                *value = Some(String::from("updated f1"));
            });
            let keypath_f2 =
                SomeStruct::f1_fw_at(SomeOtherStruct::f4_w().then(DeeplyNestedStruct::f2_w()));
            keypath_f2.get_mut(black_box(&instance), |value| {
                *value = Some(42);
            });
            black_box(())
        })
    });

    // Traditional approach: Single write guard for all operations
    group.bench_function("write_guard_single", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            let mut guard = instance.f1.write();
            if let Some(ref mut f3) = guard.f3 {
                *f3 = String::from("updated f3");
            }
            if let Some(ref mut f1) = guard.f4.f1 {
                *f1 = String::from("updated f1");
            }
            if let Some(ref mut f2) = guard.f4.f2 {
                *f2 = 42;
            }
            black_box(())
        })
    });

    // Traditional approach: Multiple write guards (less efficient)
    group.bench_function("write_guard_multiple", |b| {
        let instance = SomeStruct::new();
        b.iter(|| {
            {
                let mut guard = instance.f1.write();
                if let Some(ref mut f3) = guard.f3 {
                    *f3 = String::from("updated f3");
                }
            }
            {
                let mut guard = instance.f1.write();
                if let Some(ref mut f1) = guard.f4.f1 {
                    *f1 = String::from("updated f1");
                }
            }
            {
                let mut guard = instance.f1.write();
                if let Some(ref mut f2) = guard.f4.f2 {
                    *f2 = 42;
                }
            }
            black_box(())
        })
    });

    group.finish();
}

// ========== TOKIO RWLock BENCHMARKS ==========

#[cfg(feature = "tokio")]
#[derive(Kp)]
#[All] // Generate all methods (readable, writable, owned)
struct TokioSomeStruct {
    f1: Arc<tokio::sync::RwLock<TokioSomeOtherStruct>>,
}

#[cfg(feature = "tokio")]
#[derive(Kp)]
#[All] // Generate all methods (readable, writable, owned)
struct TokioSomeOtherStruct {
    f3: Option<String>,
    f4: TokioDeeplyNestedStruct,
}

#[cfg(feature = "tokio")]
#[derive(Kp)]
#[All] // Generate all methods (readable, writable, owned)
struct TokioDeeplyNestedStruct {
    f1: Option<String>,
    f2: Option<i32>,
}

#[cfg(feature = "tokio")]
impl TokioSomeStruct {
    fn new() -> Self {
        Self {
            f1: Arc::new(tokio::sync::RwLock::new(TokioSomeOtherStruct {
                f3: Some(String::from("value")),
                f4: TokioDeeplyNestedStruct {
                    f1: Some(String::from("value")),
                    f2: Some(12),
                },
            })),
        }
    }
}

// Benchmark: Read access through Arc<tokio::sync::RwLock<...>> with deeply nested keypath
#[cfg(feature = "tokio")]
fn bench_tokio_rwlock_read_deeply_nested_keypath(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("tokio_rwlock_read_deeply_nested");

    // Keypath approach: TokioSomeStruct -> Arc<TokioRwLock<TokioSomeOtherStruct>> -> TokioSomeOtherStruct -> TokioDeeplyNestedStruct -> f1
    group.bench_function("keypath", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let keypath = TokioSomeStruct::f1_fr_at(
                    TokioSomeOtherStruct::f4_r().then(TokioDeeplyNestedStruct::f1_r()),
                );
                keypath
                    .get(black_box(&instance), |value| {
                        black_box(value);
                    })
                    .await;
            })
        })
    });

    // Traditional approach: Manual read guard and nested access
    group.bench_function("read_guard", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let guard = instance.f1.read().await;
                let _value = black_box(&guard.f4.f1);
            })
        })
    });

    group.finish();
}

// Benchmark: Write access through Arc<tokio::sync::RwLock<...>> with deeply nested keypath
#[cfg(feature = "tokio")]
fn bench_tokio_rwlock_write_deeply_nested_keypath(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("tokio_rwlock_write_deeply_nested");

    // Keypath approach: TokioSomeStruct -> Arc<TokioRwLock<TokioSomeOtherStruct>> -> TokioSomeOtherStruct -> TokioDeeplyNestedStruct -> f1
    group.bench_function("keypath", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let keypath = TokioSomeStruct::f1_fw_at(
                    TokioSomeOtherStruct::f4_w().then(TokioDeeplyNestedStruct::f1_w()),
                );
                keypath
                    .get_mut(black_box(&instance), |value| {
                        *value = Some(String::from("new value"));
                    })
                    .await;
            })
        })
    });

    // Traditional approach: Manual write guard and nested unwraps
    group.bench_function("write_guard", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let mut guard = instance.f1.write().await;
                if let Some(f4) = guard.f4.f1.as_mut() {
                    *f4 = String::from("new value");
                }
                black_box(())
            })
        })
    });

    group.finish();
}

// Benchmark: Write access to f2 (Option<i32>) in deeply nested structure with Tokio
#[cfg(feature = "tokio")]
fn bench_tokio_rwlock_write_deeply_nested_f2(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("tokio_rwlock_write_deeply_nested_f2");

    // Keypath approach
    group.bench_function("keypath", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let keypath = TokioSomeStruct::f1_fw_at(
                    TokioSomeOtherStruct::f4_w().then(TokioDeeplyNestedStruct::f2_w()),
                );
                keypath
                    .get_mut(black_box(&instance), |value| {
                        *value = Some(42);
                    })
                    .await;
            })
        })
    });

    // Traditional approach
    group.bench_function("write_guard", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let mut guard = instance.f1.write().await;
                if let Some(ref mut f2) = guard.f4.f2 {
                    *f2 = 42;
                }
                black_box(())
            })
        })
    });

    group.finish();
}

// Benchmark: Read access to f3 (Option<String>) in SomeOtherStruct with Tokio
#[cfg(feature = "tokio")]
fn bench_tokio_rwlock_read_f3(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("tokio_rwlock_read_f3");

    // Keypath approach
    group.bench_function("keypath", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let keypath = TokioSomeStruct::f1_fr_at(TokioSomeOtherStruct::f3_r());
                keypath
                    .get(black_box(&instance), |value| {
                        black_box(value);
                    })
                    .await;
            })
        })
    });

    // Traditional approach
    group.bench_function("read_guard", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let guard = instance.f1.read().await;
                let _value = black_box(&guard.f3);
            })
        })
    });

    group.finish();
}

// Benchmark: Write access to f3 (Option<String>) in SomeOtherStruct with Tokio
#[cfg(feature = "tokio")]
fn bench_tokio_rwlock_write_f3(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("tokio_rwlock_write_f3");

    // Keypath approach
    group.bench_function("keypath", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let keypath = TokioSomeStruct::f1_fw_at(TokioSomeOtherStruct::f3_w());
                keypath
                    .get_mut(black_box(&instance), |value| {
                        *value = Some(String::from("updated f3"));
                    })
                    .await;
            })
        })
    });

    // Traditional approach
    group.bench_function("write_guard", |b| {
        let instance = TokioSomeStruct::new();
        b.iter(|| {
            rt.block_on(async {
                let mut guard = instance.f1.write().await;
                if let Some(ref mut f3) = guard.f3 {
                    *f3 = String::from("updated f3");
                }
                black_box(())
            })
        })
    });

    group.finish();
}

#[cfg(not(feature = "tokio"))]
criterion_group!(
    benches,
    bench_rwlock_write_deeply_nested_keypath,
    bench_rwlock_write_deeply_nested_f2,
    bench_rwlock_write_f3,
    bench_rwlock_multiple_writes,
);

#[cfg(feature = "tokio")]
criterion_group!(
    benches,
    bench_rwlock_write_deeply_nested_keypath,
    bench_rwlock_write_deeply_nested_f2,
    bench_rwlock_write_f3,
    bench_rwlock_multiple_writes,
    bench_tokio_rwlock_read_deeply_nested_keypath,
    bench_tokio_rwlock_write_deeply_nested_keypath,
    bench_tokio_rwlock_write_deeply_nested_f2,
    bench_tokio_rwlock_read_f3,
    bench_tokio_rwlock_write_f3,
);

criterion_main!(benches);
