use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::rc::Rc;
use std::sync::Arc;
use key_paths_derive::Kp;

#[derive(Debug, Kp)]
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
    let _option_path = AllContainersTest::option_field();
    let _vec_path = AllContainersTest::vec_field();
    let _box_path = AllContainersTest::box_field();
    let _rc_path = AllContainersTest::rc_field();
    let _arc_path = AllContainersTest::arc_field();
    // Test sets
    let _hashset_path = AllContainersTest::hashset_field();
    AllContainersTest::hashset_field()
    let _btreeset_path = AllContainersTest::btreeset_field();

    // Test queues and lists
    let _vecdeque_path = AllContainersTest::vecdeque_field();
    let _linkedlist_path = AllContainersTest::linkedlist_field();
    let _binaryheap_path = AllContainersTest::binaryheap_field();

    // Test maps
    let _hashmap_path = AllContainersTest::hashmap_field();
    let _btreemap_path = AllContainersTest::btreemap_field();
    let empty_touple = AllContainersTest::empty_touple();
    println!("All containers generated successfully!");
}
