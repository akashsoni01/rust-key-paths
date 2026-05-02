// cargo check --example box_keypath_arcswap --features arcswap
// cargo run --example box_keypath_arcswap --features arcswap
use key_paths_derive::Kp;
use rust_key_paths::ChainExt;
use std::sync::Arc;

#[derive(Debug, Kp, Clone)]
struct SomeComplexStruct {
    scsf: Box<SomeOtherStruct>,
}

#[derive(Debug, Kp, Clone)]
struct SomeOtherStruct {
    sosf: Arc<arcswap::ArcSwap<OneMoreStruct>>,
}

#[allow(dead_code)]
#[derive(Debug, Kp, Clone)]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

#[derive(Debug, Kp, Clone)]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Kp, Clone)]
struct DarkStruct {
    dsf: String,
    /// Hot-reload style snapshot (`Arc<ArcSwap<…>>`); use `then_lock(DarkStruct::hot())` from a `&DarkStruct`.
    hot: Arc<arcswap::ArcSwap<String>>,
}

fn init_via_keypaths() -> SomeComplexStruct {
    let mut root = SomeComplexStruct {
        scsf: Box::new(SomeOtherStruct {
            sosf: Arc::new(arcswap::ArcSwap::from_pointee(OneMoreStruct {
                omsf: String::new(),
                omse: SomeEnum::A(String::new()),
            })),
        }),
    };

    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf_kp())
        .get_mut(&mut root)
        .map(|slot| {
            slot.store(Arc::new(OneMoreStruct {
                omsf: "omsf_value".to_string(),
                omse: SomeEnum::B(DarkStruct {
                    dsf: "dark_value".to_string(),
                    hot: Arc::new(arcswap::ArcSwap::from_pointee("hot_value".to_string())),
                }),
            }));
        });

    root
}

fn main() {
    let instance = init_via_keypaths();

    let dsf = SomeComplexStruct::scsf().then_lock(
        SomeOtherStruct::sosf()
            .then(OneMoreStruct::omse())
            .then(SomeEnum::b())
            .then(DarkStruct::dsf()),
    );
    assert_eq!(dsf.get(&instance), Some(&"dark_value".to_string()));
    assert_eq!(instance.scsf.sosf.load().omsf, "omsf_value");

    // `ArcSwap` on `DarkStruct`: derive generates `DarkStruct::hot()` as `LockKp` (same as `Arc<RwLock<…>>`).
    let inner = instance.scsf.sosf.load();
    if let SomeEnum::B(ref dark) = inner.omse {
        assert_eq!(DarkStruct::hot().get(dark), Some(&"hot_value".to_string()));
    } else {
        panic!("expected B variant");
    }
}
