//! Benchmark: reading and writing the deepest value (leaf) through async locks only.
//!
//! Structure: Root -> Arc<TokioMutex<L1>> -> L1 -> L2 -> L3 -> leaf i32
//! (One async Mutex level, same depth as deep_chain_leaf but async-only)
//!
//! Compares:
//! - Keypath approach: Kp.then_async(AsyncLockKp).then().then() chain
//! - Direct lock approach: tokio_mutex.lock().await, then access leaf

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_key_paths::async_lock::{AsyncLockKp, TokioMutexAccess};
use rust_key_paths::Kp;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Root -> Arc<TokioMutex<L1>>
#[derive(Clone)]
struct Root {
    tokio_mutex: Arc<tokio::sync::Mutex<Level1>>,
}

// L1 -> Level2 (plain)
#[derive(Clone)]
struct Level1 {
    inner: Level2,
}

// L2 -> Level3 (plain)
#[derive(Clone)]
struct Level2 {
    inner: Level3,
}

// L3 -> leaf i32
#[derive(Clone)]
struct Level3 {
    leaf: i32,
}

fn make_root() -> Root {
    Root {
        tokio_mutex: Arc::new(tokio::sync::Mutex::new(Level1 {
            inner: Level2 {
                inner: Level3 { leaf: 42 },
            },
        })),
    }
}

#[inline(never)]
fn build_and_get<'a>(root: &'a Root, rt: &Runtime) -> Option<&'a i32> {
    let prev: rust_key_paths::KpType<
        Arc<tokio::sync::Mutex<Level1>>,
        Arc<tokio::sync::Mutex<Level1>>,
    > = Kp::new(|t: &_| Some(t), |t: &mut _| Some(t));
    let next: rust_key_paths::KpType<Level1, i32> = Kp::new(
        |l: &Level1| Some(&l.inner.inner.leaf),
        |l: &mut Level1| Some(&mut l.inner.inner.leaf),
    );
    let async_l1 = AsyncLockKp::new(prev, TokioMutexAccess::new(), next);

    let kp_root_to_lock: rust_key_paths::KpType<Root, Arc<tokio::sync::Mutex<Level1>>> =
        Kp::new(
            |r: &Root| Some(&r.tokio_mutex),
            |r: &mut Root| Some(&mut r.tokio_mutex),
        );

    let step1 = kp_root_to_lock.then_async(async_l1);
    rt.block_on(step1.get(root))
}

#[inline(never)]
fn build_and_get_mut<'a>(root: &'a mut Root, rt: &Runtime) -> Option<&'a mut i32> {
    let prev: rust_key_paths::KpType<
        Arc<tokio::sync::Mutex<Level1>>,
        Arc<tokio::sync::Mutex<Level1>>,
    > = Kp::new(|t: &_| Some(t), |t: &mut _| Some(t));
    let next: rust_key_paths::KpType<Level1, i32> = Kp::new(
        |l: &Level1| Some(&l.inner.inner.leaf),
        |l: &mut Level1| Some(&mut l.inner.inner.leaf),
    );
    let async_l1 = AsyncLockKp::new(prev, TokioMutexAccess::new(), next);

    let kp_root_to_lock: rust_key_paths::KpType<Root, Arc<tokio::sync::Mutex<Level1>>> =
        Kp::new(
            |r: &Root| Some(&r.tokio_mutex),
            |r: &mut Root| Some(&mut r.tokio_mutex),
        );

    let step1 = kp_root_to_lock.then_async(async_l1);
    rt.block_on(step1.get_mut(root))
}

fn bench_deep_chain_async_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("deep_chain_async_read");

    group.bench_function("keypath", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let leaf = build_and_get(root_ref, &rt);
            black_box(leaf);
        })
    });

    group.bench_function("direct_locks", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            rt.block_on(async {
                let guard = root_ref.tokio_mutex.lock().await;
                let leaf = &guard.inner.inner.leaf;
                black_box(leaf);
            })
        })
    });

    group.finish();
}

fn bench_deep_chain_async_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("deep_chain_async_write");

    group.bench_function("keypath", |b| {
        let mut root = make_root();
        b.iter(|| {
            if let Some(l) = build_and_get_mut(black_box(&mut root), &rt) {
                *l = 99;
            }
        })
    });

    group.bench_function("direct_locks", |b| {
        let mut root = make_root();
        b.iter(|| {
            rt.block_on(async {
                let mut guard = root.tokio_mutex.lock().await;
                guard.inner.inner.leaf = 99;
                black_box(&mut guard.inner.inner.leaf);
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_deep_chain_async_read,
    bench_deep_chain_async_write,
);
criterion_main!(benches);
