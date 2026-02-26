//! GPU-scale parallel validation and calculation using keypaths.
//!
//! This module provides types and helpers for CPU↔GPU-style pipelines: validate
//! collections in parallel via keypaths, extract data for transfer, run parallel
//! pre/post processing, and choose dispatch parameters. Use with
//! [ParallelCollectionKeyPath](crate::query_par::ParallelCollectionKeyPath) and
//! [KpType](rust_key_paths::KpType) (e.g. from `#[derive(Kp)]`).

#![cfg(feature = "rayon")]

use crate::query_par::ParallelCollectionKeyPath;
use rayon::prelude::*;
use rust_key_paths::KpType;

// ══════════════════════════════════════════════════════════════════════════
// 1. CORE TYPES - Compute state and GPU pipeline
// ══════════════════════════════════════════════════════════════════════════

/// CPU-side compute state: buffers, kernels, results, metadata.
#[derive(Clone, Debug, Default)]
pub struct ComputeState {
    pub buffers: Vec<GpuBuffer>,
    pub kernels: Vec<GpuKernel>,
    pub results: Vec<f32>,
    pub metadata: ComputeMetadata,
}

/// Buffer of floats (e.g. for GPU transfer).
#[derive(Clone, Debug)]
pub struct GpuBuffer {
    pub data: Vec<f32>,
    pub size: usize,
}

/// Kernel descriptor (name, workgroup size).
#[derive(Clone, Debug)]
pub struct GpuKernel {
    pub name: String,
    pub workgroup_size: u32,
}

/// Metadata for a compute run.
#[derive(Clone, Debug, Default)]
pub struct ComputeMetadata {
    pub device_id: u32,
    pub timestamp: u64,
}

/// HVM2-style interaction net: nodes and active redex pairs.
#[derive(Clone, Debug, Default)]
pub struct InteractionNet {
    pub nodes: Vec<NetNode>,
    pub active_pairs: Vec<(u32, u32)>,
}

#[derive(Clone, Debug)]
pub struct NetNode {
    pub kind: NodeKind,
    pub ports: [u32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Era,
    Con,
    Dup,
    Ref,
}

/// Full pipeline: CPU state + GPU buffer ids + reduction net.
#[derive(Clone, Debug)]
pub struct GpuComputePipeline {
    pub cpu_state: ComputeState,
    pub gpu_buffer_ids: Vec<u32>,
    pub reduction_net: InteractionNet,
}

/// Data extracted for GPU transfer.
#[derive(Clone, Debug)]
pub struct GpuBufferData {
    pub nodes: Vec<NetNode>,
    pub pairs: Vec<(u32, u32)>,
    pub node_count: u32,
}

/// Suggested GPU dispatch configuration from keypath queries.
#[derive(Clone, Debug)]
pub struct GpuDispatchConfig {
    pub workgroup_size: u32,
    pub workgroup_count: u32,
    pub use_local_memory: bool,
}

// ══════════════════════════════════════════════════════════════════════════
// 2. PARALLEL STRATEGY - When to use parallel vs sequential
// ══════════════════════════════════════════════════════════════════════════

/// Parallel execution strategy for collection operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelStrategy {
    /// Small collections or cheap ops: run sequentially.
    Sequential,
    /// Independent data transformations (map, filter, etc.).
    DataParallel,
    /// Independent function calls.
    TaskParallel,
    /// Tree-like recursive structures.
    Recursive,
}

/// Minimum collection size to consider parallel execution.
pub const DEFAULT_PARALLEL_THRESHOLD: usize = 1000;

