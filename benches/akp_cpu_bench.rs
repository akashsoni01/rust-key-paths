//! Benchmark: run numeric keypath extraction + transform (same as wgpu shader) on CPU
//! sequential, Rayon parallel, and GPU. Reuses [key_paths_iter::wgpu] prior art (NumericAKp,
//! AKpTier, numeric_akp_f32, User from akp_wgpu_runner).
//!
//! Run: `cargo bench --bench akp_cpu_bench`
//! Requires key-paths-iter with features = ["rayon", "gpu"] in dev-dependencies.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use key_paths_iter::wgpu::{
    cpu_transform_f32, numeric_akp_f32, GpuValue, NumericAKp, WgpuContext,
};
use rayon::prelude::*;
use std::sync::Arc;

/// Same shape as in akp_wgpu_runner example (prior art).
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct User {
    name: String,
    score: f32,
}

fn make_users(n: usize) -> Vec<User> {
    (0..n)
        .map(|i| User {
            name: format!("user_{}", i),
            score: (i as f32) * 0.1,
        })
        .collect()
}

// ─── Sequential: one root at a time, extract + CPU transform ──────────────────

fn run_numeric_sequential(roots: &[User], kp: &NumericAKp) -> Vec<f32> {
    let mut out = Vec::with_capacity(roots.len());
    for root in roots {
        let v = (kp.extractor)(root as &dyn std::any::Any);
        out.push(match v {
            Some(GpuValue::F32(f)) => cpu_transform_f32(f),
            _ => 0.0,
        });
    }
    out
}

// ─── Rayon parallel: one root per unit of work, extract + CPU transform ───────

fn run_numeric_rayon(roots: &[User], kp: &NumericAKp) -> Vec<f32> {
    roots
        .par_iter()
        .map(|root| {
            match (kp.extractor)(root as &dyn std::any::Any) {
                Some(GpuValue::F32(f)) => cpu_transform_f32(f),
                _ => 0.0,
            }
        })
        .collect()
}

// ─── GPU: flat buffer, one dispatch (same math as shader) ────────────────────

fn run_numeric_gpu(roots: &[User], kp: &NumericAKp, ctx: &WgpuContext) -> Vec<f32> {
    let flat: Vec<f32> = roots
        .iter()
        .map(|root| {
            match (kp.extractor)(root as &dyn std::any::Any) {
                Some(GpuValue::F32(f)) => f,
                _ => 0.0,
            }
        })
        .collect();
    ctx.transform_f32_gpu(&flat).unwrap_or(flat)
}

fn bench_akp_numeric(c: &mut Criterion) {
    let score_kp = numeric_akp_f32::<User>(|u| Some(u.score), "input * 2.0 + 1.0");
    let score_kp = Arc::new(score_kp);

    let wgpu_ctx = WgpuContext::new().ok();

    let mut group = c.benchmark_group("akp_numeric_transform");
    group.sample_size(50);

    for n_roots in [1_000_usize, 10_000, 50_000, 100_000] {
        group.bench_function(format!("sequential_{}", n_roots), |b| {
            b.iter_batched(
                || make_users(n_roots),
                |roots| run_numeric_sequential(black_box(&roots), black_box(score_kp.as_ref())),
                BatchSize::SmallInput,
            );
        });

        group.bench_function(format!("rayon_parallel_{}", n_roots), |b| {
            b.iter_batched(
                || make_users(n_roots),
                |roots| run_numeric_rayon(black_box(&roots), black_box(score_kp.as_ref())),
                BatchSize::SmallInput,
            );
        });

        if let Some(ref ctx) = wgpu_ctx {
            group.bench_function(format!("gpu_{}", n_roots), |b| {
                b.iter_batched(
                    || make_users(n_roots),
                    |roots| run_numeric_gpu(black_box(&roots), black_box(score_kp.as_ref()), ctx),
                    BatchSize::SmallInput,
                );
            });
        }
    }

    group.finish();
}

criterion_group!(akp_benches, bench_akp_numeric);
criterion_main!(akp_benches);
