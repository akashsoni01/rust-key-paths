use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct BasicTest {
    // Basic containers
    vec_field: Vec<String>,
    option_field: Option<String>,
    box_field: Box<String>,
    rc_field: std::rc::Rc<String>,
    arc_field: std::sync::Arc<String>,
    
    // Collections
    hashset_field: std::collections::HashSet<String>,
    btreeset_field: std::collections::BTreeSet<String>,
    vecdeque_field: std::collections::VecDeque<String>,
    linkedlist_field: std::collections::LinkedList<String>,
    binaryheap_field: std::collections::BinaryHeap<String>,
    
    // Maps
    hashmap_field: std::collections::HashMap<String, i32>,
    btreemap_field: std::collections::BTreeMap<String, i32>,
}

fn main() {
    println!("Basic containers test");
    
    // Test basic containers
    let _vec_path = BasicTest::vec_field_r();
    let _option_path = BasicTest::option_field_fr();
    let _box_path = BasicTest::box_field_r();
    let _rc_path = BasicTest::rc_field_r();
    let _arc_path = BasicTest::arc_field_r();
    
    // Test collections
    let _hashset_path = BasicTest::hashset_field_r();
    let _btreeset_path = BasicTest::btreeset_field_r();
    let _vecdeque_path = BasicTest::vecdeque_field_r();
    let _linkedlist_path = BasicTest::linkedlist_field_r();
    let _binaryheap_path = BasicTest::binaryheap_field_r();
    
    // Test maps
    let _hashmap_path = BasicTest::hashmap_field_r();
    let _btreemap_path = BasicTest::btreemap_field_r();
    
    println!("All basic container keypaths generated successfully!");
}
