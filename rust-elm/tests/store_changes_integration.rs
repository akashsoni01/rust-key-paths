use std::time::Duration;

use key_paths_derive::{Cp, FieldDiff, Kp};
use rust_elm::{start_runtime, 
    allow_state_clones, panic_on_state_clone, Runtime, RuntimeConfig, Environment, Program, Cmd,
    Sub,
};

panic_on_state_clone! {
    #[derive(Default, PartialEq, Eq, Debug, Kp, Hash, FieldDiff)]
    struct App {
        count: i32,
        label: String,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Kp, Cp)]
enum Action {
    Inc,
    Relabel,
}

fn init() -> (App, Cmd<Action>) {
    (
        App {
            count: 0,
            label: "init".into(),
        },
        Cmd::none(),
    )
}

fn update(app: &mut App, action: Action) -> Cmd<Action> {
    match action {
        Action::Inc => {
            app.count += 1;
            Cmd::none()
        }
        Action::Relabel => {
            app.label = "updated".into();
            Cmd::none()
        }
    }
}

fn subs(_: &App) -> Sub<Action> {
    Sub::none()
}

#[test]
fn subscribe_changes_emits_changed_paths_without_cloning_state() {
    let runtime = start_runtime(
        Program::new(init, update, subs),
        Environment::new(),
        RuntimeConfig::new(16),
    );
    let store = runtime.store();

    // subscribe_changes must not deep-clone App (only hash under lock).
    let mut sub = allow_state_clones(0, || store.subscribe_changes());

    store.dispatch(Action::Inc);
    std::thread::sleep(Duration::from_millis(150));

    let change = sub
        .wait_next(Duration::from_secs(1))
        .expect("change after Inc");
    assert_eq!(change.paths, vec![AppField::Count]);

    let count = change
        .paths
        .iter()
        .find_map(|path| match path {
            AppField::Count => Some(sub.binding().with(|app| app.count)),
            _ => None,
        })
        .expect("count path");
    assert_eq!(count, 1);

    store.dispatch(Action::Relabel);
    std::thread::sleep(Duration::from_millis(150));

    let change = sub
        .wait_next(Duration::from_secs(1))
        .expect("change after Relabel");
    assert_eq!(change.paths, vec![AppField::Label]);

    runtime.shutdown();
}

#[test]
fn store_binding_projects_without_clone() {
    let runtime = start_runtime(
        Program::new(init, update, subs),
        Environment::new(),
        RuntimeConfig::new(16),
    );
    let store = runtime.store();

    allow_state_clones(0, || {
        let binding = store.binding();
        assert_eq!(binding.with(|app| app.count), 0);
        store.dispatch(Action::Inc);
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(binding.with(|app| app.count), 1);
    });

    runtime.shutdown();
}
