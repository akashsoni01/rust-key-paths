//! Integration test: all 9 possible nesting combinations in one place.
//!
//! The 9 pairs (First → Second) are:
//! 1. Kp → Kp      2. Kp → LockKp    3. Kp → AsyncKp
//! 4. LockKp → Kp  5. LockKp → LockKp  6. LockKp → AsyncKp
//! 7. AsyncKp → Kp  8. AsyncKp → LockKp  9. AsyncKp → AsyncKp

#![cfg(all(feature = "tokio", feature = "parking_lot"))]

use rust_key_paths::async_lock::{AsyncLockKp, TokioMutexAccess};
use rust_key_paths::lock::{LockKp, ArcMutexAccess, ParkingLotMutexAccess};
use rust_key_paths::{Kp, KpType};
use std::sync::{Arc, Mutex};

// -----------------------------------------------------------------------------
// 1. Kp → Kp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root1 {
    a: A1,
}
#[derive(Clone)]
struct A1 {
    b: i32,
}

// -----------------------------------------------------------------------------
// 2. Kp → LockKp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root2 {
    m: Arc<Mutex<B2>>,
}
#[derive(Clone)]
struct B2 {
    x: i32,
}

// -----------------------------------------------------------------------------
// 3. Kp → AsyncKp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root3 {
    t: Arc<tokio::sync::Mutex<C3>>,
}
#[derive(Clone)]
struct C3 {
    y: i32,
}

// -----------------------------------------------------------------------------
// 4. LockKp → Kp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root4 {
    m: Arc<Mutex<D4>>,
}
#[derive(Clone)]
struct D4 {
    e: E4,
}
#[derive(Clone)]
struct E4 {
    z: i32,
}

// -----------------------------------------------------------------------------
// 5. LockKp → LockKp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root5 {
    m1: Arc<Mutex<F5>>,
}
#[derive(Clone)]
struct F5 {
    m2: Arc<Mutex<G5>>,
}
#[derive(Clone)]
struct G5 {
    v: i32,
}

// -----------------------------------------------------------------------------
// 6. LockKp → AsyncKp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root6 {
    m: Arc<Mutex<H6>>,
}
#[derive(Clone)]
struct H6 {
    t: Arc<tokio::sync::Mutex<I6>>,
}
#[derive(Clone)]
struct I6 {
    w: i32,
}

// -----------------------------------------------------------------------------
// 7. AsyncKp → Kp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root7 {
    t: Arc<tokio::sync::Mutex<J7>>,
}
#[derive(Clone)]
struct J7 {
    k: i32,
}

// -----------------------------------------------------------------------------
// 8. AsyncKp → LockKp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root8 {
    t: Arc<tokio::sync::Mutex<L8>>,
}
#[derive(Clone)]
struct L8 {
    m: Arc<parking_lot::Mutex<M8>>,
}
#[derive(Clone)]
struct M8 {
    n: i32,
}

// -----------------------------------------------------------------------------
// 9. AsyncKp → AsyncKp
// -----------------------------------------------------------------------------
#[derive(Clone)]
struct Root9 {
    t1: Arc<tokio::sync::Mutex<N9>>,
}
#[derive(Clone)]
struct N9 {
    t2: Arc<tokio::sync::Mutex<P9>>,
}
#[derive(Clone)]
struct P9 {
    q: i32,
}

