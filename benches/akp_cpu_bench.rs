//! Benchmark: run numeric keypath extraction + transform (same as wgpu shader) on CPU
//! sequential, Rayon parallel, and GPU. Two ways to build the numeric keypath:
//! - **closure**: [numeric_akp_f32] (manual extractor)
//! - **Kp/Pkp**: [IntoNumericAKp] with derived keypath (e.g. `User::score().into_numeric_akp(...)`)
//!
//! Run: `cargo bench --bench akp_cpu_bench`
//! Requires key-paths-iter with features = ["rayon", "gpu"] in dev-dependencies.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use key_paths_derive::Kp;
use key_paths_iter::wgpu::{
    cpu_transform_f32, numeric_akp_f32, GpuValue, IntoNumericAKp, NumericAKp, WgpuContext,
};
use rayon::prelude::*;
use std::sync::Arc;

/// Manual struct for closure-based benchmark (no derive).
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct UserManual {
    name: String,
    score: f32,
}

/// Same shape with Kp derive for IntoNumericAKp benchmark.
#[derive(Kp, Clone, Debug)]
#[allow(dead_code)]
struct UserKp {
    name: String,
    score: f32,
}

fn make_users_manual(n: usize) -> Vec<UserManual> {
    (0..n)
        .map(|i| UserManual {
            name: format!("user_{}", i),
            score: (i as f32) * 0.1,
        })
        .collect()
}

fn make_users_kp(n: usize) -> Vec<UserKp> {
    (0..n)
        .map(|i| UserKp {
            name: format!("user_{}", i),
            score: (i as f32) * 0.1,
        })
        .collect()
}

// ─── Sequential: one root at a time, extract + CPU transform ──────────────────

fn run_numeric_sequential(roots: &[impl AsAny], kp: &NumericAKp) -> Vec<f32> {
    let mut out = Vec::with_capacity(roots.len());
    for root in roots {
        let v = (kp.extractor)(root.as_any());
        out.push(match v {
            Some(GpuValue::F32(f)) => cpu_transform_f32(f),
            _ => 0.0,
        });
    }
    out
}

fn run_numeric_rayon(roots: &[impl AsAny + Sync], kp: &NumericAKp) -> Vec<f32> {
    roots
        .par_iter()
        .map(|root| {
            match (kp.extractor)(root.as_any()) {
                Some(GpuValue::F32(f)) => cpu_transform_f32(f),
                _ => 0.0,
            }
        })
        .collect()
}

fn run_numeric_gpu(roots: &[impl AsAny], kp: &NumericAKp, ctx: &WgpuContext) -> Vec<f32> {
    let flat: Vec<f32> = roots
        .iter()
        .map(|root| match (kp.extractor)(root.as_any()) {
            Some(GpuValue::F32(f)) => f,
            _ => 0.0,
        })
        .collect();
    ctx.transform_f32_gpu(&flat).unwrap_or(flat)
}

trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}
impl AsAny for UserManual {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl AsAny for UserKp {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn bench_akp_numeric(c: &mut Criterion) {
    let score_kp = numeric_akp_f32::<UserManual>(|u| Some(u.score), "input * 2.0 + 1.0");
    let score_kp = Arc::new(score_kp);
    let wgpu_ctx = WgpuContext::new().ok();

    let mut group = c.benchmark_group("akp_numeric_transform_closure");
    group.sample_size(50);

    for n_roots in [1_000_usize, 10_000, 50_000, 100_000] {
        group.bench_function(format!("sequential_{}", n_roots), |b| {
            b.iter_batched(
                || make_users_manual(n_roots),
                |roots| run_numeric_sequential(black_box(&roots), black_box(score_kp.as_ref())),
                BatchSize::SmallInput,
            );
        });

        group.bench_function(format!("rayon_parallel_{}", n_roots), |b| {
            b.iter_batched(
                || make_users_manual(n_roots),
                |roots| run_numeric_rayon(black_box(&roots), black_box(score_kp.as_ref())),
                BatchSize::SmallInput,
            );
        });

        if let Some(ref ctx) = wgpu_ctx {
            group.bench_function(format!("gpu_{}", n_roots), |b| {
                b.iter_batched(
                    || make_users_manual(n_roots),
                    |roots| run_numeric_gpu(black_box(&roots), black_box(score_kp.as_ref()), ctx),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();

    // Kp derive + IntoNumericAKp (reference-based; same workload)
    let score_numeric_kp = UserKp::score().into_numeric_akp("input * 2.0 + 1.0");
    let score_numeric_kp = Arc::new(score_numeric_kp);

    let mut group_kp = c.benchmark_group("akp_numeric_transform_from_kp");
    group_kp.sample_size(50);

    for n_roots in [1_000_usize, 10_000, 50_000, 100_000] {
        group_kp.bench_function(format!("sequential_{}", n_roots), |b| {
            b.iter_batched(
                || make_users_kp(n_roots),
                |roots| run_numeric_sequential(black_box(&roots), black_box(score_numeric_kp.as_ref())),
                BatchSize::SmallInput,
            );
        });

        group_kp.bench_function(format!("rayon_parallel_{}", n_roots), |b| {
            b.iter_batched(
                || make_users_kp(n_roots),
                |roots| run_numeric_rayon(black_box(&roots), black_box(score_numeric_kp.as_ref())),
                BatchSize::SmallInput,
            );
        });

        if let Some(ref ctx) = wgpu_ctx {
            group_kp.bench_function(format!("gpu_{}", n_roots), |b| {
                b.iter_batched(
                    || make_users_kp(n_roots),
                    |roots| run_numeric_gpu(black_box(&roots), black_box(score_numeric_kp.as_ref()), ctx),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group_kp.finish();
}

criterion_group!(akp_benches, bench_akp_numeric);
criterion_main!(akp_benches);
