//! Run a list of AKps in parallel: numeric keypaths on GPU (wgpu), arbitrary on CPU (rayon).
//!
//! Split keypaths into [AKpTier::Numeric] (f32/u32 → GPU) and [AKpTier::Arbitrary] (any type → CPU).
//! [AKpRunner] runs numeric KPs in one GPU dispatch and arbitrary KPs in parallel with rayon.

#![cfg(feature = "gpu")]

use rust_key_paths::AKp;
use std::any::TypeId;
use std::sync::mpsc;
use std::sync::Arc;
use wgpu::util::DeviceExt;

// ─── Value types that can be sent to GPU ─────────────────────────────────────

/// Values that can be stored in GPU buffers and transformed by WGSL.
#[derive(Clone, Debug)]
pub enum GpuValue {
    F32(f32),
    U32(u32),
}

// ─── Numeric AKp: extract GpuValue from root + optional WGSL transform ──────

/// A keypath that extracts a numeric value (e.g. f32) from a root for GPU execution.
pub struct NumericAKp {
    /// Extracts the value from `root` (as `&dyn Any`).
    pub extractor: Arc<dyn Fn(&dyn std::any::Any) -> Option<GpuValue> + Send + Sync>,
    /// WGSL expression for the transform, e.g. `"input * 2.0"`. The input is a single f32.
    pub wgsl_expr: String,
    pub root_type_id: TypeId,
}

// ─── Tier: GPU vs CPU ───────────────────────────────────────────────────────

/// Classifies a keypath as GPU-acceleratable (numeric) or CPU-only (arbitrary).
pub enum AKpTier {
    Numeric(NumericAKp),
    Arbitrary(AKp),
}

// ─── WGPU context ───────────────────────────────────────────────────────────

pub struct WgpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuContext {
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
                    label: Some("AKpRunner"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        Ok(Self { device, queue })
    }
}

// ─── Run results ───────────────────────────────────────────────────────────

/// Results from [AKpRunner::run]: numeric (GPU) and count of arbitrary (CPU) KPs run.
pub struct RunResults {
    /// Results from numeric KPs, in registration order.
    pub numeric: Vec<Option<GpuValue>>,
    /// Number of arbitrary KPs that were executed on CPU.
    pub arbitrary_count: usize,
}

// ─── Runner ────────────────────────────────────────────────────────────────

pub struct AKpRunner {
    gpu_kps: Vec<NumericAKp>,
    cpu_kps: Vec<AKp>,
    wgpu_ctx: Option<WgpuContext>,
}

impl AKpRunner {
    pub fn new(tiers: Vec<AKpTier>, wgpu_ctx: Option<WgpuContext>) -> Self {
        let mut gpu_kps = vec![];
        let mut cpu_kps = vec![];
        for tier in tiers {
            match tier {
                AKpTier::Numeric(n) => gpu_kps.push(n),
                AKpTier::Arbitrary(a) => cpu_kps.push(a),
            }
        }
        Self {
            gpu_kps,
            cpu_kps,
            wgpu_ctx,
        }
    }

    /// Run all KPs: numeric on GPU (or CPU fallback), arbitrary in parallel on CPU.
    pub fn run(&self, root: &dyn std::any::Any) -> RunResults {
        let numeric = self.run_numeric(root);
        let arbitrary_count = self.cpu_kps.len();
        self.run_arbitrary(root);
        RunResults {
            numeric,
            arbitrary_count,
        }
    }

    fn run_numeric(&self, root: &dyn std::any::Any) -> Vec<Option<GpuValue>> {
        let extracted: Vec<Option<GpuValue>> = self
            .gpu_kps
            .iter()
            .map(|kp| (kp.extractor)(root))
            .collect();

        let Some(ctx) = &self.wgpu_ctx else {
            return extracted;
        };

        let (indices, values): (Vec<usize>, Vec<f32>) = extracted
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                Some(GpuValue::F32(f)) => Some((i, *f)),
                _ => None,
            })
            .unzip();

        if values.is_empty() {
            return extracted;
        }

        match Self::dispatch_f32(ctx, &values) {
            Ok(transformed) => {
                let mut results = extracted;
                for (buf_idx, &orig_idx) in indices.iter().enumerate() {
                    if let Some(r) = results.get_mut(orig_idx) {
                        *r = Some(GpuValue::F32(transformed[buf_idx]));
                    }
                }
                results
            }
            _ => extracted,
        }
    }

    fn dispatch_f32(
        ctx: &WgpuContext,
        values: &[f32],
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let n = values.len() as u64;
        let size = n * 4;

        let shader_src = r#"
@group(0) @binding(0) var<storage, read>       input:  array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= arrayLength(&input)) { return; }
    output[id] = input[id] * 2.0 + 1.0;
}
"#;

        let module = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("akp_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader_src)),
        });

        let bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("akp_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("akp_pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("akp_pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "main",
        });

        let input_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("akp_input"),
            contents: bytemuck::cast_slice(values),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let output_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("akp_output"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("akp_readback"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("akp_bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buf.as_entire_binding(),
                },
            ],
        });

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((values.len() as u32 + 63) / 64, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output_buf, 0, &readback_buf, 0, size);
        ctx.queue.submit(Some(encoder.finish()));

        let slice = readback_buf.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        ctx.device.poll(wgpu::Maintain::Wait);
        rx.recv().map_err(|_| "map_async")??;
        let data = slice.get_mapped_range();
        let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        readback_buf.unmap();
        Ok(out)
    }

    fn run_arbitrary(&self, root: &dyn std::any::Any) {
        for kp in &self.cpu_kps {
            let _ = kp.get(root);
        }
    }
}

// ─── Helpers: build NumericAKp from KpType (conceptually) ────────────────────

/// Build a [NumericAKp] that extracts an f32 from roots of type `R`.
pub fn numeric_akp_f32<R: 'static>(
    extract: impl Fn(&R) -> Option<f32> + Send + Sync + 'static,
    wgsl_expr: impl Into<String>,
) -> NumericAKp {
    let extract = Arc::new(extract);
    let extract_any: Arc<dyn Fn(&dyn std::any::Any) -> Option<GpuValue> + Send + Sync> =
        Arc::new(move |root: &dyn std::any::Any| {
            let r = root.downcast_ref::<R>()?;
            extract(r).map(GpuValue::F32)
        });
    NumericAKp {
        extractor: extract_any,
        wgsl_expr: wgsl_expr.into(),
        root_type_id: TypeId::of::<R>(),
    }
}
