//! Template: **RwStore parallel read correctness** — Copy-only [`output_state`], replay golden, 100 threads.
//!
//! Use this as a starting point for:
//! - UI / worker threads reading state without cloning `CounterState`
//! - Verifying mutations under concurrent `dispatch` from many threads
//! - [`benches/counter.rs`](../benches/counter.rs) and [`benches/rw_calculator.rs`](../benches/rw_calculator.rs)
//!
//! ```bash
//! cargo run -p rust-elm --example read_correctness --release
//! ```

#[path = "counter/common.rs"]
mod common;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use common::{init, BucketAction, CounterAction};
use rust_elm::{
    allow_state_clones, panic_on_state_clone, Cmd, Environment, Program, RuntimeConfig, ReadStore,
    RwRuntime, RwStore, Sub,
};

// ── Tunables (match `benches/counter.rs` defaults) ───────────────────────────

const WRITER_THREADS: usize = 100;
const INCS_PER_THREAD: usize = 50;
const READER_THREADS: usize = 100;
const READS_PER_THREAD: usize = 200;
const PAGE_VIEWS_KEY: &str = "page_views";
const BUS_CAPACITY: usize = 65_536;

panic_on_state_clone! {
    #[derive(Debug, PartialEq, Eq, key_paths_derive::Kp)]
    struct CounterState {
        a: common::Bucket,
        b: common::Bucket,
        c: common::Bucket,
    }
}

fn program_init() -> (CounterState, Cmd<CounterAction>) {
    let (s, cmd) = init();
    (
        CounterState {
            a: s.a,
            b: s.b,
            c: s.c,
        },
        cmd,
    )
}

fn program_update(state: &mut CounterState, action: CounterAction) -> Cmd<CounterAction> {
    match action {
        CounterAction::A(a) => common::reduce_bucket(&mut state.a, a),
        CounterAction::B(b) => common::reduce_bucket(&mut state.b, b),
        CounterAction::C(c) => common::reduce_bucket(&mut state.c, c),
    }
    Cmd::none()
}

fn subscriptions(_: &CounterState) -> Sub<CounterAction> {
    Sub::none()
}

// ── Copy-only read snapshot (no `CounterState` / `HashMap` clone) ─────────────

/// Hot-path view for parallel readers — every field is [`Copy`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CounterOutput {
    requests: u64,
    page_views: u64,
    errors: u64,
}

/// Read totals under one `RwLock` read guard; clone only `Copy` values out.
fn output_state(read: &ReadStore<CounterState, CounterAction>) -> CounterOutput {
    read.with_read(|s| CounterOutput {
        requests: s.a.get("requests").copied().unwrap_or(0),
        page_views: s.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0),
        errors: s.c.get("errors").copied().unwrap_or(0),
    })
}

fn inc_page_views() -> CounterAction {
    CounterAction::B(BucketAction::Inc(PAGE_VIEWS_KEY.into()))
}

/// Local golden: replay recorded actions through the same reducer (single-threaded).
fn replay_golden(actions: &[CounterAction]) -> CounterOutput {
    let (mut state, _) = program_init();
    for action in actions {
        program_update(&mut state, action.clone());
    }
    CounterOutput {
        requests: state.a.get("requests").copied().unwrap_or(0),
        page_views: state.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0),
        errors: state.c.get("errors").copied().unwrap_or(0),
    }
}

fn drain_page_views(read: &ReadStore<CounterState, CounterAction>, expected: u64) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if read.with_read(|s| s.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0)) == expected {
            thread::sleep(Duration::from_millis(5));
            if read.with_read(|s| s.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0)) == expected {
                return;
            }
        }
        thread::sleep(Duration::from_micros(50));
    }
    let got = read.with_read(|s| s.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0));
    panic!("drain timeout: page_views {got}, expected {expected}");
}

// ── Parallel writers ─────────────────────────────────────────────────────────

fn parallel_writes(
    store: &RwStore<CounterState, CounterAction>,
) -> Vec<CounterAction> {
    let action = inc_page_views();
    let recorded = Arc::new(Mutex::new(Vec::<CounterAction>::new()));
    let mut handles = Vec::with_capacity(WRITER_THREADS);

    for _ in 0..WRITER_THREADS {
        let store = store.clone();
        let recorded = Arc::clone(&recorded);
        let action = action.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..INCS_PER_THREAD {
                recorded.lock().expect("record").push(action.clone());
                store.dispatch(action.clone());
            }
        }));
    }

    for handle in handles {
        handle.join().expect("writer join");
    }

    recorded.lock().expect("record").clone()
}

// ── Parallel Copy-only readers ───────────────────────────────────────────────

/// Many threads call [`output_state`] concurrently; each snapshot must match `expected`.
fn parallel_read_verify(
    read: &ReadStore<CounterState, CounterAction>,
    expected: CounterOutput,
) -> Result<(), String> {
    let read = read.clone();
    let failures = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut handles = Vec::with_capacity(READER_THREADS);

    for tid in 0..READER_THREADS {
        let read = read.clone();
        let failures = Arc::clone(&failures);
        handles.push(thread::spawn(move || {
            for _ in 0..READS_PER_THREAD {
                let snap = allow_state_clones(0, || output_state(&read));
                if snap != expected {
                    failures
                        .lock()
                        .expect("failures")
                        .push(format!("reader {tid}: got {snap:?}, want {expected:?}"));
                    return;
                }
            }
        }));
    }

    for handle in handles {
        handle.join().expect("reader join");
    }

    let msgs = failures.lock().expect("failures").clone();
    if msgs.is_empty() {
        Ok(())
    } else {
        Err(msgs.join("; "))
    }
}

// ── Verification ─────────────────────────────────────────────────────────────

fn verify_parallel_correctness(
    store: &RwStore<CounterState, CounterAction>,
    recorded: &[CounterAction],
) -> Result<(), String> {
    let expected_total = (WRITER_THREADS * INCS_PER_THREAD) as u64;
    let read = store.read_store();

    drain_page_views(&read, expected_total);

    let golden = replay_golden(recorded);
    let actual = allow_state_clones(0, || output_state(&read));

    if actual != golden {
        return Err(format!(
            "output_state vs replay: actual {actual:?}, golden {golden:?}"
        ));
    }

    if actual.page_views != expected_total {
        return Err(format!(
            "mutation count: page_views {} (expected {expected_total})",
            actual.page_views
        ));
    }

    if actual.requests != 0 || actual.errors != 0 {
        return Err(format!(
            "bucket isolation failed: requests={}, errors={} (want 0, 0)",
            actual.requests, actual.errors
        ));
    }

    parallel_read_verify(&read, actual)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== read_correctness: RwStore template ===");
    println!(
        "writers: {WRITER_THREADS}×{INCS_PER_THREAD} Inc | readers: {READER_THREADS}×{READS_PER_THREAD} output_state"
    );

    let runtime = RwRuntime::from_program(
        Program::new(program_init, program_update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(BUS_CAPACITY),
    )?;
    let store = runtime.rw_store();

    let recorded = parallel_writes(&store);

    match verify_parallel_correctness(&store, &recorded) {
        Ok(()) => {
            let out = allow_state_clones(0, || output_state(&store.read_store()));
            println!("PASS — output_state {out:?} matches replay + {READER_THREADS} parallel readers");
        }
        Err(err) => {
            eprintln!("FAIL — {err}");
            std::process::exit(1);
        }
    }

    runtime.shutdown();
    Ok(())
}
