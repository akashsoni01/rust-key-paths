//! Same `CounterState { a, b, c }` as [`counter`](counter.rs), on [`RwRuntime`] / [`RwStore`].
//!
//! Reads borrow under a **read lock** — `CounterState` is **not cloned** on the main thread.
//!
//! Compare all three backends: `cargo run -p rust-elm --example swap_counter --features arc-swap`
//!
//! ```bash
//! cargo run -p rust-elm --example rw_counter
//! ```

#[path = "counter/common.rs"]
mod common;

use std::time::Duration;

use common::{BucketAction, CounterAction};
use rust_elm::{
    allow_state_clones, panic_on_state_clone, Cmd, Environment, Program, RuntimeConfig, RwRuntime,
    Sub,
};
use rust_key_paths::Kp as KpPath;

panic_on_state_clone! {
    #[derive(Debug, PartialEq, Eq, key_paths_derive::Kp)]
    struct CounterState {
        a: common::Bucket,
        b: common::Bucket,
        c: common::Bucket,
    }
}

fn init() -> (CounterState, Cmd<CounterAction>) {
    let (s, cmd) = common::init();
    (
        CounterState {
            a: s.a,
            b: s.b,
            c: s.c,
        },
        cmd,
    )
}

fn update(state: &mut CounterState, action: CounterAction) -> Cmd<CounterAction> {
    match action {
        CounterAction::A(a) => common::reduce_bucket(&mut state.a, a),
        CounterAction::B(b) => common::reduce_bucket(&mut state.b, b),
        CounterAction::C(c) => common::reduce_bucket(&mut state.c, c),
    }
    Cmd::none()
}

fn subscriptions(_: &CounterState) -> Sub<CounterAction> {
    Sub::none()
}

fn b_lens() -> impl rust_elm::keypath::RefKpTrait<CounterState, common::Bucket> + Clone {
    fn get(s: &CounterState) -> Option<&common::Bucket> {
        Some(&s.b)
    }
    fn get_mut(s: &mut CounterState) -> Option<&mut common::Bucket> {
        Some(&mut s.b)
    }
    KpPath::new(get, get_mut)
}

fn b_page_views(state: &CounterState) -> u64 {
    state.b.get("page_views").copied().unwrap_or(0)
}

fn demo_read_b(store: &rust_elm::RwStore<CounterState, CounterAction>) {
    println!("\n--- read_store().with_read (zero CounterState clone) ---");
    allow_state_clones(0, || {
        let views = store.read_store().with_read(|s| b_page_views(s));
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
            b_page_views(&owned)
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
