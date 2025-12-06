use keypaths_proc::Keypaths;
use std::sync::Arc;

#[derive(Debug, Clone, Keypaths)]
struct SomeStruct {
    value: Option<String>,
}

// Example struct demonstrating all nested container combinations
#[derive(Debug, Clone, Keypaths)]
#[All]
struct NestedContainerExample {
    // Option<Box<T>>
    option_box_field: Option<Box<String>>,
    // Option<Rc<T>>
    option_rc_field: Option<std::rc::Rc<String>>,
    // Option<Arc<T>>
    option_arc_field: Option<std::sync::Arc<String>>,
    // Box<Option<T>>
    box_option_field: Box<Option<i32>>,
    // Rc<Option<T>>
    rc_option_field: std::rc::Rc<Option<i32>>,
    // Arc<Option<T>>
    arc_option_field: std::sync::Arc<Option<i32>>,
    // // Vec<Option<T>>
    // vec_option_field: Vec<Option<f64>>,
    // // Option<Vec<T>>
    // option_vec_field: Option<Vec<bool>>,
    // // HashMap<K, Option<V>>
    // hashmap_option_field: HashMap<String, Option<usize>>,
    // // Option<HashMap<K, V>>
    // option_hashmap_field: Option<HashMap<String, usize>>,
    // // Support added for even random containers not even possible in real life
    value: Option<Arc<Box<SomeStruct>>>,

}

