use std::rc::Rc;

use key_paths_derive::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
struct SomeComplexStruct {
    scsf: Rc<SomeOtherStruct>,

}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Rc::new(SomeOtherStruct {
                sosf: OneMoreStruct {
                    omsf: String::from("no value for now"),
                    omse: SomeEnum::B(DarkStruct { dsf: String::from("dark field") }),
                },
            }),
        }
    }
}

#[derive(Debug, Keypaths)]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

#[derive(Debug, Casepaths)]
enum SomeEnum {
    A(String), 
    B(DarkStruct)
}

#[derive(Debug, Keypaths)]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum
}

#[derive(Debug, Keypaths)]
struct DarkStruct {
    dsf: String
}

fn main() {
    let op = SomeComplexStruct::scsf_fr()
        .then(SomeOtherStruct::sosf_fr())
        .then(OneMoreStruct::omse_fr())
        .then(SomeEnum::b_case_r())
        .then(DarkStruct::dsf_fr());
    let mut instance = SomeComplexStruct::new();
    if let Some(omsf) = op.get( &instance) {
    // *omsf =
    //     String::from("we can change the field with the other way unclocked by keypaths");
        println!("instance = {:?}", instance);
    }

}
