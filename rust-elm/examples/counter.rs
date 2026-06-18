//! Three named counter buckets (`a`, `b`, `c`) in one root struct on [`TeaStore`].
//!
//! **Keypaths** focus dispatch and reads on one bucket (e.g. `b`) without cloning
//! the whole struct on the **main thread** — but see clone table below.
//!
//! | Read API | Main thread clone |
//! |----------|-------------------|
//! | `view.try_with_snapshot(\|s\| s.b.get(...))` | **None** — borrow `b` through `Arc<CounterState>` |
//! | `b_scope.subscribe_state().next()` | **Only `b` HashMap** (parent `Arc` is refcount) |
//! | `store.state()` | **Full `CounterState`** (all three maps) |
//!
//! **Reducer thread:** TeaStore always publishes `Arc::new(state.clone())` — the **entire
//! root struct** is cloned on the reducer when pushing snapshots / RPC replies.
//!
//! Compare all three backends: `cargo run -p rust-elm --example swap_counter --features arc-swap`
//!
//! ```bash
//! cargo run -p rust-elm --example counter
//! ```
//!
//! See [`book/counter.md`](../book/counter.md).

#[path = "counter/common.rs"]
mod common;

use std::time::Duration;

use common::{b_lens, b_page_views, init, update, subscriptions, BucketAction, CounterAction};
use rust_elm::{Environment, Program, RuntimeConfig, TeaRuntime, TeaStoreError};

fn demo_borrow_b(store: &rust_elm::TeaStore<common::CounterState, CounterAction>) -> Result<(), TeaStoreError> {
    println!("\n--- borrow `b` only (no HashMap clone on main) ---");
    let views = store
        .view_store()
        .try_with_snapshot(|s| b_page_views(s))?;
    println!("try_with_snapshot b.page_views={views}");
    Ok(())
}

fn demo_scoped_b(store: &rust_elm::TeaStore<common::CounterState, CounterAction>) -> Result<(), TeaStoreError> {
    println!("\n--- ScopedTeaStore on `b` (dispatch + subscribe child only) ---");
    let b_scope = store.scope(b_lens(), CounterAction::b_cp());
    let mut sub = b_scope.subscribe_state()?;

    b_scope.dispatch(BucketAction::Inc("page_views".into()));
    b_scope.dispatch(BucketAction::Inc("page_views".into()));

    if let Some(owned_b) = sub.wait_next(Duration::from_millis(200)) {
        println!(
            "scoped subscribe — owned clone of **b only**: page_views={}",
            owned_b.get("page_views").copied().unwrap_or(0)
        );
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = TeaRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(32),
    )?;
    let store = runtime.tea_store();

    println!("=== CounterState {{ a, b, c }} on TeaStore ===");

    store.dispatch(CounterAction::B(BucketAction::Inc("page_views".into())));
    std::thread::sleep(Duration::from_millis(50));

    demo_borrow_b(&store)?;
    demo_scoped_b(&store)?;

    println!("\nCompare Tea / Rw / Swap: cargo run -p rust-elm --example swap_counter --features arc-swap");

    runtime.shutdown();
    println!("\ncounter example OK");
    Ok(())
}
