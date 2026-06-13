use rust_elm::{
    reducer::coerce_fn, reducers, Cmd, CombineReducers, Reduce, Reducer, ReducerProgram, Runtime,
    panic_on_state_clone, Sub, Environment,
};

panic_on_state_clone! {
    #[derive(Default, Debug, PartialEq, Eq)]
    struct App {
        count: i32,
        doubled: i32,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    N(i32),
}

fn init() -> (App, Cmd<Action>) {
    (App::default(), Cmd::none())
}

fn subscriptions(_: &App) -> Sub<Action> {
    Sub::none()
}

fn count_reducer(state: &mut App, action: Action) -> Cmd<Action> {
    if let Action::N(n) = action {
        state.count += n;
    }
    Cmd::none()
}

fn double_reducer(state: &mut App, action: Action) -> Cmd<Action> {
    if let Action::N(n) = action {
        state.doubled += n * 2;
    }
    Cmd::none()
}

#[test]
fn combine_reducers_apply_in_order() {
    let combined = CombineReducers((coerce_fn(count_reducer), coerce_fn(double_reducer)));
    let mut app = App::default();
    combined.reduce(&mut app, Action::N(3));
    assert_eq!(app.count, 3);
    assert_eq!(app.doubled, 6);
}

#[test]
fn reducers_macro_matches_manual_combine() {
    let manual = CombineReducers((coerce_fn(count_reducer), coerce_fn(double_reducer)));
    let from_macro = reducers![count_reducer, double_reducer];

    let mut a = App::default();
    let mut b = App::default();
    manual.reduce(&mut a, Action::N(5));
    from_macro.reduce(&mut b, Action::N(5));
    assert_eq!(a, b);
}

#[test]
fn reduce_wrapper_is_reducer() {
    let r = Reduce::new(count_reducer);
    let mut app = App::default();
    r.reduce(&mut app, Action::N(1));
    assert_eq!(app.count, 1);
}

#[test]
fn runtime_from_reducer_program() {
    let program = ReducerProgram::new(
        reducers![count_reducer, double_reducer],
        init,
        subscriptions,
    );
    let runtime = Runtime::from_reducer_program(program, Environment::new(), 8);
    runtime.dispatch(Action::N(4));
    std::thread::sleep(std::time::Duration::from_millis(100));
    let state = runtime.state.lock();
    assert_eq!(state.count, 4);
    assert_eq!(state.doubled, 8);
    drop(state);
    runtime.shutdown();
}
