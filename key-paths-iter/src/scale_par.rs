//! GPU-scale parallel validation and calculation using keypaths.
//!
//! This module provides types and helpers for CPU↔GPU-style pipelines: validate
//! collections in parallel via keypaths, extract data for transfer, run parallel
//! pre/post processing, and choose dispatch parameters. Use with
//! [ParallelCollectionKeyPath](crate::query_par::ParallelCollectionKeyPath) and
//! [KpType](rust_key_paths::KpType) (e.g. from `#[derive(Kp)]`).
//!
//! Enable the `gpu` feature for wgpu-based compute (HVM2-style reduction shader).

#![cfg(feature = "rayon")]

use crate::query_par::ParallelCollectionKeyPath;
use rayon::prelude::*;
use rust_key_paths::KpType;

// ══════════════════════════════════════════════════════════════════════════
// 1. CORE TYPES - GPU-compatible with proper derives
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

impl GpuBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0.0; size],
            size,
        }
    }

    pub fn from_data(data: Vec<f32>) -> Self {
        let size = data.len();
        Self { data, size }
    }
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

/// Node kind in the interaction net (repr u32 for GPU).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum NodeKind {
    Era = 0,
    Con = 1,
    Dup = 2,
    Ref = 3,
}

/// GPU-compatible net node (16-byte aligned, safe for bytemuck when feature "gpu").
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "gpu", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct NetNode {
    /// Node kind as u32 (0=Era, 1=Con, 2=Dup, 3=Ref)
    pub kind: u32,
    pub port0: u32,
    pub port1: u32,
    pub port2: u32,
}

impl NetNode {
    pub fn new(kind: NodeKind, ports: [u32; 3]) -> Self {
        Self {
            kind: kind as u32,
            port0: ports[0],
            port1: ports[1],
            port2: ports[2],
        }
    }

    pub fn kind(&self) -> NodeKind {
        match self.kind {
            0 => NodeKind::Era,
            1 => NodeKind::Con,
            2 => NodeKind::Dup,
            3 => NodeKind::Ref,
            _ => NodeKind::Era,
        }
    }

    pub fn ports(&self) -> [u32; 3] {
        [self.port0, self.port1, self.port2]
    }
}

/// GPU-compatible redex pair (8-byte aligned).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "gpu", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct RedexPair {
    pub left: u32,
    pub right: u32,
}

impl From<(u32, u32)> for RedexPair {
    fn from((left, right): (u32, u32)) -> Self {
        Self { left, right }
    }
}

impl From<RedexPair> for (u32, u32) {
    fn from(pair: RedexPair) -> Self {
        (pair.left, pair.right)
    }
}

/// Full pipeline: CPU state + GPU buffer ids + reduction net.
#[derive(Clone, Debug)]
pub struct GpuComputePipeline {
    pub cpu_state: ComputeState,
    pub gpu_buffer_ids: Vec<u32>,
    pub reduction_net: InteractionNet,
}

/// Data extracted for GPU transfer (GPU-compatible layout when using RedexPair).
#[derive(Clone, Debug)]
pub struct GpuBufferData {
    pub nodes: Vec<NetNode>,
    /// Pairs as (u32, u32) for API compatibility; use `.pairs_redo()` for GPU buffer.
    pub pairs: Vec<(u32, u32)>,
    pub node_count: u32,
}

impl GpuBufferData {
    /// Pairs as `RedexPair` for bytemuck/wgpu (when feature "gpu").
    #[cfg(feature = "gpu")]
    pub fn pairs_redo(&self) -> Vec<RedexPair> {
        self.pairs.iter().map(|&p| RedexPair::from(p)).collect()
    }
}

/// Suggested GPU dispatch configuration from keypath queries.
#[derive(Clone, Debug)]
pub struct GpuDispatchConfig {
    pub workgroup_size: u32,
    pub workgroup_count: u32,
    pub use_local_memory: bool,
}

// ══════════════════════════════════════════════════════════════════════════
// 2. PARALLEL STRATEGY
// ══════════════════════════════════════════════════════════════════════════

/// Parallel execution strategy for collection operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelStrategy {
    Sequential,
    DataParallel,
    TaskParallel,
    Recursive,
}

pub const DEFAULT_PARALLEL_THRESHOLD: usize = 1000;

