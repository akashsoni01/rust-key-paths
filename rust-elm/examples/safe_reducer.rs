//! Safe reducer updates: [`safe_reduce_update`] is the default (no state revert).
//! Use [`safe_reduce_rollback`] / [`RollbackCatchReducer`] only when you want rollback.
//!
//! ```bash
//! cargo run -p rust-elm --example safe_reducer
//! ```

use rust_elm::{
    safe_reduce_rollback, safe_reduce_update, CatchReducer, Cmd, Reduce, Reducer,
    RollbackCatchReducer, SafeReduceError,
};

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
    // ── Default: safe_reduce_update (runtime + CatchReducer) ──
    let mut state = Counter { n: 0 };
    let _ = safe_reduce_update(&mut state, risky_reducer, Action::Panic);
    println!("safe_reduce_update: {state:?} (partial mutation kept)");
    assert_eq!(state.n, 999);

    let mut state = Counter { n: 0 };
    let safe = CatchReducer::new(
        Reduce::new(risky_reducer),
        |_: SafeReduceError| Cmd::none(),
    );
    safe.reduce(&mut state, Action::Panic);
    println!("CatchReducer: {state:?} (same — no revert)");
    assert_eq!(state.n, 999);

    // ── Opt-in rollback via checkpoint ──
    let mut state = Counter { n: 0 };
    let mut checkpoint = state.clone();
    let _ = safe_reduce_rollback(&mut state, &mut checkpoint, risky_reducer, Action::Inc);
    let _ = safe_reduce_rollback(&mut state, &mut checkpoint, risky_reducer, Action::Panic);
    println!("safe_reduce_rollback: {state:?} (rolled back via checkpoint swap)");
    assert_eq!(state.n, 1);

    let mut state = Counter { n: 0 };
    let rollback = RollbackCatchReducer::new(
        Reduce::new(risky_reducer),
        |_: SafeReduceError| Cmd::none(),
        &state,
    );
    rollback.reduce(&mut state, Action::Inc);
    rollback.reduce(&mut state, Action::Panic);
    println!("RollbackCatchReducer: {state:?}");
    assert_eq!(state.n, 1);

    println!("safe_reducer example OK");
}