/// Chooses parallel strategy based on estimated collection size.
#[inline]
pub fn strategy_for_size(len: usize, threshold: usize) -> ParallelStrategy {
    if len >= threshold {
        ParallelStrategy::DataParallel
    } else {
        ParallelStrategy::Sequential
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 3. KEYPATH-BASED VALIDATION - Parallel validation before GPU transfer
// ══════════════════════════════════════════════════════════════════════════

/// Validates that all nodes have ports within a maximum value (parallel).
pub fn validate_nodes_parallel<Root>(
    nodes_kp: &KpType<'static, Root, Vec<NetNode>>,
    root: &Root,
    max_port: u32,
) -> bool
where
    Root: Send + Sync,
    NetNode: Send + Sync,
{
    nodes_kp.par_all(root, |node| node.ports.iter().all(|&p| p < max_port))
}

/// Validates that all active pairs reference valid node indices (parallel).
pub fn validate_active_pairs<Root>(
    nodes_kp: &KpType<'static, Root, Vec<NetNode>>,
    pairs_kp: &KpType<'static, Root, Vec<(u32, u32)>>,
    root: &Root,
) -> bool
where
    Root: Send + Sync,
    NetNode: Send + Sync,
{
    let node_count = nodes_kp.par_count(root);
    pairs_kp.par_all(root, |&(a, b)| {
        (a as usize) < node_count && (b as usize) < node_count
    })
}

/// Full validation for a GPU pipeline: nodes and active pairs.
pub fn validate_for_gpu(
    pipeline: &GpuComputePipeline,
    nodes_kp: &KpType<'static, GpuComputePipeline, Vec<NetNode>>,
    pairs_kp: &KpType<'static, GpuComputePipeline, Vec<(u32, u32)>>,
    max_port: u32,
) -> Result<(), String> {
    if !validate_nodes_parallel(nodes_kp, pipeline, max_port) {
        return Err("Invalid node ports".into());
    }
    if !validate_active_pairs(nodes_kp, pairs_kp, pipeline) {
        return Err("Invalid active pairs".into());
    }
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════
// 4. DATA EXTRACTION - Prepare data for GPU transfer
// ══════════════════════════════════════════════════════════════════════════

/// Extracts nodes and pairs via keypaths for GPU transfer.
pub fn extract_gpu_data(
    pipeline: &GpuComputePipeline,
    nodes_kp: &KpType<'static, GpuComputePipeline, Vec<NetNode>>,
    pairs_kp: &KpType<'static, GpuComputePipeline, Vec<(u32, u32)>>,
) -> GpuBufferData {
    let nodes = nodes_kp
        .get(pipeline)
        .map(|v| v.clone())
        .unwrap_or_default();
    let pairs = pairs_kp
        .get(pipeline)
        .map(|v| v.clone())
        .unwrap_or_default();
    GpuBufferData {
        node_count: nodes.len() as u32,
        nodes,
        pairs,
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 5. PARALLEL PRE/POST PROCESSING - Using keypath get_mut + par_iter_mut
// ══════════════════════════════════════════════════════════════════════════

/// Sorts active pairs in place (parallel sort) via keypath.
pub fn preprocess_sort_pairs(
    pipeline: &mut GpuComputePipeline,
    pairs_kp: &KpType<'static, GpuComputePipeline, Vec<(u32, u32)>>,
) {
    if let Some(pairs) = pairs_kp.get_mut(pipeline) {
        pairs.par_sort_unstable_by_key(|&(a, b)| a.min(b));
    }
}

/// Parallel count of nodes matching a predicate (e.g. post-GPU analysis).
pub fn count_nodes_by_kind<Root>(
    nodes_kp: &KpType<'static, Root, Vec<NetNode>>,
    root: &Root,
    kind: NodeKind,
) -> usize
where
    Root: Send + Sync,
    NetNode: Send + Sync,
{
    nodes_kp.par_count_by(root, |node| node.kind == kind)
}

/// Writes back GPU results into the pipeline's nodes via keypath.
pub fn process_gpu_results(
    pipeline: &mut GpuComputePipeline,
    nodes_kp: &KpType<'static, GpuComputePipeline, Vec<NetNode>>,
    gpu_results: GpuBufferData,
) {
    if let Some(nodes) = nodes_kp.get_mut(pipeline) {
        *nodes = gpu_results.nodes;
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 6. ADAPTIVE DISPATCH - Choose GPU config from keypath queries
// ══════════════════════════════════════════════════════════════════════════

/// Suggests GPU dispatch config from pair and node counts.
pub fn adaptive_gpu_dispatch(
    pipeline: &GpuComputePipeline,
    nodes_kp: &KpType<'static, GpuComputePipeline, Vec<NetNode>>,
    pairs_kp: &KpType<'static, GpuComputePipeline, Vec<(u32, u32)>>,
) -> GpuDispatchConfig {
    let pair_count = pairs_kp.par_count(pipeline);
    let node_count = nodes_kp.par_count(pipeline);
    GpuDispatchConfig {
        workgroup_size: if pair_count < 1000 { 64 } else { 256 },
        workgroup_count: (pair_count as u32 + 255) / 256,
        use_local_memory: node_count < 100_000,
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 7. SLICE HELPER - Batch extraction for streaming CPU↔GPU
// ══════════════════════════════════════════════════════════════════════════

/// Returns a cloned slice of the collection at the keypath (for batching).
pub fn slice_collection<Root, Item>(
    kp: &KpType<'static, Root, Vec<Item>>,
    root: &Root,
    start: usize,
    end: usize,
) -> Vec<Item>
where
    Root: Send + Sync,
    Item: Clone + Send + Sync,
{
    kp.get(root)
        .map(|v| {
            let end = end.min(v.len());
            let start = start.min(end);
            v[start..end].to_vec()
        })
        .unwrap_or_default()
}

// ══════════════════════════════════════════════════════════════════════════
// 8. PARALLEL BUFFER CALCULATION - Scale buffers (e.g. double values)
// ══════════════════════════════════════════════════════════════════════════

/// Applies a parallel transformation to each buffer's data via keypath.
/// Use when you have a keypath to `Vec<GpuBuffer>`; mutates in place.
pub fn par_scale_buffers<Root>(
    buffers_kp: &KpType<'static, Root, Vec<GpuBuffer>>,
    root: &mut Root,
    scale: f32,
) where
    Root: Send + Sync,
{
    if let Some(buffers) = buffers_kp.get_mut(root) {
        buffers.par_iter_mut().for_each(|buf| {
            buf.data.par_iter_mut().for_each(|x| *x *= scale);
        });
    }
}

/// Parallel validation: all buffers have non-empty data.
pub fn par_validate_buffers_non_empty<Root>(
    buffers_kp: &KpType<'static, Root, Vec<GpuBuffer>>,
    root: &Root,
) -> bool
where
    Root: Send + Sync,
{
    buffers_kp.par_all(root, |buf| !buf.data.is_empty())
}

/// Parallel map over buffer data (e.g. extract flat f32 for GPU).
pub fn par_flat_map_buffer_data<Root>(
    buffers_kp: &KpType<'static, Root, Vec<GpuBuffer>>,
    root: &Root,
) -> Vec<f32>
where
    Root: Send + Sync,
{
    buffers_kp.par_flat_map(root, |buf| buf.data.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_key_paths::Kp;

    #[test]
    fn test_validate_nodes_and_pairs() {
        let nodes_kp: KpType<'static, GpuComputePipeline, Vec<NetNode>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.reduction_net.nodes),
            |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.nodes),
        );
        let pairs_kp: KpType<'static, GpuComputePipeline, Vec<(u32, u32)>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.reduction_net.active_pairs),
            |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.active_pairs),
        );

        let pipeline = GpuComputePipeline {
            cpu_state: ComputeState::default(),
            gpu_buffer_ids: vec![],
            reduction_net: InteractionNet {
                nodes: vec![
                    NetNode {
                        kind: NodeKind::Dup,
                        ports: [0, 1, 2],
                    },
                    NetNode {
                        kind: NodeKind::Era,
                        ports: [10, 20, 30],
                    },
                ],
                active_pairs: vec![(0, 1)],
            },
        };

        assert!(validate_nodes_parallel(&nodes_kp, &pipeline, 1000));
        assert!(validate_active_pairs(&nodes_kp, &pairs_kp, &pipeline));
        assert!(validate_for_gpu(&pipeline, &nodes_kp, &pairs_kp, 1000).is_ok());
    }

    #[test]
    fn test_adaptive_dispatch() {
        let nodes_kp: KpType<'static, GpuComputePipeline, Vec<NetNode>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.reduction_net.nodes),
            |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.nodes),
        );
        let pairs_kp: KpType<'static, GpuComputePipeline, Vec<(u32, u32)>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.reduction_net.active_pairs),
            |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.active_pairs),
        );

        let pipeline = GpuComputePipeline {
            cpu_state: ComputeState::default(),
            gpu_buffer_ids: vec![],
            reduction_net: InteractionNet {
                nodes: (0..500).map(|_| NetNode { kind: NodeKind::Ref, ports: [0, 0, 0] }).collect(),
                active_pairs: (0..2000).map(|i| (i as u32, (i + 1) as u32)).collect(),
            },
        };

        let config = adaptive_gpu_dispatch(&pipeline, &nodes_kp, &pairs_kp);
        assert_eq!(config.workgroup_size, 256);
        assert!(config.workgroup_count > 0);
    }
}
