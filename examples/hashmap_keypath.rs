use std::collections::HashMap;

use key_paths_core::KeyPaths;
// use key_paths_core::KeyPaths;
use key_paths_derive::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
#[All]
struct SomeComplexStruct {
    scsf: HashMap<String, SomeOtherStruct>,
}

impl SomeComplexStruct {
    // fn scsf_fr() -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
    //     KeyPaths::failable_readable(
    //         |root: & SomeComplexStruct|
    //         {
    //             root.scsf.first()
    //         }
    //     )
    // }

    // fn scsf_fr_at(index:  String) -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
    //     KeyPaths::failable_readable(
    //         move |root: & SomeComplexStruct|
    //         {
    //             root.scsf.get(&index)
    //         }
    //     )
    // }

    // fn scsf_fw() -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
    //     KeyPaths::failable_writable(
    //         |root: &mut SomeComplexStruct|
    //         {
    //             root.scsf.first_mut()
    //         }
    //     )
    // }

    // fn scsf_fw_at(index: String) -> KeyPaths<SomeComplexStruct, SomeOtherStruct>
    // {
    //     KeyPaths::failable_writable(
    //         move |root: &mut SomeComplexStruct|
    //         {
    //             root.scsf.get_mut(&index)
    //         }
    //     )
    // }
}
impl SomeComplexStruct {
    fn new() -> Self {
        let mut x = HashMap::new();
        x.insert(
            "0".to_string(),
            SomeOtherStruct {
                sosf: OneMoreStruct {
                    omsf: String::from("no value for now"),
                    omse: SomeEnum::B(DarkStruct {
                        dsf: String::from("dark field"),
                    }),
                },
            },
        );

        x.insert(
            "1".to_string(),
            SomeOtherStruct {
                sosf: OneMoreStruct {
                    omsf: String::from("no value for now"),
                    omse: SomeEnum::B(DarkStruct {
                        dsf: String::from("dark field"),
                    }),
                },
            },
        );

        Self { scsf: x }
    }
}

#[derive(Debug, Keypaths)]
#[All]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

#[derive(Debug, Casepaths)]
enum SomeEnum {
    A(Vec<String>),
    B(DarkStruct),
}

#[derive(Debug, Keypaths)]
#[All]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Keypaths)]
#[All]
struct DarkStruct {
    dsf: String,
}

fn main() {
    let op = SomeComplexStruct::scsf_fw_at("1".to_string())
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw());
    let mut instance = SomeComplexStruct::new();
    let omsf = op.get_mut(&mut instance);
    *omsf.unwrap() =
        String::from("we can change the field with the other way unlocked by keypaths");
    println!("instance = {:?}", instance);

    let op: KeyPaths<SomeComplexStruct, String> = SomeComplexStruct::scsf_fw_at("0".to_string())
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw());
    let mut instance = SomeComplexStruct::new();
    let omsf = op.get_mut(&mut instance);
    *omsf.unwrap() =
        String::from("we can change the field with the other way unlocked by keypaths");
    println!("instance = {:?}", instance);
}
