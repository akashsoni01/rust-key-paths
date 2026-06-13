use key_paths_core::{Readable, Writable};
pub use rust_key_paths::{EnumKp, EnumKpType, Kp, KpType};

/// Lens focusing `Part` within parent state `Whole`.
pub type StateKey<'a, Whole, Part> = KpType<'a, Whole, Part>;

/// Extract-only action variant accessor (reference-shaped).
pub type ActionCase<'a, Whole, Part> = KpType<'a, Whole, Part>;

/// Extract + embed action variant accessor.
pub type ActionEnum<'a, Whole, Part> = EnumKpType<'a, Whole, Part>;

/// Extract and embed child actions through a parent enum variant.
pub trait ActionEnumCase<PA, CA> {
    fn extract_ref<'a>(&self, action: &'a PA) -> Option<&'a CA>;
    fn embed_child(&self, child: CA) -> PA;
    fn embed_fn(&self) -> fn(CA) -> PA;
}

impl<PA: 'static, CA: 'static> ActionEnumCase<PA, CA> for EnumKpType<'static, PA, CA> {
    fn extract_ref<'a>(&self, action: &'a PA) -> Option<&'a CA> {
        self.get(action)
    }

    fn embed_child(&self, child: CA) -> PA {
        self.embed(child)
    }

    fn embed_fn(&self) -> fn(CA) -> PA {
        EnumKp::embed_fn(self)
    }
}

/// Build a copyable action keypath from fn-pointer extractors and a variant constructor.
///
/// Prefer wiring extractors from `#[derive(Kp)]` variant accessors:
///
/// ```ignore
/// fn get_child(a: &Action) -> Option<&ChildAction> { Action::child().get_ref(a) }
/// fn get_child_mut(a: &mut Action) -> Option<&mut ChildAction> { Action::child().get_mut_ref(a) }
/// let action_kp = action_enum(get_child, get_child_mut, Action::Child);
/// ```
///
/// Or pair directly at the call site without storing:
///
/// ```ignore
/// Action::child().with_embed(Action::Child)
/// ```
pub fn action_enum<PA: 'static, CA: 'static>(
    get: for<'b> fn(&'b PA) -> Option<&'b CA>,
    set: for<'b> fn(&'b mut PA) -> Option<&'b mut CA>,
    embed: fn(CA) -> PA,
) -> EnumKpType<'static, PA, CA> {
    EnumKp::new(Kp::new(get, set), embed)
}

/// Embed a child action into a parent enum variant via [`ActionEnumCase`].
pub fn wrap_action<PA, CA, K>(action_kp: &K, child: CA) -> PA
where
    K: ActionEnumCase<PA, CA>,
{
    action_kp.embed_child(child)
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

/// Extract a child action variant from a parent action enum.
pub fn extract_action<'a, PA, CA, K>(action_kp: &K, action: &'a PA) -> Option<&'a CA>
where
    K: ActionEnumCase<PA, CA>,
{
    action_kp.extract_ref(action)
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
        action_enum(get_counter, get_counter_mut, AppAction::Counter)
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
