use rust_keypaths::{OptionalKeyPath, KeyPath, EnumKeyPaths, variant_of};

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

struct DeeperStruct {
    desf: Option<Box<String>>
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
    
    // Create keypaths for each level of nesting
    let scsf_kp = OptionalKeyPath::new(|s: &SomeComplexStruct| s.scsf.as_ref());
    let sosf_kp = OptionalKeyPath::new(|s: &SomeOtherStruct| s.sosf.as_ref());
    let omse_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omse.as_ref());
    let dsf_kp = OptionalKeyPath::new(|d: &DarkStruct| d.dsf.as_ref());
    
    // Create enum variant keypath for SomeEnum::B
    let enum_b_kp = variant_of(|e: &SomeEnum| {
        if let SomeEnum::B(ds) = e {
            Some(ds)
        } else {
            None
        }
    });
    
    // Chain keypaths to read dsf field using enum keypath
    let chained_dsf_kp = scsf_kp
        .then(sosf_kp)
        .then(omse_kp)
        .then(enum_b_kp)
        .then(dsf_kp);
    
    // Access dsf using the chained keypath with enum variant
    if let Some(dsf_value) = chained_dsf_kp.get(&instance) {
        println!("dsf field value (chained with enum keypath): {:?}", dsf_value);
    }
    
    // Create and chain keypath for omsf field (separate instances since then consumes)
    // Chain: SomeComplexStruct -> scsf -> SomeOtherStruct -> sosf -> OneMoreStruct -> omsf
    let scsf_kp_omsf = OptionalKeyPath::new(|s: &SomeComplexStruct| s.scsf.as_ref());
    let sosf_kp_omsf = OptionalKeyPath::new(|s: &SomeOtherStruct| s.sosf.as_ref());
    let omsf_kp = OptionalKeyPath::new(|o: &OneMoreStruct| o.omsf.as_ref());
    
    // Chain the keypaths using then function
    let chained_omsf_kp = scsf_kp_omsf
        .then(sosf_kp_omsf)
        .then(omsf_kp);
    
    // Access omsf using the chained keypath
    if let Some(omsf_value) = chained_omsf_kp.get(&instance) {
        println!("omsf field value (chained keypath): {:?}", omsf_value);
    }
}

