//! **Exponential backoff retry** — flaky fetch with sleep between attempts.
//!
//! Compares (two sequential runs):
//! - [`Effect::retry`] — immediate re-attempts (no delay)
//! - [`Effect::retry_backoff`] — exponential sleep between attempts
//!
//! ```bash
//! cargo run -p rust-elm --example backoff_retry --release
//! ```

use rust_elm::{start_runtime, Cmd, Effect, Environment, Program, RuntimeConfig, Sub};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

const FAILURES_BEFORE_SUCCESS: u32 = 3;

static ATTEMPTS: AtomicU32 = AtomicU32::new(0);
static FAILS_REMAINING: AtomicU32 = AtomicU32::new(0);

#[derive(Default, Debug, PartialEq, Eq)]
struct App {
    result: Option<Result<String, u32>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Action {
    RunImmediate,
    RunBackoff,
    Done(Result<String, u32>),
}

fn reset_flaky() {
    FAILS_REMAINING.store(FAILURES_BEFORE_SUCCESS, Ordering::SeqCst);
    ATTEMPTS.store(0, Ordering::SeqCst);
}

fn init() -> (App, Cmd<Action>) {
    (App::default(), Cmd::none())
}

async fn simulate_fetch() -> Result<String, ()> {
    tokio::time::sleep(Duration::from_millis(5)).await;
    ATTEMPTS.fetch_add(1, Ordering::SeqCst);
    if FAILS_REMAINING.fetch_sub(1, Ordering::SeqCst) > 0 {
        Err(())
    } else {
        Ok("payload-v1".into())
    }
}

fn fetch_effect() -> Effect<Action> {
    Effect::from_fn(|| {
        Box::pin(async move {
            match simulate_fetch().await {
                Ok(data) => Ok(Action::Done(Ok(data))),
                Err(()) => Err(rust_elm::EffectError::TaskFailed("fetch failed")),
            }
        })
    })
}

fn immediate_retry_effect() -> Effect<Action> {
    Effect::retry(6, fetch_effect())
}

fn backoff_retry_effect() -> Effect<Action> {
    Effect::retry_backoff(
        6,
        Duration::from_millis(50),
        Duration::from_secs(2),
        fetch_effect(),
    )
}

fn update(app: &mut App, action: Action) -> Cmd<Action> {
    match action {
        Action::RunImmediate => Cmd::single(immediate_retry_effect()),
        Action::RunBackoff => Cmd::single(backoff_retry_effect()),
        Action::Done(result) => {
            app.result = Some(result);
            Cmd::none()
        }
    }
}

fn subscriptions(_: &App) -> Sub<Action> {
    Sub::none()
}

fn run_case(label: &str, start: Action) -> (Result<String, u32>, u32, Duration) {
    reset_flaky();
    let wall = Instant::now();
    let runtime = start_runtime(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(64),
    );
    runtime.dispatch(start);

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let done = runtime.state.lock().result.clone();
        if let Some(result) = done {
            let attempts = ATTEMPTS.load(Ordering::SeqCst);
            let elapsed = wall.elapsed();
            runtime.shutdown();
            println!("{label}: result={result:?} attempts={attempts} elapsed={elapsed:.2?}");
            return (result, attempts, elapsed);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("{label}: timed out");
}

fn main() {
    println!("=== backoff_retry (fail {FAILURES_BEFORE_SUCCESS}x then succeed) ===\n");

    let (imm_result, imm_attempts, imm_elapsed) =
        run_case("Effect::retry (immediate)", Action::RunImmediate);
    let (back_result, back_attempts, back_elapsed) =
        run_case("Effect::retry_backoff", Action::RunBackoff);

    assert_eq!(imm_result, Ok("payload-v1".into()));
    assert_eq!(back_result, Ok("payload-v1".into()));
    assert_eq!(imm_attempts, FAILURES_BEFORE_SUCCESS + 1);
    assert_eq!(back_attempts, FAILURES_BEFORE_SUCCESS + 1);
    assert!(
        back_elapsed > imm_elapsed,
        "backoff should take longer due to sleeps ({back_elapsed:?} vs {imm_elapsed:?})"
    );

    println!();
    println!("backoff_retry example OK");
}
