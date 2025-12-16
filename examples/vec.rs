use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
// use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
#[All]
struct SomeComplexStruct {
    scsf: Vec<SomeOtherStruct>,
}

// impl SomeComplexStruct {
//     fn scsf_fr() -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
//         OptionalKeyPath::new(
//             |root: & SomeComplexStruct|
//             {
//                 root.scsf.first()
//             }
//         )
//     }

//     fn scsf_fr_at(index: &'static usize) -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
//         OptionalKeyPath::new(
//             |root: & SomeComplexStruct|
//             {
//                 root.scsf.get(*index)
//             }
//         )
//     }

//     fn scsf_fw() -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
//         WritableOptionalKeyPath::new(
//             |root: &mut SomeComplexStruct|
//             {
//                 root.scsf.first_mut()
//             }
//         )
//     }

//     fn scsf_fw_at(index: usize) -> KeyPaths<SomeComplexStruct, SomeOtherStruct> {
//         WritableOptionalKeyPath::new(
//             move |root: &mut SomeComplexStruct|
//             {
//                 root.scsf.get_mut(index)
//             }
//         )
//     }

// }
impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: vec![
                SomeOtherStruct {
                    sosf: OneMoreStruct {
                        omsf: String::from("no value for now"),
                        omse: SomeEnum::B(DarkStruct {
                            dsf: String::from("dark field"),
                        }),
                    },
                },
                SomeOtherStruct {
                    sosf: OneMoreStruct {
                        omsf: String::from("no value for now"),
                        omse: SomeEnum::B(DarkStruct {
                            dsf: String::from("dark field"),
                        }),
                    },
                },
            ],
        }
    }
}

#[derive(Debug, Keypaths)]
#[All]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

#[derive(Debug, Casepaths)]
#[All]
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
    // let x = ;
    let op = SomeComplexStruct::scsf_fw_at(1)
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw());
    let mut instance = SomeComplexStruct::new();
    if let Some(omsf) = op.get_mut(&mut instance) {
        *omsf = String::from("we can change the field with the other way unlocked by keypaths");
    }
    println!("instance = {:?}", instance);

    let op = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw());
    let mut instance = SomeComplexStruct::new();
    if let Some(omsf) = op.get_mut(&mut instance) {
        *omsf = String::from("we can change the field with the other way unlocked by keypaths");
    }
    println!("instance = {:?}", instance);
}
