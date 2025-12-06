use std::sync::Arc;
use parking_lot::RwLock;
use key_paths_derive::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
#[All]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
    scfs2: Arc<RwLock<SomeOtherStruct>>
}

#[derive(Debug, Keypaths)]
#[All]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Casepaths)]
enum SomeEnum {
    A(String),
    B(Box<DarkStruct>),
}

#[derive(Debug, Keypaths)]
#[All]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Keypaths)]
#[All]
struct DarkStruct {
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
            scfs2: Arc::new(
                RwLock::new(
                    SomeOtherStruct {
                        sosf: Some(OneMoreStruct {
                            omsf: Some(String::from("no value for now")),
                            omse: Some(SomeEnum::B(Box::new(DarkStruct {
                                dsf: Some(String::from("dark field")),
                            }))),
                        }),
                    }
                )
            )
        }
    }
}
fn main() {
    let dsf_kp = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw().for_box());

    let mut instance = SomeComplexStruct::new();
    // let omsf = dsf_kp.get_mut(&mut instance);
    // **omsf =
    //     String::from("we can change the field with the other way unlocked by keypaths");
    // println!("instance = {:?}", instance);
    let omsf = dsf_kp.get_mut(&mut instance);
    {
        *omsf = String::from("This is changed 🖖🏿");
        println!("instance = {:?}", instance);
    
    }
}
