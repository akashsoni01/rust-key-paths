//! RwStore counter — 100-thread parallel `Inc` + zero-clone [`output_state`] reads.
//!
//! Domain: [`examples/counter/common.rs`](../examples/counter/common.rs) (`CounterState { a, b, c }`).
//!
//! ```bash
//! cargo bench -p rust-elm --bench counter -- --noplot
//! ```

#[path = "../examples/counter/common.rs"]
mod counter_common;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use counter_common::{init, update, BucketAction, CounterAction, CounterState};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rust_elm::{allow_state_clones, Environment, Program, RuntimeConfig, RwRuntime, Sub};

const THREADS: usize = 100;
const INCS_PER_THREAD: usize = 50;
const PAGE_VIEWS_KEY: &str = "page_views";

/// Copy-only counter totals — no `CounterState` or `HashMap` clone on the read path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CounterOutput {
    requests: u64,
    page_views: u64,
    errors: u64,
}

fn output_state(read: &rust_elm::ReadStore<CounterState, CounterAction>) -> CounterOutput {
    read.with_read(|s| CounterOutput {
        requests: s.a.get("requests").copied().unwrap_or(0),
        page_views: s.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0),
        errors: s.c.get("errors").copied().unwrap_or(0),
    })
}

fn replay(actions: &[CounterAction]) -> CounterOutput {
    let (mut state, _) = init();
    for action in actions {
        update(&mut state, action.clone());
    }
    CounterOutput {
        requests: state.a.get("requests").copied().unwrap_or(0),
        page_views: state.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0),
        errors: state.c.get("errors").copied().unwrap_or(0),
    }
}

fn inc_page_views() -> CounterAction {
    CounterAction::B(BucketAction::Inc(PAGE_VIEWS_KEY.into()))
}

fn subs(_: &CounterState) -> Sub<CounterAction> {
    Sub::none()
}

fn drain_page_views(
    read: &rust_elm::ReadStore<CounterState, CounterAction>,
    expected: u64,
) {
    let deadline = Instant::now() + Duration::from_secs(120);
    while Instant::now() < deadline {
        if read.with_read(|s| s.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0) == expected) {
            thread::sleep(Duration::from_millis(5));
            if read.with_read(|s| s.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0) == expected) {
                return;
            }
        }
        thread::sleep(Duration::from_micros(50));
    }
    let got = read.with_read(|s| s.b.get(PAGE_VIEWS_KEY).copied().unwrap_or(0));
    panic!("drain timeout: page_views {got}, expected {expected}");
}

fn parallel_correctness_100_threads(
    store: &rust_elm::RwStore<CounterState, CounterAction>,
) -> Result<(), String> {
    let action = inc_page_views();
    let recorded = Arc::new(Mutex::new(Vec::<CounterAction>::new()));
    let mut handles = Vec::with_capacity(THREADS);

    for _ in 0..THREADS {
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
        handle.join().expect("thread join");
    }

    let expected_total = (THREADS * INCS_PER_THREAD) as u64;
    let read = store.read_store();
    drain_page_views(&read, expected_total);

    let actions = recorded.lock().expect("record").clone();
    let golden = replay(&actions);
    let actual = allow_state_clones(0, || output_state(&read));

    if actual != golden {
        return Err(format!(
            "output_state mismatch: got {actual:?}, replay golden {golden:?}"
        ));
    }

    if actual.page_views != expected_total {
        return Err(format!(
            "page_views mutation wrong: got {}, expected {expected_total}",
            actual.page_views
        ));
    }

    if actual.requests != 0 || actual.errors != 0 {
        return Err(format!(
            "untouched buckets mutated: requests={}, errors={}",
            actual.requests, actual.errors
        ));
    }

    Ok(())
}

fn bench_parallel_read_copy_fields(c: &mut Criterion) {
    let mut group = c.benchmark_group("rw_counter_read_output_state");
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
            for _ in 0..100 {
                store.dispatch(inc_page_views());
            }

            let read = store.read_store();
            let start = Instant::now();

            thread::scope(|scope| {
                for _ in 0..THREADS {
                    let read = read.clone();
                    scope.spawn(move || {
                        for _ in 0..iters / THREADS as u64 {
                            let out = allow_state_clones(0, || output_state(&read));
                            black_box(out);
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
    let mut group = c.benchmark_group("rw_counter_parallel_dispatch");
    let total = (THREADS * INCS_PER_THREAD) as u64;
    group.throughput(Throughput::Elements(total));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(8));

    group.bench_function("100_threads_inc_page_views", |b| {
        b.iter_custom(|iters| {
            let runtime = RwRuntime::from_program(
                Program::new(init, update, subs),
                Environment::new(),
                RuntimeConfig::new(65_536),
            )
            .expect("RwRuntime");
            let store = runtime.rw_store();
            let action = inc_page_views();

            let start = Instant::now();
            for _ in 0..iters {
                thread::scope(|scope| {
                    for _ in 0..THREADS {
                        let store = store.clone();
                        let action = action.clone();
                        scope.spawn(move || {
                            for _ in 0..INCS_PER_THREAD {
                                store.dispatch(action.clone());
                            }
                        });
                    }
                });
            }

            let read = store.read_store();
            let expected = (iters as usize * THREADS * INCS_PER_THREAD) as u64;
            drain_page_views(&read, expected);

            let out = allow_state_clones(0, || output_state(&read));
            assert_eq!(
                out.page_views, expected,
                "page_views must reflect all parallel Inc mutations"
            );
            assert_eq!(out.requests, 0, "bucket a must be untouched");
            assert_eq!(out.errors, 0, "bucket c must be untouched");

            let elapsed = start.elapsed();
            runtime.shutdown();
            elapsed
        });
    });

    group.finish();
}

fn bench_correctness_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("rw_counter_correctness");
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
