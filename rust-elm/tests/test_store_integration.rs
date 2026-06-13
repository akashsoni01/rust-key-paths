use std::time::Duration;

use rust_elm::{Cmd, Effect, ExhaustiveTestStore, TestStoreError};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Counter {
    n: i32,
}

fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
    match msg {
        0 => Cmd::single(Effect::task(1, || Box::pin(async { Ok(10) }))),
        n => {
            s.n = n;
            Cmd::none()
        }
    }
}

#[test]
fn test_store_effect_chain() {
    let mut store = ExhaustiveTestStore::new(Counter::default(), update);
    store.send(0);
    store.receive(10);
    store.finish();
    assert_eq!(store.state.n, 10);
}

#[test]
fn test_store_send_with_and_receive_timeout() {
    let mut store = ExhaustiveTestStore::new(Counter::default(), update);
    store.send_with(7, |s| {
        s.n = 7;
    });
    store.finish();

    let mut store2 = ExhaustiveTestStore::new(Counter::default(), update);
    store2.send(0);
    assert!(store2
        .receive_timeout(10, Duration::from_millis(500))
        .is_ok());
    store2.finish();
}

#[test]
fn test_store_receive_timeout_errors() {
    let mut store = ExhaustiveTestStore::new(Counter::default(), update);
    assert_eq!(
        store.receive_timeout(99, Duration::from_millis(50)),
        Err(TestStoreError::ReceiveTimeout)
    );
}
