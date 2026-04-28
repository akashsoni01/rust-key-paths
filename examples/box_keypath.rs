use key_paths_derive::Kp;
use rust_key_paths::KpTrait;

#[derive(Debug, Kp, Default, Clone)]
struct SomeComplexStruct {
    scsf: Box<SomeOtherStruct>,
}

#[derive(Debug, Kp, Default, Clone)]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
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

/// Build a fully initialized `SomeComplexStruct` by mutating via keypaths only
/// (no direct struct literals for nested data — tests keypath traversal through Box).
fn init_via_keypaths() -> SomeComplexStruct {
    let mut root = SomeComplexStruct::default();

    // kp: root -> scsf (Box) -> sosf -> omsf (String)
    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omsf())
        .get_mut(&mut root)
        .map(|s| *s = "omsf_value".to_string());

    // kp: root -> scsf -> sosf -> omse (SomeEnum); set variant to B
    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .get_mut(&mut root)
        .map(|e| *e = SomeEnum::B(DarkStruct::default()));

    // kp: root -> scsf -> sosf -> omse -> B payload -> dsf
    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf())
        .get_mut(&mut root)
        .map(|s| *s = "dark_value".to_string());

    root
}

fn main() {
    let mut instance = init_via_keypaths();

    // Read back via same keypaths to verify
    let omsf = SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omsf())
        .get(&instance);
    assert_eq!(omsf, Some(&"omsf_value".to_string()));

    let kp = SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf());

        println!("size of kp = {:?}", size_of_val(&kp));
    let dsf = SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf())
        .get(&instance);
    assert_eq!(dsf, Some(&"dark_value".to_string()));

    if let Some(val) = kp.get_mut_ref(&mut instance) {
        *val = "changed".to_string()
    }

    if let Some(val) = kp.get_mut_ref(&mut instance) {
                *val = "changed".to_string()

    }

    if let Some(val) = kp.get_mut_ref(&mut instance) {
                *val = "changed3".to_string()

    }

        println!("size of kp = {:?}", size_of_val(&kp));

    /*
            Kp - struct 8 genric
            KpType - typealias 2 genric
            KpTrait - to enforce use to use kp
            KpTraitType - 2 gen
            Accessor - acccessor fns into a triat - get, get_mut
            OptionalAccessor - get_optionl, get_mut_optional
            HofExt - map, filter, flatmap .....
            Chain - then
            CoercionTrait - to_box, to_arc
            ChainExt - then_async, then_pin_fut, then_sync
    */
    println!("{:?}", instance);
}
