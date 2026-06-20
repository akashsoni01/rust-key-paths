//! **One runtime per worker** — handlers clone [`RwStore`] (recommended API/server pattern).
//!
//! Simulates two Actix/Axum worker processes. Each worker:
//! 1. Boots **one** [`RwRuntime`] at thread start
//! 2. Serves many fake HTTP requests by cloning `store` (cheap `Arc` handle)
//! 3. Reads session state with **Copy-only** `output_state` (no `AppState` clone)
//! 4. Shuts down runtime when the worker exits
//!
//! ```bash
//! cargo run -p rust-elm --example api_worker --release
//! ```
//!
//! See [`book/api_worker.md`](../book/api_worker.md).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use rust_elm::{
    allow_state_clones, Cmd, Environment, Program, RuntimeConfig, RwRuntime, Sub,
};

// ── Domain ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Session {
    page_views: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AppState {
    sessions: HashMap<u64, Session>,
    total_requests: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    /// First touch for a session id (lazy create).
    OpenSession(u64),
    /// Mutate session — like POST /track.
    IncPageView(u64),
    /// Global counter — every HTTP hit.
    RecordRequest,
}

fn init() -> (AppState, Cmd<Action>) {
    (AppState::default(), Cmd::none())
}

fn update(state: &mut AppState, action: Action) -> Cmd<Action> {
    match action {
        Action::RecordRequest => {
            state.total_requests += 1;
        }
        Action::OpenSession(id) => {
            state.sessions.entry(id).or_default();
        }
        Action::IncPageView(id) => {
            if let Some(session) = state.sessions.get_mut(&id) {
                session.page_views += 1;
            }
        }
    }
    Cmd::none()
}

fn subscriptions(_: &AppState) -> Sub<Action> {
    Sub::none()
}

// ── Copy-only read snapshot (GET handlers) ───────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SessionOutput {
    page_views: u64,
    total_requests: u64,
}

fn output_session(
    store: &rust_elm::RwStore<AppState, Action>,
    session_id: u64,
) -> SessionOutput {
    store.read_store().with_read(|s| SessionOutput {
        page_views: s
            .sessions
            .get(&session_id)
            .map(|sess| sess.page_views)
            .unwrap_or(0),
        total_requests: s.total_requests,
    })
}

// ── Fake HTTP handler (one request) ──────────────────────────────────────────

/// Handler body — **`store` is cloned per request** (~40 ns), runtime is not rebooted.
fn handle_request(store: &rust_elm::RwStore<AppState, Action>, session_id: u64, post: bool) {
    store.dispatch(Action::RecordRequest);
    store.dispatch(Action::OpenSession(session_id));
    if post {
        store.dispatch(Action::IncPageView(session_id));
    }
    let _view = allow_state_clones(0, || output_session(store, session_id));
}

fn drain_total_requests(
    store: &rust_elm::RwStore<AppState, Action>,
    expected: u64,
) {
    let read = store.read_store();
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if read.with_read(|s| s.total_requests) == expected {
            thread::sleep(Duration::from_millis(2));
            if read.with_read(|s| s.total_requests) == expected {
                return;
            }
        }
        thread::sleep(Duration::from_micros(50));
    }
    panic!(
        "drain timeout: total_requests {} (expected {expected})",
        read.with_read(|s| s.total_requests)
    );
}

// ── One worker thread (= one OS thread / Actix worker) ─────────────────────────

const HANDLER_THREADS: usize = 8;
const REQUESTS_PER_HANDLER: usize = 25;

fn run_worker(worker_id: usize) -> (u64, Duration) {
    let boot_start = Instant::now();
    let runtime = RwRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig {
            bus_capacity: 4_096,
            worker_threads: 1,
            thread_name: "rust-elm-api",
        },
    )
    .expect("RwRuntime boot");
    let boot_time = boot_start.elapsed();

    // Share store across handler threads — same pattern as `web::Data<Store>` or `Arc<RwStore>`.
    let store = Arc::new(runtime.rw_store());

    let requests_done = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::with_capacity(HANDLER_THREADS);

    for handler_id in 0..HANDLER_THREADS {
        let store = Arc::clone(&store);
        let requests_done = Arc::clone(&requests_done);
        handles.push(thread::spawn(move || {
            for seq in 0..REQUESTS_PER_HANDLER {
                let session_id = (worker_id as u64 * 1_000) + (handler_id as u64 * 100) + seq as u64;
                let post = seq % 3 == 0;
                handle_request(&store, session_id, post);
                requests_done.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("handler join");
    }

    let total = (HANDLER_THREADS * REQUESTS_PER_HANDLER) as u64;
    drain_total_requests(&store, total);

    let out = allow_state_clones(0, || output_session(&store, 0));
    assert_eq!(
        out.total_requests, total,
        "worker {worker_id}: global request count"
    );

    runtime.shutdown();
    (total, boot_time)
}

fn main() {
    const WORKERS: usize = 2;

    println!("=== api_worker: one RwRuntime per worker, cloned store per request ===\n");
    println!("  workers={WORKERS}  handlers/worker={HANDLER_THREADS}  requests/handler={REQUESTS_PER_HANDLER}\n");

    let mut grand_total = 0u64;
    let mut boot_samples = Vec::with_capacity(WORKERS);

    let worker_handles: Vec<_> = (0..WORKERS)
        .map(|worker_id| thread::spawn(move || run_worker(worker_id)))
        .collect();

    for handle in worker_handles {
        let (n, boot) = handle.join().expect("worker join");
        grand_total += n;
        boot_samples.push(boot);
        println!("  worker finished: {n} requests, runtime boot {boot:.2?}");
    }

    boot_samples.sort();
    let median_boot = boot_samples[boot_samples.len() / 2];

    println!();
    println!("  grand total requests (both workers): {grand_total}");
    println!("  median runtime boot per worker:      {median_boot:.2?}");
    println!();
    println!("  Pattern: boot Runtime once → Arc<RwStore> → clone per handler/request");
    println!("  NOT:     new Runtime per HTTP request (see init_cost example)");
    println!();
    println!("api_worker OK");
}
