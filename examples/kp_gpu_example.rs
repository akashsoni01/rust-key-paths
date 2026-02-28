//! Example: GPU-aware keypaths via [key_paths_iter::kp_gpu].
//!
//! Uses KpType directly (no AKp/PKp): `.map_gpu()`, `.par_gpu()`, and [GpuKpRunner]
//! for one GPU dispatch over numeric keypaths.
//!
//! Run with: `cargo run --example kp_gpu_example`

use key_paths_derive::Kp;
use key_paths_iter::kp_gpu::{GpuKpRunner, KpGpuExt, WgpuContext};
use rust_key_paths::KpType;

#[derive(Kp, Debug)]
struct User {
    score: f32,
    age: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = match WgpuContext::new() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("No GPU available; skipping example.");
            return Ok(());
        }
    };

    let user = User {
        score: 42.0,
        age: 30,
    };

    // Single-KP HOF: .map_gpu() mirrors .map() but returns GpuKp
    let score_kp: KpType<'static, User, f32> = User::score();
    let gpu_score = score_kp.map_gpu("output[id] = input[id] * 2.0 + 1.0;");
    let result = gpu_score.run_one(&user, &ctx);
    println!("score * 2 + 1 = {:?}", result); // Some(85.0)

    // Chain with .and_then_gpu()
    let gpu_score2 = User::score().map_gpu("output[id] = input[id] * 2.0;")
        .and_then_gpu("output[id] = output[id] + 1.0;");
    let result2 = gpu_score2.run_one(&user, &ctx);
    println!("(score * 2) + 1 = {:?}", result2);

    // .par_gpu: one call to attach kernel and run over a slice
    let users = vec![
        User { score: 1.0, age: 20 },
        User { score: 2.0, age: 25 },
    ];
    let results: Vec<Option<f32>> =
        User::score().par_gpu("output[id] = input[id] * 3.0;", &users, &ctx);
    println!("par_gpu results: {:?}", results); // [Some(3.0), Some(6.0)]

    // GpuKpRunner: heterogeneous list, one GPU dispatch (all values cast to f32 in the buffer)
    let age_kp: KpType<'static, User, u32> = User::age();
    let runner = GpuKpRunner::new(&ctx)
        .add_f32(User::score().map_gpu("output[id] = input[id] * 2.0;"))
        .add_u32(age_kp.map_gpu("output[id] = input[id] + 1.0;")); // f32 buffer: use 1.0 not 1u
    let kp_results = runner.run(&user);
    println!("Runner results: {:?}", kp_results);

    Ok(())
}
