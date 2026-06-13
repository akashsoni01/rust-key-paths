use rust_elm::{Environment, Program, ReducerProgram, Runtime, Sub};
use std::time::Duration;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
struct App {
    ticks: u32,
    stream_hits: u32,
    ws_hits: u32,
    pings: u32,
    logged_in: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Login,
    Tick,
    StreamHit,
    WsHit,
    Ping,
}

fn init() -> (App, rust_elm::Cmd<Action>) {
    (App::default(), rust_elm::Cmd::none())
}

fn update(app: &mut App, action: Action) -> rust_elm::Cmd<Action> {
    match action {
        Action::Login => app.logged_in = true,
        Action::Tick => app.ticks += 1,
        Action::StreamHit => app.stream_hits += 1,
        Action::WsHit => app.ws_hits += 1,
        Action::Ping => app.pings += 1,
    }
    rust_elm::Cmd::none()
}

fn unit() {}

fn ping(_: ()) -> Action {
    Action::Ping
}

fn subscriptions(app: &App) -> Sub<Action> {
    let mut subs = Vec::new();
    if app.logged_in {
        subs.push(Sub::tick(1, Duration::from_millis(40), || Action::Tick));
        subs.push(Sub::stream(
            2,
            "demo_stream",
            Duration::from_millis(50),
            || Action::StreamHit,
        ));
        subs.push(Sub::websocket(
            3,
            "wss://demo/ws",
            Duration::from_millis(60),
            || Action::WsHit,
        ));
    }
    subs.push(Sub::map_msg(
        Sub::tick(4, Duration::from_millis(45), unit),
        ping,
    ));
    Sub::batch(subs)
}

#[test]
fn all_subscription_varieties_fire() {
    let program = Program::new(init, update, subscriptions);
    let runtime = Runtime::from_program(program, Environment::new(), 32);
    runtime.dispatch(Action::Login);
    std::thread::sleep(Duration::from_millis(350));
    let app = *runtime.state.lock();
    assert!(app.ticks >= 1, "tick: {}", app.ticks);
    assert!(app.stream_hits >= 1, "stream: {}", app.stream_hits);
    assert!(app.ws_hits >= 1, "websocket: {}", app.ws_hits);
    assert!(app.pings >= 1, "map_msg: {}", app.pings);
    runtime.shutdown();
}

#[test]
fn subscriptions_stop_when_logged_out() {
    let program = ReducerProgram::new(
        rust_elm::Reduce::new(update),
        init,
        subscriptions,
    );
    let runtime = Runtime::from_reducer_program(program, Environment::new(), 32);
    runtime.dispatch(Action::Login);
    std::thread::sleep(Duration::from_millis(200));
    let ticks_before = runtime.state.lock().ticks;

    {
        let mut guard = runtime.state.lock();
        guard.logged_in = false;
    }
    runtime.dispatch(Action::Ping);
    std::thread::sleep(Duration::from_millis(200));
    let ticks_after = runtime.state.lock().ticks;
    assert_eq!(ticks_after, ticks_before);
    runtime.shutdown();
}
