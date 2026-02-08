//! Benchmark: 10-level deep Arc<parking_lot::RwLock<T>> nesting.
//!
//! Compares:
//! - **Static keypath**: LockKp chain built once, reused (pre-built)
//! - **Dynamic keypath**: LockKp chain built each iteration
//! - **Direct lock acquire**: Manual .read() through 10 levels

#![cfg(feature = "parking_lot")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_key_paths::lock::{LockKp, ParkingLotRwLockAccess};
use rust_key_paths::Kp;
use std::sync::Arc;

use parking_lot::RwLock;

// 10-level deep: L0 -> Arc<RwLock<L1>> -> L1 -> ... -> L10 { leaf: i32 }
#[derive(Clone)]
struct L0 {
    inner: Arc<RwLock<L1>>,
}
#[derive(Clone)]
struct L1 {
    inner: Arc<RwLock<L2>>,
}
#[derive(Clone)]
struct L2 {
    inner: Arc<RwLock<L3>>,
}
#[derive(Clone)]
struct L3 {
    inner: Arc<RwLock<L4>>,
}
#[derive(Clone)]
struct L4 {
    inner: Arc<RwLock<L5>>,
}
#[derive(Clone)]
struct L5 {
    inner: Arc<RwLock<L6>>,
}
#[derive(Clone)]
struct L6 {
    inner: Arc<RwLock<L7>>,
}
#[derive(Clone)]
struct L7 {
    inner: Arc<RwLock<L8>>,
}
#[derive(Clone)]
struct L8 {
    inner: Arc<RwLock<L9>>,
}
#[derive(Clone)]
struct L9 {
    inner: Arc<RwLock<L10>>,
}
#[derive(Clone)]
struct L10 {
    leaf: i32,
}

fn make_root() -> L0 {
    let leaf = L10 { leaf: 42 };
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
fn build_read_chain() -> impl Fn(&L0) -> Option<&i32> {
    let kp_leaf: rust_key_paths::KpType<L10, i32> =
        Kp::new(|t: &L10| Some(&t.leaf), |t: &mut L10| Some(&mut t.leaf));

    // L0 -> Arc<RwLock<L1>>
    let kp0: rust_key_paths::KpType<L0, Arc<RwLock<L1>>> =
        Kp::new(|r: &L0| Some(&r.inner), |r: &mut L0| Some(&mut r.inner));
    let id1: rust_key_paths::KpType<L1, L1> =
        Kp::new(|l: &L1| Some(l), |l: &mut L1| Some(l));
    let lock0 = LockKp::new(kp0, ParkingLotRwLockAccess::new(), id1);

    // L1 -> Arc<RwLock<L2>>
    let kp1: rust_key_paths::KpType<L1, Arc<RwLock<L2>>> =
        Kp::new(|r: &L1| Some(&r.inner), |r: &mut L1| Some(&mut r.inner));
    let id2: rust_key_paths::KpType<L2, L2> =
        Kp::new(|l: &L2| Some(l), |l: &mut L2| Some(l));
    let lock1 = LockKp::new(kp1, ParkingLotRwLockAccess::new(), id2);

    // L2 -> L3
    let kp2: rust_key_paths::KpType<L2, Arc<RwLock<L3>>> =
        Kp::new(|r: &L2| Some(&r.inner), |r: &mut L2| Some(&mut r.inner));
    let id3: rust_key_paths::KpType<L3, L3> =
        Kp::new(|l: &L3| Some(l), |l: &mut L3| Some(l));
    let lock2 = LockKp::new(kp2, ParkingLotRwLockAccess::new(), id3);

    // L3 -> L4
    let kp3: rust_key_paths::KpType<L3, Arc<RwLock<L4>>> =
        Kp::new(|r: &L3| Some(&r.inner), |r: &mut L3| Some(&mut r.inner));
    let id4: rust_key_paths::KpType<L4, L4> =
        Kp::new(|l: &L4| Some(l), |l: &mut L4| Some(l));
    let lock3 = LockKp::new(kp3, ParkingLotRwLockAccess::new(), id4);

    // L4 -> L5
    let kp4: rust_key_paths::KpType<L4, Arc<RwLock<L5>>> =
        Kp::new(|r: &L4| Some(&r.inner), |r: &mut L4| Some(&mut r.inner));
    let id5: rust_key_paths::KpType<L5, L5> =
        Kp::new(|l: &L5| Some(l), |l: &mut L5| Some(l));
    let lock4 = LockKp::new(kp4, ParkingLotRwLockAccess::new(), id5);

    // L5 -> L6
    let kp5: rust_key_paths::KpType<L5, Arc<RwLock<L6>>> =
        Kp::new(|r: &L5| Some(&r.inner), |r: &mut L5| Some(&mut r.inner));
    let id6: rust_key_paths::KpType<L6, L6> =
        Kp::new(|l: &L6| Some(l), |l: &mut L6| Some(l));
    let lock5 = LockKp::new(kp5, ParkingLotRwLockAccess::new(), id6);

    // L6 -> L7
    let kp6: rust_key_paths::KpType<L6, Arc<RwLock<L7>>> =
        Kp::new(|r: &L6| Some(&r.inner), |r: &mut L6| Some(&mut r.inner));
    let id7: rust_key_paths::KpType<L7, L7> =
        Kp::new(|l: &L7| Some(l), |l: &mut L7| Some(l));
    let lock6 = LockKp::new(kp6, ParkingLotRwLockAccess::new(), id7);

    // L7 -> L8
    let kp7: rust_key_paths::KpType<L7, Arc<RwLock<L8>>> =
        Kp::new(|r: &L7| Some(&r.inner), |r: &mut L7| Some(&mut r.inner));
    let id8: rust_key_paths::KpType<L8, L8> =
        Kp::new(|l: &L8| Some(l), |l: &mut L8| Some(l));
    let lock7 = LockKp::new(kp7, ParkingLotRwLockAccess::new(), id8);

    // L8 -> L9
    let kp8: rust_key_paths::KpType<L8, Arc<RwLock<L9>>> =
        Kp::new(|r: &L8| Some(&r.inner), |r: &mut L8| Some(&mut r.inner));
    let id9: rust_key_paths::KpType<L9, L9> =
        Kp::new(|l: &L9| Some(l), |l: &mut L9| Some(l));
    let lock8 = LockKp::new(kp8, ParkingLotRwLockAccess::new(), id9);

    // L9 -> L10
    let kp9: rust_key_paths::KpType<L9, Arc<RwLock<L10>>> =
        Kp::new(|r: &L9| Some(&r.inner), |r: &mut L9| Some(&mut r.inner));
    let id10: rust_key_paths::KpType<L10, L10> =
        Kp::new(|l: &L10| Some(l), |l: &mut L10| Some(l));
    let lock9 = LockKp::new(kp9, ParkingLotRwLockAccess::new(), id10);

    let chain = lock0
        .then_lock(lock1)
        .then_lock(lock2)
        .then_lock(lock3)
        .then_lock(lock4)
        .then_lock(lock5)
        .then_lock(lock6)
        .then_lock(lock7)
        .then_lock(lock8)
        .then_lock(lock9)
        .then(kp_leaf);

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
