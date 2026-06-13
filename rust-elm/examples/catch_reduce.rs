//! `catch_reduce_panic` vs `catch_reduce` — panic handling with and without state rollback.
//!
//! ```bash
//! cargo run -p rust-elm --example catch_reduce
//! ```

use rust_elm::{catch_reduce, catch_reduce_panic, CatchReducer, Cmd, Reduce, Reducer};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Counter {
    n: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Inc,
    Panic,
}

fn risky_reducer(state: &mut Counter, action: Action) -> Cmd<Action> {
    match action {
        Action::Inc => {
            state.n += 1;
            Cmd::none()
        }
        Action::Panic => {
            state.n = 999;
            panic!("bug in reducer");
        }
    }
}

fn main() {
    // ── catch_reduce_panic: store/runtime style — survives panic, state may be partial ──
    let mut state = Counter { n: 0 };
    let action = Action::Panic;

    let _ = catch_reduce_panic(&mut state, risky_reducer, action);
    println!("catch_reduce_panic: state after panic = {state:?} (partial mutation kept)");

    // ── catch_reduce: reuse committed checkpoint — swap on panic, no pre-reduce clone ──
    let mut state = Counter { n: 0 };
    let mut checkpoint = state.clone();

    let _ = catch_reduce(&mut state, &mut checkpoint, risky_reducer, Action::Inc);
    assert_eq!(state.n, 1);
    assert_eq!(checkpoint.n, 1);

    let _ = catch_reduce(&mut state, &mut checkpoint, risky_reducer, Action::Panic);
    println!("catch_reduce: state after panic = {state:?} (rolled back via checkpoint swap)");
    assert_eq!(state.n, 1);

    // ── CatchReducer: owns the checkpoint for you ──
    let mut state = Counter { n: 0 };
    let safe = CatchReducer::new(
        Reduce::new(risky_reducer),
        |panic: rust_elm::ReducePanic| {
            eprintln!("recovered from: {:?}", panic.message());
            Cmd::none()
        },
        &state,
    );

    safe.reduce(&mut state, Action::Inc);
    safe.reduce(&mut state, Action::Panic);
    println!("CatchReducer: state after panic = {state:?}");
    assert_eq!(state.n, 1);

    println!("catch_reduce example OK");
}
