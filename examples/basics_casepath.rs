use key_paths_derive::{Casepaths, Keypaths};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Keypaths)]
struct SomeComplexStruct {
    #[Writable]
    scsf: Option<SomeOtherStruct>,
    #[Writable]
    scfs2: Arc<RwLock<SomeOtherStruct>>,
}

#[derive(Debug, Keypaths)]
struct SomeOtherStruct {
    #[Writable]
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Casepaths)]
enum SomeEnum {
    A(String),
    B(Box<DarkStruct>),
}

#[derive(Debug, Keypaths)]
struct OneMoreStruct {
    #[Writable]
    omsf: Option<String>,
    #[Writable]
    omse: Option<SomeEnum>,
}

#[derive(Debug, Keypaths)]
struct DarkStruct {
    #[Writable]
    dsf: Option<String>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(Box::new(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    }))),
                }),
            }),
            scfs2: Arc::new(RwLock::new(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(Box::new(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    }))),
                }),
            })),
        }
    }
}
fn main() {
    let dsf_kp = SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b_case())
        .then(DarkStruct::dsf().for_box());

    let mut instance = SomeComplexStruct::new();
    // let omsf = dsf_kp.get_mut(&mut instance);
    // *omsf.unwrap() =
    //     String::from("we can change the field with the other way unlocked by keypaths");
    // println!("instance = {:?}", instance);
    if let Some(omsf) = dsf_kp.get_mut(&mut instance) {
        *omsf = String::from("This is changed 🖖🏿");
        println!("instance = {:?}", instance);
    }
}
