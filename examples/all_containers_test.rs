use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::marker::PhantomData;
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use key_paths_derive::Kp;
use rust_key_paths::KpType;

#[derive(Debug, Kp)]
struct AllContainersTest {
    // Basic containers
    option_field: Option<String>,
    vec_field: Vec<String>,
    box_field: Box<String>,
    rc_field: Rc<String>,
    arc_field: Arc<String>,

    // String and owned text
    string_field: String,

    // Reference types
    static_str_field: &'static str,
    static_slice_field: &'static [u8],
    static_slice_i32: &'static [i32],
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

    // Option-of-container and container-of-Option (keypath to container reference, like Vec/HashSet)
    option_vecdeque_field: Option<VecDeque<String>>,
    vecdeque_option_field: VecDeque<Option<String>>,
    option_hashset_field: Option<HashSet<String>>,
    option_result_field: Option<Result<i32, String>>,

    // Interior mutability
    cell_field: Cell<i32>,
    refcell_field: RefCell<String>,

    // Lazy init
    once_lock_field: OnceLock<String>,

    // Marker / range
    phantom_field: PhantomData<()>,
    range_field: Range<u32>,

    // Error handling and borrow
    result_field: Result<i32, String>,
    cow_str_field: Cow<'static, String>,

    empty_tuple: (),
}

static BYTES: &[u8] = b"hello";
static INTS: &[i32] = &[1, 2, 3];

fn main() {
    println!("All containers test");

    let once_lock = OnceLock::new();
    let _ = once_lock.set("lazy".to_string());

    let data = AllContainersTest {
        option_field: Some("opt".to_string()),
        vec_field: vec!["a".to_string()],
        box_field: Box::new("boxed".to_string()),
        rc_field: Rc::new("rc".to_string()),
        arc_field: Arc::new("arc".to_string()),
        string_field: "hello".to_string(),
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
        option_vecdeque_field: Some(VecDeque::from(["ov".to_string()])),
        vecdeque_option_field: VecDeque::from([Some("vo".to_string())]),
        option_hashset_field: Some(HashSet::from(["oh".to_string()])),
        option_result_field: Some(Ok(100)),
        cell_field: Cell::new(10),
        refcell_field: RefCell::new("refcell".to_string()),
        once_lock_field: once_lock,
        phantom_field: PhantomData,
        range_field: 0..10,
        result_field: Ok(200),
        cow_str_field: Cow::Owned("cow".to_string()),
        empty_tuple: (),
    };

    // Test basic containers (derive)
    let _option_path = AllContainersTest::option_field();
    let _vec_path = AllContainersTest::vec_field();
    let _box_path = AllContainersTest::box_field();
    let _rc_path = AllContainersTest::rc_field();
    let _arc_path = AllContainersTest::arc_field();

    // String and Option-of-container / container-of-Option
    let string_kp = AllContainersTest::string_field();
    assert_eq!(string_kp.get(&data).map(|s| s.as_str()), Some("hello"));
    let _opt_vecdeque_kp = AllContainersTest::option_vecdeque_field();
    let _vecdeque_opt_kp = AllContainersTest::vecdeque_option_field();
    let _opt_hashset_kp = AllContainersTest::option_hashset_field();
    let _opt_result_kp = AllContainersTest::option_result_field();

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

    // Interior mutability, lazy, marker, range, result, cow
    let _cell_kp = AllContainersTest::cell_field();
    let _refcell_kp = AllContainersTest::refcell_field();
    let once_lock_kp = AllContainersTest::once_lock_field();
    if let Some(x) = once_lock_kp.get(&data) {
        // x is &String (inner value reference)
        assert_eq!(x.as_str(), "lazy");
    }
    let _phantom_kp = AllContainersTest::phantom_field();
    let range_kp = AllContainersTest::range_field();
    assert_eq!(range_kp.get(&data), Some(&(0..10)));
    let result_kp = AllContainersTest::result_field();
    assert_eq!(result_kp.get(&data).copied(), Some(200));
    let cow_kp = AllContainersTest::cow_str_field();
    assert_eq!(cow_kp.get(&data).map(|c| c.as_str()), Some("cow"));

    let _empty_tuple = AllContainersTest::empty_tuple();
    println!("All containers (including &'static and reference types) generated successfully!");
}
