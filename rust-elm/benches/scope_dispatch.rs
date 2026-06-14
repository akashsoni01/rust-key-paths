#![allow(dead_code, clippy::type_complexity)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use key_paths_derive::{Cp, Kp};
use rust_elm::{
    reducer::Reduce, ForEachReducer, Identifiable, IdentifiedVec, Reducer, ScopeReducer, Cmd,
};
use rust_key_paths::Kp as KpPath;

#[derive(Default, Clone)]
struct Child {
    n: i32,
}

#[derive(Default, Clone)]
struct Parent {
    child: Child,
}

#[derive(Clone, Copy, Kp, Cp)]
enum Action {
    Child(ChildAction),
}

#[derive(Clone, Copy, Kp)]
enum ChildAction {
    Inc,
}

fn child_reducer(c: &mut Child, _: ChildAction) -> Cmd<ChildAction> {
    c.n += 1;
    Cmd::none()
}

fn child_get(p: &Parent) -> Option<&Child> {
    Some(&p.child)
}
fn child_get_mut(p: &mut Parent) -> Option<&mut Child> {
    Some(&mut p.child)
}

fn bench_scope(c: &mut Criterion) {
    let scope = ScopeReducer::new(
        KpPath::new(child_get, child_get_mut),
        Action::child_cp(),
        1,
        Reduce::new(child_reducer),
    );
    let mut parent = Parent::default();
    c.bench_function("scope_reducer_inc", |b| {
        b.iter(|| {
            scope.reduce(
                black_box(&mut parent),
                black_box(Action::Child(ChildAction::Inc)),
            );
        });
    });
}

#[derive(Clone)]
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
enum RowAction {
    Row(u32, ChildAction),
}

fn rows_vec(v: &mut IdentifiedVec<u32, Row>) -> &mut IdentifiedVec<u32, Row> {
    v
}

fn bench_for_each(c: &mut Criterion) {
    let fe = ForEachReducer::new(
        rows_vec,
        RowAction::Row,
        |a| match a {
            RowAction::Row(id, ca) => Some((id, ca)),
        },
        |_| None,
        |id| 2000 + u64::from(id),
        Reduce::new(|r: &mut Row, _: ChildAction| {
            r.n += 1;
            Cmd::none()
        }),
    );
    let mut rows = IdentifiedVec::new();
    for id in 0..32 {
        rows.insert(Row { id, n: 0 });
    }
    c.bench_function("for_each_reducer_32_rows", |b| {
        b.iter(|| {
            fe.reduce(
                black_box(&mut rows),
                black_box(RowAction::Row(16, ChildAction::Inc)),
            );
        });
    });
}

criterion_group!(benches, bench_scope, bench_for_each);
criterion_main!(benches);
