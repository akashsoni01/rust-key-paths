use key_paths_derive::Kp;
use parking_lot::{Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// cargo check --example basics_casepath --features parking_lot
// cargo run --example basics_casepath --features parking_lot
#[derive(Debug, Kp)]
struct SomeComplexStruct {
    id: String,
    scfs20: Option<Vec<String>>,
    scfs18: Option<HashMap<String, String>>,
    scfs19: Option<HashSet<String>>,
    scsf: Option<Box<SomeOtherStruct>>,
    scfs2: Arc<std::sync::Mutex<SomeOtherStruct>>,
    scfs3: Arc<std::sync::RwLock<SomeOtherStruct>>,
    // scfs4: Arc<RefMut<SomeOtherStruct>>,
    // scfs5: Option<RefCell<SomeOtherStruct>>,
    scfs6: Option<std::sync::Mutex<SomeOtherStruct>>,
    scfs7: Option<std::sync::RwLock<SomeOtherStruct>>,
    scfs8: Option<Mutex<SomeOtherStruct>>,
    scfs9: Option<RwLock<SomeOtherStruct>>,

    scfs10: std::sync::Mutex<Option<SomeOtherStruct>>,
    scfs11: std::sync::RwLock<Option<SomeOtherStruct>>,
    scfs12: Mutex<Option<SomeOtherStruct>>,
    scfs13: RwLock<Option<SomeOtherStruct>>,

    scfs14: Option<std::sync::Mutex<Option<SomeOtherStruct>>>,
    scfs15: Option<std::sync::RwLock<Option<SomeOtherStruct>>>,
    scfs16: Option<Mutex<Option<SomeOtherStruct>>>,
    scfs17: Option<RwLock<Option<SomeOtherStruct>>>,

    // Locks inside Arc (LockKp: root SomeComplexStruct, value SomeOtherStruct)
    scfs_arc_pl_m: Arc<Mutex<SomeOtherStruct>>,
    scfs_arc_pl_rw: Arc<RwLock<SomeOtherStruct>>,
    scfs_arc_std_mo: Arc<std::sync::Mutex<Option<SomeOtherStruct>>>,
    scfs_arc_std_rwo: Arc<std::sync::RwLock<Option<SomeOtherStruct>>>,
    scfs_arc_pl_mo: Arc<Mutex<Option<SomeOtherStruct>>>,
    scfs_arc_pl_rwo: Arc<RwLock<Option<SomeOtherStruct>>>,
    scfs_o_arc_std_m: Option<Arc<std::sync::Mutex<SomeOtherStruct>>>,
    scfs_o_arc_std_rw: Option<Arc<std::sync::RwLock<SomeOtherStruct>>>,
    scfs_o_arc_pl_m: Option<Arc<Mutex<SomeOtherStruct>>>,
    scfs_o_arc_pl_rw: Option<Arc<RwLock<SomeOtherStruct>>>,

    // Tokio: same combinations as above — produce AsyncLockKp (root SomeComplexStruct, value SomeOtherStruct or Option<SomeOtherStruct>)
    #[cfg(feature = "tokio")]
    scfs_t_arc_m: Arc<tokio::sync::Mutex<SomeOtherStruct>>,
    #[cfg(feature = "tokio")]
    scfs_t_arc_rw: Arc<tokio::sync::RwLock<SomeOtherStruct>>,
    #[cfg(feature = "tokio")]
    scfs_t_arc_mo: Arc<tokio::sync::Mutex<Option<SomeOtherStruct>>>,
    #[cfg(feature = "tokio")]
    scfs_t_arc_rwo: Arc<tokio::sync::RwLock<Option<SomeOtherStruct>>>,
    #[cfg(feature = "tokio")]
    scfs_t_o_arc_m: Option<Arc<tokio::sync::Mutex<SomeOtherStruct>>>,
    #[cfg(feature = "tokio")]
    scfs_t_o_arc_rw: Option<Arc<tokio::sync::RwLock<SomeOtherStruct>>>,
    #[cfg(feature = "tokio")]
    scfs_t_o_arc_mo: Option<Arc<tokio::sync::Mutex<Option<SomeOtherStruct>>>>,
    #[cfg(feature = "tokio")]
    scfs_t_o_arc_rwo: Option<Arc<tokio::sync::RwLock<Option<SomeOtherStruct>>>>,
}

#[derive(Debug, Kp, Clone)]
struct SomeOtherStruct {
    sosf: Box<Option<OneMoreStruct>>,
}

#[derive(Debug, Clone, Kp)]
enum SomeEnum {
    A(String),
    B(Option<Arc<DarkStruct>>),
}

#[derive(Debug, Kp, Clone)]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Kp)]
struct DarkStruct {
    dsf: Option<String>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        let inner = SomeOtherStruct {
            sosf: Box::new(None),
        };

        Self {
            id: String::from("SomeComplexStruct"),
            scfs18: None,
            scfs19: None,
            scsf: Some(Box::new(inner.clone())),
            // Arc<std::sync::Mutex/RwLock<T>>
            scfs2: Arc::new(std::sync::Mutex::new(inner.clone())),
            scfs3: Arc::new(std::sync::RwLock::new(inner.clone())),

            // Option<std::sync::Mutex/RwLock<T>>
            scfs6: Some(std::sync::Mutex::new(inner.clone())),
            scfs7: Some(std::sync::RwLock::new(inner.clone())),

            // Option<parking_lot::Mutex/RwLock<T>>
            scfs8: Some(Mutex::new(inner.clone())),
            scfs9: Some(RwLock::new(inner.clone())),

            // std::sync::Mutex/RwLock<Option<T>>
            scfs10: std::sync::Mutex::new(Some(inner.clone())),
            scfs11: std::sync::RwLock::new(Some(inner.clone())),

            // parking_lot::Mutex/RwLock<Option<T>>
            scfs12: Mutex::new(Some(inner.clone())),
            scfs13: RwLock::new(Some(inner.clone())),

            // Option<std::sync::Mutex/RwLock<Option<T>>>
            scfs14: Some(std::sync::Mutex::new(Some(inner.clone()))),
            scfs15: Some(std::sync::RwLock::new(Some(inner.clone()))),

            // Option<parking_lot::Mutex/RwLock<Option<T>>>
            scfs16: Some(Mutex::new(Some(inner.clone()))),
            scfs17: Some(RwLock::new(Some(inner.clone()))),

            // Locks inside Arc
            scfs_arc_pl_m: Arc::new(Mutex::new(inner.clone())),
            scfs_arc_pl_rw: Arc::new(RwLock::new(inner.clone())),
            scfs_arc_std_mo: Arc::new(std::sync::Mutex::new(Some(inner.clone()))),
            scfs_arc_std_rwo: Arc::new(std::sync::RwLock::new(Some(inner.clone()))),
            scfs_arc_pl_mo: Arc::new(Mutex::new(Some(inner.clone()))),
            scfs_arc_pl_rwo: Arc::new(RwLock::new(Some(inner.clone()))),

            // Option<Arc<...>>
            scfs_o_arc_std_m: Some(Arc::new(std::sync::Mutex::new(inner.clone()))),
            scfs_o_arc_std_rw: Some(Arc::new(std::sync::RwLock::new(inner.clone()))),
            scfs_o_arc_pl_m: Some(Arc::new(Mutex::new(inner.clone()))),
            scfs_o_arc_pl_rw: Some(Arc::new(RwLock::new(inner))),

            // Tokio fields (only when feature is enabled)
            #[cfg(feature = "tokio")]
            scfs_t_arc_m: Arc::new(tokio::sync::Mutex::new(SomeOtherStruct {
                sosf: Box::new(None),
            })),
            #[cfg(feature = "tokio")]
            scfs_t_arc_rw: Arc::new(tokio::sync::RwLock::new(SomeOtherStruct {
                sosf: Box::new(None),
            })),
            #[cfg(feature = "tokio")]
            scfs_t_arc_mo: Arc::new(tokio::sync::Mutex::new(Some(SomeOtherStruct {
                sosf: Box::new(None),
            }))),
            #[cfg(feature = "tokio")]
            scfs_t_arc_rwo: Arc::new(tokio::sync::RwLock::new(Some(SomeOtherStruct {
                sosf: Box::new(None),
            }))),
            #[cfg(feature = "tokio")]
            scfs_t_o_arc_m: Some(Arc::new(tokio::sync::Mutex::new(SomeOtherStruct {
                sosf: Box::new(None),
            }))),
            #[cfg(feature = "tokio")]
            scfs_t_o_arc_rw: Some(Arc::new(tokio::sync::RwLock::new(SomeOtherStruct {
                sosf: Box::new(None),
            }))),
            #[cfg(feature = "tokio")]
            scfs_t_o_arc_mo: Some(Arc::new(tokio::sync::Mutex::new(Some(SomeOtherStruct {
                sosf: Box::new(None),
            })))),
            #[cfg(feature = "tokio")]
            scfs_t_o_arc_rwo: Some(Arc::new(tokio::sync::RwLock::new(Some(SomeOtherStruct {
                sosf: Box::new(None),
            })))),
            scfs20: Some(Vec::new()),
        }
    }
}

