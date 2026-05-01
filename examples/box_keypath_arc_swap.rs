//! Same shape as `box_keypath`, but `SomeOtherStruct.sosf` lives in an [`arc_swap::ArcSwap`].
//!
//! Requires the `arc_swap_1_9_1` feature on `rust-key-paths`:
//!
//! ```bash
//! cargo run --example box_keypath_arc_swap --features arc_swap_1_9_1
//! ```

use std::mem::size_of_val;

use arc_swap::ArcSwap;
use key_paths_derive::Kp;
use rust_key_paths::ChainExt;

#[derive(Debug, Kp, Default)]
struct SomeComplexStruct {
    scsf: Box<SomeOtherStruct>,
}

#[derive(Debug, Kp)]
struct SomeOtherStruct {
    sosf: arc_swap::ArcSwap<OneMoreStruct>,
}

impl Default for SomeOtherStruct {
    fn default() -> Self {
        Self {
            sosf: ArcSwap::from_pointee(OneMoreStruct::default()),
        }
    }
}

impl Clone for SomeOtherStruct {
    fn clone(&self) -> Self {
        Self {
            sosf: ArcSwap::new(self.sosf.load_full()),
        }
    }
}

macro_rules! kp_to_dsf {
    () => {
        OneMoreStruct::omse()
            .then(SomeEnum::b())
            .then(DarkStruct::dsf())
    };
}

macro_rules! scsf_then_lock_sosf {
    ($tail:expr) => {
        SomeComplexStruct::scsf().then_lock::<_, OneMoreStruct, _, _, _, _, _, _, _, _, _, _, _, _>(
            SomeOtherStruct::sosf().then($tail),
        )
    };
}

#[derive(Debug, Kp, Clone)]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

impl Default for SomeEnum {
    fn default() -> Self {
        SomeEnum::A(String::new())
    }
}

#[derive(Debug, Kp, Default, Clone)]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Kp, Default, Clone)]
struct DarkStruct {
    dsf: String,
}

fn init_via_keypaths() -> SomeComplexStruct {
    let mut root = SomeComplexStruct::default();

    scsf_then_lock_sosf!(OneMoreStruct::omsf())
        .get_mut(&mut root)
        .map(|s| *s = "omsf_value".to_string());

    scsf_then_lock_sosf!(OneMoreStruct::omse())
        .get_mut(&mut root)
        .map(|e| *e = SomeEnum::B(DarkStruct::default()));

    scsf_then_lock_sosf!(kp_to_dsf!())
        .get_mut(&mut root)
        .map(|s| *s = "dark_value".to_string());

    root
}

fn main() {
    let mut instance = init_via_keypaths();

    let kp = scsf_then_lock_sosf!(kp_to_dsf!());

    println!("size of kp = {}", size_of_val(&kp));

    let omsf = scsf_then_lock_sosf!(OneMoreStruct::omsf()).get(&instance);
    assert_eq!(omsf, Some(&"omsf_value".to_string()));

    let dsf = kp.get(&instance).cloned();
    assert_eq!(dsf, Some("dark_value".to_string()));
    drop(kp);

    scsf_then_lock_sosf!(kp_to_dsf!())
        .get_mut(&mut instance)
        .map(|v| *v = "changed".to_string());

    assert_eq!(scsf_then_lock_sosf!(kp_to_dsf!()).get(&instance), Some(&"changed".to_string()));

    println!("{:?}", instance);
}
