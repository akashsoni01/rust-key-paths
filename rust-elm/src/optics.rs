//! Enum casepath (prism) helpers for scoped reducers and stores.
//!
//! State keypaths use [`Kp`] + [`RefKpTrait`](key_paths_core::RefKpTrait) from `keypath` prelude.
//! Build casepaths with `#[derive(Cp)]` (`variant_cp()`), compose with [`.then()`](EnumKp::then),
//! then pass to [`ScopeReducer`](crate::ScopeReducer) or [`Store::scope`](crate::Store::scope).

pub use rust_key_paths::{
    enum_err, enum_ok, enum_some, enum_variant, variant_of, EnumKp, EnumKpType, EnumValueKpType,
    Kp,
};

/// Single-step casepath (prism): parent enum → child payload.
pub type CasePath<'a, Parent, Child> = EnumKpType<'a, Parent, Child>;

/// Extract + embed for any [`EnumKp`] (including `.then()` / `.chain()` compositions).
pub trait Casepath<Parent, Child> {
    fn extract(&self, parent: &Parent) -> Option<Child>;
    fn wrap(&self, child: Child) -> Parent;
}

impl<Parent, Child, G, S, E> Casepath<Parent, Child>
    for EnumKp<
        Parent,
        Child,
        &'static Parent,
        &'static Child,
        &'static mut Parent,
        &'static mut Child,
        G,
        S,
        E,
    >
where
    Parent: 'static,
    Child: Clone + 'static,
    G: for<'b> Fn(&'b Parent) -> Option<&'b Child>,
    S: for<'b> Fn(&'b mut Parent) -> Option<&'b mut Child>,
    E: Fn(Child) -> Parent + Clone,
{
    fn extract(&self, parent: &Parent) -> Option<Child> {
        self.get_ref(parent).cloned()
    }

    fn wrap(&self, child: Child) -> Parent {
        self.embed(child)
    }
}

#[cfg(test)]
#[allow(dead_code, clippy::bool_assert_comparison)]
mod tests {
    use super::*;
    use key_paths_core::RefKpTrait;
    use key_paths_derive::{Cp, Kp};

    #[derive(Debug, Kp, Clone, PartialEq)]
    struct AppState {
        counter: Option<CounterState>,
        label: String,
    }

    #[derive(Debug, Kp, Clone, PartialEq)]
    struct CounterState {
        value: i32,
    }

    #[derive(Debug, Kp, Clone, PartialEq, Eq, Cp)]
    enum AppAction {
        Counter(CounterAction),
        Reset,
    }

    #[derive(Debug, Kp, Clone, Copy, PartialEq, Eq, Cp)]
    enum CounterAction {
        Increment,
        Decrement,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
    enum RootAction {
        App(AppAction),
    }

    #[test]
    fn ref_kp_trait_focus_round_trip() {
        let mut state = AppState {
            counter: Some(CounterState { value: 0 }),
            label: "app".into(),
        };
        let kp = AppState::counter();
        assert_eq!(kp.focus(&state).map(|c| c.value), Some(0));
        if let Some(counter) = kp.focus_mut(&mut state) {
            counter.value = 5;
        }
        assert_eq!(state.counter.unwrap().value, 5);
    }

    #[test]
    fn casepath_extract_and_wrap_single_level() {
        let kp = AppAction::counter_cp();
        let action = AppAction::Counter(CounterAction::Increment);
        assert_eq!(kp.extract(&action), Some(CounterAction::Increment));
        assert_eq!(
            kp.wrap(CounterAction::Decrement),
            AppAction::Counter(CounterAction::Decrement)
        );
    }

    #[test]
    fn composed_casepath_chains_three_levels() {
        let kp = RootAction::app_cp()
            .then(AppAction::counter_cp());

        let root = RootAction::App(AppAction::Counter(CounterAction::Increment));
        assert_eq!(kp.extract(&root), Some(CounterAction::Increment));

        let rebuilt = kp.wrap(CounterAction::Decrement);
        assert_eq!(
            rebuilt,
            RootAction::App(AppAction::Counter(CounterAction::Decrement))
        );
    }

    #[test]
    fn then_composition_over_option_and_nested_struct() {
        let mut state = AppState {
            counter: Some(CounterState { value: 1 }),
            label: "x".into(),
        };
        let value_kp = AppState::counter().then(CounterState::value());
        assert_eq!(value_kp.get(&state).copied(), Some(1));
        if let Some(v) = value_kp.get_mut(&mut state) {
            *v = 99;
        }
        assert_eq!(state.counter.unwrap().value, 99);
    }
}
