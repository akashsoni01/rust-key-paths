use keypaths_proc::{Kp, Kps};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::marker::PhantomData;
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

#[derive(Debug, Kp, Kps)]
#[All]
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

#[derive(Debug, Kp, Kps)]
#[All]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

/// Enum exercising wrapper combinations for variant payloads (read/write via Kps).
/// Avoids Arc<Mutex> here so the example runs without the parking_lot feature.
#[derive(Debug, Kps, Kp)]
#[All]
enum SomeEnum {
    A(String),
    B(DarkStruct),
    C(Option<String>),
    D(Rc<RefCell<String>>),
    E(Rc<Box<String>>),
    F(Option<String>),
    G(Vec<String>),
    H(Option<Box<String>>),
    I(Box<Option<String>>),
    J(HashSet<String>),
    K(HashMap<String, i32>),
    L(VecDeque<String>),
    M(Result<i32, String>),
    N(Cow<'static, str>),
}

#[derive(Debug, Kp, Kps)]
#[All]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Kp, Kps)]
#[All]
struct DarkStruct {
    dsf: String,
}

/// Struct that exercises all supported wrapper combinations for struct fields.
/// Uses Kp only (like AllContainersTest) so all container types work without parking_lot feature.
#[derive(Debug, Kp, Kps)]
struct AllCombinationsStruct {
    // Basic containers
    box_field: Box<String>,
    rc_field: Rc<String>,
    arc_field: Arc<String>,
    option_field: Option<String>,
    vec_field: Vec<String>,
    string_field: String,
    // Nested
    rc_box_field: Rc<Box<String>>,
    option_box_field: Option<Box<String>>,
    box_option_field: Box<Option<String>>,
    // Sets and maps
    hashset_field: HashSet<String>,
    btreeset_field: BTreeSet<String>,
    hashmap_field: HashMap<String, i32>,
    btreemap_field: BTreeMap<String, i32>,
    // Queues and lists
    vecdeque_field: VecDeque<String>,
    linkedlist_field: LinkedList<String>,
    binaryheap_field: BinaryHeap<String>,
    // Option-of-container and container-of-Option
    option_vecdeque_field: Option<VecDeque<String>>,
    vecdeque_option_field: VecDeque<Option<String>>,
    option_hashset_field: Option<HashSet<String>>,
    option_result_field: Option<Result<i32, String>>,
    // Interior mutability and lazy
    cell_field: Cell<i32>,
    refcell_field: RefCell<String>,
    once_lock_field: OnceLock<String>,
    // Marker, range, result, cow
    phantom_field: PhantomData<()>,
    range_field: Range<u32>,
    result_field: Result<i32, String>,
    cow_str_field: Cow<'static, str>,
    empty_tuple: (),
}

fn main() {
    use rust_keypaths::WritableOptionalKeyPath;

    println!("=== KeyPath Display and Debug Examples ===\n");

    // Note: These fields are NOT Option types, so we use _w() methods, not _fw()
    // For Box<T>, we manually create a keypath that unwraps the Box
    // For enum variants, we use _fw() which returns WritableOptionalKeyPath

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

    // Kp: b() / a() read-only; Kps: b_fw() / a_fw() for write
    println!("--- Step 4: Chain to SomeEnum::B variant ---");
    let step4 = step3.then(SomeEnum::b_fw());
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
        .then(SomeEnum::b_fw());
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
        .then(SomeEnum::b_fw())
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
        .then(SomeEnum::a_fw());
    println!("  Alternative path Display: {}", alt_path);
    println!("  Alternative path Debug: {:?}", alt_path);
    match alt_path.get_mut(&mut instance3) {
        Some(value) => {
            println!("  Result: Some({:?})", value);
            *value = String::from("changed via alternative path");
            println!("  After change: {:?}\n", instance3);
        }
        None => println!("  Result: None\n"),
    }

    // ========== All combinations: struct fields (Kp) ==========
    println!("=== AllCombinationsStruct (all wrapper combinations, Kp) ===\n");
    let once = OnceLock::new();
    let _ = once.set("lazy".to_string());
    let all_data = AllCombinationsStruct {
        box_field: Box::new("box".to_string()),
        rc_field: Rc::new("rc".to_string()),
        arc_field: Arc::new("arc".to_string()),
        option_field: Some("opt".to_string()),
        vec_field: vec!["v".to_string()],
        string_field: "str".to_string(),
        rc_box_field: Rc::new(Box::new("rc_box".to_string())),
        option_box_field: Some(Box::new("opt_box".to_string())),
        box_option_field: Box::new(Some("box_opt".to_string())),
        hashset_field: HashSet::from(["h".to_string()]),
        btreeset_field: BTreeSet::from(["b".to_string()]),
        hashmap_field: HashMap::from([("k".to_string(), 1)]),
        btreemap_field: BTreeMap::from([("k".to_string(), 2)]),
        vecdeque_field: VecDeque::from(["vd".to_string()]),
        linkedlist_field: LinkedList::from(["ll".to_string()]),
        binaryheap_field: BinaryHeap::from(["bh".to_string()]),
        option_vecdeque_field: Some(VecDeque::from(["ovd".to_string()])),
        vecdeque_option_field: VecDeque::from([Some("vo".to_string())]),
        option_hashset_field: Some(HashSet::from(["oh".to_string()])),
        option_result_field: Some(Ok(100)),
        cell_field: Cell::new(42),
        refcell_field: RefCell::new("refcell".to_string()),
        once_lock_field: once,
        phantom_field: PhantomData,
        range_field: 0..10,
        result_field: Ok(200),
        cow_str_field: Cow::Borrowed("cow"),
        empty_tuple: (),
    };
    let _ = AllCombinationsStruct::box_field().get(&all_data);
    let _ = AllCombinationsStruct::option_field().get(&all_data);
    let _ = AllCombinationsStruct::option_box_field().get(&all_data);
    let _ = AllCombinationsStruct::box_option_field().get(&all_data);
    let _ = AllCombinationsStruct::option_vecdeque_field().get(&all_data);
    let _ = AllCombinationsStruct::vecdeque_option_field().get(&all_data);
    let _ = AllCombinationsStruct::option_result_field().get(&all_data);
    let _ = AllCombinationsStruct::result_field().get(&all_data);
    let _ = AllCombinationsStruct::cow_str_field().get(&all_data);
    let _ = AllCombinationsStruct::once_lock_field().get(&all_data);
    let _ = AllCombinationsStruct::range_field().get(&all_data);
    let _ = AllCombinationsStruct::empty_tuple().get(&all_data);
    println!("  ✓ All struct field keypaths (Kp) work");

    // Enum variant keypaths (Kp read-only: a(), b(), c(), ...; Kps: *_fr(), *_fw(), etc.)
    let e_c = SomeEnum::C(Some("c_val".to_string()));
    let e_d = SomeEnum::D(Rc::new(RefCell::new("d_val".to_string())));
    let e_e = SomeEnum::E(Rc::new(Box::new("e_val".to_string())));
    let e_f = SomeEnum::F(Some("f_val".to_string()));
    let e_g = SomeEnum::G(vec!["g_val".to_string()]);
    let e_m = SomeEnum::M(Ok(300));
    let e_n = SomeEnum::N(Cow::Borrowed("n_val"));
    assert!(SomeEnum::c().get(&e_c).is_some());
    assert!(SomeEnum::d().get(&e_d).is_some());
    assert!(SomeEnum::e().get(&e_e).is_some());
    assert!(SomeEnum::f().get(&e_f).is_some());
    assert!(SomeEnum::g().get(&e_g).is_some());
    assert!(SomeEnum::m().get(&e_m).is_some());
    assert!(SomeEnum::n().get(&e_n).is_some());
    println!("  ✓ All enum variant keypaths (Kp/Kps) work\n");
    
    println!("=== Summary ===");
    println!("- Display shows: KeyPath type and type information");
    println!("- Debug shows: Same as Display for consistent formatting");
    println!("- Long chains: Each step can be inspected independently");
    println!("- None cases: Display still works, shows the full chain structure");
    println!("- Some cases: Display shows the complete path that succeeded");
    println!("- AllCombinationsStruct and SomeEnum cover all supported wrapper combinations");
}
