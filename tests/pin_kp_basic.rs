#![cfg(all(feature = "pin_project", feature = "tokio"))]

use rust_key_paths::{
    Kp, PinFutureAwaitLike, SyncKeyPathLike, pin::KpThenPinFuture, pin::PinFutureAwaitKp,
    pin_future_await_kp,
};
use std::pin::Pin;

#[derive(Default)]
struct Root {
    state: State,
}

#[derive(Default)]
struct State {
    value: i32,
}

#[derive(Clone, Copy)]
struct AwaitValue;

#[async_trait::async_trait(?Send)]
impl PinFutureAwaitLike<State, i32> for AwaitValue {
    async fn get_await(&self, this: Pin<&mut State>) -> Option<i32> {
        Some(this.get_mut().value)
    }
}

struct FirstNone;
impl SyncKeyPathLike<Root, State> for FirstNone {
    fn sync_get<'a>(&self, _root: &'a Root) -> Option<&'a State> {
        None
    }
    fn sync_get_mut<'a>(&self, _root: &'a mut Root) -> Option<&'a mut State> {
        None
    }
}

#[tokio::test]
async fn pin_future_chain_get_mut_reads_value() {
    let first = Kp::new(|r: &Root| Some(&r.state), |r: &mut Root| Some(&mut r.state));
    let chain =
        KpThenPinFuture::<Root, State, i32, _, _>::new(first, PinFutureAwaitKp::new(AwaitValue));

    let mut root = Root {
        state: State { value: 77 },
    };
    let out = chain.get_mut(&mut root).await;
    assert_eq!(out, Some(77));
}

#[tokio::test]
async fn pin_future_chain_returns_none_when_first_segment_missing() {
    let chain = KpThenPinFuture::<Root, State, i32, _, _>::new(
        FirstNone,
        PinFutureAwaitKp::new(AwaitValue),
    );

    let mut root = Root::default();
    let out = chain.get_mut(&mut root).await;
    assert_eq!(out, None);
}

#[derive(Default)]
struct MacroState {
    value: i32,
}

impl MacroState {
    async fn value_await(this: Pin<&mut Self>) -> Option<i32> {
        Some(this.get_mut().value)
    }
}

#[tokio::test]
async fn pin_future_macro_builds_awaiter() {
    let kp = pin_future_await_kp!(MacroState, value_await -> i32);
    let mut state = MacroState { value: 14 };
    let out = kp.get_await(Pin::new(&mut state)).await;
    assert_eq!(out, Some(14));
}