#[tokio::test]
async fn all_nine_nesting_combinations() {
    // 1. Kp → Kp
    let root1 = Root1 { a: A1 { b: 1 } };
    let kp_ra: KpType<Root1, A1> = Kp::new(|r: &Root1| Some(&r.a), |r: &mut Root1| Some(&mut r.a));
    let kp_ab: KpType<A1, i32> = Kp::new(|a: &A1| Some(&a.b), |a: &mut A1| Some(&mut a.b));
    let chain1 = kp_ra.then(kp_ab);
    assert_eq!(chain1.get(&root1), Some(&1));

    // 2. Kp → LockKp
    let root2 = Root2 {
        m: Arc::new(Mutex::new(B2 { x: 2 })),
    };
    let kp_rm: KpType<Root2, Arc<Mutex<B2>>> =
        Kp::new(|r: &Root2| Some(&r.m), |r: &mut Root2| Some(&mut r.m));
    let lock_bx = {
        let prev: KpType<Arc<Mutex<B2>>, Arc<Mutex<B2>>> =
            Kp::new(|m: &Arc<Mutex<B2>>| Some(m), |m: &mut Arc<Mutex<B2>>| Some(m));
        let next: KpType<B2, i32> = Kp::new(|b: &B2| Some(&b.x), |b: &mut B2| Some(&mut b.x));
        LockKp::new(prev, ArcMutexAccess::new(), next)
    };
    let chain2 = kp_rm.then_lock(lock_bx);
    assert_eq!(chain2.get(&root2), Some(&2));

    // 3. Kp → AsyncKp
    let root3 = Root3 {
        t: Arc::new(tokio::sync::Mutex::new(C3 { y: 3 })),
    };
    let kp_rt: KpType<Root3, Arc<tokio::sync::Mutex<C3>>> =
        Kp::new(|r: &Root3| Some(&r.t), |r: &mut Root3| Some(&mut r.t));
    let async_cy = {
        let prev: KpType<Arc<tokio::sync::Mutex<C3>>, Arc<tokio::sync::Mutex<C3>>> =
            Kp::new(|t: &_| Some(t), |t: &mut _| Some(t));
        let next: KpType<C3, i32> = Kp::new(|c: &C3| Some(&c.y), |c: &mut C3| Some(&mut c.y));
        AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
    };
    let chain3 = kp_rt.then_async::<_, i32>(async_cy);
    assert_eq!(chain3.get(&root3).await, Some(&3));

    // 4. LockKp → Kp
    let root4 = Root4 {
        m: Arc::new(Mutex::new(D4 {
            e: E4 { z: 4 },
        })),
    };
    let lock_rd = {
        let prev: KpType<Root4, Arc<Mutex<D4>>> =
            Kp::new(|r: &Root4| Some(&r.m), |r: &mut Root4| Some(&mut r.m));
        let next: KpType<D4, D4> = Kp::new(|d: &D4| Some(d), |d: &mut D4| Some(d));
        LockKp::new(prev, ArcMutexAccess::new(), next)
    };
    let kp_de: KpType<D4, i32> = Kp::new(|d: &D4| Some(&d.e.z), |d: &mut D4| Some(&mut d.e.z));
    let chain4 = lock_rd.then(kp_de);
    assert_eq!(chain4.get(&root4), Some(&4));

    // 5. LockKp → LockKp
    let root5 = Root5 {
        m1: Arc::new(Mutex::new(F5 {
            m2: Arc::new(Mutex::new(G5 { v: 5 })),
        })),
    };
    let lock_rf = {
        let prev: KpType<Root5, Arc<Mutex<F5>>> =
            Kp::new(|r: &Root5| Some(&r.m1), |r: &mut Root5| Some(&mut r.m1));
        let next: KpType<F5, F5> = Kp::new(|f: &F5| Some(f), |f: &mut F5| Some(f));
        LockKp::new(prev, ArcMutexAccess::new(), next)
    };
    let lock_fg = {
        let prev: KpType<F5, Arc<Mutex<G5>>> =
            Kp::new(|f: &F5| Some(&f.m2), |f: &mut F5| Some(&mut f.m2));
        let next: KpType<G5, i32> = Kp::new(|g: &G5| Some(&g.v), |g: &mut G5| Some(&mut g.v));
        LockKp::new(prev, ArcMutexAccess::new(), next)
    };
    let chain5 = lock_rf.then_lock(lock_fg);
    assert_eq!(chain5.get(&root5), Some(&5));

    // 6. LockKp → AsyncKp
    let root6 = Root6 {
        m: Arc::new(Mutex::new(H6 {
            t: Arc::new(tokio::sync::Mutex::new(I6 { w: 6 })),
        })),
    };
    let lock_rh = {
        let prev: KpType<Root6, Arc<Mutex<H6>>> =
            Kp::new(|r: &Root6| Some(&r.m), |r: &mut Root6| Some(&mut r.m));
        let next: KpType<H6, H6> = Kp::new(|h: &H6| Some(h), |h: &mut H6| Some(h));
        LockKp::new(prev, ArcMutexAccess::new(), next)
    };
    let async_hi = {
        let prev: KpType<H6, Arc<tokio::sync::Mutex<I6>>> =
            Kp::new(|h: &H6| Some(&h.t), |h: &mut H6| Some(&mut h.t));
        let next: KpType<I6, i32> = Kp::new(|i: &I6| Some(&i.w), |i: &mut I6| Some(&mut i.w));
        AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
    };
    let chain6 = lock_rh.then_async::<_, i32>(async_hi);
    assert_eq!(chain6.get(&root6).await, Some(&6));

    // 7. AsyncKp → Kp
    let root7 = Root7 {
        t: Arc::new(tokio::sync::Mutex::new(J7 { k: 7 })),
    };
    let async_rj = {
        let prev: KpType<Root7, Arc<tokio::sync::Mutex<J7>>> =
            Kp::new(|r: &Root7| Some(&r.t), |r: &mut Root7| Some(&mut r.t));
        let next: KpType<J7, J7> = Kp::new(|j: &J7| Some(j), |j: &mut J7| Some(j));
        AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
    };
    let kp_jk: KpType<J7, i32> = Kp::new(|j: &J7| Some(&j.k), |j: &mut J7| Some(&mut j.k));
    let chain7 = async_rj.then(kp_jk);
    assert_eq!(chain7.get(&root7).await, Some(&7));

    // 8. AsyncKp → LockKp
    let root8 = Root8 {
        t: Arc::new(tokio::sync::Mutex::new(L8 {
            m: Arc::new(parking_lot::Mutex::new(M8 { n: 8 })),
        })),
    };
    let async_rl = {
        let prev: KpType<Root8, Arc<tokio::sync::Mutex<L8>>> =
            Kp::new(|r: &Root8| Some(&r.t), |r: &mut Root8| Some(&mut r.t));
        let next: KpType<L8, L8> = Kp::new(|l: &L8| Some(l), |l: &mut L8| Some(l));
        AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
    };
    let lock_mn = {
        let prev: KpType<L8, Arc<parking_lot::Mutex<M8>>> =
            Kp::new(|l: &L8| Some(&l.m), |l: &mut L8| Some(&mut l.m));
        let next: KpType<M8, i32> = Kp::new(|m: &M8| Some(&m.n), |m: &mut M8| Some(&mut m.n));
        LockKp::new(prev, ParkingLotMutexAccess::new(), next)
    };
    let chain8 = async_rl.then_lock(lock_mn);
    assert_eq!(chain8.get(&root8).await, Some(&8));

    // 9. AsyncKp → AsyncKp
    let root9 = Root9 {
        t1: Arc::new(tokio::sync::Mutex::new(N9 {
            t2: Arc::new(tokio::sync::Mutex::new(P9 { q: 9 })),
        })),
    };
    let async_rn = {
        let prev: KpType<Root9, Arc<tokio::sync::Mutex<N9>>> =
            Kp::new(|r: &Root9| Some(&r.t1), |r: &mut Root9| Some(&mut r.t1));
        let next: KpType<N9, N9> = Kp::new(|n: &N9| Some(n), |n: &mut N9| Some(n));
        AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
    };
    let async_pq = {
        let prev: KpType<N9, Arc<tokio::sync::Mutex<P9>>> =
            Kp::new(|n: &N9| Some(&n.t2), |n: &mut N9| Some(&mut n.t2));
        let next: KpType<P9, i32> = Kp::new(|p: &P9| Some(&p.q), |p: &mut P9| Some(&mut p.q));
        AsyncLockKp::new(prev, TokioMutexAccess::new(), next)
    };
    let chain9 = async_rn.then_async(async_pq);
    assert_eq!(chain9.get(&root9).await, Some(&9));
}
