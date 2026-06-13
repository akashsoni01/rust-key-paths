//! State/action focusing for scoped reducers and stores.
//!
//! - **Keypath (lens)** — [`Kp`] / [`KpType`] / [`StateKey`] for struct fields.
//! - **Casepath (prism)** — [`EnumKp`] / [`CasePath`] for enum variants (extract + embed).
//!
//! Build casepaths with `#[derive(Cp)]` (`variant_cp()`), compose nested actions with
//! [`.then()`](EnumKp::then) / [`.chain()`](EnumKp::chain), then pass the result to
//! [`ScopeReducer`](crate::ScopeReducer) or [`Store::scope`](crate::Store::scope):
//!
//! ```ignore
//! use key_paths_derive::{Cp, Kp};
//!
//! #[derive(Kp, Cp)]
//! enum RootAction { App(AppAction) }
//! #[derive(Kp, Cp)]
//! enum AppAction { Panel(PanelAction) }
//! #[derive(Kp, Cp)]
//! enum PanelAction { Widget(WidgetAction) }
//!
//! let action_kp = RootAction::app_cp()
//!     .then(AppAction::panel_cp())
//!     .then(PanelAction::widget_cp());
//!
//! store.scope(state_kp, action_kp);
//! action_kp.extract(&root_action);
//! action_kp.wrap(WidgetAction::Submit);
//! ```

use key_paths_core::{Readable, Writable};

pub use rust_key_paths::{
    enum_err, enum_ok, enum_some, enum_variant, variant_of, EnumKp, EnumKpType, EnumValueKpType,
    Kp, KpType,
};

/// Lens focusing `Part` within parent state `Whole`.
pub type StateKey<'a, Whole, Part> = KpType<'a, Whole, Part>;

/// Single-step casepath (prism): parent enum → child payload.
pub type CasePath<'a, Parent, Child> = EnumKpType<'a, Parent, Child>;

/// Back-compat alias of [`CasePath`].
pub type ActionCase<'a, Whole, Part> = CasePath<'a, Whole, Part>;

/// Back-compat alias of [`CasePath`].
pub type ActionEnum<'a, Whole, Part> = CasePath<'a, Whole, Part>;

/// Uniform extract + embed for any [`EnumKp`] (including `.then()` / `.chain()` compositions).
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
    Child: Copy + 'static,
    G: for<'b> Fn(&'b Parent) -> Option<&'b Child>,
    S: for<'b> Fn(&'b mut Parent) -> Option<&'b mut Child>,
    E: Fn(Child) -> Parent + Copy,
{
    fn extract(&self, parent: &Parent) -> Option<Child> {
        self.get_ref(parent).copied()
    }

    fn wrap(&self, child: Child) -> Parent {
        self.embed(child)
    }
}

/// Read a focused `Part` from `Whole` through any keypath (lens).
pub fn extract<'a, Whole, Part, K>(kp: &K, whole: &'a Whole) -> Option<&'a Part>
where
    K: Readable<&'a Whole, &'a Part>,
{
    kp.get(whole)
}

/// Mutably focus a `Part` within `Whole` through any keypath (lens).
pub fn extract_mut<'a, Whole, Part, K>(kp: &K, whole: &'a mut Whole) -> Option<&'a mut Part>
where
    K: Writable<&'a mut Whole, &'a mut Part>,
{
    Writable::set(kp, whole)
}

/// Extract a child action through any [`Casepath`] (single step or composed).
pub fn extract_action<Parent, Child, CP>(casepath: &CP, action: &Parent) -> Option<Child>
where
    CP: Casepath<Parent, Child>,
{
    casepath.extract(action)
}

/// Embed a child action through any [`Casepath`] (single step or composed).
pub fn wrap_action<Parent, Child, CP>(casepath: &CP, child: Child) -> Parent
where
    CP: Casepath<Parent, Child>,
{
    casepath.wrap(child)
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn state_key_get_and_get_mut_round_trip() {
        let mut state = AppState {
            counter: Some(CounterState { value: 0 }),
            label: "app".into(),
        };
        let kp = AppState::counter();
        assert_eq!(kp.get(&state).map(|c| c.value), Some(0));
        if let Some(counter) = kp.get_mut(&mut state) {
            counter.value = 5;
        }
        assert_eq!(state.counter.unwrap().value, 5);
    }

    #[test]
    fn casepath_extract_and_wrap_single_level() {
        let kp = AppAction::counter_cp();
        let action = AppAction::Counter(CounterAction::Increment);
        assert_eq!(extract_action(&kp, &action), Some(CounterAction::Increment));
        assert_eq!(
            wrap_action(&kp, CounterAction::Decrement),
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
