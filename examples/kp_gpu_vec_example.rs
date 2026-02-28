//! Example: Kp with **value type `Vec<V>`** running on the GPU.
//!
//! Uses [KpGpuVecExt::map_gpu_vec] to attach an element-wise WGSL kernel to a keypath
//! that points to a `Vec<f32>` (or other GpuCompatible element). One GPU dispatch over the whole vector.
//!
//! Run with: `cargo run --example kp_gpu_vec_example`

use key_paths_derive::Kp;
use key_paths_iter::kp_gpu::{KpGpuVecExt, WgpuContext};
use rust_key_paths::KpType;

#[derive(Kp, Debug)]
struct Model {
    /// Weights: vector of f32 updated on GPU
    weights: Vec<f32>,
    #[allow(dead_code)]
    bias: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = match WgpuContext::new() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("No GPU available; skipping example.");
            return Ok(());
        }
    };

    let mut model = Model {
        weights: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        bias: 0.5,
    };

    // KpType<R, Vec<f32>> → map_gpu_vec: one dispatch over the whole vector
    let weights_kp: KpType<'static, Model, Vec<f32>> = Model::weights();
    let gpu_weights = weights_kp.map_gpu_vec("output[id] = input[id] * 2.0 + 1.0;");

    // Read-only: run kernel, get transformed vector (root unchanged)
    let transformed = gpu_weights.run_one(&model, &ctx);
    println!("run_one (read-only): {:?}", transformed);
    // e.g. Some([3.0, 5.0, 7.0, 9.0, 11.0])

    // In-place: run kernel and write result back into model.weights
    let applied = gpu_weights.apply_one(&mut model, &ctx);
    println!("apply_one: {}; model.weights = {:?}", applied, model.weights);

    // Chain kernels with .and_then_gpu
    let model2 = Model {
        weights: vec![10.0, 20.0],
        bias: 0.0,
    };
    let gpu_chain = Model::weights()
        .map_gpu_vec("output[id] = input[id] * 0.5;")
        .and_then_gpu("output[id] = output[id] + 1.0;");
    let out = gpu_chain.run_one(&model2, &ctx);
    println!("chained kernel result: {:?}", out); // e.g. Some([6.0, 11.0])

    Ok(())
}
