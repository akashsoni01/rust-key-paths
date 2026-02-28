//! GPU-aware extensions for `KpType` / `Kp`.
//!
//! Design principles:
//! - No AKp / PKp — everything builds on `KpType` directly.
//! - HOF style: `.map_gpu()`, `.par_gpu()`, `.zip_gpu()` return composable values,
//!   mirroring the existing `.map()` / `.filter()` / `.then()` API.
//! - True parallelism: numeric KPs → one wgpu dispatch; arbitrary KPs → rayon.
//! - `GpuKp<R, V>` is the GPU-aware analog of `KpType<R, V>`.

#![cfg(feature = "gpu")]

use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::{mpsc, Arc};
use wgpu::util::DeviceExt;

use rust_key_paths::KpType;

// ── 1. GpuCompatible trait ───────────────────────────────────────────────────

mod sealed {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for u32 {}
    impl Sealed for i32 {}
}

/// Marker + helper for types that map to a WGSL scalar.
///
/// Implement this for your own `#[repr(C)]` newtypes if needed.
pub trait GpuCompatible: sealed::Sealed + bytemuck::Pod + bytemuck::Zeroable + Copy + 'static {
    /// WGSL type name: `"f32"`, `"u32"`, `"i32"`.
    fn wgsl_type() -> &'static str;
}

impl GpuCompatible for f32 {
    fn wgsl_type() -> &'static str {
        "f32"
    }
}
impl GpuCompatible for u32 {
    fn wgsl_type() -> &'static str {
        "u32"
    }
}
impl GpuCompatible for i32 {
    fn wgsl_type() -> &'static str {
        "i32"
    }
}

// ── 2. GpuKernel: the WGSL snippet attached to a KP ────────────────────────

/// A WGSL transform body applied element-wise.
///
/// The shader binds:
/// - `input`:  `array<T>` (read-only)
/// - `output`: `array<T>` (read-write)
/// - `id`:     `u32` — `global_invocation_id.x`
///
/// `wgsl_statement` is inserted verbatim as the loop body, e.g.:
/// ```wgsl
/// output[id] = input[id] * 2.0 + 1.0;
/// ```
#[derive(Clone, Debug)]
pub struct GpuKernel {
    pub wgsl_statement: String,
    pub workgroup_size: u32,
}

impl GpuKernel {
    pub fn new(wgsl_statement: impl Into<String>) -> Self {
        Self {
            wgsl_statement: wgsl_statement.into(),
            workgroup_size: 64,
        }
    }

    pub fn workgroup_size(mut self, size: u32) -> Self {
        self.workgroup_size = size;
        self
    }
}

// ── 3. GpuKp<R, V, E, I>: static dispatch (no trait objects) ───────────────

/// A keypath that extracts a `V` from `R` **and** knows how to transform
/// that value on the GPU.
///
/// Uses **static dispatch**: `E` and `I` are the concrete extractor/injector
/// types (e.g. closures), so calls are monomorphized with no vtable.
///
/// Created via [`KpGpuExt::map_gpu`] or [`KpGpuExt::into_gpu`].
/// Composable via [`.and_then_gpu()`] and [`.zip_gpu_same()`].
pub struct GpuKp<R: 'static, V: GpuCompatible, E, I> {
    pub(crate) extractor: E,
    pub(crate) injector: I,
    pub kernel: GpuKernel,
    pub root_type_id: TypeId,
    /// Used to tie R and V to the type without requiring them to be Send/Sync.
    _marker: PhantomData<fn() -> (R, V)>,
}

