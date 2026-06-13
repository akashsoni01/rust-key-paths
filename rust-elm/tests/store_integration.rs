use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rust_elm::{Cmd, Effect, Environment, Program, Runtime, Sub};
use key_paths_derive::{Cp, Kp};
use rust_key_paths::Kp as KpPath;

#[derive(Default, Clone, PartialEq, Eq, Debug, Kp)]
struct App {
    count: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Kp, Cp)]
enum Action {
    Inc,
    Child(ChildAction),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Kp)]
enum ChildAction {
    Bump,
}

fn init() -> (App, Cmd<Action>) {
    (App::default(), Cmd::none())
}

fn update(app: &mut App, action: Action) -> Cmd<Action> {
    match action {
        Action::Inc => {
            app.count += 1;
            Cmd::none()
        }
        Action::Child(ChildAction::Bump) => {
            app.count += 10;
            Cmd::none()
        }
    }
}

fn update_with_effect(app: &mut App, action: Action) -> Cmd<Action> {
    match action {
        Action::Inc => Cmd::single(Effect::task(1, || {
            Box::pin(async { Ok(Action::Child(ChildAction::Bump)) })
        })),
        Action::Child(ChildAction::Bump) => {
            app.count += 10;
            Cmd::none()
        }
    }
}

fn subs(_: &App) -> Sub<Action> {
    Sub::none()
}

fn child_action_kp() -> rust_key_paths::EnumKpType<'static, Action, ChildAction> {
    Action::child_cp()
}

fn count_kp() -> rust_key_paths::KpType<'static, App, i32> {
    fn get(app: &App) -> Option<&i32> {
        Some(&app.count)
    }
    fn get_mut(app: &mut App) -> Option<&mut i32> {
        Some(&mut app.count)
    }
    KpPath::new(get, get_mut)
}

#[test]
fn store_send_finishes_after_effects() {
    let runtime = Runtime::from_program(
        Program::new(init, update_with_effect, subs),
        Environment::new(),
        16,
    );
    let store = runtime.store();
    let task = store.send(Action::Inc);
    assert!(task.finish().is_ok());
    assert_eq!(store.state().count, 10);
    runtime.shutdown();
}

#[test]
fn store_subscribe_state_dedupes() {
    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), 16);
    let store = runtime.store();
    let mut sub = store.subscribe_state();
    store.dispatch(Action::Inc);
    std::thread::sleep(Duration::from_millis(150));
    let first = sub.wait_next(Duration::from_secs(1)).expect("first snapshot");
    assert_eq!(first.count, 1);
    store.dispatch(Action::Inc);
    std::thread::sleep(Duration::from_millis(150));
    let second = sub.wait_next(Duration::from_secs(1)).expect("second snapshot");
    assert_eq!(second.count, 2);
    runtime.shutdown();
}

#[test]
fn scoped_store_routes_child_actions() {
    let runtime = Runtime::from_program(Program::new(init, update, subs), Environment::new(), 16);
    let store = runtime.store();
    let child = store.scope(count_kp(), child_action_kp());
    child.dispatch(ChildAction::Bump);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(child.child_state(), Some(10));
    runtime.shutdown();
}

