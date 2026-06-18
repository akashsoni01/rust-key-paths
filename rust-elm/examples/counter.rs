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
//! Keypaths do **not** skip that. For large maps, store each bucket as
//! `Arc<HashMap<…>>` so root `clone()` only bumps Arc refcounts.
//!
//! ```bash
//! cargo run -p rust-elm --example counter
//! ```
//!
//! See [`book/counter.md`](../book/counter.md).

use std::collections::HashMap;
use std::time::Duration;

use key_paths_derive::{Cp, Kp};
use rust_elm::{
    Cmd, Environment, Program, RuntimeConfig, Sub, TeaRuntime, TeaStoreError,
};
use rust_key_paths::Kp as KpPath;

type Bucket = HashMap<String, u64>;

/// Root model — three independent counter maps.
///
/// Tip: replace `Bucket` with `Arc<Bucket>` if maps are large; then `CounterState::clone()`
/// on the reducer thread is cheap and only changed buckets need a new `Arc`.
#[derive(Clone, Debug, PartialEq, Eq, Kp)]
struct CounterState {
    a: Bucket,
    b: Bucket,
    c: Bucket,
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum BucketAction {
    Inc(String),
    Dec(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum CounterAction {
    A(BucketAction),
    B(BucketAction),
    C(BucketAction),
}

fn empty_bucket() -> Bucket {
    HashMap::new()
}

fn init() -> (CounterState, Cmd<CounterAction>) {
    let mut a = empty_bucket();
    a.insert("requests".into(), 0);
    let mut b = empty_bucket();
    b.insert("page_views".into(), 0);
    let mut c = empty_bucket();
    c.insert("errors".into(), 0);
    (
        CounterState { a, b, c },
        Cmd::none(),
    )
}

fn reduce_bucket(bucket: &mut Bucket, action: BucketAction) {
    match action {
        BucketAction::Inc(key) => {
            *bucket.entry(key).or_insert(0) += 1;
        }
        BucketAction::Dec(key) => {
            let entry = bucket.entry(key).or_insert(0);
            *entry = entry.saturating_sub(1);
        }
    }
}

fn update(state: &mut CounterState, action: CounterAction) -> Cmd<CounterAction> {
    match action {
        CounterAction::A(a) => reduce_bucket(&mut state.a, a),
        CounterAction::B(b) => reduce_bucket(&mut state.b, b),
        CounterAction::C(c) => reduce_bucket(&mut state.c, c),
    }
    Cmd::none()
}

fn subscriptions(_: &CounterState) -> Sub<CounterAction> {
    Sub::none()
}

type BucketKp = KpPath<
    CounterState,
    Bucket,
    &'static CounterState,
    &'static Bucket,
    &'static mut CounterState,
    &'static mut Bucket,
    for<'a> fn(&'a CounterState) -> Option<&'a Bucket>,
    for<'a> fn(&'a mut CounterState) -> Option<&'a mut Bucket>,
>;

fn b_lens() -> BucketKp {
    fn get(s: &CounterState) -> Option<&Bucket> {
        Some(&s.b)
    }
    fn get_mut(s: &mut CounterState) -> Option<&mut Bucket> {
        Some(&mut s.b)
    }
    KpPath::new(get, get_mut)
}

fn demo_borrow_b(store: &rust_elm::TeaStore<CounterState, CounterAction>) -> Result<(), TeaStoreError> {
    println!("\n--- borrow `b` only (no HashMap clone on main) ---");
    let views = store
        .view_store()
        .try_with_snapshot(|s| s.b.get("page_views").copied().unwrap_or(0))?;
    println!("try_with_snapshot b.page_views={views}");
    Ok(())
}

fn demo_scoped_b(store: &rust_elm::TeaStore<CounterState, CounterAction>) -> Result<(), TeaStoreError> {
    println!("\n--- ScopedTeaStore on `b` (dispatch + subscribe child only) ---");

    // Keypath: state focus `CounterState::b`, action casepath `CounterAction::b_cp`.
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

    runtime.shutdown();
    println!("\ncounter example OK");
    Ok(())
}
