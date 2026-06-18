//! Shared counter domain for `counter`, `rw_counter`, and `swap_counter` examples.

use std::collections::HashMap;

use key_paths_derive::{Cp, Kp};
use rust_elm::{Cmd, Sub};
use rust_key_paths::Kp as KpPath;

pub type Bucket = HashMap<String, u64>;

#[derive(Clone, Debug, PartialEq, Eq, Kp)]
pub struct CounterState {
    pub a: Bucket,
    pub b: Bucket,
    pub c: Bucket,
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum BucketAction {
    Inc(String),
    Dec(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum CounterAction {
    A(BucketAction),
    B(BucketAction),
    C(BucketAction),
}

pub fn empty_bucket() -> Bucket {
    HashMap::new()
}

pub fn init() -> (CounterState, Cmd<CounterAction>) {
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

pub fn reduce_bucket(bucket: &mut Bucket, action: BucketAction) {
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

pub fn update(state: &mut CounterState, action: CounterAction) -> Cmd<CounterAction> {
    match action {
        CounterAction::A(a) => reduce_bucket(&mut state.a, a),
        CounterAction::B(b) => reduce_bucket(&mut state.b, b),
        CounterAction::C(c) => reduce_bucket(&mut state.c, c),
    }
    Cmd::none()
}

pub fn subscriptions(_: &CounterState) -> Sub<CounterAction> {
    Sub::none()
}

pub type BucketKp = KpPath<
    CounterState,
    Bucket,
    &'static CounterState,
    &'static Bucket,
    &'static mut CounterState,
    &'static mut Bucket,
    for<'a> fn(&'a CounterState) -> Option<&'a Bucket>,
    for<'a> fn(&'a mut CounterState) -> Option<&'a mut Bucket>,
>;

pub fn b_lens() -> BucketKp {
    fn get(s: &CounterState) -> Option<&Bucket> {
        Some(&s.b)
    }
    fn get_mut(s: &mut CounterState) -> Option<&mut Bucket> {
        Some(&mut s.b)
    }
    KpPath::new(get, get_mut)
}

pub fn b_page_views(state: &CounterState) -> u64 {
    state.b.get("page_views").copied().unwrap_or(0)
}
