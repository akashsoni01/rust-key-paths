use rust_elm::{
    allow_state_clones, shared_get, shared_with_mut, Cmd, InMemoryStorage, ReplayHarness, Shared,
    panic_on_state_clone,
};

panic_on_state_clone! {
    #[derive(Debug, PartialEq, Eq, Default)]
    struct AppState {
        count: i32,
    }
}

struct ScopedView {
    shared: Shared<AppState>,
}

fn update(state: &mut AppState, msg: i32) -> Cmd<i32> {
    state.count += msg;
    Cmd::none()
}

#[test]
fn shared_value_visible_across_scopes() {
    let shared = Shared::new(AppState::default());
    let parent = shared.clone();
    let child = ScopedView {
        shared: shared.clone(),
    };

    shared_with_mut(&parent, |s| {
        s.count = 5;
    });
    assert_eq!(shared_get(&child.shared).count, 5);

    shared_with_mut(&child.shared, |s| {
        s.count += 2;
    });
    assert_eq!(shared_get(&parent).count, 7);
}

#[test]
fn shared_persist_and_replay_restore() {
    let storage = InMemoryStorage::new();
    let shared = Shared::new(AppState { count: 10 });
    allow_state_clones(2, || shared.persist(&storage, "app")).unwrap();

    let mut harness = ReplayHarness::new(AppState::default(), update);
    harness.send(3);
    assert_eq!(harness.state.count, 3);

    let restored =
        allow_state_clones(1, || Shared::load(&storage, "app", AppState::default())).unwrap();
    harness.restore(shared_get(&restored));
    assert_eq!(harness.state.count, 10);

    harness.send(1);
    assert_eq!(harness.state.count, 11);
}

#[cfg(feature = "serde")]
#[test]
fn file_storage_round_trip_integration() {
    use rust_elm::FileStorage;

    panic_on_state_clone! {
        #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Persisted {
            count: i32,
        }
    }

    let dir = std::env::temp_dir().join(format!("rust_elm_shared_int_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let storage = FileStorage::new(&dir);
    let shared = Shared::new(Persisted { count: 99 });
    allow_state_clones(2, || shared.persist(&storage, "app")).unwrap();
    let loaded =
        allow_state_clones(1, || Shared::load(&storage, "app", Persisted { count: 0 })).unwrap();
    assert_eq!(shared_get(&loaded).count, 99);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(feature = "serde")]
#[test]
fn replay_snapshot_record_round_trip() {
    use rust_elm::StateSnapshot;

    let mut harness = ReplayHarness::new(AppState::default(), update);
    harness.send(4);
    let snap = allow_state_clones(1, || harness.snapshot_record());
    harness.send(10);
    assert_eq!(harness.state.count, 14);

    allow_state_clones(1, || harness.restore_record(snap.clone()));
    assert_eq!(harness.state.count, 4);
    assert_eq!(snap, StateSnapshot {
        state: AppState { count: 4 }
    });
}
