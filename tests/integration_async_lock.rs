//! Integration test: keypath chaining with consistent naming across the crate.
//!
//! Convention: `then` (Kp), `then_lock` (sync LockKp), `then_async` (tokio/async keypath).
//! Target style: `root_lock.then_lock(parking_kp).then_async(async_kp).then_lock(std_lock_kp)`.

#![cfg(all(feature = "tokio", feature = "parking_lot"))]

use rust_key_paths::async_lock::{AsyncLockKp, TokioMutexAccess, TokioRwLockAccess};
use rust_key_paths::lock::{LockKp, ParkingLotMutexAccess, StdRwLockAccess};
use rust_key_paths::{Kp, KpType};
use std::sync::Arc;

// Level 3: innermost value behind std::sync::RwLock
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
    let root: Root = Arc::new(tokio::sync::Mutex::new(Level1 {
        parking: Arc::new(parking_lot::Mutex::new(Level2 {
            value: 42,
            rwlock: Arc::new(tokio::sync::RwLock::new(Level3 {
                value: std::sync::RwLock::new(7),
            })),
        })),
    }));

    // Build keypaths (same naming: _kp for keypaths, then_lock / then_async for chaining)
    let root_lock = {
        let prev: KpType<'_, Root, Arc<tokio::sync::Mutex<Level1>>> =
            Kp::new(|r: &Root| Some(r), |r: &mut Root| Some(r));
        let next: KpType<'_, Level1, Level1> =
            Kp::new(|l1: &Level1| Some(l1), |l1: &mut Level1| Some(l1));
        AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
    };

    let parking_kp = {
        let prev: KpType<'_, Level1, Arc<parking_lot::Mutex<Level2>>> =
            Kp::new(|l1: &Level1| Some(&l1.parking), |l1: &mut Level1| Some(&mut l1.parking));
        let next: KpType<'_, Level2, Level2> =
            Kp::new(|l2: &Level2| Some(l2), |l2: &mut Level2| Some(l2));
        LockKp::new(prev, ParkingLotMutexAccess::new(), next)
    };

    let async_kp = {
        let prev: KpType<'_, Level2, Arc<tokio::sync::RwLock<Level3>>> =
            Kp::new(|l2: &Level2| Some(&l2.rwlock), |l2: &mut Level2| Some(&mut l2.rwlock));
        let next: KpType<'_, Level3, Level3> =
            Kp::new(|l3: &Level3| Some(l3), |l3: &mut Level3| Some(l3));
        AsyncLockKp::new(prev, TokioRwLockAccess::new(), next)
    };

    let std_lock_kp = {
        let prev: KpType<'_, Level3, std::sync::RwLock<i32>> =
            Kp::new(|l3: &Level3| Some(&l3.value), |l3: &mut Level3| Some(&mut l3.value));
        let next: KpType<'_, i32, i32> = Kp::new(|v: &i32| Some(v), |v: &mut i32| Some(v));
        LockKp::new(prev, StdRwLockAccess::new(), next)
    };

    // Simplest composition: root_lock.then_lock(parking_kp).then_async(async_kp).then_lock(std_lock_kp)
    // When the first segment yields reference types, .then_async() requires MutValue2: BorrowMut<Value2>,
    // so we build the chain in two parts and use the same naming.
    let with_parking = root_lock.then_lock(parking_kp);

    // Read: with_parking -> Level2
    let result = with_parking.get(&root).await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().value, 42);

    // Two-step to Level3 and inner i32 (same logical chain: then_async then then_lock)
    let l2 = with_parking.get(&root).await.unwrap();
    let l3 = async_kp.get(l2).await;
    assert!(l3.is_some());
    let inner = std_lock_kp.get(l3.unwrap());
    assert_eq!(inner, Some(&7));

    // Write Level2
    let mut_root = &mut root.clone();
    with_parking.get_mut(mut_root).await.unwrap().value = 100;
    assert_eq!(with_parking.get(&root).await.unwrap().value, 100);

    // Write Level3.value (inner i32) via async_kp then std_lock_kp
    let mut_root2 = &mut root.clone();
    let mut_l2 = with_parking.get_mut(mut_root2).await.unwrap();
    let mut_l3 = async_kp.get_mut(mut_l2).await.unwrap();
    *std_lock_kp.get_mut(mut_l3).unwrap() = 99;

    let l2_again = with_parking.get(&root).await.unwrap();
    let l3_again = async_kp.get(l2_again).await;
    assert_eq!(std_lock_kp.get(l3_again.unwrap()), Some(&99));
}
