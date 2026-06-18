//! Named counters in a `HashMap<String, u64>` on [`TeaRuntime`] / [`TeaStore`].
//!
//! Demonstrates increment/decrement actions and **three ways to read snapshots**:
//!
//! | API | Clone cost on main thread |
//! |-----|---------------------------|
//! | `view.try_load()` | **None** — receive `Arc<HashMap>` (refcount only) |
//! | `view.try_with_snapshot(\|m\| …)` | **None** — borrow via `Arc` on main |
//! | `store.state()` | **Full `HashMap` clone** on main (unwraps `Arc`) |
//! | `subscribe_state().next()` | **None** — receive `Arc<HashMap>` per push |
//!
//! **Where does cloning happen?** The live `HashMap` lives only on the **reducer thread**.
//! After each `Inc` / `Dec`, the reducer clones the map (`state.clone()`) and sends
//! `Arc<HashMap>` to subscribers or RPC replies — that **full map clone is on the reducer thread**.
//! Main thread work is cheap unless you call `state()` or clone the map yourself.
//!
//! ```bash
//! cargo run -p rust-elm --example counter
//! ```
//!
//! See [`book/counter.md`](../book/counter.md) for architecture notes.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rust_elm::{
    Cmd, Environment, Program, RuntimeConfig, Sub, TeaRuntime, TeaStoreError,
};

type Counters = HashMap<String, u64>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum CounterAction {
    Inc(String),
    Dec(String),
}

fn init() -> (Counters, Cmd<CounterAction>) {
    let mut counters = HashMap::new();
    counters.insert("page_views".into(), 0);
    counters.insert("api_calls".into(), 0);
    (counters, Cmd::none())
}

fn update(counters: &mut Counters, action: CounterAction) -> Cmd<CounterAction> {
    match action {
        CounterAction::Inc(key) => {
            *counters.entry(key).or_insert(0) += 1;
        }
        CounterAction::Dec(key) => {
            let entry = counters.entry(key).or_insert(0);
            *entry = entry.saturating_sub(1);
        }
    }
    Cmd::none()
}

fn subscriptions(_: &Counters) -> Sub<CounterAction> {
    Sub::none()
}

fn read_key(snap: &Arc<Counters>, key: &str) -> u64 {
    snap.get(key).copied().unwrap_or(0)
}

fn print_snapshot(label: &str, snap: &Arc<Counters>) {
    println!(
        "{label}: page_views={} api_calls={}",
        read_key(snap, "page_views"),
        read_key(snap, "api_calls"),
    );
}

fn demo_rpc_read(store: &rust_elm::TeaStore<Counters, CounterAction>) -> Result<(), TeaStoreError> {
    println!("\n--- RPC snapshot (main waits; HashMap clone on reducer thread) ---");

    // Preferred: Arc only — reducer cloned the map when building the reply.
    let snap = store.view_store().try_load()?;
    print_snapshot("try_load (Arc on main — no extra HashMap clone)", &snap);

    // Borrow one field — no HashMap clone on main.
    let views = store
        .view_store()
        .try_with_snapshot(|m| m.get("page_views").copied().unwrap_or(0))?;
    println!("try_with_snapshot page_views={views}");

    // Full owned copy — HashMap clone on MAIN (.state() clones out of Arc).
    let owned = store.state()?;
    println!(
        "state() — HashMap cloned on main: page_views={}",
        owned.get("page_views").copied().unwrap_or(0)
    );

    Ok(())
}

fn demo_subscribe(store: &rust_elm::TeaStore<Counters, CounterAction>) -> Result<(), TeaStoreError> {
    println!("\n--- subscribe_state (reducer pushes Arc after each reduce) ---");
    let mut sub = store.subscribe_state()?;
    if let Some(snap) = sub.latest() {
        print_snapshot("initial subscribe", &snap);
    }

    store.dispatch(CounterAction::Inc("page_views".into()));
    store.dispatch(CounterAction::Inc("page_views".into()));
    store.dispatch(CounterAction::Dec("api_calls".into()));

    std::thread::sleep(Duration::from_millis(100));
    while sub.next().is_some() {}

    if let Some(snap) = sub.latest() {
        print_snapshot("latest after pushes (Arc on main)", &snap);
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

    println!("=== TeaStore counter (HashMap<String, u64>) ===");

    store.dispatch(CounterAction::Inc("page_views".into()));
    store.dispatch(CounterAction::Inc("api_calls".into()));
    store.dispatch(CounterAction::Inc("api_calls".into()));
    std::thread::sleep(Duration::from_millis(50));

    demo_rpc_read(&store)?;
    demo_subscribe(&store)?;

    runtime.shutdown();
    println!("\ncounter example OK");
    Ok(())
}
