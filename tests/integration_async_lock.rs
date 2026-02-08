//! Integration test: keypath chaining with tokio (async), parking_lot, std lock,
//! and normal Kp, using `then` (Kp) and `then_lock` (LockKp).
//!
//! Demonstrates a multi-level chain: tokio Mutex -> parking_lot -> tokio RwLock (level 3).

#![cfg(all(feature = "tokio", feature = "parking_lot"))]

use rust_key_paths::async_lock::{AsyncLockKp, TokioMutexAccess, TokioRwLockAccess};
use rust_key_paths::lock::{LockKp, ParkingLotMutexAccess, StdRwLockAccess};
use rust_key_paths::{Kp, KpType};
use std::sync::Arc;

// Level 3: innermost value behind std::sync::RwLock (tokio RwLock gives &Level3)
struct Level3 {
    value: std::sync::RwLock<i32>,
}

// Level 2: parking_lot mutex + tokio RwLock to Level3
#[derive(Clone)]
struct Level2 {
    value: i32,
    rwlock: Arc<tokio::sync::RwLock<Level3>>,
}

// Level 1: protected by parking_lot::Mutex (Arc)
#[derive(Clone)]
struct Level1 {
    parking: Arc<parking_lot::Mutex<Level2>>,
}

// Root: protected by tokio::sync::Mutex
type Root = Arc<tokio::sync::Mutex<Level1>>;

#[tokio::test]
async fn integration_async_lock_then_lock_then_chain() {
    // Root -> L1 (tokio) -> L2 (parking_lot) -> L3 (tokio RwLock)
    let root: Root = Arc::new(tokio::sync::Mutex::new(Level1 {
        parking: Arc::new(parking_lot::Mutex::new(Level2 {
            value: 42,
            rwlock: Arc::new(tokio::sync::RwLock::new(Level3 {
            value: std::sync::RwLock::new(7),
        })),
        })),
    }));

    // 1) AsyncLockKp: Root -> Level1 (tokio lock)
    let tokio_kp = {
        let prev: KpType<'_, Root, Arc<tokio::sync::Mutex<Level1>>> =
            Kp::new(|r: &Root| Some(r), |r: &mut Root| Some(r));
        let next: KpType<'_, Level1, Level1> = Kp::new(
            |l1: &Level1| Some(l1),
            |l1: &mut Level1| Some(l1),
        );
        AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
    };

    // 2) then_lock: Level1 -> Level2 (parking_lot lock)
    let with_parking = {
        let prev: KpType<'_, Level1, Arc<parking_lot::Mutex<Level2>>> = Kp::new(
            |l1: &Level1| Some(&l1.parking),
            |l1: &mut Level1| Some(&mut l1.parking),
        );
        let next: KpType<'_, Level2, Level2> = Kp::new(
            |l2: &Level2| Some(l2),
            |l2: &mut Level2| Some(l2),
        );
        let lock_kp = LockKp::new(prev, ParkingLotMutexAccess::new(), next);
        tokio_kp.then_lock(lock_kp)
    };

    // Read through chain (tokio -> parking_lot): get &Level2
    let result = with_parking.get(&root).await;
    assert!(result.is_some(), "get through tokio then_lock parking_lot");
    assert_eq!(result.unwrap().value, 42);

    // 3) Level2 -> Level3 via tokio RwLock (level 3). Use two-step get when chain output is &Level2.
    let rwlock_kp = {
        let prev: KpType<'_, Level2, Arc<tokio::sync::RwLock<Level3>>> = Kp::new(
            |l2: &Level2| Some(&l2.rwlock),
            |l2: &mut Level2| Some(&mut l2.rwlock),
        );
        let next: KpType<'_, Level3, Level3> = Kp::new(
            |l3: &Level3| Some(l3),
            |l3: &mut Level3| Some(l3),
        );
        AsyncLockKp::new(prev, TokioRwLockAccess::new(), next)
    };

    // 4) Level3 -> i32 via std::sync::RwLock (sync lock on Level3.value)
    let std_rwlock_kp = {
        let prev: KpType<'_, Level3, std::sync::RwLock<i32>> = Kp::new(
            |l3: &Level3| Some(&l3.value),
            |l3: &mut Level3| Some(&mut l3.value),
        );
        let next: KpType<'_, i32, i32> = Kp::new(|v: &i32| Some(v), |v: &mut i32| Some(v));
        LockKp::new(prev, StdRwLockAccess::new(), next)
    };

    // Read through to Level3: get Level2 then get Level3 (tokio RwLock at level 3)
    let l2 = with_parking.get(&root).await.unwrap();
    let l3 = rwlock_kp.get(l2).await;
    assert!(l3.is_some(), "get through tokio -> parking_lot -> tokio RwLock");
    // Read inner i32 through std::sync::RwLock
    let inner = std_rwlock_kp.get(l3.unwrap());
    assert_eq!(inner, Some(&7));

    // Write Level2.value via with_parking
    let mut_root = &mut root.clone();
    let mut_result = with_parking.get_mut(mut_root).await;
    assert!(mut_result.is_some());
    mut_result.unwrap().value = 100;

    let again = with_parking.get(&root).await;
    assert_eq!(again.unwrap().value, 100);

    // Write Level3.value (inner i32) via rwlock_kp then std_rwlock_kp (two-step get_mut)
    let mut_root2 = &mut root.clone();
    let mut_l2 = with_parking.get_mut(mut_root2).await.unwrap();
    let mut_l3 = rwlock_kp.get_mut(mut_l2).await.unwrap();
    let mut_inner = std_rwlock_kp.get_mut(mut_l3);
    assert!(mut_inner.is_some());
    *mut_inner.unwrap() = 99;

    let l2_again = with_parking.get(&root).await.unwrap();
    let l3_again = rwlock_kp.get(l2_again).await;
    assert!(l3_again.is_some());
    let inner_again = std_rwlock_kp.get(l3_again.unwrap());
    assert_eq!(inner_again, Some(&99));
}
