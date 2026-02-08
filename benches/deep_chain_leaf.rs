//! Benchmark: reading and writing the deepest value (leaf) through a deep nested chain.
//!
//! Structure: Root -> Arc<Mutex<L1>> (sync) -> L1 -> L2 -> Arc<TokioMutex<L3>> (async) -> L3 -> leaf i32
//!
//! Compares:
//! - Direct lock approach: sync_mutex.lock(), tokio_mutex.lock().await, then access leaf
//! - Keypath approach: LockKp.then().then().then_async().then() chain

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::{Arc, Mutex};

#[cfg(all(feature = "tokio", feature = "parking_lot"))]
use rust_key_paths::async_lock::{AsyncLockKp, TokioMutexAccess};
#[cfg(all(feature = "tokio", feature = "parking_lot"))]
use rust_key_paths::lock::{ArcMutexAccess, LockKp};
#[cfg(all(feature = "tokio", feature = "parking_lot"))]
use rust_key_paths::Kp;
#[cfg(all(feature = "tokio", feature = "parking_lot"))]
use tokio::runtime::Runtime;

#[cfg(all(feature = "tokio", feature = "parking_lot"))]
mod benches {
    use super::*;

    // Root -> Arc<Mutex<L1>>
    #[derive(Clone)]
    pub struct Root {
        pub sync_mutex: Arc<Mutex<Level1>>,
    }

    // L1 -> Level2 (plain)
    #[derive(Clone)]
    pub struct Level1 {
        pub inner: Level2,
    }

    // L2 -> Arc<TokioMutex<Level3>>
    #[derive(Clone)]
    pub struct Level2 {
        pub tokio_mutex: Arc<tokio::sync::Mutex<Level3>>,
    }

    // L3 -> leaf i32
    #[derive(Clone)]
    pub struct Level3 {
        pub leaf: i32,
    }

    pub fn make_root() -> Root {
        Root {
            sync_mutex: Arc::new(Mutex::new(Level1 {
                inner: Level2 {
                    tokio_mutex: Arc::new(tokio::sync::Mutex::new(Level3 { leaf: 42 })),
                },
            })),
        }
    }

    /// Build the deep keypath chain: LockKp(Root->L1).then(L1->L2).then(L2->tokio).then_async(tokio->L3).then(L3->leaf)
    #[inline(never)]
    pub fn build_and_get<'a>(root: &'a Root, rt: &Runtime) -> Option<&'a i32> {
        let identity_l1: rust_key_paths::KpType<Level1, Level1> =
            Kp::new(|l: &Level1| Some(l), |l: &mut Level1| Some(l));
        let kp_sync: rust_key_paths::KpType<Root, Arc<Mutex<Level1>>> =
            Kp::new(|r: &Root| Some(&r.sync_mutex), |r: &mut Root| Some(&mut r.sync_mutex));
        let lock_root_to_l1 = LockKp::new(kp_sync, ArcMutexAccess::new(), identity_l1);

        let kp_l1_inner: rust_key_paths::KpType<Level1, Level2> =
            Kp::new(|l: &Level1| Some(&l.inner), |l: &mut Level1| Some(&mut l.inner));

        let kp_l2_tokio: rust_key_paths::KpType<Level2, Arc<tokio::sync::Mutex<Level3>>> =
            Kp::new(
                |l: &Level2| Some(&l.tokio_mutex),
                |l: &mut Level2| Some(&mut l.tokio_mutex),
            );

        let async_l3 = {
            let prev: rust_key_paths::KpType<
                Arc<tokio::sync::Mutex<Level3>>,
                Arc<tokio::sync::Mutex<Level3>>,
            > = Kp::new(|t: &_| Some(t), |t: &mut _| Some(t));
            let next: rust_key_paths::KpType<Level3, Level3> =
                Kp::new(|l: &Level3| Some(l), |l: &mut Level3| Some(l));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        let kp_l3_leaf: rust_key_paths::KpType<Level3, i32> =
            Kp::new(|l: &Level3| Some(&l.leaf), |l: &mut Level3| Some(&mut l.leaf));

        let step1 = lock_root_to_l1.then(kp_l1_inner);
        let step2 = step1.then(kp_l2_tokio);
        let step3 = step2.then_async(async_l3);
        let deep_chain = step3.then(kp_l3_leaf);
        rt.block_on(deep_chain.get(root))
    }

    /// Build the deep keypath chain and mutate leaf
    #[inline(never)]
    pub fn build_and_get_mut<'a>(root: &'a mut Root, rt: &Runtime) -> Option<&'a mut i32> {
        let identity_l1: rust_key_paths::KpType<Level1, Level1> =
            Kp::new(|l: &Level1| Some(l), |l: &mut Level1| Some(l));
        let kp_sync: rust_key_paths::KpType<Root, Arc<Mutex<Level1>>> =
            Kp::new(|r: &Root| Some(&r.sync_mutex), |r: &mut Root| Some(&mut r.sync_mutex));
        let lock_root_to_l1 = LockKp::new(kp_sync, ArcMutexAccess::new(), identity_l1);

        let kp_l1_inner: rust_key_paths::KpType<Level1, Level2> =
            Kp::new(|l: &Level1| Some(&l.inner), |l: &mut Level1| Some(&mut l.inner));

        let kp_l2_tokio: rust_key_paths::KpType<Level2, Arc<tokio::sync::Mutex<Level3>>> =
            Kp::new(
                |l: &Level2| Some(&l.tokio_mutex),
                |l: &mut Level2| Some(&mut l.tokio_mutex),
            );

        let async_l3 = {
            let prev: rust_key_paths::KpType<
                Arc<tokio::sync::Mutex<Level3>>,
                Arc<tokio::sync::Mutex<Level3>>,
            > = Kp::new(|t: &_| Some(t), |t: &mut _| Some(t));
            let next: rust_key_paths::KpType<Level3, Level3> =
                Kp::new(|l: &Level3| Some(l), |l: &mut Level3| Some(l));
            AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
        };

        let kp_l3_leaf: rust_key_paths::KpType<Level3, i32> =
            Kp::new(|l: &Level3| Some(&l.leaf), |l: &mut Level3| Some(&mut l.leaf));

        let step1 = lock_root_to_l1.then(kp_l1_inner);
        let step2 = step1.then(kp_l2_tokio);
        let step3 = step2.then_async(async_l3);
        let deep_chain = step3.then(kp_l3_leaf);
        rt.block_on(deep_chain.get_mut(root))
    }
}

