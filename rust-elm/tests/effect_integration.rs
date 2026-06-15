use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use rust_elm::{Cmd, Effect, Environment, panic_on_state_clone, Program, Runtime, RuntimeConfig, RunSender};

panic_on_state_clone! {
    #[derive(Default)]
    struct Counter {
        n: i32,
    }
}

fn init() -> (Counter, Cmd<i32>) {
    (Counter::default(), Cmd::none())
}

fn subs(_: &Counter) -> rust_elm::Sub<i32> {
    rust_elm::Sub::none()
}

#[test]
fn cancel_effect_aborts_in_flight_task() {
    static STARTED: AtomicI32 = AtomicI32::new(0);
    static FINISHED: AtomicI32 = AtomicI32::new(0);

    fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
        match msg {
            0 => Cmd::single(Effect::task(42, || {
                Box::pin(async {
                    STARTED.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    FINISHED.fetch_add(1, Ordering::SeqCst);
                    Ok(99)
                })
            })),
            1 => Cmd::single(Effect::cancel(42)),
            n => {
                s.n += n;
                Cmd::none()
            }
        }
    }

    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), RuntimeConfig::new(16));
    runtime.dispatch(0);
    std::thread::sleep(Duration::from_millis(50));
    runtime.dispatch(1);
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(STARTED.load(Ordering::SeqCst), 1);
    assert_eq!(FINISHED.load(Ordering::SeqCst), 0);
    assert_eq!(runtime.state.lock().n, 0);
    runtime.shutdown();
}

#[test]
fn debounce_coalesces_rapid_fire_to_last() {
    fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
        if msg >= 100 {
            s.n = msg - 100;
            return Cmd::none();
        }
        let payload = msg + 100;
        Cmd::single(Effect::debounce(
            7,
            Duration::from_millis(80),
            Effect::from_fn(move || Box::pin(async move { Ok(payload) })),
        ))
    }

    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), RuntimeConfig::new(16));
    runtime.dispatch(1);
    runtime.dispatch(2);
    runtime.dispatch(3);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(runtime.state.lock().n, 3);
    runtime.shutdown();
}

#[test]
fn throttle_latest_runs_last_in_window() {
    fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
        if msg >= 100 {
            s.n = msg - 100;
            return Cmd::none();
        }
        let payload = msg + 100;
        Cmd::single(Effect::throttle(
            8,
            Duration::from_millis(80),
            true,
            Effect::from_fn(move || Box::pin(async move { Ok(payload) })),
        ))
    }

    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), RuntimeConfig::new(16));
    runtime.dispatch(1);
    runtime.dispatch(2);
    runtime.dispatch(9);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(runtime.state.lock().n, 9);
    runtime.shutdown();
}

#[test]
fn throttle_first_wins_when_not_latest() {
    fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
        if msg >= 100 {
            s.n = msg - 100;
            return Cmd::none();
        }
        let payload = msg + 100;
        Cmd::single(Effect::throttle(
            8,
            Duration::from_millis(120),
            false,
            Effect::from_fn(move || Box::pin(async move { Ok(payload) })),
        ))
    }

    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), RuntimeConfig::new(16));
    runtime.dispatch(4);
    runtime.dispatch(5);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(runtime.state.lock().n, 4);
    runtime.shutdown();
}

#[test]
fn run_effect_can_send_multiple_actions() {
    fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
        if msg == 0 {
            Cmd::single(Effect::from_run(|send: RunSender<i32>| {
                Box::pin(async move {
                    send.send(2);
                    send.send(3);
                    Ok(())
                })
            }))
        } else {
            s.n += msg;
            Cmd::none()
        }
    }

    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), RuntimeConfig::new(16));
    runtime.dispatch(0);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(runtime.state.lock().n, 5);
    runtime.shutdown();
}

#[test]
fn cancellable_cancel_in_flight_replaces_previous() {
    static RUNS: AtomicI32 = AtomicI32::new(0);
    static VALUE: AtomicI32 = AtomicI32::new(0);
    RUNS.store(0, Ordering::SeqCst);

    fn slow_task() -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<i32, rust_elm::EffectError>> + Send>,
    > {
        Box::pin(async {
            RUNS.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok(VALUE.load(Ordering::SeqCst) + 100)
        })
    }

    fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
        if msg >= 100 {
            s.n = msg - 100;
            return Cmd::none();
        }
        VALUE.store(msg, Ordering::SeqCst);
        Cmd::single(Effect::cancellable_with(
            55,
            true,
            Effect::task(55, slow_task),
        ))
    }

    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), RuntimeConfig::new(16));
    runtime.dispatch(1);
    std::thread::sleep(Duration::from_millis(20));
    runtime.dispatch(2);
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(RUNS.load(Ordering::SeqCst), 2);
    assert_eq!(runtime.state.lock().n, 2);
    runtime.shutdown();
}
