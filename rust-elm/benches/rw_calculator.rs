//! RwStore calculator — 100-thread parallel dispatch + zero-clone [`output_state`] reads.
//!
//! ```bash
//! cargo bench -p rust-elm --bench rw_calculator -- --noplot
//! ```

#[path = "../examples/calc_common.rs"]
mod calc_common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use calc_common::{init, update, CalcAction, CalcOp, CalcState};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rust_elm::{allow_state_clones, Environment, Program, RuntimeConfig, RwRuntime, Sub};

const THREADS: usize = 100;
const DIGITS_PER_THREAD: usize = 50;

/// Copy-only view of calculator state — no `CalcState` or `String` clone on the read path.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CalcOutput {
    lhs: Option<f64>,
    op: Option<CalcOp>,
    fresh_rhs: bool,
    display_len: usize,
}

/// Hot-path read for parallel UI threads: only [`Copy`] fields + lengths under one read lock.
fn output_state(
    read: &rust_elm::ReadStore<CalcState, CalcAction>,
) -> CalcOutput {
    read.with_read(|s| CalcOutput {
        lhs: s.lhs,
        op: s.op,
        fresh_rhs: s.fresh_rhs,
        display_len: s.display.len(),
    })
}

/// Full display check without cloning `display` — compares `&str` inside the read guard.
fn output_display_matches(
    read: &rust_elm::ReadStore<CalcState, CalcAction>,
    expected: &str,
) -> bool {
    read.with_read(|s| s.display.as_str() == expected)
}

fn replay(actions: &[CalcAction]) -> CalcState {
    let (mut state, _) = init();
    for &action in actions {
        update(&mut state, action);
    }
    state
}

fn subs(_: &CalcState) -> Sub<CalcAction> {
    Sub::none()
}

fn drain_calc_display(
    read: &rust_elm::ReadStore<CalcState, CalcAction>,
    expected_len: usize,
) {
    let deadline = Instant::now() + Duration::from_secs(120);
    while Instant::now() < deadline {
        if read.with_read(|s| s.display.len() == expected_len) {
            thread::sleep(Duration::from_millis(5));
            if read.with_read(|s| s.display.len() == expected_len) {
                return;
            }
        }
        thread::sleep(Duration::from_micros(50));
    }
    let got = read.with_read(|s| s.display.len());
    panic!("drain timeout: display len {got}, expected {expected_len}");
}

