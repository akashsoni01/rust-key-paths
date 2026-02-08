//! Benchmark: reading and writing the deepest value (leaf) through a plain Kp chain (no locks).
//!
//! Structure: Root -> L1 -> L2 -> L3 -> leaf i32
//!
//! Compares:
//! - Keypath approach: Kp.then().then().then() chain
//! - Direct field access: root.l1.inner.leaf

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_key_paths::Kp;

// Root -> Level1
#[derive(Clone)]
struct Root {
    l1: Level1,
}

// L1 -> Level2
#[derive(Clone)]
struct Level1 {
    inner: Level2,
}

// L2 -> Level3
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
        l1: Level1 {
            inner: Level2 {
                inner: Level3 { leaf: 42 },
            },
        },
    }
}

#[inline(never)]
fn build_and_get(root: &Root) -> Option<&i32> {
    let kp_l1: rust_key_paths::KpType<Root, Level1> =
        Kp::new(|r: &Root| Some(&r.l1), |r: &mut Root| Some(&mut r.l1));
    let kp_l2: rust_key_paths::KpType<Level1, Level2> =
        Kp::new(|l: &Level1| Some(&l.inner), |l: &mut Level1| Some(&mut l.inner));
    let kp_l3: rust_key_paths::KpType<Level2, Level3> =
        Kp::new(|l: &Level2| Some(&l.inner), |l: &mut Level2| Some(&mut l.inner));
    let kp_leaf: rust_key_paths::KpType<Level3, i32> =
        Kp::new(|l: &Level3| Some(&l.leaf), |l: &mut Level3| Some(&mut l.leaf));
    let step1 = kp_l1.then(kp_l2);
    let step2 = step1.then(kp_l3);
    let chain = step2.then(kp_leaf);
    chain.get(root)
}

#[inline(never)]
fn build_and_get_mut(root: &mut Root) -> Option<&mut i32> {
    let kp_l1: rust_key_paths::KpType<Root, Level1> =
        Kp::new(|r: &Root| Some(&r.l1), |r: &mut Root| Some(&mut r.l1));
    let kp_l2: rust_key_paths::KpType<Level1, Level2> =
        Kp::new(|l: &Level1| Some(&l.inner), |l: &mut Level1| Some(&mut l.inner));
    let kp_l3: rust_key_paths::KpType<Level2, Level3> =
        Kp::new(|l: &Level2| Some(&l.inner), |l: &mut Level2| Some(&mut l.inner));
    let kp_leaf: rust_key_paths::KpType<Level3, i32> =
        Kp::new(|l: &Level3| Some(&l.leaf), |l: &mut Level3| Some(&mut l.leaf));
    let step1 = kp_l1.then(kp_l2);
    let step2 = step1.then(kp_l3);
    let chain = step2.then(kp_leaf);
    chain.get_mut(root)
}

fn bench_kp_plain_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("kp_plain_read");

    // Keypath approach
    group.bench_function("keypath", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let leaf = build_and_get(root_ref);
            black_box(leaf);
        })
    });

    // Direct field access
    group.bench_function("direct", |b| {
        let root = make_root();
        b.iter(|| {
            let root_ref = black_box(&root);
            let leaf = &root_ref.l1.inner.inner.leaf;
            black_box(leaf);
        })
    });

    group.finish();
}

fn bench_kp_plain_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("kp_plain_write");

    // Keypath approach
    group.bench_function("keypath", |b| {
        let mut root = make_root();
        b.iter(|| {
            if let Some(l) = build_and_get_mut(black_box(&mut root)) {
                *l = 99;
            }
        })
    });

    // Direct field access
    group.bench_function("direct", |b| {
        let mut root = make_root();
        b.iter(|| {
            root.l1.inner.inner.leaf = 99;
            black_box(&mut root.l1.inner.inner.leaf);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_kp_plain_read, bench_kp_plain_write);
criterion_main!(benches);
