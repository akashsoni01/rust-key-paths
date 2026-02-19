use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use key_paths_derive::Kp;
use rust_key_paths::KpType;
// cargo run --example drop_test 2>&1


/// Increments a shared counter when dropped. Used to verify all fields are dropped after kp get/set.
#[derive(Clone, Debug)]
struct DropToken(Arc<AtomicUsize>);
impl Drop for DropToken {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Wraps a value and counts when dropped. Used to verify AllContainersTest fields are all dropped.
#[derive(Clone, Debug)]
struct WithDrop<T> {
    value: T,
    _token: DropToken,
}
impl<T: PartialEq> PartialEq for WithDrop<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}
impl<T: Eq> Eq for WithDrop<T> {}
impl<T: std::hash::Hash> std::hash::Hash for WithDrop<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
impl<T: PartialOrd> PartialOrd for WithDrop<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl<T: Ord> Ord for WithDrop<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}
impl<T> WithDrop<T> {
    fn new(value: T, token: DropToken) -> Self {
        Self {
            value,
            _token: token,
        }
    }
}

#[derive(Debug, Kp)]
struct AllContainersTest {
    // Basic containers (wrapped to track drops)
    option_field: Option<WithDrop<String>>,
    vec_field: Vec<WithDrop<String>>,
    box_field: Box<WithDrop<String>>,
    rc_field: Rc<WithDrop<String>>,
    arc_field: Arc<WithDrop<String>>,

    // Reference types (no ownership, no drop tracking)
    static_str_field: &'static str,
    static_slice_field: &'static [u8],
    static_slice_i32: &'static [i32],
    opt_static_str: Option<&'static str>,

    // Sets
    hashset_field: HashSet<WithDrop<String>>,
    btreeset_field: BTreeSet<WithDrop<String>>,

    // Queues and Lists
    vecdeque_field: VecDeque<WithDrop<String>>,
    linkedlist_field: LinkedList<WithDrop<String>>,
    binaryheap_field: BinaryHeap<WithDrop<String>>,

    // Maps
    hashmap_field: HashMap<WithDrop<String>, i32>,
    btreemap_field: BTreeMap<WithDrop<String>, i32>,
    empty_tuple: (),
}

static BYTES: &[u8] = b"hello";
static INTS: &[i32] = &[1, 2, 3];
// KpStatic uses const fn; can be initialized in static without LazyLock.

fn main() {
    println!("All containers test");

    let drop_count = Arc::new(AtomicUsize::new(0));
    let token = || DropToken(Arc::clone(&drop_count));

    let mut data = AllContainersTest {
        option_field: Some(WithDrop::new("opt".to_string(), token())),
        vec_field: vec![WithDrop::new("a".to_string(), token())],
        box_field: Box::new(WithDrop::new("boxed".to_string(), token())),
        rc_field: Rc::new(WithDrop::new("rc".to_string(), token())),
        arc_field: Arc::new(WithDrop::new("arc".to_string(), token())),
        static_str_field: "static",
        static_slice_field: BYTES,
        static_slice_i32: INTS,
        opt_static_str: Some("optional"),
        hashset_field: HashSet::from([WithDrop::new("s".to_string(), token())]),
        btreeset_field: BTreeSet::from([WithDrop::new("t".to_string(), token())]),
        vecdeque_field: VecDeque::from([WithDrop::new("v".to_string(), token())]),
        linkedlist_field: LinkedList::from([WithDrop::new("l".to_string(), token())]),
        binaryheap_field: BinaryHeap::from([WithDrop::new("b".to_string(), token())]),
        hashmap_field: HashMap::from([(WithDrop::new("k".to_string(), token()), 42)]),
        btreemap_field: BTreeMap::from([(WithDrop::new("k".to_string(), token()), 99)]),
        empty_tuple: (),
    };
    let expected_drops = 12usize; // option(1) + vec(1) + box(1) + rc(1) + arc(1) + hashset(1) + btreeset(1) + vecdeque(1) + linkedlist(1) + binaryheap(1) + hashmap_key(1) + btreemap_key(1)

    // Test basic containers (derive) — access via .value for wrapped fields
    let option_kp = AllContainersTest::option_field();
    assert_eq!(option_kp.get(&data).map(|w| w.value.as_str()), Some("opt"));
    let _option_path = option_kp;
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

    // --- Drop test: kp get/set on all AllContainersTest fields must not drop; then drop(data) must drop all ---
    assert_eq!(
        drop_count.load(Ordering::SeqCst),
        0,
        "no drops must happen during construction or kp use so far"
    );

    // Exercise get and get_mut on every owned-field keypath
    let _ = AllContainersTest::option_field().get(&data);
    let _ = AllContainersTest::vec_field().get(&data);
    let _ = AllContainersTest::box_field().get(&data);
    let _ = AllContainersTest::rc_field().get(&data);
    let _ = AllContainersTest::arc_field().get(&data);
    let _ = AllContainersTest::hashset_field().get(&data);
    let _ = AllContainersTest::btreeset_field().get(&data);
    let _ = AllContainersTest::vecdeque_field().get(&data);
    let _ = AllContainersTest::linkedlist_field().get(&data);
    let _ = AllContainersTest::binaryheap_field().get(&data);
    let _ = AllContainersTest::hashmap_field().get(&data);
    let _ = AllContainersTest::btreemap_field().get(&data);

    let _ = AllContainersTest::option_field().get_mut(&mut data);
    let _ = AllContainersTest::vec_field().get_mut(&mut data);
    let _ = AllContainersTest::box_field().get_mut(&mut data);
    let _ = AllContainersTest::hashset_field().get_mut(&mut data);
    let _ = AllContainersTest::btreeset_field().get_mut(&mut data);
    let _ = AllContainersTest::vecdeque_field().get_mut(&mut data);
    let _ = AllContainersTest::linkedlist_field().get_mut(&mut data);
    let _ = AllContainersTest::binaryheap_field().get_mut(&mut data);
    let _ = AllContainersTest::hashmap_field().get_mut(&mut data);
    let _ = AllContainersTest::btreemap_field().get_mut(&mut data);

    assert_eq!(
        drop_count.load(Ordering::SeqCst),
        0,
        "no drops must happen during get/get_mut"
    );

    drop(data);

    assert_eq!(
        drop_count.load(Ordering::SeqCst),
        expected_drops,
        "all {} tracked fields must be dropped after kp get/set",
        expected_drops
    );
    println!(
        "Drop test OK: all {} AllContainersTest fields dropped properly after kp get/set.",
        expected_drops
    );
}
