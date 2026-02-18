use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::rc::Rc;
use std::sync::Arc;
use key_paths_derive::Kp;
use rust_key_paths::{KpStatic, KpType};


#[derive(Debug, Kp)]
struct AllContainersTest {
    // Basic containers
    option_field: Option<String>,
    vec_field: Vec<String>,
    box_field: Box<String>,
    rc_field: Rc<String>,
    arc_field: Arc<String>,

    // Reference types
    static_str_field: &'static str,
    static_slice_field: &'static [u8],
    static_slice_i32: &'static [i32],
    // Option<Reference>
    opt_static_str: Option<&'static str>,

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
    empty_tuple: (),
}

static BYTES: &[u8] = b"hello";
static INTS: &[i32] = &[1, 2, 3];
// KpStatic uses const fn; can be initialized in static without LazyLock.

fn get_option_field(r: &AllContainersTest) -> Option<&String> {
    r.option_field.as_ref()
}
fn set_option_field(r: &mut AllContainersTest) -> Option<&mut String> {
    r.option_field.as_mut()
}

static KP: KpStatic<AllContainersTest, String> =
    KpStatic::new(get_option_field, set_option_field);

static KP2: KpStatic<AllContainersTest, String> = KpStatic::new(
    |r: &AllContainersTest| r.option_field.as_ref(),
    |r: &mut AllContainersTest| r.option_field.as_mut(),
);

fn main() {
    println!("All containers test");

    let data = AllContainersTest {
        option_field: Some("opt".to_string()),
        vec_field: vec!["a".to_string()],
        box_field: Box::new("boxed".to_string()),
        rc_field: Rc::new("rc".to_string()),
        arc_field: Arc::new("arc".to_string()),
        static_str_field: "static",
        static_slice_field: BYTES,
        static_slice_i32: INTS,
        opt_static_str: Some("optional"),
        hashset_field: HashSet::from(["s".to_string()]),
        btreeset_field: BTreeSet::from(["t".to_string()]),
        vecdeque_field: VecDeque::from(["v".to_string()]),
        linkedlist_field: LinkedList::from(["l".to_string()]),
        binaryheap_field: BinaryHeap::from(["b".to_string()]),
        hashmap_field: HashMap::from([("k".to_string(), 42)]),
        btreemap_field: BTreeMap::from([("k".to_string(), 99)]),
        empty_tuple: (),
    };

    // Static dispatch via KpStatic (no lazy init)
    assert_eq!(KP.get(&data).map(|s| s.as_str()), Some("opt"));

    // Test basic containers (derive)
    let _option_path = AllContainersTest::option_field();
    let _vec_path = AllContainersTest::vec_field();
    let _box_path = AllContainersTest::box_field();
    let _rc_path = AllContainersTest::rc_field();
    let _arc_path = AllContainersTest::arc_field();

    // Test reference types
    let static_str_kp = AllContainersTest::static_str_field();
    let static_slice_kp = AllContainersTest::static_slice_field();
    let static_slice_i32_kp = AllContainersTest::static_slice_i32();
    assert_eq!(static_str_kp.get(&data), Some(&"static"));
    assert_eq!(static_slice_kp.get(&data).map(|s| *s), Some(BYTES));
    assert_eq!(static_slice_i32_kp.get(&data).map(|s| *s), Some(INTS));
    let opt_str_kp: KpType<'static, AllContainersTest, &'static str> =
        AllContainersTest::opt_static_str();
    assert_eq!(opt_str_kp.get(&data).map(|s| *s), Some("optional"));

    // Test sets
    let _hashset_path = AllContainersTest::hashset_field();
    let _btreeset_path = AllContainersTest::btreeset_field();

    // Test queues and lists
    let _vecdeque_path = AllContainersTest::vecdeque_field();
    let _linkedlist_path = AllContainersTest::linkedlist_field();
    let _binaryheap_path = AllContainersTest::binaryheap_field();

    // Test maps
    let _hashmap_path = AllContainersTest::hashmap_field();
    let _btreemap_path = AllContainersTest::btreemap_field();
    let _empty_tuple = AllContainersTest::empty_tuple();
    println!("All containers (including &'static and reference types) generated successfully!");
}
