//! Panic handling: [`catch_reduce_panic`] is the default (no state revert).
//! Use [`catch_reduce`] / [`RollbackCatchReducer`] only when you want rollback.
//!
//! ```bash
//! cargo run -p rust-elm --example catch_reduce
//! ```

use rust_elm::{
    catch_reduce, catch_reduce_panic, CatchReducer, Cmd, Reduce, Reducer, RollbackCatchReducer,
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
    // ── Default: catch_reduce_panic (runtime + CatchReducer) ──
    let mut state = Counter { n: 0 };
    let _ = catch_reduce_panic(&mut state, risky_reducer, Action::Panic);
    println!("catch_reduce_panic: {state:?} (partial mutation kept)");
    assert_eq!(state.n, 999);

    let mut state = Counter { n: 0 };
    let safe = CatchReducer::new(
        Reduce::new(risky_reducer),
        |_: rust_elm::ReducePanic| Cmd::none(),
    );
    safe.reduce(&mut state, Action::Panic);
    println!("CatchReducer: {state:?} (same — no revert)");
    assert_eq!(state.n, 999);

    // ── Opt-in rollback via checkpoint ──
    let mut state = Counter { n: 0 };
    let mut checkpoint = state.clone();
    let _ = catch_reduce(&mut state, &mut checkpoint, risky_reducer, Action::Inc);
    let _ = catch_reduce(&mut state, &mut checkpoint, risky_reducer, Action::Panic);
    println!("catch_reduce: {state:?} (rolled back via checkpoint swap)");
    assert_eq!(state.n, 1);

    let mut state = Counter { n: 0 };
    let rollback = RollbackCatchReducer::new(
        Reduce::new(risky_reducer),
        |_: rust_elm::ReducePanic| Cmd::none(),
        &state,
    );
    rollback.reduce(&mut state, Action::Inc);
    rollback.reduce(&mut state, Action::Panic);
    println!("RollbackCatchReducer: {state:?}");
    assert_eq!(state.n, 1);

    println!("catch_reduce example OK");
}
