//! Integration test: keypath chaining with tokio (async), parking_lot, std lock,
//! and normal Kp, using `then` (Kp) and `then_lock` (LockKp).
//!
//! Demonstrates a multi-level chain: tokio async lock -> parking_lot lock -> then Kp to a field.

#![cfg(all(feature = "tokio", feature = "parking_lot"))]

use rust_key_paths::async_lock::{AsyncLockKp, TokioMutexAccess};
use rust_key_paths::lock::{LockKp, ParkingLotMutexAccess};
use rust_key_paths::{Kp, KpType};
use std::sync::Arc;

// Level 2: inner value with a field we can read (Clone required by LockKp::get)
#[derive(Clone)]
struct Level2 {
    value: i32,
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
    // Root -> L1 (tokio) -> L2 (parking_lot) -> i32 via Kp
    let root: Root = Arc::new(tokio::sync::Mutex::new(Level1 {
        parking: Arc::new(parking_lot::Mutex::new(Level2 { value: 42 })),
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

    // Write through chain (mutate Level2.value via get_mut)
    let mut_root = &mut root.clone();
    let mut_result = with_parking.get_mut(mut_root).await;
    assert!(mut_result.is_some());
    mut_result.unwrap().value = 100;

    let again = with_parking.get(&root).await;
    assert_eq!(again.unwrap().value, 100);
}
