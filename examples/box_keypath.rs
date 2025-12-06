use keypaths_proc::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
#[Writable]
struct SomeComplexStruct {
    scsf: Box<SomeOtherStruct>,
}

impl SomeComplexStruct {
    fn new() -> Self {
        Self {
            scsf: Box::new(SomeOtherStruct {
                sosf: OneMoreStruct {
                    omsf: String::from("no value for now"),
                    omse: SomeEnum::B(DarkStruct {
                        dsf: String::from("dark field"),
                    }),
                },
            }),
        }
    }
}

#[derive(Debug, Keypaths)]
#[Writable]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

#[derive(Debug, Casepaths)]
#[Writable]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

#[derive(Debug, Keypaths)]
#[Writable]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Keypaths)]
#[Writable]
struct DarkStruct {
    dsf: String,
}

fn main() {
    use rust_keypaths::WritableOptionalKeyPath;
    
    // Note: These fields are NOT Option types, so we use _w() methods, not _fw()
    // For Box<T>, we manually create a keypath that unwraps the Box
    // For enum variants, we use _case_fw() which returns WritableOptionalKeyPath
    
    // Manually create keypath to unwrap Box<SomeOtherStruct>
    let box_unwrap = WritableOptionalKeyPath::new(|s: &mut SomeComplexStruct| {
        Some(&mut *s.scsf)  // Dereference Box to get &mut SomeOtherStruct
    });
    
    let op = box_unwrap
        .then(SomeOtherStruct::sosf_fw())  // Convert to OptionalKeyPath for chaining
        .then(OneMoreStruct::omse_w().to_optional())  // Convert to OptionalKeyPath for chaining
        .then(SomeEnum::b_case_fw())  // Enum variant returns WritableOptionalKeyPath
        .then(DarkStruct::dsf_w().to_optional());  // Convert to OptionalKeyPath for chaining
    
    let mut instance = SomeComplexStruct::new();
    
    // get_mut() returns Option<&mut String> for WritableOptionalKeyPath
    if let Some(omsf) = op.get_mut(&mut instance) {
        *omsf = String::from("we can change the field with the other way unlocked by keypaths");
        println!("instance = {:?}", instance);
    }
}
