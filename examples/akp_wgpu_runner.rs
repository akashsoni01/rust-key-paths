//! Example: Run a mix of numeric (GPU) and arbitrary (CPU) keypaths in parallel via [key_paths_iter::wgpu].
//!
//! Run with: `cargo run --example akp_wgpu_runner`
//! (Builds with key-paths-iter gpu feature enabled in dev-dependencies.)

use key_paths_derive::{Akp, Kp, Pkp};
use key_paths_iter::wgpu::{numeric_akp_f32, AKpRunner, AKpTier, GpuValue, RunResults, WgpuContext};
use rust_key_paths::{AKp, KpType};

#[derive(Kp, Pkp, Akp, Debug)]
struct User {
    name: String,
    score: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user = User {
        name: "Alice".to_string(),
        score: 42.0,
    };

    // Numeric KP: score (f32) → GPU tier
    let score_tier = AKpTier::Numeric(numeric_akp_f32::<User>(
        |u| Some(u.score),
        "input * 2.0 + 1.0",
    ));

    // Arbitrary KP: name (String) → CPU tier
    let name_kp: KpType<'static, User, String> = User::name();
    let name_akp = AKp::new(name_kp);
    let name_tier = AKpTier::Arbitrary(name_akp);

    let wgpu_ctx = WgpuContext::new().ok();
    let runner = AKpRunner::new(vec![score_tier, name_tier], wgpu_ctx);

    let results: RunResults = runner.run(&user as &dyn std::any::Any);

    println!("Numeric (GPU or CPU fallback):");
    for (i, v) in results.numeric.iter().enumerate() {
        match v {
            Some(GpuValue::F32(f)) => println!("  [{}] f32 = {}", i, f),
            Some(GpuValue::U32(u)) => println!("  [{}] u32 = {}", i, u),
            None => println!("  [{}] (none)", i),
        }
    }
    println!("Arbitrary KPs run (CPU): {}", results.arbitrary_count);

    // Expect numeric[0] = 85.0 (42.0 * 2.0 + 1.0) when GPU is used
    if let Some(Some(GpuValue::F32(f))) = results.numeric.first() {
        assert!((*f - 85.0).abs() < 1e-5, "expected 85.0, got {}", f);
    }

    Ok(())
}
