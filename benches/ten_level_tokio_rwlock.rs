//! Benchmark: 10-level deep Arc<tokio::sync::RwLock<T>> nesting.
//!
//! Compares:
//! - **Static keypath**: AsyncLockKp chain built once, reused
//! - **Dynamic keypath**: Chain built each iteration
//! - **Direct lock acquire**: Manual .read().await through 10 levels

#![cfg(feature = "tokio")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use key_paths_derive::Kp;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

// 10-level deep: L0 -> Arc<tokio::sync::RwLock<L1>> -> L1 -> ... -> L10 { leaf: i32 }
// Use full path so derive emits TokioArcRwLock (inner_async)
#[derive(Clone, Kp)]
struct L0 {
    inner: Arc<tokio::sync::RwLock<L1>>,
}
#[derive(Clone, Kp)]
struct L1 {
    inner: Arc<tokio::sync::RwLock<L2>>,
}
#[derive(Clone, Kp)]
struct L2 {
    inner: Arc<tokio::sync::RwLock<L3>>,
}
#[derive(Clone, Kp)]
struct L3 {
    inner: Arc<tokio::sync::RwLock<L4>>,
}
#[derive(Clone, Kp)]
struct L4 {
    inner: Arc<tokio::sync::RwLock<L5>>,
}
#[derive(Clone, Kp)]
struct L5 {
    inner: Arc<tokio::sync::RwLock<L6>>,
}
#[derive(Clone, Kp)]
struct L6 {
    inner: Arc<tokio::sync::RwLock<L7>>,
}
#[derive(Clone, Kp)]
struct L7 {
    inner: Arc<tokio::sync::RwLock<L8>>,
}
#[derive(Clone, Kp)]
struct L8 {
    inner: Arc<tokio::sync::RwLock<L9>>,
}
#[derive(Clone, Kp)]
struct L9 {
    inner: Arc<tokio::sync::RwLock<L10>>,
}
#[derive(Clone, Kp)]
struct L10 {
    leaf: f64,
}

fn make_root() -> L0 {
    let leaf = L10 { leaf: 42.0 };
    let l9 = L9 {
        inner: Arc::new(RwLock::new(leaf)),
    };
    let l8 = L8 {
        inner: Arc::new(RwLock::new(l9)),
    };
    let l7 = L7 {
        inner: Arc::new(RwLock::new(l8)),
    };
    let l6 = L6 {
        inner: Arc::new(RwLock::new(l7)),
    };
    let l5 = L5 {
        inner: Arc::new(RwLock::new(l6)),
    };
    let l4 = L4 {
        inner: Arc::new(RwLock::new(l5)),
    };
    let l3 = L3 {
        inner: Arc::new(RwLock::new(l4)),
    };
    let l2 = L2 {
        inner: Arc::new(RwLock::new(l3)),
    };
    let l1 = L1 {
        inner: Arc::new(RwLock::new(l2)),
    };
    L0 {
        inner: Arc::new(RwLock::new(l1)),
    }
}

macro_rules! make_chain {
    () => {
        L0::inner_async()
            .then_async(L1::inner_async())
            .then_async(L2::inner_async())
            .then_async(L3::inner_async())
            .then_async(L4::inner_async())
            .then_async(L5::inner_async())
            .then_async(L6::inner_async())
            .then_async(L7::inner_async())
            .then_async(L8::inner_async())
            .then_async(L9::inner_async())
            .then(L10::leaf())
    };
}

fn bench_ten_level_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ten_level_tokio_rwlock_read");

    group.bench_function("keypath_static", |b| {
        let chain = make_chain!();
        let root = make_root();
        b.iter(|| {
            let result = rt.block_on(chain.get(black_box(&root)));
            black_box(result)
        })
    });

    group.bench_function("keypath_dynamic", |b| {
        let root = make_root();
        b.iter(|| {
            let chain = make_chain!();
            let result = rt.block_on(chain.get(black_box(&root)));
            black_box(result)
        })
    });

    group.bench_function("direct_lock", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let result = rt.block_on(async {
                let g1 = root_ref.inner.read().await;
                let g2 = g1.inner.read().await;
                let g3 = g2.inner.read().await;
                let g4 = g3.inner.read().await;
                let g5 = g4.inner.read().await;
                let g6 = g5.inner.read().await;
                let g7 = g6.inner.read().await;
                let g8 = g7.inner.read().await;
                let g9 = g8.inner.read().await;
                let g10 = g9.inner.read().await;
                g10.leaf
            });
            black_box(result)
        })
    });

    group.finish();
}

fn bench_ten_level_incr(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ten_level_tokio_rwlock_incr");

    group.bench_function("keypath_static", |b| {
        let chain = make_chain!();
        let mut root = make_root();
        b.iter(|| {
            rt.block_on(async {
                if let Some(v) = chain.get_mut(black_box(&mut root)).await {
                    *v += 0.25;
                }
            });
        })
    });

    group.bench_function("keypath_dynamic", |b| {
        let mut root = make_root();
        b.iter(|| {
            let chain = make_chain!();
            rt.block_on(async {
                if let Some(v) = chain.get_mut(black_box(&mut root)).await {
                    *v += 0.25;
                }
            });
        })
    });

    group.bench_function("direct_lock", |b| {
        let mut root = make_root();
        b.iter(|| {
            rt.block_on(async {
                let root_ref = black_box(&mut root);
                let mut g1 = root_ref.inner.write().await;
                let mut g2 = g1.inner.write().await;
                let mut g3 = g2.inner.write().await;
                let mut g4 = g3.inner.write().await;
                let mut g5 = g4.inner.write().await;
                let mut g6 = g5.inner.write().await;
                let mut g7 = g6.inner.write().await;
                let mut g8 = g7.inner.write().await;
                let mut g9 = g8.inner.write().await;
                let mut g10 = g9.inner.write().await;
                g10.leaf += 0.25;
            });
        })
    });

    group.finish();
}

criterion_group!(benches, bench_ten_level_read, bench_ten_level_incr);
criterion_main!(benches);
