use rust_keypaths::{OptionalKeyPath, KeyPath};

#[derive(Debug)]
struct SomeComplexStruct {
    scsf: Option<SomeOtherStruct>,
}

#[derive(Debug)]
struct SomeOtherStruct {
    sosf: Option<OneMoreStruct>,
}

#[derive(Debug)]
struct OneMoreStruct {
    omsf: Option<String>,
    omse: Option<SomeEnum>,
}

#[derive(Debug)]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

#[derive(Debug)]
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
    let instance = SomeComplexStruct::new();
    
    // Manual keypath from current library to read dsf field
    // Create keypaths for each level of nesting
    let scsf_kp = OptionalKeyPath::new(|s: &SomeComplexStruct| s.scsf.as_ref());
    let sosf_kp = OptionalKeyPath::new(|s: &SomeOtherStruct| s.sosf.as_ref());
    let omse_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omse.as_ref());
    let dsf_kp = OptionalKeyPath::new(|d: &DarkStruct| d.dsf.as_ref());
    
    // Manually navigate through the nested structure to read dsf
    if let Some(sos) = scsf_kp.get(&instance) {
        if let Some(oms) = sosf_kp.get(sos) {
            if let Some(enum_val) = omse_kp.get(oms) {
                // Match enum variant to extract DarkStruct
                if let SomeEnum::B(dark_struct) = enum_val {
                    if let Some(dsf_value) = dsf_kp.get(dark_struct) {
                        println!("dsf field value (manual keypath): {:?}", dsf_value);
                    }
                }
            }
        }
    }
    
    // Create and chain keypath for omsf field using appending
    // Chain: SomeComplexStruct -> scsf -> SomeOtherStruct -> sosf -> OneMoreStruct -> omsf
    let omsf_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omsf.as_ref());
    
    // Chain the keypaths using appending function
    let chained_omsf_kp = scsf_kp
        .then(sosf_kp)
        .then(omsf_kp);
    
    // Access omsf using the chained keypath
    if let Some(omsf_value) = chained_omsf_kp.get(&instance) {
        println!("omsf field value (chained keypath): {:?}", omsf_value);
    }
}

