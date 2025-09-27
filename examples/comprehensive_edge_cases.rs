use key_paths_derive::Keypaths;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, LinkedList, BinaryHeap};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Keypaths)]
struct ComprehensiveTest {
    // Basic types
    string_field: String,
    int_field: i32,
    bool_field: bool,
    
    // Basic containers - these should work
    option_string: Option<String>,
    vec_string: Vec<String>,
    box_string: Box<String>,
    rc_string: Rc<String>,
    arc_string: Arc<String>,
    
    // Collections - these should work
    hashset_string: HashSet<String>,
    btreeset_string: BTreeSet<String>,
    vecdeque_string: VecDeque<String>,
    linkedlist_string: LinkedList<String>,
    binaryheap_string: BinaryHeap<String>,
    
    // Maps - these should work
    hashmap_string_int: HashMap<String, i32>,
    btreemap_string_int: BTreeMap<String, i32>,
    
    // Nested combinations - these might have issues
    // option_box_string: Option<Box<String>>,  // This should work
    // box_option_string: Box<Option<String>>,  // This has the issue
    // vec_option_string: Vec<Option<String>>,  // This should work
    // option_vec_string: Option<Vec<String>>,  // This should work
}

fn main() {
    println!("Comprehensive edge cases test");
    
    // Test basic types
    let _string_path = ComprehensiveTest::string_field_r();
    let _int_path = ComprehensiveTest::int_field_r();
    let _bool_path = ComprehensiveTest::bool_field_r();
    
    // Test basic containers
    let _option_path = ComprehensiveTest::option_string_fr();
    let _vec_path = ComprehensiveTest::vec_string_r();
    let _box_path = ComprehensiveTest::box_string_r();
    let _rc_path = ComprehensiveTest::rc_string_r();
    let _arc_path = ComprehensiveTest::arc_string_r();
    
    // Test collections
    let _hashset_path = ComprehensiveTest::hashset_string_r();
    let _btreeset_path = ComprehensiveTest::btreeset_string_r();
    let _vecdeque_path = ComprehensiveTest::vecdeque_string_r();
    let _linkedlist_path = ComprehensiveTest::linkedlist_string_r();
    let _binaryheap_path = ComprehensiveTest::binaryheap_string_r();
    
    // Test maps
    let _hashmap_path = ComprehensiveTest::hashmap_string_int_r();
    let _btreemap_path = ComprehensiveTest::btreemap_string_int_r();
    
    println!("All comprehensive edge cases generated successfully!");
}
