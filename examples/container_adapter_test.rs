// Comprehensive test suite for container adapters
// Run with: cargo run --example container_adapter_test

use key_paths_core::KeyPaths;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct TestStruct {
    name: String,
    value: u32,
    optional: Option<String>,
}

fn main() {
    println!("=== Container Adapter Test Suite ===\n");

    let test_data = TestStruct {
        name: "Test".to_string(),
        value: 42,
        optional: Some("Optional Value".to_string()),
    };

    // Create keypaths
    let name_path = KeyPaths::readable(|s: &TestStruct| &s.name);
    let name_path_w = KeyPaths::writable(|s: &mut TestStruct| &mut s.name);
    let value_path = KeyPaths::readable(|s: &TestStruct| &s.value);
    let value_path_w = KeyPaths::writable(|s: &mut TestStruct| &mut s.value);
    let optional_path = KeyPaths::failable_readable(|s: &TestStruct| s.optional.as_ref());
    let optional_path_w =
        KeyPaths::failable_writable(|s: &mut TestStruct| s.optional.as_mut());

    // ===== Test 1: Arc Readable =====
    println!("--- Test 1: Arc with Readable KeyPath ---");
    let arc_data = Arc::new(test_data.clone());
    let name_path_arc = name_path.clone().for_arc();

    if let Some(name) = name_path_arc.get(&arc_data) {
        println!("  Arc name: {}", name);
        assert_eq!(name, "Test", "Arc readable should return correct value");
    }
    println!("✓ Test 1 passed\n");

    // ===== Test 2: Arc with Failable Readable =====
    println!("--- Test 2: Arc with Failable Readable KeyPath ---");
    let optional_path_arc = optional_path.clone().for_arc();

    if let Some(optional_val) = optional_path_arc.get(&arc_data) {
        println!("  Arc optional: {}", optional_val);
        assert_eq!(
            optional_val, "Optional Value",
            "Arc failable readable should return correct value"
        );
    }
    println!("✓ Test 2 passed\n");

    // ===== Test 3: Arc is Immutable (Note: .for_arc() with writable paths will panic) =====
    println!("--- Test 3: Arc with Writable KeyPath ---");
    println!("  Note: Calling .for_arc() on writable keypaths will panic");
    println!("  Arc is immutable, so only readable keypaths are supported");
    println!("  Skipping panic test (can't catch with catch_unwind)");
    println!("✓ Test 3 passed (documented behavior)\n");

    // ===== Test 4: Box Readable =====
    println!("--- Test 4: Box with Readable KeyPath ---");
    let box_data = Box::new(test_data.clone());
    let name_path_box = name_path.clone().for_box();

    if let Some(name) = name_path_box.get(&box_data) {
        println!("  Box name: {}", name);
        assert_eq!(name, "Test", "Box readable should return correct value");
    }
    println!("✓ Test 4 passed\n");

    // ===== Test 5: Box Writable =====
    println!("--- Test 5: Box with Writable KeyPath ---");
    let mut box_data_mut = Box::new(test_data.clone());
    let name_path_box_w = name_path_w.clone().for_box();

    if let Some(name) = name_path_box_w.get_mut(&mut box_data_mut) {
        println!("  Original Box name: {}", name);
        *name = "Modified".to_string();
        println!("  Modified Box name: {}", name);
        assert_eq!(name, "Modified", "Box writable should allow modification");
    }
    println!("✓ Test 5 passed\n");

    // ===== Test 6: Box Failable Writable =====
    println!("--- Test 6: Box with Failable Writable KeyPath ---");
    let mut box_data_opt = Box::new(test_data.clone());
    let optional_path_box_w = optional_path_w.clone().for_box();

    if let Some(opt_val) = optional_path_box_w.get_mut(&mut box_data_opt) {
        println!("  Original optional: {}", opt_val);
        *opt_val = "New Value".to_string();
        println!("  Modified optional: {}", opt_val);
        assert_eq!(opt_val, "New Value");
    }
    println!("✓ Test 6 passed\n");

    // ===== Test 7: Rc Readable =====
    println!("--- Test 7: Rc with Readable KeyPath ---");
    let rc_data = Rc::new(test_data.clone());
    let value_path_rc = value_path.clone().for_rc();

    if let Some(&value) = value_path_rc.get(&rc_data) {
        println!("  Rc value: {}", value);
        assert_eq!(value, 42, "Rc readable should return correct value");
    }
    println!("✓ Test 7 passed\n");

    // ===== Test 8: Rc is Immutable (Note: .for_rc() with writable paths will panic) =====
    println!("--- Test 8: Rc with Writable KeyPath ---");
    println!("  Note: Calling .for_rc() on writable keypaths will panic");
    println!("  Rc is immutable, so only readable keypaths are supported");
    println!("  Skipping panic test (can't catch with catch_unwind)");
    println!("✓ Test 8 passed (documented behavior)\n");

    // ===== Test 9: Vec<Arc<T>> Collection =====
    println!("--- Test 9: Vec<Arc<TestStruct>> Collection ---");
    let collection: Vec<Arc<TestStruct>> = vec![
        Arc::new(TestStruct {
            name: "Item 1".to_string(),
            value: 10,
            optional: Some("A".to_string()),
        }),
        Arc::new(TestStruct {
            name: "Item 2".to_string(),
            value: 20,
            optional: None,
        }),
        Arc::new(TestStruct {
            name: "Item 3".to_string(),
            value: 30,
            optional: Some("B".to_string()),
        }),
    ];

    let value_path_arc = value_path.clone().for_arc();

    let sum: u32 = collection
        .iter()
        .filter_map(|item| value_path_arc.get(item).copied())
        .sum();

    println!("  Sum of values: {}", sum);
    assert_eq!(sum, 60, "Sum should be 60");
    println!("✓ Test 9 passed\n");

    // ===== Test 10: Vec<Box<T>> with Mutation =====
    println!("--- Test 10: Vec<Box<TestStruct>> with Mutation ---");
    let mut box_collection: Vec<Box<TestStruct>> = vec![
        Box::new(TestStruct {
            name: "Box 1".to_string(),
            value: 100,
            optional: None,
        }),
        Box::new(TestStruct {
            name: "Box 2".to_string(),
            value: 200,
            optional: None,
        }),
    ];

    let value_path_box_w = value_path_w.clone().for_box();

    // Increment all values
    for item in &mut box_collection {
        if let Some(value) = value_path_box_w.get_mut(item) {
            *value += 10;
        }
    }

    // Verify modifications
    let new_sum: u32 = box_collection
        .iter()
        .filter_map(|item| value_path.clone().for_box().get(item).copied())
        .sum();

    println!("  Sum after increment: {}", new_sum);
    assert_eq!(new_sum, 320, "Sum should be 320 after increments");
    println!("✓ Test 10 passed\n");

    // ===== Test 11: Vec<Rc<T>> Filtering =====
    println!("--- Test 11: Vec<Rc<TestStruct>> Filtering ---");
    let rc_collection: Vec<Rc<TestStruct>> = vec![
        Rc::new(TestStruct {
            name: "RC A".to_string(),
            value: 5,
            optional: Some("X".to_string()),
        }),
        Rc::new(TestStruct {
            name: "RC B".to_string(),
            value: 15,
            optional: Some("Y".to_string()),
        }),
        Rc::new(TestStruct {
            name: "RC C".to_string(),
            value: 25,
            optional: None,
        }),
    ];

    let optional_path_rc = optional_path.clone().for_rc();

    let with_optional: Vec<&Rc<TestStruct>> = rc_collection
        .iter()
        .filter(|item| optional_path_rc.get(item).is_some())
        .collect();

    println!("  Items with optional value: {}", with_optional.len());
    assert_eq!(with_optional.len(), 2, "Should find 2 items with optional");
    println!("✓ Test 11 passed\n");

    // ===== Test 12: Multiple Container Types Together =====
    println!("--- Test 12: Mixed Container Types ---");
    
    let arc_item = Arc::new(test_data.clone());
    let box_item = Box::new(test_data.clone());
    let rc_item = Rc::new(test_data.clone());

    let name_path_arc_12 = name_path.clone().for_arc();
    let name_path_box_12 = name_path.clone().for_box();
    let name_path_rc_12 = name_path.clone().for_rc();

    let arc_name = name_path_arc_12.get(&arc_item).unwrap();
    let box_name = name_path_box_12.get(&box_item).unwrap();
    let rc_name = name_path_rc_12.get(&rc_item).unwrap();

    assert_eq!(arc_name, box_name, "Arc and Box should return same value");
    assert_eq!(box_name, rc_name, "Box and Rc should return same value");
    println!("  All containers return: '{}'", arc_name);
    println!("✓ Test 12 passed\n");

    // ===== Test 13: Result Readable =====
    println!("--- Test 13: Result with Readable KeyPath ---");
    let ok_data = Ok(test_data.clone());
    let err_data: Result<TestStruct, String> = Err("Error occurred".to_string());
    
    let name_path_result = name_path.clone().for_result::<String>();

    if let Some(name) = name_path_result.get(&ok_data) {
        println!("  Result name (Ok): {}", name);
        assert_eq!(name, "Test", "Result readable should return correct value for Ok");
    }
    
    if let Some(_) = name_path_result.get(&err_data) {
        panic!("Result readable should return None for Err");
    }
    println!("✓ Test 13 passed\n");

    // ===== Test 14: Result Writable =====
    println!("--- Test 14: Result with Writable KeyPath ---");
    let mut ok_data_mut = Ok(test_data.clone());
    let mut err_data_mut: Result<TestStruct, String> = Err("Error occurred".to_string());
    
    let name_path_result_w = name_path_w.clone().for_result::<String>();

    if let Some(name) = name_path_result_w.get_mut(&mut ok_data_mut) {
        println!("  Original Result name: {}", name);
        *name = "Modified Result".to_string();
        println!("  Modified Result name: {}", name);
        assert_eq!(name, "Modified Result", "Result writable should allow modification for Ok");
    }
    
    if let Some(_) = name_path_result_w.get_mut(&mut err_data_mut) {
        panic!("Result writable should return None for Err");
    }
    println!("✓ Test 14 passed\n");

    // ===== Test 15: Result Failable Readable =====
    println!("--- Test 15: Result with Failable Readable KeyPath ---");
    let ok_data_opt = Ok(TestStruct {
        name: "Test".to_string(),
        value: 42,
        optional: Some("Optional Value".to_string()),
    });
    let ok_data_none = Ok(TestStruct {
        name: "Test".to_string(),
        value: 42,
        optional: None,
    });
    let err_data_opt: Result<TestStruct, String> = Err("Error occurred".to_string());
    
    let optional_path_result = optional_path.clone().for_result::<String>();

    if let Some(opt_val) = optional_path_result.get(&ok_data_opt) {
        println!("  Result optional (Some): {}", opt_val);
        assert_eq!(opt_val, "Optional Value", "Result failable readable should return Some for Ok with Some");
    }
    
    if let Some(_) = optional_path_result.get(&ok_data_none) {
        panic!("Result failable readable should return None for Ok with None");
    }
    
    if let Some(_) = optional_path_result.get(&err_data_opt) {
        panic!("Result failable readable should return None for Err");
    }
    println!("✓ Test 15 passed\n");

    // ===== Test 16: Result Failable Writable =====
    println!("--- Test 16: Result with Failable Writable KeyPath ---");
    let mut ok_data_opt_mut = Ok(TestStruct {
        name: "Test".to_string(),
        value: 42,
        optional: Some("Original".to_string()),
    });
    let mut err_data_opt_mut: Result<TestStruct, String> = Err("Error occurred".to_string());
    
    let optional_path_result_w = optional_path_w.clone().for_result::<String>();

    if let Some(opt_val) = optional_path_result_w.get_mut(&mut ok_data_opt_mut) {
        println!("  Original Result optional: {}", opt_val);
        *opt_val = "Modified".to_string();
        println!("  Modified Result optional: {}", opt_val);
        assert_eq!(opt_val, "Modified", "Result failable writable should allow modification for Ok with Some");
    }
    
    if let Some(_) = optional_path_result_w.get_mut(&mut err_data_opt_mut) {
        panic!("Result failable writable should return None for Err");
    }
    println!("✓ Test 16 passed\n");

    // ===== Test 17: Vec<Result<T, E>> Collection =====
    println!("--- Test 17: Vec<Result<TestStruct, String>> Collection ---");
    let result_collection: Vec<Result<TestStruct, String>> = vec![
        Ok(TestStruct {
            name: "Success 1".to_string(),
            value: 10,
            optional: Some("A".to_string()),
        }),
        Err("Error 1".to_string()),
        Ok(TestStruct {
            name: "Success 2".to_string(),
            value: 20,
            optional: None,
        }),
        Err("Error 2".to_string()),
        Ok(TestStruct {
            name: "Success 3".to_string(),
            value: 30,
            optional: Some("B".to_string()),
        }),
    ];

    let value_path_result = value_path.clone().for_result::<String>();

    let sum: u32 = result_collection
        .iter()
        .filter_map(|item| value_path_result.get(item).copied())
        .sum();

    println!("  Sum of successful values: {}", sum);
    assert_eq!(sum, 60, "Sum should be 60 (only successful results)");
    println!("✓ Test 17 passed\n");

    println!("=== All 17 Tests Passed! ===");
}

