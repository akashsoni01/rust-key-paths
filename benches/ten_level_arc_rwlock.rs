//! Benchmark: 10-level deep Arc<parking_lot::RwLock<T>> nesting.
//!
//! Compares:
//! - **Static keypath**: LockKp chain built once, reused (pre-built)
//! - **Dynamic keypath**: LockKp chain built each iteration
//! - **Direct lock acquire**: Manual .read() through 10 levels

#![cfg(feature = "parking_lot")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use key_paths_derive::Kp;
use std::sync::Arc;

use parking_lot::RwLock;

// 10-level deep: L0 -> Arc<RwLock<L1>> -> L1 -> ... -> L10 { leaf: i32 }
#[derive(Clone, Kp)]
struct L0 {
    inner: Arc<RwLock<L1>>,
}
#[derive(Clone, Kp)]
struct L1 {
    inner: Arc<RwLock<L2>>,
}
#[derive(Clone, Kp)]
struct L2 {
    inner: Arc<RwLock<L3>>,
}
#[derive(Clone, Kp)]
struct L3 {
    inner: Arc<RwLock<L4>>,
}
#[derive(Clone, Kp)]
struct L4 {
    inner: Arc<RwLock<L5>>,
}
#[derive(Clone, Kp)]
struct L5 {
    inner: Arc<RwLock<L6>>,
}
#[derive(Clone, Kp)]
struct L6 {
    inner: Arc<RwLock<L7>>,
}
#[derive(Clone, Kp)]
struct L7 {
    inner: Arc<RwLock<L8>>,
}
#[derive(Clone, Kp)]
struct L8 {
    inner: Arc<RwLock<L9>>,
}
#[derive(Clone, Kp)]
struct L9 {
    inner: Arc<RwLock<L10>>,
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

/// Build the 10-level LockKp chain (read path)
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

/// Build and return the chain (for static reuse - caller stores it)
#[inline(never)]
fn build_chain_once() -> impl Fn(&L0) -> Option<&i32> {
    build_read_chain()
}

fn bench_ten_level_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("ten_level_arc_rwlock_read");

    // Static keypath: build chain ONCE, reuse
    group.bench_function("keypath_static", |b| {
        let chain = build_chain_once();
        let root = make_root();
        b.iter(|| {
            let result = chain(black_box(&root));
            black_box(result)
        })
    });

    // Dynamic keypath: build chain each iteration
    group.bench_function("keypath_dynamic", |b| {
        let root = make_root();
        b.iter(|| {
            let chain = build_read_chain();
            let result = chain(black_box(&root));
            black_box(result)
        })
    });

    // Direct lock acquire: manual .read() through 10 levels (guards kept in scope)
    group.bench_function("direct_lock", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let g1 = root_ref.inner.read();
            let g2 = g1.inner.read();
            let g3 = g2.inner.read();
            let g4 = g3.inner.read();
            let g5 = g4.inner.read();
            let g6 = g5.inner.read();
            let g7 = g6.inner.read();
            let g8 = g7.inner.read();
            let g9 = g8.inner.read();
            let g10 = g9.inner.read();
            black_box(g10.leaf)
        })
    });

    group.finish();
}

criterion_group! {
    benches,
    bench_ten_level_read,
}
criterion_main!(benches);
