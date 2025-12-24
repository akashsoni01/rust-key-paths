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
    
    println!("=== KeyPath Display and Debug Examples ===\n");
    
    // Note: These fields are NOT Option types, so we use _w() methods, not _fw()
    // For Box<T>, we manually create a keypath that unwraps the Box
    // For enum variants, we use _case_fw() which returns WritableOptionalKeyPath
    
    // Build a long chain step by step, showing Display/Debug at each stage
    println!("--- Step 1: Start with Box field ---");
    let step1 = SomeComplexStruct::scsf_fw();
    println!("  Display: {}", step1);
    println!("  Debug: {:?}\n", step1);
    
    println!("--- Step 2: Chain to SomeOtherStruct field ---");
    let step2 = step1.then(SomeOtherStruct::sosf_fw());
    println!("  Display: {}", step2);
    println!("  Debug: {:?}\n", step2);
    
    println!("--- Step 3: Chain to OneMoreStruct enum field ---");
    let step3 = step2.then(OneMoreStruct::omse_fw());
    println!("  Display: {}", step3);
    println!("  Debug: {:?}\n", step3);
    
    println!("--- Step 4: Chain to SomeEnum::B variant ---");
    let step4 = step3.then(SomeEnum::b_case_fw());
    println!("  Display: {}", step4);
    println!("  Debug: {:?}\n", step4);
    
    println!("--- Step 5: Final chain to DarkStruct field ---");
    let final_path = step4.then(DarkStruct::dsf_fw());
    println!("  Display: {}", final_path);
    println!("  Debug: {:?}\n", final_path);
    
    // Test with different initialization scenarios
    println!("=== Testing with Different Initializations ===\n");
    
    // Scenario 1: Normal case - all fields present (Some)
    println!("--- Scenario 1: All fields present (Some) ---");
    let mut instance1 = SomeComplexStruct::new();
    println!("  Instance: {:?}", instance1);
    println!("  Final path Display: {}", final_path);
    if let Some(value) = final_path.get_mut(&mut instance1) {
        println!("  Result: Some({:?})", value);
        *value = String::from("changed via keypath chain");
        println!("  After change: {:?}\n", instance1);
    }
    
    // Scenario 2: Enum variant mismatch - returns None
    println!("--- Scenario 2: Wrong enum variant (None) ---");
    let mut instance2 = SomeComplexStruct {
        scsf: Box::new(SomeOtherStruct {
            sosf: OneMoreStruct {
                omsf: String::from("test"),
                omse: SomeEnum::A(String::from("wrong variant")), // A instead of B
            },
        }),
    };
    println!("  Instance: {:?}", instance2);
    println!("  Final path Display: {}", final_path);
    match final_path.get_mut(&mut instance2) {
        Some(value) => println!("  Result: Some({:?})", value),
        None => println!("  Result: None (enum variant mismatch)\n"),
    }
    
    // Scenario 3: Show intermediate None in chain
    // Rebuild keypaths for checking intermediate steps
    println!("--- Scenario 3: Intermediate None in chain ---");
    let mut instance3 = SomeComplexStruct {
        scsf: Box::new(SomeOtherStruct {
            sosf: OneMoreStruct {
                omsf: String::from("test"),
                omse: SomeEnum::A(String::from("variant A")),
            },
        }),
    };
    
    // Check each step in the chain (rebuild keypaths for each check)
    println!("  Checking step 1 (Box field):");
    let check_step1 = SomeComplexStruct::scsf_fw();
    println!("    KeyPath Display: {}", check_step1);
    match check_step1.get_mut(&mut instance3) {
        Some(_) => println!("    ✓ Some - Box field exists"),
        None => println!("    ✗ None - Box field missing"),
    }
    
    println!("  Checking step 2 (SomeOtherStruct field):");
    let check_step2 = SomeComplexStruct::scsf_fw().then(SomeOtherStruct::sosf_fw());
    println!("    KeyPath Display: {}", check_step2);
    match check_step2.get_mut(&mut instance3) {
        Some(_) => println!("    ✓ Some - Field exists"),
        None => println!("    ✗ None - Field missing"),
    }
    
    println!("  Checking step 3 (OneMoreStruct enum field):");
    let check_step3 = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw());
    println!("    KeyPath Display: {}", check_step3);
    match check_step3.get_mut(&mut instance3) {
        Some(_) => println!("    ✓ Some - Enum field exists"),
        None => println!("    ✗ None - Enum field missing"),
    }
    
    println!("  Checking step 4 (SomeEnum::B variant):");
    let check_step4 = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_fw());
    println!("    KeyPath Display: {}", check_step4);
    println!("    KeyPath Debug: {:?}", check_step4);
    println!("    KeyPath Debug (detailed):");
    println!("    {:#?}", check_step4);
    match check_step4.trace_chain(&mut instance3) {
        Ok(()) => println!("    ✓ Some - Variant B exists"),
        Err(msg) => println!("    ✗ Chain broken: {}", msg),
    }
    println!();
    
    println!("  Checking final path:");
    let check_final = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::b_case_fw())
        .then(DarkStruct::dsf_fw());
    println!("    KeyPath Display: {}", check_final);
    println!("    KeyPath Debug: {:?}", check_final);
    println!("    KeyPath Debug (detailed):");
    println!("    {:#?}", check_final);
    match check_final.trace_chain(&mut instance3) {
        Ok(()) => println!("    ✓ Some - Full chain successful"),
        Err(msg) => println!("    ✗ Chain broken: {}", msg),
    }
    println!();
    
    // Scenario 4: Create alternative chain for variant A
    println!("--- Scenario 4: Alternative chain for SomeEnum::A ---");
    let alt_path = SomeComplexStruct::scsf_fw()
        .then(SomeOtherStruct::sosf_fw())
        .then(OneMoreStruct::omse_fw())
        .then(SomeEnum::a_case_fw());
    println!("  Alternative path Display: {}", alt_path);
    println!("  Alternative path Debug: {:?}", alt_path);
    match alt_path.get_mut(&mut instance3) {
        Some(value) => {
            println!("  Result: Some({:?})", value);
            *value = String::from("changed via alternative path");
            println!("  After change: {:?}\n", instance3);
        },
        None => println!("  Result: None\n"),
    }
    
    println!("=== Summary ===");
    println!("- Display shows: KeyPath type and type information");
    println!("- Debug shows: Same as Display for consistent formatting");
    println!("- Long chains: Each step can be inspected independently");
    println!("- None cases: Display still works, shows the full chain structure");
    println!("- Some cases: Display shows the complete path that succeeded");
}
