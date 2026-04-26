use rust_key_paths::lock::{ArcMutexAccess, LockKp, StdRwLockAccess};
use rust_key_paths::{Kp, KpType};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug)]
struct RootMutex {
    inner: Arc<Mutex<i32>>,
}

impl RootMutex {
    fn inner_prev<'a>() -> Kp<
        RootMutex,
        Arc<Mutex<i32>>,
        &'static RootMutex,
        &'static Arc<Mutex<i32>>,
        &'static mut RootMutex,
        &'static mut Arc<Mutex<i32>>,
        impl Fn(&'a RootMutex) -> Option<&'a Arc<Mutex<i32>>>,
        impl Fn(&'a mut RootMutex) -> Option<&'a mut Arc<Mutex<i32>>>,
    > {
        return Kp::new(
            |r: &RootMutex| Some(&r.inner),
            |r: &mut RootMutex| Some(&mut r.inner),
        );
    }
    fn inner() {
        let next = Kp::identity();

        let lock_kp = LockKp::new(prev, StdRwLockAccess::new(), next);
    }
}

#[derive(Debug)]
struct RootRw {
    inner: RwLock<i32>,
}

#[test]
fn lock_kp_arc_mutex_read_and_update() {
    let mut root = RootMutex {
        inner: Arc::new(Mutex::new(7)),
    };

    let prev: KpType<'_, RootMutex, Arc<Mutex<i32>>> = Kp::new(
        |r: &RootMutex| Some(&r.inner),
        |r: &mut RootMutex| Some(&mut r.inner),
    );
    let next: KpType<'_, i32, i32> = Kp::new(|v: &i32| Some(v), |v: &mut i32| Some(v));

    let lock_kp: LockKp<
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
    > = LockKp::new(prev, ArcMutexAccess::new(), next);

    assert_eq!(lock_kp.get(&root), Some(7));
    assert!(lock_kp.update(&mut root, |v| *v = 42));
    assert_eq!(lock_kp.get(&root), Some(42));
}

#[test]
fn lock_kp_std_rwlock_read_and_update() {
    let mut root = RootRw {
        inner: RwLock::new(11),
    };

    let prev: KpType<'_, RootRw, RwLock<i32>> = Kp::new(
        |r: &RootRw| Some(&r.inner),
        |r: &mut RootRw| Some(&mut r.inner),
    );
    let next: KpType<'_, i32, i32> = Kp::new(|v: &i32| Some(v), |v: &mut i32| Some(v));

    let lock_kp = LockKp::new(prev, StdRwLockAccess::new(), next);

    assert_eq!(lock_kp.get(&root), Some(11));
    assert!(lock_kp.update(&mut root, |v| *v += 9));
    assert_eq!(lock_kp.get(&root), Some(20));
}
