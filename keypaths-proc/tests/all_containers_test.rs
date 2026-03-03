//! Integration test: Kp derive with Option, Box, Rc, Arc, Vec, Option<Vec> and AllContainersTest.

use keypaths_proc::Kp;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::marker::PhantomData;
use std::ops::Range;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone, Kp)]
struct ContainerFields {
    name: String,
    age: Option<u32>,
    boxed: Box<String>,
    rc: std::rc::Rc<i32>,
    arc: Arc<bool>,
    vec: Vec<String>,
    opt_vec: Option<Vec<String>>,
}

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

    // Option-of-container and container-of-Option
    option_vecdeque_field: Option<VecDeque<String>>,
    vecdeque_option_field: VecDeque<Option<String>>,
    option_hashset_field: Option<HashSet<String>>,
    // option_result_field: Option<Result<i32, String>>,  // Kp macro currently generates wrong type for Option<Result<>> fr

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
    cow_str_field: Cow<'static, str>,

    empty_tuple: (),
}

#[test]
fn test_container_keypaths() {
    let value = ContainerFields {
        name: "Alice".to_string(),
        age: Some(30),
        boxed: Box::new("boxed".to_string()),
        rc: std::rc::Rc::new(42),
        arc: Arc::new(true),
        vec: vec!["a".into(), "b".into(), "c".into()],
        opt_vec: Some(vec!["one".into(), "two".into()]),
    };

    assert_eq!(ContainerFields::name_r().get(&value), "Alice");
    assert_eq!(ContainerFields::age_fr().get(&value), Some(&30u32));
    assert_eq!(ContainerFields::boxed_r().get(&value).as_str(), "boxed");
    assert_eq!(*ContainerFields::rc_r().get(&value), 42);
    assert!(*ContainerFields::arc_r().get(&value));
    assert_eq!(ContainerFields::vec_r().get(&value).len(), 3);
    assert_eq!(ContainerFields::vec_r().get(&value)[0], "a");
    assert_eq!(ContainerFields::vec_fr().get(&value), Some(&"a".to_string()));
    assert_eq!(ContainerFields::vec_fr_at(1).get(&value), Some(&"b".to_string()));
    assert_eq!(ContainerFields::opt_vec_fr().get(&value), Some(&"one".to_string()));
    assert_eq!(ContainerFields::opt_vec_fr_at(1).get(&value), Some(&"two".to_string()));
}

#[test]
fn test_all_containers_keypaths() {
    let value = AllContainersTest {
        option_field: Some("opt".to_string()),
        vec_field: vec!["a".into(), "b".into()],
        box_field: Box::new("box".to_string()),
        rc_field: Rc::new("rc".to_string()),
        arc_field: Arc::new("arc".to_string()),
        string_field: "string".to_string(),
        static_str_field: "static",
        static_slice_field: &[1u8, 2, 3],
        static_slice_i32: &[10i32, 20],
        opt_static_str: Some("opt_static"),
        hashset_field: HashSet::from(["h1".into(), "h2".into()]),
        btreeset_field: BTreeSet::from(["b1".into(), "b2".into()]),
        vecdeque_field: VecDeque::from(["vd1".into(), "vd2".into()]),
        linkedlist_field: LinkedList::from(["ll1".into(), "ll2".into()]),
        binaryheap_field: BinaryHeap::from(["bh1".into(), "bh2".into()]),
        hashmap_field: HashMap::from([("k1".into(), 1), ("k2".into(), 2)]),
        btreemap_field: BTreeMap::from([("a".into(), 1), ("b".into(), 2)]),
        option_vecdeque_field: Some(VecDeque::from(["ovd".into()])),
        vecdeque_option_field: VecDeque::from([Some("vo1".into()), None]),
        option_hashset_field: Some(HashSet::from(["ohs".into()])),
        cell_field: Cell::new(7),
        refcell_field: RefCell::new("refcell".to_string()),
        once_lock_field: OnceLock::new(),
        phantom_field: PhantomData,
        range_field: 0..10,
        result_field: Ok(100),
        cow_str_field: Cow::Borrowed("cow"),
        empty_tuple: (),
    };

    // Basic containers
    assert_eq!(AllContainersTest::string_field_r().get(&value), "string");
    assert_eq!(AllContainersTest::vec_field_r().get(&value).len(), 2);
    assert_eq!(AllContainersTest::box_field_r().get(&value).as_str(), "box");
    assert_eq!(AllContainersTest::rc_field_r().get(&value).as_str(), "rc");
    assert_eq!(AllContainersTest::arc_field_r().get(&value).as_str(), "arc");
    assert_eq!(AllContainersTest::option_field_r().get(&value).as_ref(), Some(&"opt".to_string()));

    // Reference types
    assert_eq!(*AllContainersTest::static_str_field_r().get(&value), "static");
    assert_eq!(AllContainersTest::opt_static_str_fr().get(&value), Some(&"opt_static"));

    let x = crate::AllContainersTest::vec_field_r().map_optional(|x| { x.first() });
    // Collections
    assert_eq!(AllContainersTest::vecdeque_field_r().get(&value).len(), 2);
    assert_eq!(AllContainersTest::hashmap_field_r().get(&value).len(), 2);
    assert_eq!(AllContainersTest::btreemap_field_r().get(&value).len(), 2);

    // Result
    assert_eq!(AllContainersTest::result_field_fr().get(&value), Some(&100));

    // Cell / RefCell (reference to container)
    assert_eq!(AllContainersTest::cell_field_r().get(&value).get(), 7);
    assert_eq!(AllContainersTest::refcell_field_r().get(&value).borrow().as_str(), "refcell");

    // Range, PhantomData, empty tuple
    assert_eq!(AllContainersTest::range_field_r().get(&value).start, 0);
    assert_eq!(AllContainersTest::range_field_r().get(&value).end, 10);
    assert_eq!(*AllContainersTest::empty_tuple_r().get(&value), ());
}