/// Parallel dispatch + verify `output_state` / display match replayed golden state.
fn parallel_correctness_100_threads(
    store: &rust_elm::RwStore<CalcState, CalcAction>,
) -> Result<(), String> {
    store.dispatch(CalcAction::Clear);

    let recorded = Arc::new(Mutex::new(Vec::<CalcAction>::new()));
    let mut handles = Vec::with_capacity(THREADS);

    for _ in 0..THREADS {
        let store = store.clone();
        let recorded = Arc::clone(&recorded);
        handles.push(thread::spawn(move || {
            for _ in 0..DIGITS_PER_THREAD {
                recorded.lock().expect("record").push(CalcAction::Digit(1));
                store.dispatch(CalcAction::Digit(1));
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread join");
    }

    let expected_len = THREADS * DIGITS_PER_THREAD;
    let read = store.read_store();
    drain_calc_display(&read, expected_len);

    let actions = recorded.lock().expect("record").clone();
    let expected = replay(&actions);

    let copy_out = allow_state_clones(0, || output_state(&read));
    let expected_out = CalcOutput {
        lhs: expected.lhs,
        op: expected.op,
        fresh_rhs: expected.fresh_rhs,
        display_len: expected.display.len(),
    };

    if copy_out != expected_out {
        return Err(format!(
            "output_state mismatch: got {copy_out:?}, expected {expected_out:?}"
        ));
    }

    if !output_display_matches(&read, &expected.display) {
        return Err(format!(
            "display mismatch: expected {:?}",
            expected.display
        ));
    }

    let expected_len = THREADS * DIGITS_PER_THREAD;
    if expected.display.len() != expected_len {
        return Err(format!(
            "unexpected display length: {} (want {expected_len})",
            expected.display.len()
        ));
    }

    Ok(())
}

fn bench_parallel_read_copy_fields(c: &mut Criterion) {
    let mut group = c.benchmark_group("rw_calculator_read_output_state");
    group.throughput(Throughput::Elements(1));

    group.bench_function("100_threads_copy_read", |b| {
        b.iter_custom(|iters| {
            let runtime = RwRuntime::from_program(
                Program::new(init, update, subs),
                Environment::new(),
                RuntimeConfig::new(65_536),
            )
            .expect("RwRuntime");
            let store = runtime.rw_store();
            store.dispatch(CalcAction::Clear);
            for _ in 0..100 {
                store.dispatch(CalcAction::Digit(1));
            }

            let read = store.read_store();
            let counter = Arc::new(AtomicUsize::new(0));
            let start = Instant::now();

            thread::scope(|scope| {
                for _ in 0..THREADS {
                    let read = read.clone();
                    let counter = Arc::clone(&counter);
                    scope.spawn(move || {
                        for _ in 0..iters / THREADS as u64 {
                            let out = allow_state_clones(0, || output_state(&read));
                            black_box(out);
                            counter.fetch_add(1, Ordering::Relaxed);
                        }
                    });
                }
            });

            let elapsed = start.elapsed();
            runtime.shutdown();
            elapsed
        });
    });

    group.finish();
}

fn bench_parallel_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("rw_calculator_parallel_dispatch");
    let total = (THREADS * DIGITS_PER_THREAD) as u64;
    group.throughput(Throughput::Elements(total));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(8));

    group.bench_function("100_threads_digit1", |b| {
        b.iter_custom(|iters| {
            let runtime = RwRuntime::from_program(
                Program::new(init, update, subs),
                Environment::new(),
                RuntimeConfig::new(65_536),
            )
            .expect("RwRuntime");
            let store = runtime.rw_store();

            let start = Instant::now();
            for _ in 0..iters {
                store.dispatch(CalcAction::Clear);
                thread::scope(|scope| {
                    for _ in 0..THREADS {
                        let store = store.clone();
                        scope.spawn(move || {
                            for _ in 0..DIGITS_PER_THREAD {
                                store.dispatch(CalcAction::Digit(1));
                            }
                        });
                    }
                });
            }

            let read = store.read_store();
            drain_calc_display(&read, THREADS * DIGITS_PER_THREAD);
            let ok = read.with_read(|s| {
                s.display.len() == THREADS * DIGITS_PER_THREAD
                    && s.display.bytes().all(|b| b == b'1')
            });
            assert!(ok, "parallel dispatch display must be all 1s after drain");

            let copy_ok = allow_state_clones(0, || {
                let out = output_state(&read);
                out.lhs.is_none() && out.op.is_none() && out.display_len == THREADS * DIGITS_PER_THREAD
            });
            assert!(copy_ok, "output_state must match expected Copy fields");

            let elapsed = start.elapsed();
            runtime.shutdown();
            elapsed
        });
    });

    group.finish();
}

fn bench_correctness_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("rw_calculator_correctness");
    group.sample_size(10);

    group.bench_function("100_threads_output_state_vs_replay", |b| {
        b.iter(|| {
            let runtime = RwRuntime::from_program(
                Program::new(init, update, subs),
                Environment::new(),
                RuntimeConfig::new(65_536),
            )
            .expect("RwRuntime");
            let store = runtime.rw_store();
            parallel_correctness_100_threads(&store).expect("correctness");
            runtime.shutdown();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_correctness_check,
    bench_parallel_read_copy_fields,
    bench_parallel_dispatch,
);
criterion_main!(benches);
