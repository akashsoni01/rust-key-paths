// Test cases for reference keypath support
// Run with: cargo run --example reference_test

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u32,
}

fn main() {
    println!("=== Reference KeyPath Tests ===\n");

    let people = vec![
        Person {
            name: "Akash".to_string(),
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

    // Test 1: Basic get with readable keypath
    println!("--- Test 1: get with Readable KeyPath ---");
    let name_path = KeyPath::new(|p: &Person| &p.name);

    let person_refs: Vec<&Person> = people.iter().collect();
    for person_ref in &person_refs {
        let name = name_path.get(person_ref);
        println!("  Name: {}", name);
        assert!(!name.is_empty(), "Name should not be empty");
    }
    println!("✓ Test 1 passed\n");

    // Test 2: get returns correct values
    println!("--- Test 2: get Value Correctness ---");
    let age_path = KeyPath::new(|p: &Person| &p.age);

    let first_ref = &people[0];
    let age = age_path.get(first_ref);
    println!("  First person age: {}", age);
    assert_eq!(*age, 30, "Age should be 30");
    println!("✓ Test 2 passed\n");

    // Test 3: get with nested references
    println!("--- Test 3: Nested References ---");
    let refs_of_refs: Vec<&&Person> = person_refs.iter().collect();
    for ref_ref in &refs_of_refs {
        // Need to deref once to get &Person, then use get
        let name = name_path.get(*ref_ref);
        println!("  Nested ref name: {}", name);
    }
    println!("✓ Test 3 passed\n");

    // Test 4: Writable keypaths don't have get() method
    println!("--- Test 4: Writable KeyPath (no get method) ---");
    let name_path_w = WritableKeyPath::new(|p: &mut Person| &mut p.name);

    // WritableKeyPath doesn't have get(), only get_mut()
    // This test demonstrates that writable paths are for mutation only
    println!("  WritableKeyPath only has get_mut(), not get()");
    println!("✓ Test 4 passed\n");

    // Test 5: get_mut with mutable references
    println!("--- Test 5: get_mut with Mutable References ---");
    let mut people_mut = people.clone();
    let name_path_w = WritableKeyPath::new(|p: &mut Person| &mut p.name);

    let person_mut_ref = &mut people_mut[0];
    let name = name_path_w.get_mut(person_mut_ref);
    println!("  Original name: {}", name);
    *name = "Akash Smith".to_string();
    println!("  Modified name: {}", name);
    assert_eq!(name, "Akash Smith");
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
            name: "Akash".to_string(),
            manager: Some("Bob".to_string()),
        },
        Employee {
            name: "Charlie".to_string(),
            manager: None,
        },
    ];

    let manager_path = OptionalKeyPath::new(|e: &Employee| e.manager.as_ref());
    let employee_refs: Vec<&Employee> = employees.iter().collect();

    for emp_ref in &employee_refs {
        match manager_path.get(emp_ref) {
            Some(manager) => println!("  {} has manager: {}", emp_ref.name, manager),
            None => println!("  {} has no manager", emp_ref.name),
        }
    }
    println!("✓ Test 6 passed\n");

    // Test 7: Comparison between get with different references
    println!("--- Test 7: get with Different References ---");
    let owned_person = &people[0];
    let ref_person = &people[0];

    // Using get with direct reference
    let name1 = name_path.get(owned_person);
    println!("  get() result: {}", name1);

    // Using get with another reference
    let name2 = name_path.get(ref_person);
    println!("  get() result: {}", name2);
    assert_eq!(name1, name2, "Both should return the same value");
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
        let age = age_path.get(person_ref);
        if *age > 40 {
            count += 1;
        }
    }
    println!("  Found {} people over 40 (using references)", count);
    println!("✓ Test 8 passed\n");

    println!("=== All Tests Passed! ===");
}
