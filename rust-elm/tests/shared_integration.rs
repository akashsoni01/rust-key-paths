use rust_elm::{Cmd, InMemoryStorage, ReplayHarness, Shared};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct AppState {
    count: i32,
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

    parent.with_mut(|s| {
        s.count = 5;
    });
    assert_eq!(child.shared.get().count, 5);

    child.shared.with_mut(|s| {
        s.count += 2;
    });
    assert_eq!(parent.get().count, 7);
}

#[test]
fn shared_persist_and_replay_restore() {
    let storage = InMemoryStorage::new();
    let shared = Shared::new(AppState { count: 10 });
    shared.persist(&storage, "app").unwrap();

    let mut harness = ReplayHarness::new(AppState::default(), update);
    harness.send(3);
    assert_eq!(harness.state.count, 3);

    let restored = Shared::load(&storage, "app", AppState::default()).unwrap();
    harness.restore(restored.get());
    assert_eq!(harness.state.count, 10);

    harness.send(1);
    assert_eq!(harness.state.count, 11);
}

#[cfg(feature = "serde")]
#[test]
fn file_storage_round_trip_integration() {
    use rust_elm::FileStorage;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Persisted {
        count: i32,
    }

    let dir = std::env::temp_dir().join(format!("rust_elm_shared_int_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let storage = FileStorage::new(&dir);
    let shared = Shared::new(Persisted { count: 99 });
    shared.persist(&storage, "app").unwrap();
    let loaded = Shared::load(&storage, "app", Persisted { count: 0 }).unwrap();
    assert_eq!(loaded.get().count, 99);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(feature = "serde")]
#[test]
fn replay_snapshot_record_round_trip() {
    use rust_elm::StateSnapshot;

    let mut harness = ReplayHarness::new(AppState::default(), update);
    harness.send(4);
    let snap = harness.snapshot_record();
    harness.send(10);
    assert_eq!(harness.state.count, 14);

    harness.restore_record(snap.clone());
    assert_eq!(harness.state.count, 4);
    assert_eq!(snap, StateSnapshot {
        state: AppState { count: 4 }
    });
}
