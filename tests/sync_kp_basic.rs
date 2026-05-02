use rust_key_paths::sync_kp::{ArcMutexAccess, StdRwLockAccess, SyncKp};
use rust_key_paths::{Kp, KpType};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug)]
struct RootMutex {
    inner: Arc<Mutex<i32>>,
}

#[derive(Debug)]
struct RootRw {
    inner: RwLock<i32>,
}

#[test]
fn sync_kp_arc_mutex_read_and_set() {
    let root = RootMutex {
        inner: Arc::new(Mutex::new(7)),
    };

    let prev: KpType<'_, RootMutex, Arc<Mutex<i32>>> = Kp::new(
        |r: &RootMutex| Some(&r.inner),
        |r: &mut RootMutex| Some(&mut r.inner),
    );
    let next: KpType<'_, i32, i32> = Kp::new(|v: &i32| Some(v), |v: &mut i32| Some(v));

    let lock_kp: SyncKp<
        RootMutex,
        Arc<Mutex<i32>>,
        i32,
        i32,
        &RootMutex,
        &Arc<Mutex<i32>>,
        &i32,
        &i32,
        &mut RootMutex,
        &mut Arc<Mutex<i32>>,
        &mut i32,
        &mut i32,
        fn(&RootMutex) -> Option<&Arc<Mutex<i32>>>,
        fn(&mut RootMutex) -> Option<&mut Arc<Mutex<i32>>>,
        ArcMutexAccess<i32>,
        fn(&i32) -> Option<&i32>,
        fn(&mut i32) -> Option<&mut i32>,
    > = SyncKp::new(prev, ArcMutexAccess::new(), next);

    assert_eq!(lock_kp.get(&root), Some(&7));
    assert!(lock_kp.set(&root, |v| *v = 42).is_ok());
    assert_eq!(lock_kp.get(&root), Some(&42));
}

#[test]
fn sync_kp_std_rwlock_read_and_set() {
    let root = RootRw {
        inner: RwLock::new(11),
    };

    let prev: KpType<'_, RootRw, RwLock<i32>> = Kp::new(
        |r: &RootRw| Some(&r.inner),
        |r: &mut RootRw| Some(&mut r.inner),
    );
    let next: KpType<'_, i32, i32> = Kp::new(|v: &i32| Some(v), |v: &mut i32| Some(v));

    let lock_kp = SyncKp::new(prev, StdRwLockAccess::new(), next);

    assert_eq!(lock_kp.get(&root), Some(&11));
    assert!(lock_kp.set(&root, |v| *v += 9).is_ok());
    assert_eq!(lock_kp.get(&root), Some(&20));
}
