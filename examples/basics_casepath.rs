use parking_lot::RwLock;
use std::sync::Arc;
use key_paths_derive::Kp;

#[derive(Debug, Kp)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
    scfs2: Arc<RwLock<SomeOtherStruct>>,
}

#[derive(Debug, Kp)]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Kp)]
enum SomeEnum {
    A(String),
    B(Box<DarkStruct>),
}

#[derive(Debug, Kp)]
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
    let mut instance = SomeComplexStruct::new();

    {
        // For Option fields, use () methods; chain with .then() for nested access
        let scsf_kp = SomeComplexStruct::scsf();
        let sosf_kp = scsf_kp.then(SomeOtherStruct::sosf());
        let omse_kp = sosf_kp.then(OneMoreStruct::omse());
        let b_kp = omse_kp.then(SomeEnum::b());
        let dsf_kp = b_kp.then(DarkStruct::dsf());

        if let Some(omsf) = dsf_kp.get_mut(&mut instance) {
            *omsf = String::from("This is changed 🖖🏿");
        }
    } // keypaths dropped here, borrow released

    println!("instance = {:?}", instance);
}