impl<R: 'static, V: GpuCompatible, E, I> GpuKp<R, V, E, I>
where
    E: Fn(&R) -> Option<V> + Send + Sync,
    I: Fn(&mut R, V) + Send + Sync,
{
    /// Compose: apply `next` kernel *after* `self` kernel.
    pub fn and_then_gpu(self, next_kernel: impl Into<String>) -> GpuKp<R, V, E, I> {
        let combined = format!(
            "{}\n    {}",
            self.kernel.wgsl_statement,
            next_kernel.into()
        );
        GpuKp {
            kernel: GpuKernel::new(combined),
            extractor: self.extractor,
            injector: self.injector,
            root_type_id: self.root_type_id,
            _marker: PhantomData,
        }
    }

    /// Run this single GpuKp on GPU, returning the transformed value.
    ///
    /// For batch use prefer [`GpuKpRunner`].
    pub fn run_one(&self, root: &R, ctx: &WgpuContext) -> Option<V> {
        let raw = (self.extractor)(root)?;
        let results = ctx.dispatch_scalar::<V>(&[raw], &self.kernel).ok()?;
        results.into_iter().next()
    }

    /// Run over a slice of roots — **one GPU dispatch** for all of them.
    pub fn run_many(&self, roots: &[R], ctx: &WgpuContext) -> Vec<Option<V>> {
        let (indices, values): (Vec<usize>, Vec<V>) = roots
            .iter()
            .enumerate()
            .filter_map(|(i, r)| (self.extractor)(r).map(|v| (i, v)))
            .unzip();

        let Ok(transformed) = ctx.dispatch_scalar::<V>(&values, &self.kernel) else {
            return vec![None; roots.len()];
        };

        let mut out: Vec<Option<V>> = vec![None; roots.len()];
        for (buf_idx, &orig_idx) in indices.iter().enumerate() {
            out[orig_idx] = Some(transformed[buf_idx]);
        }
        out
    }

    /// Write the GPU-transformed value back into each root (in-place mutation).
    pub fn apply_many(&self, roots: &mut [R], ctx: &WgpuContext) {
        let results = self.run_many(roots, ctx);
        for (root, result) in roots.iter_mut().zip(results.into_iter()) {
            if let Some(v) = result {
                (self.injector)(root, v);
            }
        }
    }
}

// ── 4. HOF extensions on KpType ─────────────────────────────────────────────

/// GPU extensions for [`KpType`]; mirrors `.map()` / `.filter()` but produces [`GpuKp`].
pub trait KpGpuExt<R: 'static, V: GpuCompatible> {
    /// Lift `self` into a `GpuKp` with an identity (pass-through) kernel.
    fn into_gpu(self) -> GpuKp<R, V, impl Fn(&R) -> Option<V> + Send + Sync, impl Fn(&mut R, V) + Send + Sync>;

    /// Attach a WGSL element-wise transform, producing a `GpuKp` (static dispatch).
    fn map_gpu(self, wgsl_statement: impl Into<String>) -> GpuKp<R, V, impl Fn(&R) -> Option<V> + Send + Sync, impl Fn(&mut R, V) + Send + Sync>;

    /// Run `self` across a `roots` slice: one GPU dispatch, transformed results.
    fn par_gpu(
        self,
        wgsl_statement: impl Into<String>,
        roots: &[R],
        ctx: &WgpuContext,
    ) -> Vec<Option<V>>;
}

impl<R, V> KpGpuExt<R, V> for KpType<'static, R, V>
where
    R: 'static,
    V: GpuCompatible,
{
    fn into_gpu(self) -> GpuKp<R, V, impl Fn(&R) -> Option<V> + Send + Sync, impl Fn(&mut R, V) + Send + Sync> {
        self.map_gpu("output[id] = input[id];")
    }

    fn map_gpu(self, wgsl_statement: impl Into<String>) -> GpuKp<R, V, impl Fn(&R) -> Option<V> + Send + Sync, impl Fn(&mut R, V) + Send + Sync> {
        let kp = Arc::new(self);
        GpuKp {
            extractor: {
                let kp = Arc::clone(&kp);
                move |r: &R| kp.get(r).copied()
            },
            injector: {
                let kp = Arc::clone(&kp);
                move |r: &mut R, val: V| {
                    if let Some(slot) = kp.get_mut(r) {
                        *slot = val;
                    }
                }
            },
            kernel: GpuKernel::new(wgsl_statement),
            root_type_id: TypeId::of::<R>(),
            _marker: PhantomData,
        }
    }

    fn par_gpu(
        self,
        wgsl_statement: impl Into<String>,
        roots: &[R],
        ctx: &WgpuContext,
    ) -> Vec<Option<V>> {
        self.map_gpu(wgsl_statement).run_many(roots, ctx)
    }
}

// ── 4b. KpType<R, Vec<V>>: GPU over a vector (static dispatch) ────────────────

/// A keypath that extracts a `Vec<V>` from `R` and runs an element-wise GPU kernel over it.
///
/// Static dispatch: `E` and `I` are the concrete extractor/injector types.
pub struct GpuKpVec<R: 'static, V: GpuCompatible, E, I> {
    extractor: E,
    injector: I,
    pub kernel: GpuKernel,
    pub root_type_id: TypeId,
    /// Used to tie R and V to the type without requiring them to be Send/Sync.
    _marker: PhantomData<fn() -> (R, V)>,
}

