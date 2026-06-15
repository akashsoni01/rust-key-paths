//! State/action focusing for scoped reducers and stores.
//!
//! - **Keypath (lens)** — [`Kp`] / [`StateKp`] for struct fields, bounded by [`KpTrait`].
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

use key_paths_core::KpTrait;

pub use rust_key_paths::{
    enum_err, enum_ok, enum_some, enum_variant, variant_of, EnumKp, EnumKpType, EnumValueKpType,
    Kp,
};

/// Lens focusing `Part` within parent state `Whole`.
///
/// Matches the concrete [`Kp`] returned by `#[derive(Kp)]` field accessors (closure-backed
/// `G`/`S`) and manual `Kp::new` fn-pointer paths alike.
pub type StateKp<Whole, Part, G, Set> = Kp<
    Whole,
    Part,
    &'static Whole,
    &'static Part,
    &'static mut Whole,
    &'static mut Part,
    G,
    Set,
>;

/// Reference-shaped state keypath — static dispatch via [`KpTrait`].
///
/// ```ignore
/// fn validate_field<F, R, V>(kp: F, root: &R) -> Option<&V>
/// where
///     F: KpTrait<R, V, &'static R, &'static V, &'static mut R, &'static mut V>,
/// {
///     kp.get(root)
/// }
/// ```
pub trait StateKeypath<Whole, Part>:
    KpTrait<
        Whole,
        Part,
        &'static Whole,
        &'static Part,
        &'static mut Whole,
        &'static mut Part,
    > + StateLens<Whole, Part>
where
    Whole: 'static,
    Part: 'static,
{
}

impl<K, Whole, Part> StateKeypath<Whole, Part> for K
where
    Whole: 'static,
    Part: 'static,
    K: KpTrait<
            Whole,
            Part,
            &'static Whole,
            &'static Part,
            &'static mut Whole,
            &'static mut Part,
        > + StateLens<Whole, Part>,
{
}

/// Read/write navigation used by scoped reducers and stores.
pub trait StateLens<Whole, Part> {
    fn focus<'a>(&self, whole: &'a Whole) -> Option<&'a Part>;
    fn focus_mut<'a>(&self, whole: &'a mut Whole) -> Option<&'a mut Part>;
}

impl<'a, R, V, G, Set> StateLens<R, V> for Kp<R, V, &'a R, &'a V, &'a mut R, &'a mut V, G, Set>
where
    G: for<'b> Fn(&'b R) -> Option<&'b V>,
    Set: for<'b> Fn(&'b mut R) -> Option<&'b mut V>,
{
    fn focus<'b>(&self, whole: &'b R) -> Option<&'b V> {
        self.get_ref(whole)
    }

    fn focus_mut<'b>(&self, whole: &'b mut R) -> Option<&'b mut V> {
        self.get_mut_ref(whole)
    }
}

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

/// Read a focused `Part` from `Whole` through any [`StateLens`].
pub fn extract<'a, Whole, Part, K>(kp: &K, whole: &'a Whole) -> Option<&'a Part>
where
    K: StateLens<Whole, Part>,
{
    kp.focus(whole)
}

/// Mutably focus a `Part` within `Whole` through any [`StateLens`].
pub fn extract_mut<'a, Whole, Part, K>(kp: &K, whole: &'a mut Whole) -> Option<&'a mut Part>
where
    K: StateLens<Whole, Part>,
{
    kp.focus_mut(whole)
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
#[allow(dead_code, clippy::bool_assert_comparison)]
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

    fn accepts_state_kp<K>(kp: K) -> K
    where
        K: KpTrait<
            AppState,
            CounterState,
            &'static AppState,
            &'static CounterState,
            &'static mut AppState,
            &'static mut CounterState,
        >,
    {
        kp
    }

    #[test]
    fn state_key_get_and_get_mut_round_trip() {
        let mut state = AppState {
            counter: Some(CounterState { value: 0 }),
            label: "app".into(),
        };
        let kp = accepts_state_kp(AppState::counter());
        assert_eq!(kp.get(&state).map(|c| c.value), Some(0));
        assert_eq!(extract(&kp, &state).map(|c| c.value), Some(0));
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