/// Exercise every **sync** `LockKp` keypath: `get` and `get_mut` on the `SomeOtherStruct` value
/// (exercises `rust_key_paths::lock::LockKp<…>` paths produced by the derive).
fn exercise_all_sync_lock_kps_get(r: &SomeComplexStruct) {
    // Arc<std::sync::{Mutex,RwLock}<SomeOtherStruct>>
    assert!(SomeComplexStruct::scfs2().get(r).is_some());
    assert!(SomeComplexStruct::scfs3().get(r).is_some());
    // Arc + parking_lot
    assert!(SomeComplexStruct::scfs_arc_pl_m().get(r).is_some());
    assert!(SomeComplexStruct::scfs_arc_pl_rw().get(r).is_some());
    // Arc + Option<…> in the lock (value type is `SomeOtherStruct` after unwrapping)
    assert!(SomeComplexStruct::scfs_arc_std_mo().get(r).is_some());
    assert!(SomeComplexStruct::scfs_arc_std_rwo().get(r).is_some());
    assert!(SomeComplexStruct::scfs_arc_pl_mo().get(r).is_some());
    assert!(SomeComplexStruct::scfs_arc_pl_rwo().get(r).is_some());
    // Option<Arc<…>> outer
    assert!(SomeComplexStruct::scfs_o_arc_std_m().get(r).is_some());
    assert!(SomeComplexStruct::scfs_o_arc_std_rw().get(r).is_some());
    assert!(SomeComplexStruct::scfs_o_arc_pl_m().get(r).is_some());
    assert!(SomeComplexStruct::scfs_o_arc_pl_rw().get(r).is_some());
    println!("exercise_all_sync_lock_kps_get: ok");
}
fn exercise_all_sync_lock_kps_get_mut(m: &mut SomeComplexStruct) {
    assert!(SomeComplexStruct::scfs2().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs3().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_arc_pl_m().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_arc_pl_rw().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_arc_std_mo().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_arc_std_rwo().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_arc_pl_mo().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_arc_pl_rwo().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_o_arc_std_m().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_o_arc_std_rw().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_o_arc_pl_m().get_mut(m).is_some());
    assert!(SomeComplexStruct::scfs_o_arc_pl_rw().get_mut(m).is_some());
    println!("exercise_all_sync_lock_kps_get_mut: ok");
}