#[inline]
pub fn strategy_for_size(len: usize, threshold: usize) -> ParallelStrategy {
    if len >= threshold {
        ParallelStrategy::DataParallel
    } else {
        ParallelStrategy::Sequential
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 3. KEYPATH-BASED VALIDATION
// ══════════════════════════════════════════════════════════════════════════

/// Validates that all nodes have ports within a maximum value (parallel).
pub fn validate_nodes_parallel<Root>(
    nodes_kp: &KpType<'static, Root, Vec<NetNode>>,
    root: &Root,
    max_port: u32,
) -> bool
where
    Root: Send + Sync,
{
    nodes_kp.par_all(root, |node| node.ports().iter().all(|&p| p < max_port))
}

/// Validates that all active pairs reference valid node indices (parallel).
pub fn validate_active_pairs<Root>(
    nodes_kp: &KpType<'static, Root, Vec<NetNode>>,
    pairs_kp: &KpType<'static, Root, Vec<(u32, u32)>>,
    root: &Root,
) -> bool
where
    Root: Send + Sync,
{
    let node_count = nodes_kp.par_count(root);
    pairs_kp.par_all(root, |&(a, b)| {
        (a as usize) < node_count && (b as usize) < node_count
    })
}

/// Full validation for a GPU pipeline.
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
// 4. DATA EXTRACTION
// ══════════════════════════════════════════════════════════════════════════

/// Extracts nodes and pairs via keypaths for GPU transfer.
pub fn extract_gpu_data(
    pipeline: &GpuComputePipeline,
    nodes_kp: &KpType<'static, GpuComputePipeline, Vec<NetNode>>,
    pairs_kp: &KpType<'static, GpuComputePipeline, Vec<(u32, u32)>>,
) -> GpuBufferData {
    let nodes = nodes_kp
        .get(pipeline)
        .map(|v| v.to_vec())
        .unwrap_or_default();
    let pairs = pairs_kp
        .get(pipeline)
        .map(|v| v.to_vec())
        .unwrap_or_default();
    GpuBufferData {
        node_count: nodes.len() as u32,
        nodes,
        pairs,
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 5. PARALLEL PRE/POST PROCESSING (uses get_mut from KpType)
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

/// Parallel count of nodes matching a predicate.
pub fn count_nodes_by_kind<Root>(
    nodes_kp: &KpType<'static, Root, Vec<NetNode>>,
    root: &Root,
    kind: NodeKind,
) -> usize
where
    Root: Send + Sync,
{
    nodes_kp.par_count_by(root, |node| node.kind() == kind)
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
// 6. ADAPTIVE DISPATCH
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
// 7. SLICE HELPER
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
// 8. PARALLEL BUFFER CALCULATION
// ══════════════════════════════════════════════════════════════════════════

/// Applies a parallel transformation to each buffer's data via keypath.
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

// ══════════════════════════════════════════════════════════════════════════
// 9. GPU COMPUTE (wgpu) - behind feature "gpu"
// ══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "gpu")]
mod gpu_impl {
    use super::*;
    use std::borrow::Cow;
    use std::sync::mpsc;
    use wgpu::util::DeviceExt;

    /// GPU compute context for HVM2-style reductions.
    pub struct GpuCompute {
        device: wgpu::Device,
        queue: wgpu::Queue,
        reduce_pipeline: wgpu::ComputePipeline,
        bind_group_layout: wgpu::BindGroupLayout,
    }

    impl GpuCompute {
        /// Initialize GPU compute context (sync via pollster).
        pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            pollster::block_on(Self::new_async())
        }

        pub async fn new_async() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            let instance = wgpu::Instance::default();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .ok_or("No GPU adapter found")?;

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("HVM2 Compute Device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await?;

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("HVM2 Reduction Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/hvm_reduce.wgsl"))),
            });

            let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("HVM2 Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("HVM2 Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

            let reduce_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("HVM2 Reduction Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
            });

            Ok(Self {
                device,
                queue,
                reduce_pipeline,
                bind_group_layout,
            })
        }

        /// Execute HVM2 reduction on GPU; returns updated nodes.
        pub fn execute_reduction(
            &self,
            data: &GpuBufferData,
            config: &GpuDispatchConfig,
        ) -> Result<GpuBufferData, Box<dyn std::error::Error + Send + Sync>> {
            let pairs_redo = data.pairs_redo();
            let nodes_bytes = bytemuck::cast_slice(&data.nodes);
            let pairs_bytes = bytemuck::cast_slice(&pairs_redo);

            let nodes_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Nodes Buffer"),
                contents: nodes_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            });

            let pairs_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Pairs Buffer"),
                contents: pairs_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            });

            let metadata: [u32; 4] = [data.node_count, data.pairs.len() as u32, 0u32, 0u32];
            let metadata_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Metadata Buffer"),
                contents: bytemuck::cast_slice(&metadata),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            });

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("HVM2 Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: nodes_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: pairs_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: metadata_buffer.as_entire_binding(),
                    },
                ],
            });

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("HVM2 Command Encoder"),
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("HVM2 Reduction Pass"),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.reduce_pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups(config.workgroup_count, 1, 1);
            }

            let staging_size = nodes_bytes.len() as u64;
            let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Staging Buffer"),
                size: staging_size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            encoder.copy_buffer_to_buffer(&nodes_buffer, 0, &staging_buffer, 0, staging_size);
            self.queue.submit(Some(encoder.finish()));

            let buffer_slice = staging_buffer.slice(..);
            let (tx, rx) = mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            self.device.poll(wgpu::Maintain::Wait);
            rx.recv().map_err(|_| "map_async callback never ran")??;

            let data_view = buffer_slice.get_mapped_range();
            let result_nodes: Vec<NetNode> = bytemuck::cast_slice(&data_view).to_vec();
            drop(data_view);
            staging_buffer.unmap();

            Ok(GpuBufferData {
                nodes: result_nodes,
                pairs: data.pairs.clone(),
                node_count: data.node_count,
            })
        }
    }

    /// Complete GPU reduction pipeline using keypaths (sync).
    pub fn run_gpu_reduction_pipeline(
        pipeline: &mut GpuComputePipeline,
        nodes_kp: &KpType<'static, GpuComputePipeline, Vec<NetNode>>,
        pairs_kp: &KpType<'static, GpuComputePipeline, Vec<(u32, u32)>>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        validate_for_gpu(pipeline, nodes_kp, pairs_kp, 1_000_000)?;
        preprocess_sort_pairs(pipeline, pairs_kp);
        let gpu_data = extract_gpu_data(pipeline, nodes_kp, pairs_kp);
        let config = adaptive_gpu_dispatch(pipeline, nodes_kp, pairs_kp);
        let gpu_compute = GpuCompute::new()?;
        let results = gpu_compute.execute_reduction(&gpu_data, &config)?;
        process_gpu_results(pipeline, nodes_kp, results);
        let reduction_count = count_nodes_by_kind(nodes_kp, pipeline, NodeKind::Era);
        Ok(reduction_count)
    }
}

