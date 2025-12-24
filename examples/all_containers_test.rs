use keypaths_proc::Keypaths;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, LinkedList, BinaryHeap};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Keypaths)]
struct AllContainersTest {
    // Basic containers
    option_field: Option<String>,
    vec_field: Vec<String>,
    box_field: Box<String>,
    rc_field: Rc<String>,
    arc_field: Arc<String>,
    
    // Sets
    hashset_field: HashSet<String>,
    btreeset_field: BTreeSet<String>,
    
    // Queues and Lists
    vecdeque_field: VecDeque<String>,
    linkedlist_field: LinkedList<String>,
    binaryheap_field: BinaryHeap<String>,
    
    // Maps
    hashmap_field: HashMap<String, i32>,
    btreemap_field: BTreeMap<String, i32>,
    empty_touple: (),
}

fn main() {
    println!("All containers test");
    
    // Test basic containers
    let _option_path = AllContainersTest::option_field_fr();
    let _vec_path = AllContainersTest::vec_field_r();
    let _box_path = AllContainersTest::box_field_r();
    let _rc_path = AllContainersTest::rc_field_r();
    let _arc_path = AllContainersTest::arc_field_r();
    
    // Test sets
    let _hashset_path = AllContainersTest::hashset_field_r();
    let _btreeset_path = AllContainersTest::btreeset_field_r();
    
    // Test queues and lists
    let _vecdeque_path = AllContainersTest::vecdeque_field_r();
    let _linkedlist_path = AllContainersTest::linkedlist_field_r();
    let _binaryheap_path = AllContainersTest::binaryheap_field_r();
    
    // Test maps
    let _hashmap_path = AllContainersTest::hashmap_field_r();
    let _btreemap_path = AllContainersTest::btreemap_field_r();
    let empty_touple = AllContainersTest::empty_touple_fr();
    println!("All containers generated successfully!");
}
