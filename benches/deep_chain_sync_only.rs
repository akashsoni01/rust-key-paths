//! Benchmark: reading and writing the deepest value (leaf) through sync locks only.
//!
//! Structure: Root -> Arc<Mutex<L1>> -> L1 -> L2 -> Arc<Mutex<L3>> -> L3 -> leaf i32
//!
//! Compares:
//! - Keypath approach: LockKp.then().then_lock().then() chain (two sync Mutex levels)
//! - Direct lock approach: sync_mutex1.lock(), sync_mutex2.lock(), then access leaf

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_key_paths::lock::{ArcMutexAccess, LockKp};
use rust_key_paths::Kp;
use std::sync::{Arc, Mutex};

// Root -> Arc<Mutex<L1>>
#[derive(Clone)]
struct Root {
    sync_mutex_1: Arc<Mutex<Level1>>,
}

// L1 -> Level2 (plain)
#[derive(Clone)]
struct Level1 {
    inner: Level2,
}

// L2 -> Arc<Mutex<L3>>
#[derive(Clone)]
struct Level2 {
    sync_mutex_2: Arc<Mutex<Level3>>,
}

// L3 -> leaf i32
#[derive(Clone)]
struct Level3 {
    leaf: i32,
}

fn make_root() -> Root {
    Root {
        sync_mutex_1: Arc::new(Mutex::new(Level1 {
            inner: Level2 {
                sync_mutex_2: Arc::new(Mutex::new(Level3 { leaf: 42 })),
            },
        })),
    }
}

#[inline(never)]
fn build_and_get(root: &Root) -> Option<&i32> {
    let identity_l1: rust_key_paths::KpType<Level1, Level1> =
        Kp::new(|l: &Level1| Some(l), |l: &mut Level1| Some(l));
    let kp_sync1: rust_key_paths::KpType<Root, Arc<Mutex<Level1>>> =
        Kp::new(|r: &Root| Some(&r.sync_mutex_1), |r: &mut Root| Some(&mut r.sync_mutex_1));
    let lock_root_to_l1 = LockKp::new(kp_sync1, ArcMutexAccess::new(), identity_l1);

    let kp_l1_inner: rust_key_paths::KpType<Level1, Level2> =
        Kp::new(|l: &Level1| Some(&l.inner), |l: &mut Level1| Some(&mut l.inner));

    let identity_l3: rust_key_paths::KpType<Level3, Level3> =
        Kp::new(|l: &Level3| Some(l), |l: &mut Level3| Some(l));
    let kp_sync2: rust_key_paths::KpType<Level2, Arc<Mutex<Level3>>> =
        Kp::new(
            |l: &Level2| Some(&l.sync_mutex_2),
            |l: &mut Level2| Some(&mut l.sync_mutex_2),
        );
    let lock_l2_to_l3 = LockKp::new(kp_sync2, ArcMutexAccess::new(), identity_l3);

    let kp_l3_leaf: rust_key_paths::KpType<Level3, i32> =
        Kp::new(|l: &Level3| Some(&l.leaf), |l: &mut Level3| Some(&mut l.leaf));

    let step1 = lock_root_to_l1.then(kp_l1_inner);
    let step2 = step1.then_lock(lock_l2_to_l3);
    let chain = step2.then(kp_l3_leaf);
    chain.get(root)
}

#[inline(never)]
fn build_and_get_mut(root: &mut Root) -> Option<&mut i32> {
    let identity_l1: rust_key_paths::KpType<Level1, Level1> =
        Kp::new(|l: &Level1| Some(l), |l: &mut Level1| Some(l));
    let kp_sync1: rust_key_paths::KpType<Root, Arc<Mutex<Level1>>> =
        Kp::new(|r: &Root| Some(&r.sync_mutex_1), |r: &mut Root| Some(&mut r.sync_mutex_1));
    let lock_root_to_l1 = LockKp::new(kp_sync1, ArcMutexAccess::new(), identity_l1);

    let kp_l1_inner: rust_key_paths::KpType<Level1, Level2> =
        Kp::new(|l: &Level1| Some(&l.inner), |l: &mut Level1| Some(&mut l.inner));

    let identity_l3: rust_key_paths::KpType<Level3, Level3> =
        Kp::new(|l: &Level3| Some(l), |l: &mut Level3| Some(l));
    let kp_sync2: rust_key_paths::KpType<Level2, Arc<Mutex<Level3>>> =
        Kp::new(
            |l: &Level2| Some(&l.sync_mutex_2),
            |l: &mut Level2| Some(&mut l.sync_mutex_2),
        );
    let lock_l2_to_l3 = LockKp::new(kp_sync2, ArcMutexAccess::new(), identity_l3);

    let kp_l3_leaf: rust_key_paths::KpType<Level3, i32> =
        Kp::new(|l: &Level3| Some(&l.leaf), |l: &mut Level3| Some(&mut l.leaf));

    let step1 = lock_root_to_l1.then(kp_l1_inner);
    let step2 = step1.then_lock(lock_l2_to_l3);
    let chain = step2.then(kp_l3_leaf);
    chain.get_mut(root)
}

fn bench_deep_chain_sync_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_chain_sync_read");

    group.bench_function("keypath", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let leaf = build_and_get(root_ref);
            black_box(leaf);
        })
    });

    group.bench_function("direct_locks", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let guard1 = root_ref.sync_mutex_1.lock().unwrap();
            let guard2 = guard1.inner.sync_mutex_2.lock().unwrap();
            let leaf = &guard2.leaf;
            black_box(leaf);
        })
    });

    group.finish();
}

fn bench_deep_chain_sync_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_chain_sync_write");

    group.bench_function("keypath", |b| {
        let mut root = make_root();
        b.iter(|| {
            if let Some(l) = build_and_get_mut(black_box(&mut root)) {
                *l = 99;
            }
        })
    });

    group.bench_function("direct_locks", |b| {
        let mut root = make_root();
        b.iter(|| {
            let mut guard1 = root.sync_mutex_1.lock().unwrap();
            let mut guard2 = guard1.inner.sync_mutex_2.lock().unwrap();
            guard2.leaf = 99;
            black_box(&mut guard2.leaf);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_deep_chain_sync_read,
    bench_deep_chain_sync_write,
);
criterion_main!(benches);
