//! Benchmark: scale_par keypath-based parallel vs sequential.
//!
//! Compares:
//! - Buffer scaling: sequential (nested for_each) vs keypath par_scale_buffers
//! - Validation (all non-empty): sequential iter().all vs par_validate_buffers_non_empty
//! - Count by predicate: sequential filter().count vs par_count_by (nodes by kind)

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use key_paths_iter::query_par::ParallelCollectionKeyPath;
use key_paths_iter::scale_par::{
    par_scale_buffers, par_validate_buffers_non_empty, ComputeState, GpuComputePipeline,
    GpuBuffer, InteractionNet, NetNode, NodeKind,
};
use rust_key_paths::Kp;
use rust_key_paths::KpType;

fn make_pipeline(num_buffers: usize, buffer_len: usize, num_nodes: usize, num_pairs: usize) -> GpuComputePipeline {
    GpuComputePipeline {
        cpu_state: ComputeState {
            buffers: (0..num_buffers)
                .map(|_| GpuBuffer {
                    data: vec![1.0_f32; buffer_len],
                    size: buffer_len,
                })
                .collect(),
            kernels: vec![],
            results: vec![],
            metadata: Default::default(),
        },
        gpu_buffer_ids: (0..num_buffers as u32).collect(),
        reduction_net: InteractionNet {
            nodes: (0..num_nodes)
                .map(|i| {
                    let kind = match i % 4 {
                        0 => NodeKind::Era,
                        1 => NodeKind::Con,
                        2 => NodeKind::Dup,
                        _ => NodeKind::Ref,
                    };
                    NetNode::new(kind, [i as u32 % 1000, (i + 1) as u32 % 1000, (i + 2) as u32 % 1000])
                })
                .collect(),
            active_pairs: (0..num_pairs).map(|i| (i as u32 % num_nodes as u32, (i + 1) as u32 % num_nodes as u32)).collect(),
        },
    }
}

fn sequential_scale_buffers(pipeline: &mut GpuComputePipeline, scale: f32) {
    for buf in &mut pipeline.cpu_state.buffers {
        for x in &mut buf.data {
            *x *= scale;
        }
    }
}

fn sequential_validate_buffers_non_empty(pipeline: &GpuComputePipeline) -> bool {
    pipeline.cpu_state.buffers.iter().all(|b| !b.data.is_empty())
}

fn sequential_count_nodes_era(pipeline: &GpuComputePipeline) -> usize {
    pipeline.reduction_net.nodes.iter().filter(|n| n.kind() == NodeKind::Era).count()
}

fn bench_buffer_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("scale_par_buffer_scale");
    group.sample_size(50);

    for (num_buffers, buffer_len) in [(100, 1000), (500, 2000), (1000, 1000)] {
        group.bench_function(format!("sequential_{}buf_x{}", num_buffers, buffer_len), |b| {
            b.iter_batched(
                || make_pipeline(num_buffers, buffer_len, 1000, 2000),
                |mut pipeline| {
                    sequential_scale_buffers(black_box(&mut pipeline), 2.0);
                },
                BatchSize::SmallInput,
            );
        });

        let buffers_kp: KpType<'static, GpuComputePipeline, Vec<GpuBuffer>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.cpu_state.buffers),
            |p: &mut GpuComputePipeline| Some(&mut p.cpu_state.buffers),
        );
        group.bench_function(format!("keypath_par_{}buf_x{}", num_buffers, buffer_len), |b| {
            b.iter_batched(
                || make_pipeline(num_buffers, buffer_len, 1000, 2000),
                |mut pipeline| {
                    par_scale_buffers(black_box(&buffers_kp), black_box(&mut pipeline), 2.0);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("scale_par_validation");
    group.sample_size(50);

    for (num_buffers, buffer_len) in [(500, 500), (2000, 500)] {
        let pipeline = make_pipeline(num_buffers, buffer_len, 5000, 10000);

        group.bench_function(
            format!("sequential_all_non_empty_{}buf", num_buffers),
            |b| b.iter(|| sequential_validate_buffers_non_empty(black_box(&pipeline))),
        );

        let buffers_kp: KpType<'static, GpuComputePipeline, Vec<GpuBuffer>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.cpu_state.buffers),
            |p: &mut GpuComputePipeline| Some(&mut p.cpu_state.buffers),
        );
        group.bench_function(
            format!("keypath_par_all_non_empty_{}buf", num_buffers),
            |b| b.iter(|| par_validate_buffers_non_empty(black_box(&buffers_kp), black_box(&pipeline))),
        );
    }

    group.finish();
}

fn bench_count_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("scale_par_count_by");
    group.sample_size(50);

    for num_nodes in [5_000_usize, 50_000, 100_000] {
        let pipeline = make_pipeline(100, 100, num_nodes, num_nodes * 2);

        group.bench_function(format!("sequential_count_era_{}nodes", num_nodes), |b| {
            b.iter(|| sequential_count_nodes_era(black_box(&pipeline)));
        });

        let nodes_kp: KpType<'static, GpuComputePipeline, Vec<NetNode>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.reduction_net.nodes),
            |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.nodes),
        );
        group.bench_function(format!("keypath_par_count_by_era_{}nodes", num_nodes), |b| {
            b.iter(|| nodes_kp.par_count_by(black_box(&pipeline), |n| n.kind() == NodeKind::Era));
        });
    }

    group.finish();
}

criterion_group!(
    scale_par_benches,
    bench_buffer_scale,
    bench_validation,
    bench_count_by,
);
criterion_main!(scale_par_benches);
