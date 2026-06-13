use key_paths_core::{Readable, Writable};
use rust_key_paths::Kp;

/// Lens focusing `Part` within parent state `Whole`.
pub type StateKey<'a, Whole, Part> = KpType<'a, Whole, Part>;

/// Prism focusing child action `Part` within parent action enum `Whole`.
pub type ActionCase<'a, Whole, Part> = KpType<'a, Whole, Part>;

/// Re-export the standard reference keypath alias used throughout the crate.
pub type KpType<'a, R, V> = Kp<
    R,
    V,
    &'a R,
    &'a V,
    &'a mut R,
    &'a mut V,
    for<'b> fn(&'b R) -> Option<&'b V>,
    for<'b> fn(&'b mut R) -> Option<&'b mut V>,
>;

/// Wrap a child action into a parent enum variant.
pub fn wrap_action<Whole, Part>(embed: fn(Part) -> Whole, part: Part) -> Whole {
    embed(part)
}

pub fn extract<'a, Whole, Part, K>(kp: &K, whole: &'a Whole) -> Option<&'a Part>
where
    K: Readable<&'a Whole, &'a Part>,
{
    kp.get(whole)
}

/// Extract mutably — used by scoped reducers to focus child actions.
pub fn extract_mut<'a, Whole, Part, K>(kp: &K, whole: &'a mut Whole) -> Option<&'a mut Part>
where
    K: Writable<&'a mut Whole, &'a mut Part>,
{
    Writable::set(kp, whole)
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

    #[derive(Debug, Kp, Clone, PartialEq)]
    enum CounterAction {
        Increment,
        Decrement,
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
        let kp = AppAction::counter();
        assert!(extract(&kp, &action).is_some());
        assert!(matches!(
            extract(&kp, &action),
            Some(CounterAction::Increment)
        ));
    }

    #[test]
    fn wrap_action_embeds_child() {
        let wrapped = wrap_action(AppAction::Counter, CounterAction::Decrement);
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