impl<R: 'static, V: GpuCompatible, E, I> GpuKpVec<R, V, E, I>
where
    E: Fn(&R) -> Option<Vec<V>> + Send + Sync,
    I: Fn(&mut R, Vec<V>) + Send + Sync,
{
    /// Run the GPU kernel on the vector at `root`; returns the transformed vector.
    pub fn run_one(&self, root: &R, ctx: &WgpuContext) -> Option<Vec<V>> {
        let values = (self.extractor)(root)?;
        if values.is_empty() {
            return Some(values);
        }
        ctx.dispatch_scalar::<V>(&values, &self.kernel).ok()
    }

    /// Run the kernel and write the result back into the root (in-place).
    pub fn apply_one(&self, root: &mut R, ctx: &WgpuContext) -> bool {
        if let Some(transformed) = self.run_one(root, ctx) {
            (self.injector)(root, transformed);
            true
        } else {
            false
        }
    }

    /// Chain another WGSL statement.
    pub fn and_then_gpu(self, next_kernel: impl Into<String>) -> GpuKpVec<R, V, E, I> {
        let combined = format!(
            "{}\n    {}",
            self.kernel.wgsl_statement,
            next_kernel.into()
        );
        GpuKpVec {
            kernel: GpuKernel::new(combined),
            extractor: self.extractor,
            injector: self.injector,
            root_type_id: self.root_type_id,
            _marker: PhantomData,
        }
    }
}

/// GPU extensions for keypaths whose **value type is `Vec<V>`**.
pub trait KpGpuVecExt<R: 'static, V: GpuCompatible> {
    /// Attach an element-wise WGSL transform over the vector (static dispatch).
    fn map_gpu_vec(self, wgsl_statement: impl Into<String>) -> GpuKpVec<R, V, impl Fn(&R) -> Option<Vec<V>> + Send + Sync, impl Fn(&mut R, Vec<V>) + Send + Sync>;
}

impl<R, V> KpGpuVecExt<R, V> for KpType<'static, R, Vec<V>>
where
    R: 'static,
    V: GpuCompatible,
{
    fn map_gpu_vec(self, wgsl_statement: impl Into<String>) -> GpuKpVec<R, V, impl Fn(&R) -> Option<Vec<V>> + Send + Sync, impl Fn(&mut R, Vec<V>) + Send + Sync> {
        let kp = Arc::new(self);
        GpuKpVec {
            extractor: {
                let kp = Arc::clone(&kp);
                move |r: &R| kp.get(r).map(|vec_ref| vec_ref.iter().copied().collect())
            },
            injector: {
                let kp = Arc::clone(&kp);
                move |r: &mut R, new_vec: Vec<V>| {
                    if let Some(slot) = kp.get_mut(r) {
                        *slot = new_vec;
                    }
                }
            },
            kernel: GpuKernel::new(wgsl_statement),
            root_type_id: TypeId::of::<R>(),
            _marker: PhantomData,
        }
    }
}

// ── 5. zip_gpu: run TWO GpuKps with one shader ──────────────────────────────

/// Run two `GpuKp`s on the **same root** (static dispatch).
pub fn zip_gpu<R: 'static, A: GpuCompatible, B: GpuCompatible, E1, I1, E2, I2>(
    kp_a: &GpuKp<R, A, E1, I1>,
    kp_b: &GpuKp<R, B, E2, I2>,
    root: &R,
    ctx: &WgpuContext,
) -> (Option<A>, Option<B>)
where
    E1: Fn(&R) -> Option<A> + Send + Sync,
    I1: Fn(&mut R, A) + Send + Sync,
    E2: Fn(&R) -> Option<B> + Send + Sync,
    I2: Fn(&mut R, B) + Send + Sync,
{
    let a = kp_a.run_one(root, ctx);
    let b = kp_b.run_one(root, ctx);
    (a, b)
}

/// Run two same-type `GpuKp<R, V>` with **one GPU dispatch** (packed buffer).
pub fn zip_gpu_same<R: 'static, V: GpuCompatible, E1, I1, E2, I2>(
    kp_a: &GpuKp<R, V, E1, I1>,
    kp_b: &GpuKp<R, V, E2, I2>,
    root: &R,
    ctx: &WgpuContext,
) -> (Option<V>, Option<V>)
where
    E1: Fn(&R) -> Option<V> + Send + Sync,
    I1: Fn(&mut R, V) + Send + Sync,
    E2: Fn(&R) -> Option<V> + Send + Sync,
    I2: Fn(&mut R, V) + Send + Sync,
{
    let a_raw = (kp_a.extractor)(root);
    let b_raw = (kp_b.extractor)(root);

    match (a_raw, b_raw) {
        (Some(a), Some(b)) => {
            let merged_kernel = format!(
                "if (id == 0u) {{ {} }} else {{ {} }}",
                kp_a.kernel.wgsl_statement,
                kp_b.kernel.wgsl_statement,
            );
            let merged = GpuKernel::new(merged_kernel);
            let results = ctx.dispatch_scalar::<V>(&[a, b], &merged).ok();
            match results.as_deref() {
                Some([ra, rb]) => (Some(*ra), Some(*rb)),
                _ => (None, None),
            }
        }
        (a, b) => (a, b),
    }
}

