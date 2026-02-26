//! Example: GPU-scale validations and calculations using [key-paths-iter] [scale_par].
//!
//! Demonstrates parallel validation of compute state (nodes, active pairs), data extraction
//! for GPU transfer, adaptive dispatch config, and parallel buffer scaling — all via keypaths.

use key_paths_iter::scale_par::{
    adaptive_gpu_dispatch, count_nodes_by_kind, extract_gpu_data, par_flat_map_buffer_data,
    par_scale_buffers, par_validate_buffers_non_empty, preprocess_sort_pairs, process_gpu_results,
    slice_collection, validate_for_gpu, GpuBuffer, GpuComputePipeline, InteractionNet, NetNode,
    NodeKind,
};
use rust_key_paths::Kp;
use rust_key_paths::KpType;

fn main() {
    // Keypaths into GpuComputePipeline (manual Kp::new; in real code you could #[derive(Kp)] on your own type)
    let nodes_kp: KpType<'static, GpuComputePipeline, Vec<NetNode>> = Kp::new(
        |p: &GpuComputePipeline| Some(&p.reduction_net.nodes),
        |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.nodes),
    );
    let pairs_kp: KpType<'static, GpuComputePipeline, Vec<(u32, u32)>> = Kp::new(
        |p: &GpuComputePipeline| Some(&p.reduction_net.active_pairs),
        |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.active_pairs),
    );

    // Build a pipeline: CPU state with buffers + reduction net (nodes + active pairs)
    let mut pipeline = GpuComputePipeline {
        cpu_state: key_paths_iter::scale_par::ComputeState {
            buffers: vec![
                GpuBuffer {
                    data: vec![1.0_f32; 100],
                    size: 100,
                },
                GpuBuffer {
                    data: vec![2.0_f32; 200],
                    size: 200,
                },
            ],
            kernels: vec![],
            results: vec![],
            metadata: Default::default(),
        },
        gpu_buffer_ids: vec![0, 1],
        reduction_net: InteractionNet {
            nodes: (0..500)
                .map(|i| NetNode {
                    kind: if i % 3 == 0 {
                        NodeKind::Dup
                    } else if i % 3 == 1 {
                        NodeKind::Era
                    } else {
                        NodeKind::Ref
                    },
                    ports: [i as u32 % 100, (i + 1) as u32 % 100, (i + 2) as u32 % 100],
                })
                .collect(),
            active_pairs: (0..2000).map(|i| (i as u32 % 500, (i + 1) as u32 % 500)).collect(),
        },
    };

    println!("=== scale_par: validations and GPU-style calculations ===\n");

    // 1) Parallel validation before GPU transfer
    match validate_for_gpu(&pipeline, &nodes_kp, &pairs_kp, 1000) {
        Ok(()) => println!("Validation OK: nodes and active pairs valid for GPU."),
        Err(e) => println!("Validation failed: {}", e),
    }

    // 2) Adaptive GPU dispatch from keypath queries
    let config = adaptive_gpu_dispatch(&pipeline, &nodes_kp, &pairs_kp);
    println!(
        "Dispatch config: workgroup_size={}, workgroup_count={}, use_local_memory={}",
        config.workgroup_size, config.workgroup_count, config.use_local_memory
    );

    // 3) Extract data for GPU transfer (keypath-based)
    let gpu_data = extract_gpu_data(&pipeline, &nodes_kp, &pairs_kp);
    println!(
        "Extracted for GPU: {} nodes, {} pairs (node_count={})",
        gpu_data.nodes.len(),
        gpu_data.pairs.len(),
        gpu_data.node_count
    );

    // 4) Preprocess: parallel sort active pairs in place
    preprocess_sort_pairs(&mut pipeline, &pairs_kp);
    println!("Preprocessed: active_pairs sorted in place (parallel).");

    // 5) Slice for batching (e.g. stream to GPU in chunks)
    let batch = slice_collection(&pairs_kp, &pipeline, 0, 100);
    println!("First batch of pairs (slice 0..100): {} pairs.", batch.len());

    // 6) Keypath to buffers: we need a keypath to pipeline.cpu_state.buffers
    let buffers_kp: KpType<'static, GpuComputePipeline, Vec<GpuBuffer>> = Kp::new(
        |p: &GpuComputePipeline| Some(&p.cpu_state.buffers),
        |p: &mut GpuComputePipeline| Some(&mut p.cpu_state.buffers),
    );

    // Parallel validation: all buffers non-empty
    let buffers_ok = par_validate_buffers_non_empty(&buffers_kp, &pipeline);
    println!("Buffers validation (par_all non-empty): {}.", buffers_ok);

    // Parallel calculation: scale all buffer data (simulate GPU-style transform on CPU)
    par_scale_buffers(&buffers_kp, &mut pipeline, 2.0);
    let flat: Vec<f32> = par_flat_map_buffer_data(&buffers_kp, &pipeline);
    println!(
        "After par_scale_buffers(2.0): flat data len={}, first few: {:?}",
        flat.len(),
        &flat[..5.min(flat.len())]
    );

    // 7) Simulate GPU results write-back
    let mut updated_nodes = gpu_data.nodes.clone();
    updated_nodes.truncate(10);
    process_gpu_results(
        &mut pipeline,
        &nodes_kp,
        key_paths_iter::scale_par::GpuBufferData {
            nodes: updated_nodes,
            pairs: gpu_data.pairs,
            node_count: gpu_data.node_count,
        },
    );
    println!("Wrote back GPU results (process_gpu_results) into pipeline nodes.");

    // 8) Post-GPU analysis: parallel count by node kind
    let era_count = count_nodes_by_kind(&nodes_kp, &pipeline, NodeKind::Era);
    let dup_count = count_nodes_by_kind(&nodes_kp, &pipeline, NodeKind::Dup);
    println!(
        "Post-GPU counts (par_count_by): Era={}, Dup={}",
        era_count, dup_count
    );

    println!("\nDone: validations and calculations on GPU-style data via keypaths.");
}
