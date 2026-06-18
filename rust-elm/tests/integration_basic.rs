use rust_elm::{start_runtime, Cmd, Effect, Program, ReplayHarness, Runtime, RuntimeConfig, Sub, TestRuntime, Environment, panic_on_state_clone};

panic_on_state_clone! {
    #[derive(Default, PartialEq, Eq, Debug)]
    struct AppState {
        count: i32,
    }
}

fn init() -> (AppState, Cmd<i32>) {
    (AppState { count: 0 }, Cmd::none())
}

fn update(state: &mut AppState, msg: i32) -> Cmd<i32> {
    state.count += msg;
    Cmd::none()
}

fn subscriptions(_: &AppState) -> Sub<i32> {
    Sub::none()
}

#[test]
fn program_init_and_update_via_test_runtime() {
    let mut rt = TestRuntime::new(AppState::default(), update);
    rt.send(2);
    rt.send(3);
    assert_eq!(rt.state.count, 5);
    rt.assert_no_pending_effects();
}

#[test]
fn replay_harness_replays_actions() {
    let mut harness = ReplayHarness::new(AppState::default(), update);
    harness.replay([1, 2, 3]);
    assert_eq!(harness.state.count, 6);
    assert_eq!(harness.log.len(), 3);
}

#[test]
fn runtime_dispatches_and_updates_state() {
    let program = Program::new(init, update, subscriptions);
    let runtime = start_runtime(program, Environment::new(), RuntimeConfig::new(8));
    runtime.dispatch(10);
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_eq!(runtime.state.lock().count, 10);
    runtime.shutdown();
}

#[test]
fn effect_task_produces_follow_up_message() {
    fn update_with_task(state: &mut AppState, msg: i32) -> Cmd<i32> {
        if msg == 0 {
            Cmd::single(Effect::task(1, || Box::pin(async { Ok(7) })))
        } else {
            state.count += msg;
            Cmd::none()
        }
    }
    let program = Program::new(init, update_with_task, subscriptions);
    let runtime = start_runtime(program, Environment::new(), RuntimeConfig::new(8));
    runtime.dispatch(0);
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert_eq!(runtime.state.lock().count, 7);
    runtime.shutdown();
}