// ── 6. GpuKpRunner: run a heterogeneous list of GpuKps in parallel ──────────

trait ErasedGpuKp<R>: Send + Sync {
    fn extract_f32(&self, root: &R) -> Option<f32>;
    fn inject_f32(&self, root: &mut R, val: f32);
    fn kernel(&self) -> &GpuKernel;
}

impl<R: 'static, E, I> ErasedGpuKp<R> for GpuKp<R, f32, E, I>
where
    E: Fn(&R) -> Option<f32> + Send + Sync,
    I: Fn(&mut R, f32) + Send + Sync,
{
    fn extract_f32(&self, root: &R) -> Option<f32> {
        (self.extractor)(root)
    }
    fn inject_f32(&self, root: &mut R, val: f32) {
        (self.injector)(root, val);
    }
    fn kernel(&self) -> &GpuKernel {
        &self.kernel
    }
}

impl<R: 'static, E, I> ErasedGpuKp<R> for GpuKp<R, u32, E, I>
where
    E: Fn(&R) -> Option<u32> + Send + Sync,
    I: Fn(&mut R, u32) + Send + Sync,
{
    fn extract_f32(&self, root: &R) -> Option<f32> {
        (self.extractor)(root).map(|v| v as f32)
    }
    fn inject_f32(&self, root: &mut R, val: f32) {
        (self.injector)(root, val as u32);
    }
    fn kernel(&self) -> &GpuKernel {
        &self.kernel
    }
}

impl<R: 'static, E, I> ErasedGpuKp<R> for GpuKp<R, i32, E, I>
where
    E: Fn(&R) -> Option<i32> + Send + Sync,
    I: Fn(&mut R, i32) + Send + Sync,
{
    fn extract_f32(&self, root: &R) -> Option<f32> {
        (self.extractor)(root).map(|v| v as f32)
    }
    fn inject_f32(&self, root: &mut R, val: f32) {
        (self.injector)(root, val as i32);
    }
    fn kernel(&self) -> &GpuKernel {
        &self.kernel
    }
}

/// Run a **heterogeneous list** of `GpuKp`s on a single root in one GPU dispatch.
#[derive(Clone, Debug)]
pub struct KpResult {
    pub index: usize,
    pub value: Option<f32>,
}

/// Run a heterogeneous list of `GpuKp`s on a single root in one GPU dispatch.
pub struct GpuKpRunner<'ctx, R: 'static> {
    gpu_kps: Vec<Box<dyn ErasedGpuKp<R>>>,
    ctx: &'ctx WgpuContext,
}

