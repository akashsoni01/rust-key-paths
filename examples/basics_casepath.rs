use key_paths_derive::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
}


#[derive(Debug, Keypaths)]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug, Casepaths)]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

#[derive(Debug, Keypaths)]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug, Keypaths)]
struct DarkStruct {
    dsf: Option<String>,
}


impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Some(SomeOtherStruct {
                sosf: Some(OneMoreStruct {
                    omsf: Some(String::from("no value for now")),
                    omse: Some(SomeEnum::B(DarkStruct {
                        dsf: Some(String::from("dark field")),
                    })),
                }),
            }),
        }
    }
}
fn main() {
    let dsf_kp = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_w())
        .then(DarkStruct::dsf_fw());

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
