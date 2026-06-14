//! Minimal subscription demo — tick + map_msg.
//!
//! ```bash
//! cargo run -p rust-elm --example subscriptions
//! ```

use rust_elm::{Cmd, Environment, Program, Runtime, Sub};
use std::time::Duration;

#[derive(Default, Debug)]
struct App {
    ticks: u32,
    pings: u32,
}

#[derive(Clone, Copy)]
enum Action {
    Tick,
    Ping,
}

fn init() -> (App, Cmd<Action>) {
    (App::default(), Cmd::none())
}

fn update(app: &mut App, action: Action) -> Cmd<Action> {
    match action {
        Action::Tick => app.ticks += 1,
        Action::Ping => app.pings += 1,
    }
    Cmd::none()
}

fn unit() {}

fn ping(_: ()) -> Action {
    Action::Ping
}

fn subscriptions(_: &App) -> Sub<Action> {
    Sub::batch([
        Sub::tick(1, Duration::from_millis(100), || Action::Tick),
        Sub::map_msg(
            Sub::tick(2, Duration::from_millis(120), unit),
            ping,
        ),
    ])
}

fn main() {
    let program = Program::new(init, update, subscriptions);
    let runtime = Runtime::from_program(program, Environment::new(), 32);
    std::thread::sleep(Duration::from_millis(400));
    let ticks = runtime.state.lock().ticks;
    let pings = runtime.state.lock().pings;
    println!("ticks={ticks} pings={pings}");
    runtime.shutdown();
}
