use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
    // scsf2: Option<SomeOtherStruct>,
}

impl SomeComplexStruct {
    // read only keypath = field_name_r
    // fn r() -> KeyPaths<SomeComplexStruct, SomeOtherStruct>{
    //     KeyPaths::readable(get)
    // }

    // write only keypath = field_name_w
    // fn w() -> KeyPaths<>{}

    // failable read only keypath = field_name_fr
    fn scsf_fr() -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
        KeyPaths::failable_readable(|root: &SomeComplexStruct| root.scsf.as_ref())
    }

    // failable writeable keypath = field_name_fw
    fn scsf_fw() -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
        KeyPaths::failable_writable(|root: &mut SomeComplexStruct| root.scsf.as_mut())
    }
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: OneMoreStruct {
                    omsf: String::from("no value for now"),
                },
            }),
        }
    }
}

#[derive(Debug, Keypaths)]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

#[derive(Debug, Keypaths)]
struct OneMoreStruct {
    omsf: String,
}

fn main() {
    // imparitive way
    // let mut instance = SomeComplexStruct::new();
    // if let  Some(inner_filed) = &mut instance.scsf {
    //     let inner_most_field = &mut inner_filed.sosf.omsf;
    //     *inner_most_field = String::from("we can change the field with the imparitive");
    // }
    // println!("instance = {:?}", instance);

    // the other way
    // SomeComplexStruct -> SomeOtherStruct -> OneMoreStruct -> omsf

    // let scsfp: KeyPaths<SomeComplexStruct, SomeOtherStruct> = SomeComplexStruct::scsf_fw();
    // let sosfp: key_paths_core::KeyPaths<SomeOtherStruct, OneMoreStruct> =
    //     SomeOtherStruct::sosf_fw();
    // let omsfp: key_paths_core::KeyPaths<OneMoreStruct, String> = OneMoreStruct::omsf_fw();
    // let op: KeyPaths<SomeComplexStruct, String> = scsfp.then(sosfp).then(omsfp);
    // let mut instance = SomeComplexStruct::new();
    // let omsf = op.get_mut(&mut instance);
    // *omsf.unwrap() =
    //     String::from("we can change the field with the other way unlocked by keypaths");
    // println!("instance = {:?}", instance);

    // syntictic suger to do what we just do with other way
    // SomeComplexStruct -> SomeOtherStruct -> OneMoreStruct -> omsf

    let op = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omsf_fw());
    let mut instance = SomeComplexStruct::new();
    let omsf = op.get_mut(&mut instance);
    *omsf.unwrap() =
        String::from("we can change the field with the other way unlocked by keypaths");
    println!("instance = {:?}", instance);
}
