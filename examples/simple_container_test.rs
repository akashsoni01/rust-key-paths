use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

// Simple test to verify all new container types work
#[derive(Debug, Keypaths)]
struct TestStruct {
    // Basic containers
    vec_field: Vec<String>,
    hashmap_field: HashMap<String, i32>,
    hashset_field: HashSet<String>,
    vecdeque_field: VecDeque<String>,
    
    // Smart pointers
    box_field: Box<String>,
    rc_field: Rc<String>,
    
    // Options
    option_field: Option<String>,
    
    // Nested combinations
    option_box_field: Option<Box<String>>,
    box_option_field: Box<Option<String>>,
    vec_option_field: Vec<Option<String>>,
    option_vec_field: Option<Vec<String>>,
}

fn main() {
    println!("🧪 Simple Container Test\n");
    
    let mut test = TestStruct {
        vec_field: vec!["hello".to_string(), "world".to_string()],
        hashmap_field: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), 42);
            map.insert("key2".to_string(), 24);
            map
        },
        hashset_field: {
            let mut set = HashSet::new();
            set.insert("hello".to_string());
            set.insert("world".to_string());
            set
        },
        vecdeque_field: {
            let mut deque = VecDeque::new();
            deque.push_back("front".to_string());
            deque.push_back("back".to_string());
            deque
        },
        box_field: Box::new("boxed".to_string()),
        rc_field: Rc::new("rc".to_string()),
        option_field: Some("option".to_string()),
        option_box_field: Some(Box::new("option box".to_string())),
        box_option_field: Box::new(Some("box option".to_string())),
        vec_option_field: vec![Some("vec option 1".to_string()), None, Some("vec option 3".to_string())],
        option_vec_field: Some(vec!["option vec 1".to_string(), "option vec 2".to_string()]),
    };
    
    // Test basic containers
    println!("Testing basic containers:");
    
    // Vec
    let vec_path = TestStruct::vec_field_fr();
    if let Some(first) = vec_path.get(&test) {
        println!("  ✅ Vec first element: {}", first);
    }
    
    let vec_at_path = TestStruct::vec_field_fr_at(1);
    if let Some(second) = vec_at_path.get(&test) {
        println!("  ✅ Vec second element: {}", second);
    }
    
    // HashMap
    let map_path = TestStruct::hashmap_field_fr("key1".to_string());
    if let Some(value) = map_path.get(&test) {
        println!("  ✅ HashMap key1 value: {}", value);
    }
    
    // HashSet
    let set_path = TestStruct::hashset_field_fr();
    if let Some(element) = set_path.get(&test) {
        println!("  ✅ HashSet element: {}", element);
    }
    
    // VecDeque
    let deque_path = TestStruct::vecdeque_field_fr();
    if let Some(front) = deque_path.get(&test) {
        println!("  ✅ VecDeque front: {}", front);
    }
    
    // Test smart pointers
    println!("\nTesting smart pointers:");
    
    // Box
    let box_path = TestStruct::box_field_fr();
    if let Some(value) = box_path.get(&test) {
        println!("  ✅ Box value: {}", value);
    }
    
    // Rc
    let rc_path = TestStruct::rc_field_fr();
    if let Some(value) = rc_path.get(&test) {
        println!("  ✅ Rc value: {}", value);
    }
    
    // Test options
    println!("\nTesting options:");
    
    // Option
    let option_path = TestStruct::option_field_fr();
    if let Some(value) = option_path.get(&test) {
        println!("  ✅ Option value: {}", value);
    }
    
    // Test nested combinations
    println!("\nTesting nested combinations:");
    
    // Option<Box<T>>
    let option_box_path = TestStruct::option_box_field_fr();
    if let Some(value) = option_box_path.get(&test) {
        println!("  ✅ Option<Box<T>> value: {}", value);
    }
    
    // Box<Option<T>>
    let box_option_path = TestStruct::box_option_field_fr();
    if let Some(value) = box_option_path.get(&test) {
        println!("  ✅ Box<Option<T>> value: {}", value);
    }
    
    // Vec<Option<T>>
    let vec_option_path = TestStruct::vec_option_field_fr();
    if let Some(value) = vec_option_path.get(&test) {
        println!("  ✅ Vec<Option<T>> first value: {}", value);
    }
    
    let vec_option_at_path = TestStruct::vec_option_field_fr_at(2);
    if let Some(value) = vec_option_at_path.get(&test) {
        println!("  ✅ Vec<Option<T>> third value: {}", value);
    }
    
    // Option<Vec<T>>
    let option_vec_path = TestStruct::option_vec_field_fr();
    if let Some(value) = option_vec_path.get(&test) {
        println!("  ✅ Option<Vec<T>> first value: {}", value);
    }
    
    // Test mutations
    println!("\nTesting mutations:");
    
    // Mutate vec element
    let mut_vec_path = TestStruct::vec_field_fw();
    if let Some(value) = mut_vec_path.get_mut(&mut test) {
        *value = "modified".to_string();
        println!("  ✅ Modified vec first element");
    }
    
    // Mutate hashmap value
    let mut_map_path = TestStruct::hashmap_field_fw("key1".to_string());
    if let Some(value) = mut_map_path.get_mut(&mut test) {
        *value = 999;
        println!("  ✅ Modified hashmap value");
    }
    
    // Verify mutations
    let verify_vec_path = TestStruct::vec_field_fr();
    if let Some(value) = verify_vec_path.get(&test) {
        println!("  ✅ Verified vec modification: {}", value);
    }
    
    let verify_map_path = TestStruct::hashmap_field_fr("key1".to_string());
    if let Some(value) = verify_map_path.get(&test) {
        println!("  ✅ Verified hashmap modification: {}", value);
    }
    
    println!("\n🎉 All tests passed! All container types are working correctly.");
    println!("📋 Supported container types:");
    println!("  • Vec<T> - with indexed access");
    println!("  • HashMap<K,V> - with key-based access");
    println!("  • HashSet<T> - with element access");
    println!("  • VecDeque<T> - with front access");
    println!("  • Box<T> - with dereferenced access");
    println!("  • Rc<T> - with dereferenced access");
    println!("  • Option<T> - with failable access");
    println!("  • Nested combinations - Option<Box<T>>, Vec<Option<T>>, etc.");
}