#[cfg(feature = "gpu")]
pub use gpu_impl::{run_gpu_reduction_pipeline, GpuCompute};

// ══════════════════════════════════════════════════════════════════════════
// 10. TESTS
// ══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use rust_key_paths::Kp;

    fn create_test_pipeline() -> GpuComputePipeline {
        GpuComputePipeline {
            cpu_state: ComputeState::default(),
            gpu_buffer_ids: vec![],
            reduction_net: InteractionNet {
                nodes: vec![
                    NetNode::new(NodeKind::Dup, [0, 1, 2]),
                    NetNode::new(NodeKind::Era, [10, 20, 30]),
                    NetNode::new(NodeKind::Con, [5, 6, 7]),
                ],
                active_pairs: vec![(0, 1), (1, 2)],
            },
        }
    }

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

        let pipeline = create_test_pipeline();

        assert!(validate_nodes_parallel(&nodes_kp, &pipeline, 1000));
        assert!(validate_active_pairs(&nodes_kp, &pairs_kp, &pipeline));
        assert!(validate_for_gpu(&pipeline, &nodes_kp, &pairs_kp, 1000).is_ok());
    }

    #[test]
    fn test_extract_gpu_data() {
        let nodes_kp: KpType<'static, GpuComputePipeline, Vec<NetNode>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.reduction_net.nodes),
            |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.nodes),
        );
        let pairs_kp: KpType<'static, GpuComputePipeline, Vec<(u32, u32)>> = Kp::new(
            |p: &GpuComputePipeline| Some(&p.reduction_net.active_pairs),
            |p: &mut GpuComputePipeline| Some(&mut p.reduction_net.active_pairs),
        );

        let pipeline = create_test_pipeline();
        let gpu_data = extract_gpu_data(&pipeline, &nodes_kp, &pairs_kp);

        assert_eq!(gpu_data.node_count, 3);
        assert_eq!(gpu_data.nodes.len(), 3);
        assert_eq!(gpu_data.pairs.len(), 2);
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

        let pipeline = create_test_pipeline();
        let config = adaptive_gpu_dispatch(&pipeline, &nodes_kp, &pairs_kp);

        assert!(config.workgroup_size > 0);
        assert!(config.workgroup_count > 0);
    }
}