fn main() {
    println!("=== Nested Container Options Example ===\n");

    let mut example = NestedContainerExample {
        value: Some(Arc::new(Box::new(SomeStruct { value: Some(String::from("Hello, world!")) }))),
        option_box_field: Some(Box::new("jkhkhjhk".to_string())),
        option_rc_field: Some(std::rc::Rc::new("World".to_string())),
        option_arc_field: Some(std::sync::Arc::new("Rust".to_string())),
        box_option_field: Box::new(Some(42)),
        rc_option_field: std::rc::Rc::new(Some(100)),
        arc_option_field: std::sync::Arc::new(Some(200)),
        // vec_option_field: vec![Some(3.14), None, Some(2.71)],
        // option_vec_field: Some(vec![true, false, true]),
        // hashmap_option_field: {
        //     let mut map = HashMap::new();
        //     map.insert("key1".to_string(), Some(10));
        //     map.insert("key2".to_string(), None);
        //     map.insert("key3".to_string(), Some(20));
        //     map
        // },
        // option_hashmap_field: {
        //     let mut map = HashMap::new();
        //     map.insert("a".to_string(), 1);
        //     map.insert("b".to_string(), 2);
        //     Some(map)
        // },
    };
    println!("Value");
    if let Some(value) = NestedContainerExample::value_fr().then(SomeStruct::value_fr().for_box()).get(&example) {
        // *value = String::from("changed");
        println!("   Changed value: {:?}", value);
    }

    // Test Option<Box<T>>
    println!("1. Option<Box<T>>:");
    if let Some(value) = NestedContainerExample::option_box_field_fr().get(&example) {
        println!("   Read value: {}", value);
    }
    
    if let Some(value) = NestedContainerExample::option_box_field_fw().get_mut(&mut example) {
        *value = "kjlkjljljk".to_string();
        println!("   Changed value: {}", value);
    }
    println!();

    let x = NestedContainerExample::option_rc_field_fr();
    // crate::NestedContainerExample::
    // Test Option<Rc<T>>
    println!("2. Option<Rc<T>>:");
    if let Some(value) = NestedContainerExample::option_rc_field_fr().get(&example) {
        println!("   Read value: {}", value);
    }
    println!();

    // Test Option<Arc<T>>
    println!("3. Option<Arc<T>>:");
    if let Some(value) = NestedContainerExample::option_arc_field_fr().get(&example) {
        println!("   Read value: {}", value);
    }
    println!();

    // Test Box<Option<T>>
    println!("4. Box<Option<T>>:");
    // Read the inner i32 if Some
    if let Some(value) = NestedContainerExample::box_option_field_fr().get(&example) {
        println!("   Inner value: {}", value);
    }
    
    if let Some(value) = NestedContainerExample::box_option_field_fw().get_mut(&mut example) {
        *value = 99;
        println!("   Changed inner value: {}", value);
    }
    println!();

    // Test Rc<Option<T>>
    println!("5. Rc<Option<T>>:");
    if let Some(value) = NestedContainerExample::rc_option_field_fr().get(&example) {
        println!("   Inner value: {}", value);
    }
    println!();

    // Test Arc<Option<T>>
    println!("6. Arc<Option<T>>:");
    if let Some(value) = NestedContainerExample::arc_option_field_fr().get(&example) {
        println!("   Inner value: {}", value);
    }
    println!();
    //
    // // Test Vec<Option<T>>
    // println!("7. Vec<Option<T>>:");
    // for (i, val) in example.vec_option_field.iter().enumerate() {
    //     println!("   Index {}: {:?}", i, val);
    // }
    //
    // if let Some(value) = NestedContainerExample::vec_option_field_fr().get(&example) {
    //     println!("   First value: {}", value);
    // }
    //
    // if let Some(value) = NestedContainerExample::vec_option_field_fr_at(0).get(&example) {
    //     println!("   Value at index 0: {}", value);
    // }
    //
    // if let Some(value) = NestedContainerExample::vec_option_field_fw_at(2).get_mut(&mut example) {
    //     *value = 1.41;
    //     println!("   Changed value at index 2: {}", value);
    // }
    // println!();
    //
    // // Test Option<Vec<T>>
    // println!("8. Option<Vec<T>>:");
    // if let Some(value) = NestedContainerExample::option_vec_field_fr().get(&example) {
    //     println!("   First value: {}", value);
    // }
    //
    // if let Some(value) = NestedContainerExample::option_vec_field_fw().get_mut(&mut example) {
    //     *value = false;
    //     println!("   Changed value: {}", value);
    // }
    //
    // if let Some(value) = NestedContainerExample::option_vec_field_fr_at(1).get(&example) {
    //     println!("   Value at index 1: {}", value);
    // }
    // println!();
    //
    // // Test HashMap<K, Option<V>>
    // println!("9. HashMap<K, Option<V>>:");
    // for (key, val) in example.hashmap_option_field.iter() {
    //     println!("   {}: {:?}", key, val);
    // }
    //
    // let key = "key1".to_string();
    // // fr returns the inner usize (unwrap Option inside HashMap)
    // if let Some(value) = NestedContainerExample::hashmap_option_field_fr(key.clone()).get(&example) {
    //     println!("   Value for 'key1': {}", value);
    // }
    //
    // if let Some(value) = NestedContainerExample::hashmap_option_field_fw(key).get_mut(&mut example) {
    //     *value = 50;
    //     println!("   Changed value for 'key1': {}", value);
    // }
    // println!();
    //
    // // Test Option<HashMap<K, V>>
    // println!("10. Option<HashMap<K, V>>:");
    // for (key, val) in example.option_hashmap_field.as_ref().unwrap().iter() {
    //     println!("   {}: {:?}", key, val);
    // }
    //
    // let key = "a".to_string();
    // if let Some(value) = NestedContainerExample::option_hashmap_field_fr(key.clone()).get(&example) {
    //     println!("   Value for 'a': {}", value);
    // }
    //
    // if let Some(value) = NestedContainerExample::option_hashmap_field_fw(key).get_mut(&mut example) {
    //     *value = 100;
    //     println!("   Changed value for 'a': {}", value);
    // }
    // println!();

    // Demonstrate composition
    println!("=== Composition Example ===");
    
    // Compose Option<Box<T>> with another struct field
    #[derive(Debug, Keypaths)]
    struct Outer {
        inner: Option<Box<NestedContainerExample>>,
    }
    
    let mut outer = Outer {
        inner: Some(Box::new(example.clone())),
    };
    
    // This would work when we have methods available
    // let path = Outer::inner_fr().then(NestedContainerExample::option_box_field_fr());
    println!("Composition ready for outer structures");
    
    println!("\n=== All tests completed successfully! ===");
}

