#![cfg(feature = "tokio")]

use rust_key_paths::async_lock::{AsyncLockKp, TokioMutexAccess, TokioRwLockAccess};
use rust_key_paths::{Kp, KpType};
use std::sync::Arc;

#[derive(Clone)]
struct Root {
    m: Arc<tokio::sync::Mutex<Inner>>,
    r: Arc<tokio::sync::RwLock<i32>>,
}

#[derive(Clone)]
struct Inner {
    v: i32,
}

#[tokio::test]
async fn async_lock_kp_mutex_get_and_update() {
    let root = Root {
        m: Arc::new(tokio::sync::Mutex::new(Inner { v: 7 })),
        r: Arc::new(tokio::sync::RwLock::new(11)),
    };

    let prev: KpType<'_, Root, Arc<tokio::sync::Mutex<Inner>>> =
        Kp::new(|r: &Root| Some(&r.m), |r: &mut Root| Some(&mut r.m));
    let next: KpType<'_, Inner, i32> =
        Kp::new(|i: &Inner| Some(&i.v), |i: &mut Inner| Some(&mut i.v));
    let kp = AsyncLockKp::new(prev, TokioMutexAccess::new(), next);

    assert_eq!(kp.get(&root).await, Some(7));
    assert!(kp.update(&root, |v| *v = 42).await);
    assert_eq!(kp.get(&root).await, Some(42));
}

#[tokio::test]
async fn async_lock_kp_rwlock_get_and_missing_edge_case() {
    let root = Root {
        m: Arc::new(tokio::sync::Mutex::new(Inner { v: 1 })),
        r: Arc::new(tokio::sync::RwLock::new(9)),
    };

    let prev: KpType<'_, Root, Arc<tokio::sync::RwLock<i32>>> =
        Kp::new(|r: &Root| Some(&r.r), |r: &mut Root| Some(&mut r.r));
    let next: KpType<'_, i32, i32> = Kp::new(|v: &i32| Some(v), |v: &mut i32| Some(v));
    let kp = AsyncLockKp::new(prev, TokioRwLockAccess::new(), next);

    assert_eq!(kp.get(&root).await, Some(9));
    assert!(kp.update(&root, |v| *v += 5).await);
    assert_eq!(kp.get(&root).await, Some(14));
}
