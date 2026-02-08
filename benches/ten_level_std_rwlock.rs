//! Benchmark: 10-level deep Arc<std::sync::RwLock<T>> nesting.
//!
//! Compares:
//! - **Static keypath**: LockKp chain built once, reused (pre-built)
//! - **Dynamic keypath**: LockKp chain built each iteration
//! - **Direct lock acquire**: Manual .read() through 10 levels

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use key_paths_derive::Kp;
use std::sync::{Arc, RwLock};

// 10-level deep: L0 -> Arc<std::sync::RwLock<L1>> -> L1 -> ... -> L10 { leaf: i32 }
// Use full path std::sync::RwLock so derive emits StdArcRwLock (inner_lock) not parking_lot
#[derive(Clone, Kp)]
struct L0 {
    inner: Arc<std::sync::RwLock<L1>>,
}
#[derive(Clone, Kp)]
struct L1 {
    inner: Arc<std::sync::RwLock<L2>>,
}
#[derive(Clone, Kp)]
struct L2 {
    inner: Arc<std::sync::RwLock<L3>>,
}
#[derive(Clone, Kp)]
struct L3 {
    inner: Arc<std::sync::RwLock<L4>>,
}
#[derive(Clone, Kp)]
struct L4 {
    inner: Arc<std::sync::RwLock<L5>>,
}
#[derive(Clone, Kp)]
struct L5 {
    inner: Arc<std::sync::RwLock<L6>>,
}
#[derive(Clone, Kp)]
struct L6 {
    inner: Arc<std::sync::RwLock<L7>>,
}
#[derive(Clone, Kp)]
struct L7 {
    inner: Arc<std::sync::RwLock<L8>>,
}
#[derive(Clone, Kp)]
struct L8 {
    inner: Arc<std::sync::RwLock<L9>>,
}
#[derive(Clone, Kp)]
struct L9 {
    inner: Arc<std::sync::RwLock<L10>>,
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

fn build_read_chain() -> impl Fn(&L0) -> Option<&f64> {
    let chain = L0::inner_lock()
        .then_lock(L1::inner_lock())
        .then_lock(L2::inner_lock())
        .then_lock(L3::inner_lock())
        .then_lock(L4::inner_lock())
        .then_lock(L5::inner_lock())
        .then_lock(L6::inner_lock())
        .then_lock(L7::inner_lock())
        .then_lock(L8::inner_lock())
        .then_lock(L9::inner_lock())
        .then(L10::leaf());

    move |root: &L0| chain.get(root)
}

#[inline(never)]
fn build_chain_once() -> impl Fn(&L0) -> Option<&f64> {
    build_read_chain()
}

fn bench_ten_level_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("ten_level_std_rwlock_read");

    group.bench_function("keypath_static", |b| {
        let chain = build_chain_once();
        let root = make_root();
        b.iter(|| {
            let result = chain(black_box(&root));
            black_box(result)
        })
    });

    group.bench_function("keypath_dynamic", |b| {
        let root = make_root();
        b.iter(|| {
            let chain = build_read_chain();
            let result = chain(black_box(&root));
            black_box(result)
        })
    });

    group.bench_function("direct_lock", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let g1 = root_ref.inner.read().unwrap();
            let g2 = g1.inner.read().unwrap();
            let g3 = g2.inner.read().unwrap();
            let g4 = g3.inner.read().unwrap();
            let g5 = g4.inner.read().unwrap();
            let g6 = g5.inner.read().unwrap();
            let g7 = g6.inner.read().unwrap();
            let g8 = g7.inner.read().unwrap();
            let g9 = g8.inner.read().unwrap();
            let g10 = g9.inner.read().unwrap();
            black_box(g10.leaf)
        })
    });

    group.finish();
}

fn bench_ten_level_incr(c: &mut Criterion) {
    let mut group = c.benchmark_group("ten_level_std_rwlock_incr");

    group.bench_function("keypath_static", |b| {
        let chain = L0::inner_lock()
            .then_lock(L1::inner_lock())
            .then_lock(L2::inner_lock())
            .then_lock(L3::inner_lock())
            .then_lock(L4::inner_lock())
            .then_lock(L5::inner_lock())
            .then_lock(L6::inner_lock())
            .then_lock(L7::inner_lock())
            .then_lock(L8::inner_lock())
            .then_lock(L9::inner_lock())
            .then(L10::leaf());
        let mut root = make_root();
        b.iter(|| {
            let _ = chain.set(black_box(&mut root), |v| *v += 0.25);
        })
    });

    group.bench_function("keypath_dynamic", |b| {
        let mut root = make_root();
        b.iter(|| {
            let chain = L0::inner_lock()
                .then_lock(L1::inner_lock())
                .then_lock(L2::inner_lock())
                .then_lock(L3::inner_lock())
                .then_lock(L4::inner_lock())
                .then_lock(L5::inner_lock())
                .then_lock(L6::inner_lock())
                .then_lock(L7::inner_lock())
                .then_lock(L8::inner_lock())
                .then_lock(L9::inner_lock())
                .then(L10::leaf());
            let _ = chain.set(black_box(&mut root), |v| *v += 0.25);
        })
    });

    group.bench_function("direct_lock", |b| {
        let mut root = make_root();
        b.iter(|| {
            let root_ref = black_box(&mut root);
            let mut g1 = root_ref.inner.write().unwrap();
            let mut g2 = g1.inner.write().unwrap();
            let mut g3 = g2.inner.write().unwrap();
            let mut g4 = g3.inner.write().unwrap();
            let mut g5 = g4.inner.write().unwrap();
            let mut g6 = g5.inner.write().unwrap();
            let mut g7 = g6.inner.write().unwrap();
            let mut g8 = g7.inner.write().unwrap();
            let mut g9 = g8.inner.write().unwrap();
            let mut g10 = g9.inner.write().unwrap();
            g10.leaf += 0.25;
        })
    });

    group.finish();
}

criterion_group!(benches, bench_ten_level_read, bench_ten_level_incr);
criterion_main!(benches);