#[cfg(all(feature = "tokio", feature = "parking_lot"))]
fn bench_deep_chain_leaf_read(c: &mut Criterion) {
    use crate::benches::{build_and_get, make_root, Level3};

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("deep_chain_leaf_read");

    // Keypath approach
    group.bench_function("keypath", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let leaf = build_and_get(root_ref, &rt);
            black_box(leaf);
        })
    });

    // Direct lock approach
    group.bench_function("direct_locks", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            rt.block_on(async {
                let tokio_mutex: Arc<tokio::sync::Mutex<Level3>> = {
                    let guard = root_ref.sync_mutex.lock().unwrap();
                    Arc::clone(&guard.inner.tokio_mutex)
                };
                let guard = tokio_mutex.lock().await;
                black_box(&guard.leaf);
            })
        })
    });

    group.finish();
}

#[cfg(all(feature = "tokio", feature = "parking_lot"))]
fn bench_deep_chain_leaf_write(c: &mut Criterion) {
    use crate::benches::{build_and_get_mut, make_root, Level3};

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("deep_chain_leaf_write");

    // Keypath approach
    group.bench_function("keypath", |b| {
        let mut root = make_root();
        b.iter(|| {
            if let Some(l) = build_and_get_mut(black_box(&mut root), &rt) {
                *l = 99;
            }
        })
    });

    // Direct lock approach
    group.bench_function("direct_locks", |b| {
        let root = make_root();
        b.iter(|| {
            rt.block_on(async {
                let tokio_mutex: Arc<tokio::sync::Mutex<Level3>> = {
                    let guard = root.sync_mutex.lock().unwrap();
                    Arc::clone(&guard.inner.tokio_mutex)
                };
                let mut guard = tokio_mutex.lock().await;
                guard.leaf = 99;
                black_box(&mut guard.leaf);
            })
        })
    });

    group.finish();
}

#[cfg(all(feature = "tokio", feature = "parking_lot"))]
criterion_group!(
    benches,
    bench_deep_chain_leaf_read,
    bench_deep_chain_leaf_write,
);

#[cfg(not(all(feature = "tokio", feature = "parking_lot")))]
fn bench_dummy(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_chain_leaf");
    group.bench_function("skipped_no_tokio_parking_lot", |b| b.iter(|| 0_u64));
    group.finish();
}

#[cfg(not(all(feature = "tokio", feature = "parking_lot")))]
criterion_group!(benches, bench_dummy);

criterion_main!(benches);
