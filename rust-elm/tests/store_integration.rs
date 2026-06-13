use std::time::Duration;

use rust_elm::{Cmd, Effect, Environment, Program, Runtime, Sub};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
struct App {
    count: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    Inc,
    Child(ChildAction),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ChildAction {
    Bump,
}

fn init() -> (App, Cmd<Action>) {
    (App::default(), Cmd::none())
}

fn update(app: &mut App, action: Action) -> Cmd<Action> {
    match action {
        Action::Inc => {
            app.count += 1;
            Cmd::none()
        }
        Action::Child(ChildAction::Bump) => {
            app.count += 10;
            Cmd::none()
        }
    }
}

fn update_with_effect(app: &mut App, action: Action) -> Cmd<Action> {
    match action {
        Action::Inc => Cmd::single(Effect::task(1, || {
            Box::pin(async { Ok(Action::Child(ChildAction::Bump)) })
        })),
        Action::Child(ChildAction::Bump) => {
            app.count += 10;
            Cmd::none()
        }
    }
}

fn subs(_: &App) -> Sub<Action> {
    Sub::none()
}

fn embed(a: ChildAction) -> Action {
    Action::Child(a)
}

fn child_state(app: &App) -> Option<i32> {
    Some(app.count)
}

#[test]
fn store_send_finishes_after_effects() {
    let runtime = Runtime::from_program(
        Program::new(init, update_with_effect, subs),
        Environment::new(),
        16,
    );
    let store = runtime.store();
    let task = store.send(Action::Inc);
    assert!(task.finish().is_ok());
    assert_eq!(store.state().count, 10);
    runtime.shutdown();
}

#[test]
fn store_subscribe_state_dedupes() {
    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), 16);
    let store = runtime.store();
    let mut sub = store.subscribe_state();
    store.dispatch(Action::Inc);
    std::thread::sleep(Duration::from_millis(150));
    let first = sub.wait_next(Duration::from_secs(1)).expect("first snapshot");
    assert_eq!(first.count, 1);
    store.dispatch(Action::Inc);
    std::thread::sleep(Duration::from_millis(150));
    let second = sub.wait_next(Duration::from_secs(1)).expect("second snapshot");
    assert_eq!(second.count, 2);
    runtime.shutdown();
}

#[test]
fn scoped_store_routes_child_actions() {
    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), 16);
    let store = runtime.store();
    let child = store.scope(child_state, embed);
    child.dispatch(ChildAction::Bump);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(child.child_state(), Some(10));
    runtime.shutdown();
}
