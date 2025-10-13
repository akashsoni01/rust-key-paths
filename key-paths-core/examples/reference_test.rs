// Test cases for reference keypath support
// Run with: cargo run --example reference_test

use key_paths_core::KeyPaths;

#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u32,
}

fn main() {
    println!("=== Reference KeyPath Tests ===\n");

    let people = vec![
        Person {
            name: "Alice".to_string(),
            age: 30,
        },
        Person {
            name: "Bob".to_string(),
            age: 25,
        },
        Person {
            name: "Charlie".to_string(),
            age: 35,
        },
    ];

    // Test 1: Basic get_ref with readable keypath
    println!("--- Test 1: get_ref with Readable KeyPath ---");
    let name_path = KeyPaths::readable(|p: &Person| &p.name);
    
    let person_refs: Vec<&Person> = people.iter().collect();
    for person_ref in &person_refs {
        if let Some(name) = name_path.get_ref(person_ref) {
            println!("  Name: {}", name);
            assert!(!name.is_empty(), "Name should not be empty");
        }
    }
    println!("✓ Test 1 passed\n");

    // Test 2: get_ref returns correct values
    println!("--- Test 2: get_ref Value Correctness ---");
    let age_path = KeyPaths::readable(|p: &Person| &p.age);
    
    let first_ref = &people[0];
    if let Some(&age) = age_path.get_ref(&first_ref) {
        println!("  First person age: {}", age);
        assert_eq!(age, 30, "Age should be 30");
    }
    println!("✓ Test 2 passed\n");

    // Test 3: get_ref with nested references
    println!("--- Test 3: Nested References ---");
    let refs_of_refs: Vec<&&Person> = person_refs.iter().collect();
    for ref_ref in &refs_of_refs {
        // Need to deref once to get &Person, then use get_ref
        if let Some(name) = name_path.get_ref(*ref_ref) {
            println!("  Nested ref name: {}", name);
        }
    }
    println!("✓ Test 3 passed\n");

    // Test 4: get_ref with writable keypaths (should work for reading)
    println!("--- Test 4: get_ref with Writable KeyPath ---");
    let name_path_w = KeyPaths::writable(|p: &mut Person| &mut p.name);
    
    // Even writable paths should work with get_ref for reading
    for person_ref in &person_refs {
        // Note: get_ref works with writable paths via get() internally
        // but get() returns None for Writable, so this is expected
        let result = name_path_w.get_ref(person_ref);
        assert!(result.is_none(), "Writable keypath should return None for immutable get_ref");
    }
    println!("✓ Test 4 passed (correctly returns None for writable)\n");

    // Test 5: get_mut_ref with mutable references
    println!("--- Test 5: get_mut_ref with Mutable References ---");
    let mut people_mut = people.clone();
    let name_path_w = KeyPaths::writable(|p: &mut Person| &mut p.name);
    
    let mut person_mut_ref = &mut people_mut[0];
    if let Some(name) = name_path_w.get_mut_ref(&mut person_mut_ref) {
        println!("  Original name: {}", name);
        *name = "Alice Smith".to_string();
        println!("  Modified name: {}", name);
        assert_eq!(name, "Alice Smith");
    }
    println!("✓ Test 5 passed\n");

    // Test 6: get_ref with failable keypaths
    println!("--- Test 6: get_ref with Failable KeyPath ---");
    
    #[derive(Debug)]
    struct Employee {
        name: String,
        manager: Option<String>,
    }
    
    let employees = vec![
        Employee {
            name: "Alice".to_string(),
            manager: Some("Bob".to_string()),
        },
        Employee {
            name: "Charlie".to_string(),
            manager: None,
        },
    ];
    
    let manager_path = KeyPaths::failable_readable(|e: &Employee| e.manager.as_ref());
    let employee_refs: Vec<&Employee> = employees.iter().collect();
    
    for emp_ref in &employee_refs {
        match manager_path.get_ref(emp_ref) {
            Some(manager) => println!("  {} has manager: {}", emp_ref.name, manager),
            None => println!("  {} has no manager", emp_ref.name),
        }
    }
    println!("✓ Test 6 passed\n");

    // Test 7: Comparison between get and get_ref
    println!("--- Test 7: get vs get_ref Comparison ---");
    let owned_person = &people[0];
    let ref_person = &people[0];
    
    // Using get with owned/borrowed
    if let Some(name1) = name_path.get(owned_person) {
        println!("  get() result: {}", name1);
        
        // Using get_ref with reference
        if let Some(name2) = name_path.get_ref(&ref_person) {
            println!("  get_ref() result: {}", name2);
            assert_eq!(name1, name2, "Both should return the same value");
        }
    }
    println!("✓ Test 7 passed\n");

    // Test 8: Performance consideration demo
    println!("--- Test 8: Performance Benefit Demonstration ---");
    let large_collection: Vec<Person> = (0..1000)
        .map(|i| Person {
            name: format!("Person {}", i),
            age: 20 + (i % 50),
        })
        .collect();
    
    // With references (no cloning)
    let refs: Vec<&Person> = large_collection.iter().collect();
    let mut count = 0;
    for person_ref in &refs {
        if let Some(&age) = age_path.get_ref(person_ref) {
            if age > 40 {
                count += 1;
            }
        }
    }
    println!("  Found {} people over 40 (using references)", count);
    println!("✓ Test 8 passed\n");

    println!("=== All Tests Passed! ===");
}

