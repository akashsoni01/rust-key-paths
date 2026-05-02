// cargo check --example box_keypath_arcswap --features arc-swap
// cargo run --example box_keypath_arcswap --features arc-swap
use key_paths_derive::Kp;
use rust_key_paths::ChainExt;
use std::sync::Arc;

#[derive(Debug, Kp, Clone)]
struct SomeComplexStruct {
    scsf: Box<SomeOtherStruct>,
}

/// Arc-swap fields use **`sosf()`** → **`LockKp`** into **`OneMoreStruct`** only (no **`sosf_kp()`**).
/// Enum snapshots use **`SomeEnum::b()`** → **`LockKp`** (no **`b_lock()`**).
///
/// Chain after **`outer.then_lock(Self::sosf())`** with **`.then(OneMoreStruct::…)`**, and nest swaps with **`.then_lock(SomeEnum::b())`**.
#[derive(Debug, Kp, Clone)]
struct SomeOtherStruct {
    sosf: Arc<arc_swap::ArcSwap<OneMoreStruct>>,
}

#[allow(dead_code)]
#[derive(Debug, Kp, Clone)]
enum SomeEnum {
    A(String),
    /// Snapshot behind `ArcSwap` (typical hot-reload / config pattern).
    B(Arc<arc_swap::ArcSwap<DarkStruct>>),
}

#[derive(Debug, Kp, Clone)]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Kp, Clone)]
struct DarkStruct {
    dsf: String,
    hot: Arc<arc_swap::ArcSwap<String>>,
}

fn init_via_keypaths() -> SomeComplexStruct {
    let mut root = SomeComplexStruct {
        scsf: Box::new(SomeOtherStruct {
            sosf: Arc::new(arc_swap::ArcSwap::from_pointee(OneMoreStruct {
                omsf: String::new(),
                omse: SomeEnum::A(String::new()),
            })),
        }),
    };

    // `get_mut` / `get` use `LockAccess::lock_write` / `lock_read` → `ArcSwap::load` (not `load_full`).
    SomeComplexStruct::scsf()
        .then_lock(SomeOtherStruct::sosf())
        .get_mut(&mut root)
        .map(|inner| {
            inner.omsf = "omsf_value".to_string();
            inner.omse = SomeEnum::B(Arc::new(arc_swap::ArcSwap::from_pointee(DarkStruct {
                dsf: "dark_value".to_string(),
                hot: Arc::new(arc_swap::ArcSwap::from_pointee("hot_value".to_string())),
            })));
        });

    root
}

fn main() {
    let instance = init_via_keypaths();

    let kp_dsf = SomeComplexStruct::scsf().then_lock(
        SomeOtherStruct::sosf()
            .then(OneMoreStruct::omse())
            .then_lock(SomeEnum::b())
            .then(DarkStruct::dsf()),
    );
    println!("size_of_val(&kp_dsf) = {}", std::mem::size_of_val(&kp_dsf));
    assert_eq!(kp_dsf.get(&instance).map(|s| s.as_str()), Some("dark_value"));

    let kp_hot = SomeComplexStruct::scsf()
    .then_lock(SomeOtherStruct::sosf())
    .then(OneMoreStruct::omse())
    .then_lock(SomeEnum::b())
    .then_lock(DarkStruct::hot());

    println!("size_of_val(&kp_hot) = {}", std::mem::size_of_val(&kp_hot));
    assert_eq!(kp_hot.get(&instance).map(|s| s.as_str()), Some("hot_value"));

    let kp_omsf = SomeComplexStruct::scsf().then_lock(
        SomeOtherStruct::sosf().then(OneMoreStruct::omsf()),
    );
    assert_eq!(kp_omsf.get(&instance).map(|s| s.as_str()), Some("omsf_value"));
}
