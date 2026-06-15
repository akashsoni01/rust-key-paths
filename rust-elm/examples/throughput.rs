//! One-shot throughput printout for book estimates.
//!
//! ```bash
//! cargo run -p rust-elm --example throughput --release
//! ```

use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use rust_elm::{Cmd, Environment, Program, Runtime, RuntimeConfig, Sub};

#[derive(Default, Clone)]
struct State {
    map: HashMap<u64, u64>,
}

#[derive(Clone, Copy)]
enum Action {
    Insert(u64),
}

fn init() -> (State, Cmd<Action>) {
    (State::default(), Cmd::none())
}

fn update(state: &mut State, action: Action) -> Cmd<Action> {
    match action {
        Action::Insert(k) => {
            state.map.insert(k, k);
        }
    }
    Cmd::none()
}

fn subs(_: &State) -> Sub<Action> {
    Sub::none()
}

fn drain(store: &rust_elm::Store<State, Action>, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while store.state().map.len() < expected {
        assert!(
            Instant::now() < deadline,
            "queue did not drain (got {})",
            store.state().map.len()
        );
        thread::sleep(Duration::from_micros(50));
    }
}

fn run_parallel(threads: usize, per: usize, bus: usize) -> (f64, u64) {
    let total = threads * per;
    let runtime = Runtime::from_program(
        Program::new(init, update, subs),
        Environment::new(),
        RuntimeConfig::new(bus),
    );
    let store = runtime.store();
    let start = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let store = store.clone();
            thread::spawn(move || {
                for i in 0..per {
                    store.dispatch(Action::Insert(t as u64 * per as u64 + i as u64));
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    drain(&store, total);
    let tps = total as f64 / start.elapsed().as_secs_f64();
    let dropped = runtime.bus.dropped_count();
    runtime.shutdown();
    (tps, dropped)
}

fn main() {
    let n = 1_000_000u64;
    let start = Instant::now();
    let mut map = HashMap::with_capacity(n as usize);
    for i in 0..n {
        map.insert(i, i);
    }
    let raw_tps = n as f64 / start.elapsed().as_secs_f64();
    println!("raw HashMap<u64,u64>::insert (1M keys): {raw_tps:.0} TPS");

    let (single, _) = run_parallel(1, 100_000, 65_536);
    println!("rust-elm single-thread (100k, bus=65536): {single:.0} TPS");

    for bus in [4_096, 16_384, 65_536, 131_072] {
        let (tps, dropped) = run_parallel(50, 1_000, bus);
        println!("50 threads x 1k = 50k (bus={bus}): {tps:.0} TPS, dropped={dropped}");
    }
}
