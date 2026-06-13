use key_paths_derive::{Cp, Kp};
use rust_key_paths::EnumKpType;

#[derive(Debug, Clone, Copy, PartialEq, Kp, Cp)]
enum Action {
    Child(ChildAction),
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Kp, Cp)]
enum ChildAction {
    Inc(i32),
    Reset,
}

#[test]
fn tuple_variant_casepath_round_trips() {
    let kp: EnumKpType<'static, Action, ChildAction> = Action::child_cp();

    let embedded = kp.embed(ChildAction::Inc(5));
    assert_eq!(embedded, Action::Child(ChildAction::Inc(5)));

    assert_eq!(kp.get_ref(&embedded), Some(&ChildAction::Inc(5)));
    assert_eq!(kp.get_ref(&Action::Tick), None);
}

#[test]
fn casepath_get_mut_focuses_payload() {
    let kp = Action::child_cp();
    let mut action = Action::Child(ChildAction::Inc(1));

    if let Some(child) = kp.get_mut(&mut action) {
        *child = ChildAction::Reset;
    }
    assert_eq!(action, Action::Child(ChildAction::Reset));
}

#[test]
fn unit_variant_casepath_embeds_and_extracts() {
    let kp: EnumKpType<'static, Action, ()> = Action::tick_cp();

    let embedded = kp.embed(());
    assert_eq!(embedded, Action::Tick);

    assert!(kp.get_ref(&Action::Tick).is_some());
    assert!(kp.get_ref(&Action::Child(ChildAction::Reset)).is_none());
}

#[test]
fn kp_and_cp_accessors_coexist() {
    let action = Action::Child(ChildAction::Inc(7));

    // `Kp` derive: extract-only accessor.
    assert_eq!(Action::child().get_ref(&action), Some(&ChildAction::Inc(7)));
    // `Cp` derive: extract + embed accessor.
    assert_eq!(
        Action::child_cp().embed(ChildAction::Inc(7)),
        action
    );
}
