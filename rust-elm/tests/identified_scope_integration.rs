use rust_elm::{
    reducer::Reduce, Identifiable, IdentifiedVec, ScopeReducer, IfLetReducer, Cmd, Reducer,
    Effect,
};
use rust_key_paths::Kp;

#[test]
fn identified_vec_ordering_and_remove() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Node {
        id: u32,
        label: &'static str,
    }

    impl Identifiable for Node {
        type Id = u32;
        fn id(&self) -> u32 {
            self.id
        }
    }

    let mut nodes = IdentifiedVec::new();
    nodes.insert(Node { id: 1, label: "a" });
    nodes.insert(Node { id: 2, label: "b" });
    nodes.reorder(1, 0);
    assert_eq!(nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![2, 1]);
    nodes.remove(2);
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes.get(1).unwrap().label, "a");
}

#[test]
fn scope_child_state_isolated_from_sibling_field() {
    #[derive(Default)]
    struct Child {
        n: i32,
    }

    #[derive(Default)]
    struct Parent {
        child: Option<Child>,
        other: i32,
    }

    #[derive(Clone, Copy)]
    enum PA {
        Child(CA),
        TouchOther,
    }

    #[derive(Clone, Copy)]
    enum CA {
        Inc,
    }

    fn child_r(c: &mut Child, _: CA) -> Cmd<CA> {
        c.n += 1;
        Cmd::none()
    }

    fn embed(c: CA) -> PA {
        PA::Child(c)
    }

    fn extract(a: PA) -> Option<CA> {
        match a {
            PA::Child(c) => Some(c),
            _ => None,
        }
    }

    fn get_child(p: &Parent) -> Option<&Child> {
        p.child.as_ref()
    }

    fn get_child_mut(p: &mut Parent) -> Option<&mut Child> {
        p.child.as_mut()
    }

    let scope = ScopeReducer::new(
        Kp::new(get_child, get_child_mut),
        embed,
        extract,
        1,
        Reduce::new(child_r),
    );
    let mut parent = Parent {
        child: Some(Child::default()),
        other: 5,
    };
    scope.reduce(&mut parent, PA::Child(CA::Inc));
    scope.reduce(&mut parent, PA::TouchOther);
    assert_eq!(parent.child.unwrap().n, 1);
    assert_eq!(parent.other, 5);
}

#[test]
fn if_let_dismiss_returns_cancel_effect() {
    #[derive(Default)]
    struct Child {
        n: i32,
    }

    #[derive(Default)]
    struct Parent {
        child: Option<Child>,
    }

    #[derive(Clone, Copy)]
    enum PA {
        Child(CA),
        Dismiss,
    }

    #[derive(Clone, Copy)]
    enum CA {
        Inc,
    }

    fn child_r(_: &mut Child, _: CA) -> Cmd<CA> {
        Cmd::none()
    }

    let if_let = IfLetReducer::new(
        Kp::new(
            |p: &Parent| p.child.as_ref(),
            |p: &mut Parent| p.child.as_mut(),
        ),
        |c| PA::Child(c),
        |a| match a {
            PA::Child(c) => Some(c),
            _ => None,
        },
        |a| matches!(a, PA::Dismiss),
        |p| {
            p.child = None;
        },
        99,
        Reduce::new(child_r),
    );

    let mut parent = Parent {
        child: Some(Child::default()),
    };
    let cmd = if_let.reduce(&mut parent, PA::Dismiss);
    assert!(parent.child.is_none());
    assert!(matches!(
        cmd.into_effects().first(),
        Some(Effect::Cancel { id: 99 })
    ));
}

#[cfg(feature = "serde")]
#[test]
fn identified_vec_serde_round_trip() {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Row {
        id: u64,
        n: i32,
    }

    impl Identifiable for Row {
        type Id = u64;
        fn id(&self) -> u64 {
            self.id
        }
    }

    let mut vec = IdentifiedVec::new();
    vec.insert(Row { id: 1, n: 1 });
    let json = serde_json::to_string(&vec).unwrap();
    let back: IdentifiedVec<u64, Row> = serde_json::from_str(&json).unwrap();
    assert_eq!(back.get(1).unwrap().n, 1);
}