// Async `LockKp` / `Option<Arc<tokio::...>>` combinations: exercise separately with
// `cargo run --example ... --features tokio` on a type that does not also need `parking_lot`
// in the same `#[derive(Kp)]` struct, or fix derive classification for `tokio`+`parking_lot` together.
fn main() {
    let mut instance = SomeComplexStruct::new();

    SomeComplexStruct::scfs2()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b()) // Generated by Casepaths macro
        .then(DarkStruct::dsf())
        .get_mut(&mut instance)
        .map(|x| {
            *x = String::from("🖖🏿🖖🏿🖖🏿🖖🏿");
        });

    SomeComplexStruct::scfs_arc_pl_rw()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b()) // Generated by Casepaths macro
        .then(DarkStruct::dsf())
        .get_mut(&mut instance)
        .map(|x| {
            *x = String::from("🖖🏿🖖🏿🖖🏿🖖🏿");
        });

    let kp = SomeComplexStruct::scfs3()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b()) // Generated by Casepaths macro
        .then(DarkStruct::dsf());
    println!("size of kp ===== {:?}", size_of_val(&kp));
    // LockKp for Arc<parking_lot::RwLock<Option<SomeOtherStruct>>> should yield value SomeOtherStruct
    let mut x_pl_rw = false;
    if let Some(_) = SomeComplexStruct::scfs_arc_pl_rwo().get(&instance) {
        x_pl_rw = true;
    }
    // Same for Arc<std::sync::Mutex<Option<SomeOtherStruct>>>
    let mut x_std_m = false;
    if let Some(_) = SomeComplexStruct::scfs_arc_std_mo().get(&instance) {
        x_std_m = true;
    }
    // And Arc<std::sync::RwLock<Option<SomeOtherStruct>>>
    let mut x_std_rw = false;
    if let Some(_) = SomeComplexStruct::scfs_arc_std_rwo().get(&instance) {
        x_std_rw = true;
    }
    // And Arc<parking_lot::Mutex<Option<SomeOtherStruct>>>
    let mut x_pl_m = false;
    if let Some(_) = SomeComplexStruct::scfs_arc_pl_mo().get(&instance) {
        x_pl_m = true;
    }

    let _x = SomeComplexStruct::scfs18_at("testing".to_string());
    let _x = SomeComplexStruct::scfs20();
    let _x = SomeComplexStruct::scfs20_at(0);

    exercise_all_sync_lock_kps_get(&instance);
    exercise_all_sync_lock_kps_get_mut(&mut instance);
    println!("x = {:?}", _x);
    assert!(x_pl_rw);
    assert!(x_std_m);
    assert!(x_std_rw);
    assert!(x_pl_m);
    // if let Some(omsf) = SomeComplexStruct::scfs2_lock()
    //     .then(SomeOtherStruct::sosf())
    //     .then(OneMoreStruct::omse())
    //     .then(SomeEnum::b()) // Generated by Casepaths macro
    //     .then(DarkStruct::dsf())
    //     .get_mut(&mut instance)
    // {
    //     *omsf = String::from("This is changed 🖖🏿🖖🏿🖖🏿🖖🏿🖖🏿🖖🏿🖖🏿");
    // }

    // println!("instance = {:?}", instance.scsf.unwrap().sosf.unwrap().omse.unwrap());
    // output - instance = B(DarkStruct { dsf: Some("🖖🏿🖖🏿🖖🏿🖖🏿") })
}
