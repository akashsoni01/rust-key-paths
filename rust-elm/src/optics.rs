//! State/action focusing for scoped reducers and stores.
//!
//! This module is a thin layer over the `rust_key_paths` prior art:
//!
//! - **Keypath (lens)** — [`Kp`] / [`KpType`] focus a struct field of parent state.
//! - **Casepath (prism)** — [`EnumKp`] / [`EnumKpType`] extract *and* embed an enum
//!   action variant. Build them with [`variant_of`] / [`enum_variant`] (or the
//!   `Option`/`Result` shortcuts [`enum_some`], [`enum_ok`], [`enum_err`]), or pair a
//!   `#[derive(Kp)]` variant accessor with its constructor via [`Kp::with_embed`].

use key_paths_core::{Readable, Writable};

pub use rust_key_paths::{
    enum_err, enum_ok, enum_some, enum_variant, variant_of, EnumKp, EnumKpType, Kp, KpType,
};

/// Lens focusing `Part` within parent state `Whole` (keypath).
pub type StateKey<'a, Whole, Part> = KpType<'a, Whole, Part>;

/// Prism focusing child action `Part` within parent action enum `Whole` (casepath).
pub type ActionCase<'a, Whole, Part> = EnumKpType<'a, Whole, Part>;

/// Alias of [`ActionCase`]: extract + embed action variant accessor (casepath).
pub type ActionEnum<'a, Whole, Part> = EnumKpType<'a, Whole, Part>;

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

/// Extract a child action variant from a parent action enum through a casepath.
pub fn extract_action<'a, PA: 'static, CA: 'static>(
    action_kp: &EnumKpType<'static, PA, CA>,
    action: &'a PA,
) -> Option<&'a CA> {
    action_kp.get_ref(action)
}

/// Embed a child action into its parent enum variant through a casepath.
pub fn wrap_action<PA: 'static, CA: 'static>(
    action_kp: &EnumKpType<'static, PA, CA>,
    child: CA,
) -> PA {
    action_kp.embed(child)
}

#[cfg(test)]
mod tests {
    use super::*;
    use key_paths_derive::Kp;

    #[derive(Debug, Kp, Clone, PartialEq)]
    struct AppState {
        counter: Option<CounterState>,
        label: String,
    }

    #[derive(Debug, Kp, Clone, PartialEq)]
    struct CounterState {
        value: i32,
    }

    #[derive(Debug, Kp, Clone, PartialEq)]
    enum AppAction {
        Counter(CounterAction),
        Reset,
    }

    #[derive(Debug, Kp, Clone, Copy, PartialEq)]
    enum CounterAction {
        Increment,
        Decrement,
    }

    fn get_counter(a: &AppAction) -> Option<&CounterAction> {
        AppAction::counter().get_ref(a)
    }

    fn get_counter_mut(a: &mut AppAction) -> Option<&mut CounterAction> {
        AppAction::counter().get_mut_ref(a)
    }

    fn counter_action_kp() -> EnumKpType<'static, AppAction, CounterAction> {
        variant_of(get_counter, get_counter_mut, AppAction::Counter)
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

    #[test]
    fn action_case_extracts_variant() {
        let action = AppAction::Counter(CounterAction::Increment);
        let kp = counter_action_kp();
        assert!(extract_action(&kp, &action).is_some());
        assert!(matches!(
            extract_action(&kp, &action),
            Some(CounterAction::Increment)
        ));
    }

    #[test]
    fn derived_with_embed_matches_action_enum() {
        let action = AppAction::Counter(CounterAction::Increment);
        let derived = AppAction::counter().with_embed(AppAction::Counter);
        let stored = counter_action_kp();
        assert_eq!(
            derived.get_ref(&action).copied(),
            stored.get(&action).copied()
        );
        assert_eq!(
            derived.embed(CounterAction::Decrement),
            stored.embed(CounterAction::Decrement)
        );
    }

    #[test]
    fn wrap_action_embeds_child() {
        let kp = counter_action_kp();
        let wrapped = wrap_action(&kp, CounterAction::Decrement);
        assert_eq!(wrapped, AppAction::Counter(CounterAction::Decrement));
    }

    #[test]
    fn manual_kp_option_box_chain() {
        #[derive(Kp, Debug, PartialEq)]
        struct Root {
            inner: Option<Box<Leaf>>,
        }

        #[derive(Kp, Debug, PartialEq)]
        struct Leaf {
            n: u32,
        }

        let mut root = Root {
            inner: Some(Box::new(Leaf { n: 7 })),
        };
        let kp = Root::inner().then(Leaf::n());
        assert_eq!(kp.get(&root).copied(), Some(7));
        if let Some(n) = kp.get_mut(&mut root) {
            *n = 42;
        }
        assert_eq!(root.inner.unwrap().n, 42);
    }
}
