//! Compare **TeaStore**, **RwStore**, and **SwapStore** on the same `CounterState { a, b, c }`.
//!
//! All three support keypath-scoped dispatch to bucket `b`. They differ in how state is
//! stored and what gets cloned when you read `b`:
//!
//! | Backend | Main read `b` | Reducer on each update | Reader synchronization |
//! |---------|---------------|------------------------|-------------------------|
//! | **TeaStore** | borrow via RPC `Arc` | **clone full struct** → push `Arc` | channel RPC / push |
//! | **RwStore** | borrow under **read lock** | **in-place mutate** (no struct clone) | `RwLock::read` |
//! | **SwapStore** | borrow via **atomic** `Arc` load | **clone full struct** → `ArcSwap::store` | lock-free load |
//!
//! Tea and Swap both clone the entire root on the reducer when publishing a new snapshot.
//! Rw avoids that. Tea and Swap readers borrow without cloning on main; Swap adds lock-free
//! concurrent reads (no `RwLock`).
//!
//! ```bash
//! cargo run -p rust-elm --example swap_counter --features arc-swap
//! ```
//!
//! See also: [`counter.rs`](counter.rs), [`rw_counter.rs`](rw_counter.rs), [`book/counter.md`](../book/counter.md).

#[path = "counter_common.rs"]
mod common;

use std::time::Duration;

use common::{b_lens, b_page_views, init, update, subscriptions, BucketAction, CounterAction};
use rust_elm::{
    Environment, Program, RuntimeConfig, RwRuntime, SwapRuntime, TeaRuntime,
};

fn print_comparison_table() {
    println!("=== Counter backends — reading bucket `b` only ===\n");
    println!("| Backend   | Main thread read `b`     | Reducer on update          | Reader sync        |");
    println!("|-----------|----------------------------|----------------------------|--------------------|");
    println!("| TeaStore  | borrow via Arc (RPC)       | clone full CounterState    | channel            |");
    println!("| RwStore   | borrow under read lock     | in-place mutate            | RwLock::read       |");
    println!("| SwapStore | borrow via ArcSwap load    | clone full CounterState    | atomic (no lock)   |");
    println!();
}

fn demo_tea() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- TeaRuntime / TeaStore ---");
    let runtime = TeaRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(32),
    )?;
    let store = runtime.tea_store();

    store.dispatch(CounterAction::B(BucketAction::Inc("page_views".into())));
    store.dispatch(CounterAction::B(BucketAction::Inc("page_views".into())));
    std::thread::sleep(Duration::from_millis(50));

    let views = store
        .view_store()
        .try_with_snapshot(|s| b_page_views(s))?;
    println!("try_with_snapshot b.page_views={views} (main: borrow Arc — reducer cloned full struct)");

    let b_scope = store.scope(b_lens(), CounterAction::b_cp());
    b_scope.dispatch(BucketAction::Inc("page_views".into()));
    std::thread::sleep(Duration::from_millis(50));

    let views = store
        .view_store()
        .try_with_snapshot(|s| b_page_views(s))?;
    println!("after scoped Inc: b.page_views={views}");

    runtime.shutdown();
    Ok(())
}

fn demo_rw() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- RwRuntime / RwStore ---");
    let runtime = RwRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(32),
    )?;
    let store = runtime.rw_store();

    store.dispatch(CounterAction::B(BucketAction::Inc("page_views".into())));
    store.dispatch(CounterAction::B(BucketAction::Inc("page_views".into())));
    std::thread::sleep(Duration::from_millis(50));

    let views = store
        .read_store()
        .with_read(|s| b_page_views(s));
    println!("with_read b.page_views={views} (main: borrow under read lock — no struct clone)");

    let b_scope = store.scope(b_lens(), CounterAction::b_cp());
    b_scope.dispatch(BucketAction::Inc("page_views".into()));
    std::thread::sleep(Duration::from_millis(50));

    let views = b_scope
        .read_binding()
        .with_read(|b| b.get("page_views").copied().unwrap_or(0))
        .unwrap_or(0);
    println!("read_binding after scoped Inc: b.page_views={views}");
    println!("(Rw zero-clone proof: cargo run -p rust-elm --example rw_counter)");

    runtime.shutdown();
    Ok(())
}

fn demo_swap() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- SwapRuntime / SwapStore (ArcSwap) ---");
    let runtime = SwapRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(32),
    )?;
    let store = runtime.swap_store();
    let snapshots = store.snapshot_store();

    store.dispatch(CounterAction::B(BucketAction::Inc("page_views".into())));
    store.dispatch(CounterAction::B(BucketAction::Inc("page_views".into())));
    std::thread::sleep(Duration::from_millis(50));

    let views = snapshots.with_snapshot(|s| b_page_views(s));
    println!(
        "with_snapshot b.page_views={views} (main: lock-free borrow — reducer cloned full struct)"
    );

    let b_scope = store.scope(b_lens(), CounterAction::b_cp());
    b_scope.dispatch(BucketAction::Inc("page_views".into()));
    std::thread::sleep(Duration::from_millis(50));

    let views = b_scope
        .snapshot_binding()
        .with_snapshot(|b| b.get("page_views").copied().unwrap_or(0))
        .unwrap_or(0);
    println!("snapshot_binding after scoped Inc: b.page_views={views}");

    let arc = snapshots.load();
    println!(
        "load() → Arc<CounterState> refcount only; b.page_views={}",
        b_page_views(&arc)
    );

    runtime.shutdown();
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_comparison_table();
    demo_tea()?;
    demo_rw()?;
    demo_swap()?;
    println!("\nswap_counter comparison OK");
    Ok(())
}