impl<'ctx, R: 'static + Send + Sync> GpuKpRunner<'ctx, R> {
    pub fn new(ctx: &'ctx WgpuContext) -> Self {
        Self {
            gpu_kps: vec![],
            ctx,
        }
    }

    pub fn add_f32<E, I>(mut self, kp: GpuKp<R, f32, E, I>) -> Self
    where
        E: Fn(&R) -> Option<f32> + Send + Sync + 'static,
        I: Fn(&mut R, f32) + Send + Sync + 'static,
    {
        self.gpu_kps.push(Box::new(kp));
        self
    }

    pub fn add_u32<E, I>(mut self, kp: GpuKp<R, u32, E, I>) -> Self
    where
        E: Fn(&R) -> Option<u32> + Send + Sync + 'static,
        I: Fn(&mut R, u32) + Send + Sync + 'static,
    {
        self.gpu_kps.push(Box::new(kp));
        self
    }

    pub fn add_i32<E, I>(mut self, kp: GpuKp<R, i32, E, I>) -> Self
    where
        E: Fn(&R) -> Option<i32> + Send + Sync + 'static,
        I: Fn(&mut R, i32) + Send + Sync + 'static,
    {
        self.gpu_kps.push(Box::new(kp));
        self
    }

    /// Run all registered GpuKps on `root` — **one GPU dispatch**.
    pub fn run(&self, root: &R) -> Vec<KpResult> {
        let (indices, values): (Vec<usize>, Vec<f32>) = self
            .gpu_kps
            .iter()
            .enumerate()
            .filter_map(|(i, kp)| kp.extract_f32(root).map(|v| (i, v)))
            .unzip();

        if values.is_empty() {
            return self
                .gpu_kps
                .iter()
                .enumerate()
                .map(|(i, _)| KpResult {
                    index: i,
                    value: None,
                })
                .collect();
        }

        let branches: String = indices
            .iter()
            .enumerate()
            .map(|(buf_idx, &kp_idx)| {
                let stmt = &self.gpu_kps[kp_idx].kernel().wgsl_statement;
                if buf_idx == 0 {
                    format!("    if (id == {buf_idx}u) {{ {stmt} }}")
                } else {
                    format!(" else if (id == {buf_idx}u) {{ {stmt} }}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let merged = GpuKernel::new(branches);

        let transformed = self
            .ctx
            .dispatch_scalar::<f32>(&values, &merged)
            .unwrap_or_else(|_| values.clone());

        let mut out: Vec<KpResult> = self
            .gpu_kps
            .iter()
            .enumerate()
            .map(|(i, _)| KpResult {
                index: i,
                value: None,
            })
            .collect();

        for (buf_idx, &kp_idx) in indices.iter().enumerate() {
            out[kp_idx].value = Some(transformed[buf_idx]);
        }
        out
    }

    /// Run all KPs across **many roots** — one GPU dispatch for the entire batch.
    pub fn run_many(&self, roots: &[R]) -> Vec<Vec<KpResult>> {
        let n_kps = self.gpu_kps.len();

        let flat: Vec<f32> = roots
            .iter()
            .flat_map(|root| {
                (0..n_kps)
                    .map(|ki| self.gpu_kps[ki].extract_f32(root).unwrap_or(0.0))
                    .collect::<Vec<_>>()
            })
            .collect();

        let branches: String = (0..n_kps)
            .map(|ki| {
                let stmt = &self.gpu_kps[ki].kernel().wgsl_statement;
                let cond = format!("(id % {n}u == {ki}u)", n = n_kps, ki = ki);
                if ki == 0 {
                    format!("    if {cond} {{ {stmt} }}")
                } else {
                    format!(" else if {cond} {{ {stmt} }}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let merged = GpuKernel::new(branches);

        let transformed = self
            .ctx
            .dispatch_scalar::<f32>(&flat, &merged)
            .unwrap_or_else(|_| flat.clone());

        roots
            .iter()
            .enumerate()
            .map(|(ri, _)| {
                (0..n_kps)
                    .map(|ki| KpResult {
                        index: ki,
                        value: Some(transformed[ri * n_kps + ki]),
                    })
                    .collect()
            })
            .collect()
    }
}

// ── 7. WgpuContext + dispatch_scalar ────────────────────────────────────────

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
                    label: Some("GpuKp"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        Ok(Self { device, queue })
    }

    /// Core GPU dispatch: `input` → WGSL kernel → `output`, same element type.
    pub fn dispatch_scalar<V: GpuCompatible>(
        &self,
        values: &[V],
        kernel: &GpuKernel,
    ) -> Result<Vec<V>, Box<dyn std::error::Error + Send + Sync>> {
        let n = values.len();
        if n == 0 {
            return Ok(vec![]);
        }
        let size = (n * std::mem::size_of::<V>()) as u64;
        let wgsl_t = V::wgsl_type();
        let ws = kernel.workgroup_size;

        let shader_src = format!(
            r#"
@group(0) @binding(0) var<storage, read>       input:  array<{wgsl_t}>;
@group(0) @binding(1) var<storage, read_write> output: array<{wgsl_t}>;

@compute @workgroup_size({ws})
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{
    let id = gid.x;
    if (id >= arrayLength(&input)) {{ return; }}
{body}
}}
"#,
            wgsl_t = wgsl_t,
            ws = ws,
            body = kernel.wgsl_statement,
        );

        let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gkp_shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(shader_src)),
        });

        let bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "main",
        });

        let input_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gkp_in"),
            contents: bytemuck::cast_slice(values),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let output_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gkp_out"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gkp_rb"),
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bgl,
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

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((n as u32 + ws - 1) / ws, 1, 1);
        }
        enc.copy_buffer_to_buffer(&output_buf, 0, &readback_buf, 0, size);
        self.queue.submit(Some(enc.finish()));

        let slice = readback_buf.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().map_err(|_| "map_async channel closed")??;

        let mapped = slice.get_mapped_range();
        let out: Vec<V> = bytemuck::cast_slice(&mapped).to_vec();
        drop(mapped);
        readback_buf.unmap();
        Ok(out)
    }
}
