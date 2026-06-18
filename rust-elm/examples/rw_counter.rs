//! Same `CounterState { a, b, c }` as [`counter`](counter.rs), on [`RwRuntime`] / [`RwStore`].
//!
//! With RwStore the live model stays in `Arc<RwLock<CounterState>>`. Reads borrow under a
//! **read lock** — the root struct and individual buckets are **not cloned**.
//!
//! | Read API | Clones `CounterState`? |
//! |----------|------------------------|
//! | `read_store().with_read(\|s\| s.b.get(...))` | **No** — borrow root, read field `b` |
//! | `b_scope.read_binding().with_read(\|m\| …)` | **No** — keypath focus, borrow `b` only |
//! | `store.state()` | **Yes** — full struct clone (avoid on hot paths) |
//! | `subscribe_state().next()` | **Yes** — clones full struct under read lock |
//!
//! This example uses [`panic_on_state_clone!`] + [`allow_state_clones`] to **panic** if a read
//! path accidentally clones `CounterState`. Compare with `counter.rs` (TeaStore), where the
//! reducer clones the entire struct on every publish even when you only need `b`.
//!
//! ```bash
//! cargo run -p rust-elm --example rw_counter
//! ```

use std::collections::HashMap;
use std::time::Duration;

use key_paths_derive::{Cp, Kp};
use rust_elm::{
    allow_state_clones, panic_on_state_clone, Cmd, Environment, Program, RuntimeConfig, RwRuntime,
    Sub,
};
use rust_key_paths::Kp as KpPath;

type Bucket = HashMap<String, u64>;

panic_on_state_clone! {
    #[derive(Debug, PartialEq, Eq, Kp)]
    struct CounterState {
        a: Bucket,
        b: Bucket,
        c: Bucket,
    }
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

fn demo_read_b(store: &rust_elm::RwStore<CounterState, CounterAction>) {
    println!("\n--- read_store().with_read (zero CounterState clone) ---");
    allow_state_clones(0, || {
        let views = store
            .read_store()
            .with_read(|s| s.b.get("page_views").copied().unwrap_or(0));
        println!("with_read b.page_views={views}");
    });
}

fn demo_scoped_binding(store: &rust_elm::RwStore<CounterState, CounterAction>) {
    println!("\n--- ScopedRwStore::read_binding (keypath → borrow `b` only) ---");
    let b_scope = store.scope(b_lens(), CounterAction::b_cp());

    allow_state_clones(0, || {
        let before = b_scope
            .read_binding()
            .with_read(|b| b.get("page_views").copied().unwrap_or(0))
            .unwrap_or(0);
        println!("read_binding before dispatch: page_views={before}");
    });

    b_scope.dispatch(BucketAction::Inc("page_views".into()));
    b_scope.dispatch(BucketAction::Inc("page_views".into()));
    std::thread::sleep(Duration::from_millis(50));

    allow_state_clones(0, || {
        let after = b_scope
            .read_binding()
            .with_read(|b| b.get("page_views").copied().unwrap_or(0))
            .unwrap_or(0);
        println!("read_binding after 2× Inc: page_views={after}");
    });
}

fn demo_state_clones(store: &rust_elm::RwStore<CounterState, CounterAction>) {
    println!("\n--- store.state() — intentional full CounterState clone ---");
    allow_state_clones(1, || {
        let owned = store.state();
        println!(
            "state() cloned entire struct: b.page_views={}",
            owned.b.get("page_views").copied().unwrap_or(0)
        );
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = RwRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(32),
    )?;
    let store = runtime.rw_store();

    println!("=== CounterState {{ a, b, c }} on RwStore (no clone on read) ===");

    store.dispatch(CounterAction::B(BucketAction::Inc("page_views".into())));
    std::thread::sleep(Duration::from_millis(50));

    demo_read_b(&store);
    demo_scoped_binding(&store);
    demo_state_clones(&store);

    runtime.shutdown();
    println!("\nrw_counter example OK");
    Ok(())
}
