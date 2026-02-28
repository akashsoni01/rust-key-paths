//! **Finance-focused GPU example**: Monte Carlo option valuation, batch option intrinsics,
//! and portfolio stress-test — all driven by keypaths and [key_paths_iter::kp_gpu].
//!
//! Demonstrates real-life scale: hundreds of thousands of payoff paths and thousands of
//! positions in a single GPU dispatch.
//!
//! Run with: `cargo run --example kp_gpu_practical_app`

use key_paths_derive::Kp;
use key_paths_iter::kp_gpu::{KpGpuExt, KpGpuVecExt, WgpuContext};
use std::time::Instant;

// ─── Monte Carlo: one option, many paths ───────────────────────────────────────

/// Simulated discounted payoffs for a European option (e.g. call/put from paths).
#[derive(Kp, Debug)]
struct MonteCarloOption {
    /// One payoff per path (e.g. max(0, S_T - K) discounted).
    payoffs: Vec<f32>,
}

// ─── Batch of options (e.g. book of vanilla options) ───────────────────────────

/// Single option contract: we precompute intrinsic = spot - strike for the GPU batch.
#[derive(Kp, Debug, Clone)]
struct OptionContract {
    #[allow(dead_code)]
    spot: f32,
    #[allow(dead_code)]
    strike: f32,
    /// Precomputed spot - strike; GPU will cap at 0 for call intrinsic.
    intrinsic: f32,
}

// ─── Portfolio stress: factor sensitivities ───────────────────────────────────

/// Risk position with sensitivities to risk factors (e.g. rates, equity, vol).
#[derive(Kp, Debug)]
struct RiskPosition {
    /// Sensitivity to each factor; we flatten many positions for one big GPU vector.
    sensitivities: Vec<f32>,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = match WgpuContext::new() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("No GPU available; skipping example.");
            return Ok(());
        }
    };

    println!("═══ GPU in real-life finance ═══\n");

    // ─── 1. Monte Carlo: 500k paths, one dispatch ─────────────────────────────
    const N_PATHS: usize = 5000_000;
    let payoffs: Vec<f32> = (0..N_PATHS)
        .map(|i| {
            // Simulate payoffs (e.g. S_T - K with some negative for OTM)
            let t = i as f32 / N_PATHS as f32;
            (t * 20.0 - 8.0) + (i as f32 * 0.00001).sin()
        })
        .collect();

    let mc_option = MonteCarloOption { payoffs };

    let gpu_payoffs = MonteCarloOption::payoffs()
        .map_gpu_vec("output[id] = max(0.0, input[id]);"); // ensure non-negative payoffs

    let t0 = Instant::now();
    let capped = gpu_payoffs.run_one(&mc_option, &ctx).unwrap_or_default();
    let gpu_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let option_value: f32 = if capped.is_empty() {
        0.0
    } else {
        capped.iter().sum::<f32>() / capped.len() as f32
    };

    println!("1. Monte Carlo option ({} paths)", N_PATHS);
    println!("   GPU: max(0, payoff) over all paths → option value = {:.6}", option_value);
    println!("   Dispatch + readback: {:.2} ms\n", gpu_ms);

    // ─── 2. Batch of options: intrinsic value for many contracts ───────────────
    const N_OPTIONS: usize = 2000;
    let options: Vec<OptionContract> = (0..N_OPTIONS)
        .map(|i| {
            let spot = 100.0 + (i as f32 * 0.1).sin() * 10.0;
            let strike = 98.0 + (i as f32 * 0.07).cos() * 5.0;
            OptionContract {
                spot,
                strike,
                intrinsic: spot - strike,
            }
        })
        .collect();

    let gpu_intrinsic =
        OptionContract::intrinsic().map_gpu("output[id] = max(0.0, input[id]);");
    let t1 = Instant::now();
    let intrinsics: Vec<Option<f32>> = gpu_intrinsic.run_many(&options, &ctx);
    let batch_ms = t1.elapsed().as_secs_f64() * 1000.0;

    let total_intrinsic: f32 = intrinsics.iter().filter_map(|x| *x).sum();
    let in_the_money = intrinsics.iter().filter(|x| x.unwrap_or(0.0) > 0.0).count();

    println!("2. Batch option intrinsics ({} contracts)", N_OPTIONS);
    println!("   One GPU dispatch: max(0, spot - strike) for all");
    println!("   Total intrinsic: {:.2}, ITM count: {}", total_intrinsic, in_the_money);
    println!("   Time: {:.2} ms\n", batch_ms);

    // ─── 3. Portfolio stress: factor sensitivities × shock ─────────────────────
    const N_POSITIONS: usize = 50;
    const N_FACTORS: usize = 20;
    const STRESS_FACTOR: f32 = 1.5;

    let sensitivities: Vec<f32> = (0..N_POSITIONS * N_FACTORS)
        .map(|i| ((i as f32 * 0.01).sin() * 100.0) as f32)
        .collect();

    let portfolio = RiskPosition { sensitivities };
    let gpu_stress = RiskPosition::sensitivities().map_gpu_vec(format!(
        "output[id] = input[id] * {:.1};",
        STRESS_FACTOR
    ));

    let t2 = Instant::now();
    let stressed = gpu_stress.run_one(&portfolio, &ctx).unwrap_or_default();
    let stress_ms = t2.elapsed().as_secs_f64() * 1000.0;

    let total_stressed_pnl: f32 = stressed.iter().sum();

    println!("3. Portfolio stress test ({} positions × {} factors = {} values)",
        N_POSITIONS, N_FACTORS, N_POSITIONS * N_FACTORS);
    println!("   GPU: sensitivities × {:.1} in one dispatch", STRESS_FACTOR);
    println!("   Total stressed PnL (sum): {:.2}", total_stressed_pnl);
    println!("   Time: {:.2} ms\n", stress_ms);

    println!("═══ Done: GPU handled {} + {} + {} elements in 3 dispatches ═══",
        N_PATHS, N_OPTIONS, N_POSITIONS * N_FACTORS);

    Ok(())
}
