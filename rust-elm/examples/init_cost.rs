//! Runtime / store init cost — helps decide if per-request bootstrap is viable.
//!
//! ```bash
//! cargo run -p rust-elm --example init_cost --release
//! ```
//!
//! See [book/README.md](../book/README.md) § **Runtime init cost estimates**.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use rust_elm::{
    Cmd, Environment, Program, Runtime, RuntimeConfig, RwRuntime, Sub, TeaRuntime,
};

#[derive(Default, Clone)]
struct State {
    map: HashMap<String, u64>,
}

#[derive(Clone, Copy)]
enum Action {
    Ping,
}

fn init() -> (State, Cmd<Action>) {
    let mut map = HashMap::with_capacity(8);
    map.insert("requests".into(), 0);
    (State { map }, Cmd::none())
}

fn update(state: &mut State, action: Action) -> Cmd<Action> {
    match action {
        Action::Ping => {
            *state.map.entry("requests".into()).or_insert(0) += 1;
        }
    }
    Cmd::none()
}

fn subs(_: &State) -> Sub<Action> {
    Sub::none()
}

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}

fn bench<F: FnMut()>(label: &str, runs: usize, mut f: F) -> Duration {
    let mut samples = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        f();
        samples.push(start.elapsed());
    }
    let med = median(samples);
    println!("  {label:<42} {med:>8.2?}  (median of {runs})");
    med
}

fn main() {
    const RUNS: usize = 20;
    let env = Environment::new();

    // Minimal config — typical for a dedicated API worker.
    let lean = RuntimeConfig {
        bus_capacity: 256,
        worker_threads: 1,
        thread_name: "rust-elm-lean",
    };

    // Default config — Tokio workers = logical CPU count, bus 4096.
    let default = RuntimeConfig::default();

    let program = || Program::new(init, update, subs);

    println!("=== rust-elm init cost (release) ===\n");

    println!("── Without runtime (program / state only) ──");
    bench("program init() only", RUNS, || {
        let (_state, _cmd) = init();
    });

    println!();
    println!("── Full runtime bootstrap (lean: 1 Tokio worker, bus 256) ──");
    bench("Mutex Store Runtime boot+shutdown", RUNS, || {
        let rt = Runtime::from_program(program(), env.clone(), lean.clone()).expect("boot");
        rt.shutdown();
    });
    bench("RwStore Runtime boot+shutdown", RUNS, || {
        let rt = RwRuntime::from_program(program(), env.clone(), lean.clone()).expect("boot");
        rt.shutdown();
    });
    bench("TeaStore Runtime boot+shutdown", RUNS, || {
        let rt = TeaRuntime::from_program(program(), env.clone(), lean.clone()).expect("boot");
        rt.shutdown();
    });

    println!();
    println!("── Full runtime bootstrap (default RuntimeConfig) ──");
    bench("Mutex Store Runtime boot+shutdown", RUNS, || {
        let rt = Runtime::from_program(program(), env.clone(), default.clone()).expect("boot");
        rt.shutdown();
    });
    bench("RwStore Runtime boot+shutdown", RUNS, || {
        let rt = RwRuntime::from_program(program(), env.clone(), default.clone()).expect("boot");
        rt.shutdown();
    });
    bench("TeaStore Runtime boot+shutdown", RUNS, || {
        let rt = TeaRuntime::from_program(program(), env.clone(), default.clone()).expect("boot");
        rt.shutdown();
    });

    println!();
    println!("── Boot without shutdown (thread + Tokio until drop) ──");
    bench("Mutex Store boot only", RUNS, || {
        let rt = Runtime::from_program(program(), env.clone(), lean.clone()).expect("boot");
        std::mem::forget(rt);
    });
    bench("RwStore boot only", RUNS, || {
        let rt = RwRuntime::from_program(program(), env.clone(), lean.clone()).expect("boot");
        std::mem::forget(rt);
    });

    println!();
    println!("── Marginal cost after runtime is warm (reuse pattern) ──");
    let rt = Runtime::from_program(program(), env, lean).expect("boot");
    let store = rt.store();
    bench("Store handle clone", RUNS, || {
        let _s = store.clone();
    });
    bench("dispatch enqueue only", RUNS, || {
        store.dispatch(Action::Ping);
    });
    rt.shutdown();

    println!();
    println!("── Verdict ──");
    println!("  Per-request FULL runtime boot+shutdown is **~100–400 µs** (lean–default config).");
    println!("  program init() alone is **~400 ns**.");
    println!("  Reuse one runtime per process/worker; clone Store/RwStore handles per task.");
    println!();
    println!("  Design patterns:");
    println!("    • One runtime at process start (Actix/Axum worker, tokio main, UI app)");
    println!("    • Session key in state + route actions (not one runtime per HTTP request)");
    println!("    • ScopedStore / RwStore::scope for per-tenant child state");
    println!("    • Fresh domain state without runtime: call init() + reduce in tests only");
    println!("    • Per-request isolation: reset action or IfLet dismiss, not new Runtime");
}
