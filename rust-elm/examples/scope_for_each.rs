//! Scope + ForEach reducers without Runtime — pure reduce only.
//!
//! ```bash
//! cargo run -p rust-elm --example scope_for_each
//! ```

use key_paths_derive::{Cp, Kp};
use rust_elm::{
    reducer::Reduce, ForEachReducer, Identifiable, IdentifiedVec, Reducer, ScopeReducer, Cmd,
};
use rust_key_paths::Kp as KpPath;

fn main() {
    scope_demo();
    for_each_demo();
}

fn scope_demo() {
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
    println!("scope: child.n={} other={}", parent.child.n, parent.other);
}

fn for_each_demo() {
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
    println!("for_each: row.n={}", rows.get(1).unwrap().n);
}
