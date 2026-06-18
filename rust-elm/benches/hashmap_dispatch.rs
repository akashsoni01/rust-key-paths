//! Throughput benchmarks: raw `HashMap::insert` vs end-to-end `Store::dispatch`.
//!
//! Run: `cargo bench -p rust-elm --bench hashmap_dispatch -- --noplot`

use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rust_elm::{start_runtime, Cmd, Environment, Program, Runtime, RuntimeConfig, Sub};

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
            Cmd::none()
        }
    }
}

fn subs(_: &State) -> Sub<Action> {
    Sub::none()
}

fn bench_raw_hashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("raw_hashmap_insert");
    group.throughput(Throughput::Elements(1));
    group.bench_function("u64_insert", |b| {
        let mut map = HashMap::with_capacity(1024);
        let mut i = 0u64;
        b.iter(|| {
            map.insert(black_box(i), black_box(i));
            i = i.wrapping_add(1);
        });
    });
    group.finish();
}

fn drain_store(store: &rust_elm::Store<State, Action>, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while store.state().map.len() < expected {
        assert!(Instant::now() < deadline, "reducer did not drain queue in time");
        thread::sleep(Duration::from_micros(100));
    }
}

fn bench_runtime_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("runtime_hashmap_insert");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("single_thread_dispatch", |b| {
        b.iter_custom(|iters| {
            let program = Program::new(init, update, subs);
            let runtime = start_runtime(program, Environment::new(), RuntimeConfig::new(16_384));
            let store = runtime.store();
            let start = Instant::now();
            for i in 0..iters {
                store.dispatch(Action::Insert(i));
            }
            drain_store(&store, iters as usize);
            let elapsed = start.elapsed();
            runtime.shutdown();
            elapsed
        });
    });
    group.finish();
}

criterion_group!(benches, bench_raw_hashmap, bench_runtime_dispatch);
criterion_main!(benches);