/// Scoped stores keep only a copyable keypath handle — not extra clones of focused state.
///
/// `child_state` intentionally clones the parent once per call via [`Store::state`]; this test
/// checks that keypath scoping itself does not add hidden parent clones, retain child payload
/// past returned values, or leak shared handles after the scoped store is dropped.
#[test]
fn scoped_store_keypath_does_not_retain_extra_state() {
    static PARENT_CLONES: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct ChildPanel {
        count: i32,
        probe: Arc<AtomicUsize>,
    }

    impl PartialEq for ChildPanel {
        fn eq(&self, other: &Self) -> bool {
            self.count == other.count && Arc::ptr_eq(&self.probe, &other.probe)
        }
    }

    impl Clone for ChildPanel {
        fn clone(&self) -> Self {
            Self {
                count: self.count,
                probe: Arc::clone(&self.probe),
            }
        }
    }

    #[derive(PartialEq, Debug)]
    struct PanelApp {
        child: ChildPanel,
    }

    impl Clone for PanelApp {
        fn clone(&self) -> Self {
            PARENT_CLONES.fetch_add(1, Ordering::SeqCst);
            Self {
                child: self.child.clone(),
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug, Kp)]
    enum PanelAction {
        Bump,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug, Kp, Cp)]
    enum PanelParentAction {
        Panel(PanelAction),
    }

    fn panel_action_kp() -> rust_key_paths::EnumKpType<'static, PanelParentAction, PanelAction> {
        PanelParentAction::panel_cp()
    }

    fn panel_init() -> (PanelApp, Cmd<PanelParentAction>) {
        let probe = Arc::new(AtomicUsize::new(0));
        (
            PanelApp {
                child: ChildPanel { count: 0, probe },
            },
            Cmd::none(),
        )
    }

    fn panel_update(app: &mut PanelApp, action: PanelParentAction) -> Cmd<PanelParentAction> {
        match action {
            PanelParentAction::Panel(PanelAction::Bump) => {
                app.child.count += 1;
                Cmd::none()
            }
        }
    }

    fn panel_subs(_: &PanelApp) -> Sub<PanelParentAction> {
        Sub::none()
    }

    fn child_panel_kp() -> rust_key_paths::KpType<'static, PanelApp, ChildPanel> {
        fn get(app: &PanelApp) -> Option<&ChildPanel> {
            Some(&app.child)
        }
        fn get_mut(app: &mut PanelApp) -> Option<&mut ChildPanel> {
            Some(&mut app.child)
        }
        KpPath::new(get, get_mut)
    }

    PARENT_CLONES.store(0, Ordering::SeqCst);

    let runtime = Runtime::from_program(
        Program::new(panel_init, panel_update, panel_subs),
        Environment::new(),
        16,
    );
    let store = runtime.store();

    let probe = store.state().child.probe;
    assert_eq!(Arc::strong_count(&probe), 2, "store + local probe handle");
    assert_eq!(PARENT_CLONES.load(Ordering::SeqCst), 1, "store.state clones parent once");

    {
        let scoped = store.scope(child_panel_kp(), panel_action_kp());
        let scoped_clone = scoped.clone();

        assert_eq!(
            PARENT_CLONES.load(Ordering::SeqCst),
            1,
            "creating/cloning scoped store must not clone parent state"
        );
        assert_eq!(
            Arc::strong_count(&probe),
            2,
            "scoped store must not retain child payload"
        );

        scoped.dispatch(PanelAction::Bump);
        std::thread::sleep(Duration::from_millis(100));

        {
            let child = scoped.child_state().expect("child present");
            assert_eq!(child.count, 1);
            assert_eq!(
                Arc::strong_count(&probe),
                3,
                "child_state returns an owned child snapshot"
            );
            assert_eq!(
                PARENT_CLONES.load(Ordering::SeqCst),
                2,
                "child_state clones parent exactly once per call"
            );
        }
        assert_eq!(
            Arc::strong_count(&probe),
            2,
            "dropping owned child snapshot releases shared probe"
        );

        {
            let mut sub = scoped.subscribe_state();
            store.dispatch(PanelParentAction::Panel(PanelAction::Bump));
            let deadline = std::time::Instant::now() + Duration::from_secs(1);
            let child = loop {
                if let Some(child) = sub.next() {
                    break child;
                }
                if std::time::Instant::now() >= deadline {
                    panic!("scoped subscription timed out");
                }
                std::thread::sleep(Duration::from_millis(5));
            };
            assert_eq!(child.count, 2);
            assert_eq!(
                Arc::strong_count(&probe),
                4,
                "store + local handle + subscriber snapshot + returned child"
            );
            assert!(
                PARENT_CLONES.load(Ordering::SeqCst) >= 3,
                "parent subscription clones for snapshots; keypath only projects them"
            );
        }
        assert_eq!(
            Arc::strong_count(&probe),
            2,
            "dropping subscribed child and subscriber releases extra probe handles"
        );

        drop(scoped_clone);
        drop(scoped);
        assert_eq!(
            Arc::strong_count(&probe),
            2,
            "dropping scoped stores must not retain child payload"
        );
    }

    let parent_clones_after_scope = PARENT_CLONES.load(Ordering::SeqCst);
    drop(store);
    runtime.shutdown();
    assert_eq!(
        Arc::strong_count(&probe),
        1,
        "runtime shutdown drops store-held child payload"
    );
    assert_eq!(
        PARENT_CLONES.load(Ordering::SeqCst),
        parent_clones_after_scope,
        "dropping scoped stores must not leave extra parent clones"
    );
}
