//! Example: wgpu runner using **Kp** and **PKp** with the derive macro and a functional API.
//!
//! - Uses `#[derive(Kp, Pkp, Akp)]`; keypaths come from `User::score()` / `User::name()`.
//! - Numeric tier is built with [IntoNumericAKp]: `User::score().into_numeric_akp(wgsl)` (reference-based, no clone of root).
//! - Uses [Kp::map] for a derived keypath: e.g. `score_kp.map(|s: &f32| *s)` or a mapped view; arbitrary tier uses `AKp::new(name_kp)`.
//!
//! Run with: `cargo run --example kp_pkp_wgpu`

use key_paths_derive::{Akp, Kp, Pkp};
use key_paths_iter::wgpu::{IntoNumericAKp, AKpRunner, AKpTier, GpuValue, RunResults, WgpuContext};
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

    // Numeric: typed Kp from derive → into_numeric_akp (uses get by reference; only f32 is copied)
    let score_tier =
        AKpTier::Numeric(User::score().into_numeric_akp("input * 2.0 + 1.0"));

    // Functional API: Kp::map takes a reference (no copy of value in the get path)
    let score_kp = User::score();
    let doubled = score_kp.map(|s: &f32| *s * 2.0);
    let _ = doubled.get(&user); // Some(84.0)

    // Arbitrary: Kp from derive → AKp
    let name_kp: KpType<'static, User, String> = User::name();
    let name_tier = AKpTier::Arbitrary(AKp::new(name_kp));

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

    if let Some(Some(GpuValue::F32(f))) = results.numeric.first() {
        assert!((*f - 85.0).abs() < 1e-5, "expected 85.0, got {}", f);
    }

    Ok(())
}
