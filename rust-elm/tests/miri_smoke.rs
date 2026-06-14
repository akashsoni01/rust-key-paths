//! Miri-safe synchronous tests — no threads, no Tokio runtime.
//!
//! Run: `./scripts/miri-rust-elm.sh` or `cargo +nightly miri test -p rust-elm --test miri_smoke`

use key_paths_derive::{Cp, Kp};
use rust_elm::{
    reducer::Reduce, CatchReducer, Cmd, ForEachReducer, Identifiable, IdentifiedVec, Reducer,
    ReducePanic, ScopeReducer, Shared, Sub,
};
use rust_key_paths::Kp as KpPath;
use std::time::Duration;

#[test]
fn miri_identified_vec_insert_remove() {
    #[derive(Clone, PartialEq, Eq, Debug)]
    struct Row {
        id: u32,
        v: i32,
    }
    impl Identifiable for Row {
        type Id = u32;
        fn id(&self) -> u32 {
            self.id
        }
    }
    let mut vec = IdentifiedVec::new();
    vec.insert(Row { id: 1, v: 10 });
    vec.insert(Row { id: 2, v: 20 });
    vec.remove(1);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.get(2).unwrap().v, 20);
}

#[test]
fn miri_scope_routes_child_action_only() {
    #[derive(Default)]
    struct Child {
        n: i32,
    }
    #[derive(Default)]
    struct Parent {
        child: Child,
        other: i32,
    }
    #[derive(Clone, Copy, Kp, Cp)]
    enum PA {
        Child(CA),
        TouchOther,
    }
    #[derive(Clone, Copy, Kp)]
    enum CA {
        Inc,
    }
    fn child_r(c: &mut Child, _: CA) -> Cmd<CA> {
        c.n += 1;
        Cmd::none()
    }
    fn child_get(p: &Parent) -> Option<&Child> {
        Some(&p.child)
    }
    fn child_get_mut(p: &mut Parent) -> Option<&mut Child> {
        Some(&mut p.child)
    }
    let scope = ScopeReducer::new(
        KpPath::new(child_get, child_get_mut),
        PA::child_cp(),
        1,
        Reduce::new(child_r),
    );
    let mut parent = Parent::default();
    scope.reduce(&mut parent, PA::Child(CA::Inc));
    scope.reduce(&mut parent, PA::TouchOther);
    assert_eq!(parent.child.n, 1);
    assert_eq!(parent.other, 0);
}

#[test]
fn miri_for_each_routes_by_id() {
    #[derive(Clone, PartialEq, Eq, Debug)]
    struct Row {
        id: u32,
        n: i32,
    }
    impl Identifiable for Row {
        type Id = u32;
        fn id(&self) -> u32 {
            self.id
        }
    }
    #[derive(Clone, Copy, Kp)]
    enum RA {
        Row(u32, CA),
    }
    #[derive(Clone, Copy, Kp)]
    enum CA {
        Inc,
    }
    fn row_r(r: &mut Row, _: CA) -> Cmd<CA> {
        r.n += 1;
        Cmd::none()
    }
    fn rows_vec(v: &mut IdentifiedVec<u32, Row>) -> &mut IdentifiedVec<u32, Row> {
        v
    }

    let fe = ForEachReducer::new(
        rows_vec,
        RA::Row,
        |a| match a {
            RA::Row(id, ca) => Some((id, ca)),
        },
        |_| None,
        |id| 1000 + u64::from(id),
        Reduce::new(row_r),
    );
    let mut rows = IdentifiedVec::new();
    rows.insert(Row { id: 1, n: 0 });
    fe.reduce(&mut rows, RA::Row(1, CA::Inc));
    assert_eq!(rows.get(1).unwrap().n, 1);
}

#[test]
fn miri_shared_mutate_and_read() {
    let shared = Shared::new(0i32);
    shared.with_mut(|n| *n = 5);
    assert_eq!(shared.get(), 5);
}

#[test]
fn miri_sub_description_is_pure_data() {
    fn tick() -> i32 {
        1
    }
    let sub = Sub::batch([
        Sub::tick(1, Duration::from_millis(1), tick),
        Sub::none(),
    ]);
    assert_eq!(sub.id(), Some(1));
}

#[test]
fn miri_catch_reducer_survives_panic() {
    #[derive(Default, Debug, PartialEq)]
    struct S {
        n: i32,
    }
    #[derive(Clone, Copy)]
    enum A {
        Boom,
    }
    fn boom(s: &mut S, _: A) -> Cmd<A> {
        s.n = 99;
        panic!("miri test panic");
    }
    let r = CatchReducer::new(Reduce::new(boom), |_: ReducePanic| Cmd::none());
    let mut s = S::default();
    r.reduce(&mut s, A::Boom);
    assert_eq!(s.n, 99);
}
