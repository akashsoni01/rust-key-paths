use key_paths_derive::Kp;
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, LinkedList, BinaryHeap};
use std::sync::{Arc, Mutex as StdMutex, RwLock as StdRwLock};
use std::rc::Rc;

#[derive(Kp)]
struct AllWrapperTypes {
    // Basic types
    basic: String,
    
    // Option
    opt_string: Option<String>,
    
    // Vec
    vec_numbers: Vec<i32>,
    
    // Box
    boxed_value: Box<i32>,
    
    // Arc/Rc
    arc_value: Arc<String>,
    rc_value: Rc<String>,
    
    // HashMap and BTreeMap
    hash_map: HashMap<String, i32>,
    btree_map: BTreeMap<String, i32>,
    
    // Sets
    hash_set: HashSet<String>,
    btree_set: BTreeSet<String>,
    
    // VecDeque and LinkedList
    vec_deque: VecDeque<i32>,
    linked_list: LinkedList<i32>,
    
    // BinaryHeap
    binary_heap: BinaryHeap<i32>,
    
    // Result
    result_value: Result<String, String>,
    
    // Mutex and RwLock
    mutex_value: StdMutex<i32>,
    rwlock_value: StdRwLock<String>,
}

#[test]
fn test_basic_type() {
    let data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test basic type
    let basic_kp = AllWrapperTypes::basic();
    assert_eq!(basic_kp.get(&data), Some(&"hello".to_string()));
}

#[test]
fn test_option_type() {
    let data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test Option - should unwrap and access inner String
    let opt_kp = AllWrapperTypes::opt_string();
    assert_eq!(opt_kp.get(&data), Some(&"world".to_string()));
}

#[test]
fn test_option_none() {
    let data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: None,
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test Option None - should return None
    let opt_kp = AllWrapperTypes::opt_string();
    assert_eq!(opt_kp.get(&data), None);
}

#[test]
fn test_vec_type() {
    let data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test Vec - should access first element
    let vec_kp = AllWrapperTypes::vec_numbers();
    assert_eq!(vec_kp.get(&data), Some(&1));
}

#[test]
fn test_box_type() {
    let data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test Box - should deref to inner value (returns &i32, not &Box<i32>)
    let box_kp = AllWrapperTypes::boxed_value();
    let value = box_kp.get(&data);
    assert_eq!(value, Some(&42));
    
    // Verify the correct type signature
    let _typed: rust_key_paths::KpType<'static, AllWrapperTypes, i32> = box_kp;
}

#[test]
fn test_arc_type() {
    let data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test Arc - should deref to inner value
    let arc_kp = AllWrapperTypes::arc_value();
    assert_eq!(arc_kp.get(&data), Some(&"arc".to_string()));
}

#[test]
fn test_result_type() {
    let data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test Result - should access Ok value
    let result_kp = AllWrapperTypes::result_value();
    assert_eq!(result_kp.get(&data), Some(&"success".to_string()));
}

#[test]
fn test_result_err() {
    let data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Err("error".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test Result Err - should return None
    let result_kp = AllWrapperTypes::result_value();
    assert_eq!(result_kp.get(&data), None);
}

#[test]
fn test_mutable_basic_type() {
    let mut data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test mutable access to basic type
    let basic_kp = AllWrapperTypes::basic();
    basic_kp.get_mut(&mut data).map(|v| *v = "modified".to_string());
    assert_eq!(data.basic, "modified");
}

#[test]
fn test_mutable_option_type() {
    let mut data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test mutable access to Option inner value
    let opt_kp = AllWrapperTypes::opt_string();
    opt_kp.get_mut(&mut data).map(|v| *v = "modified".to_string());
    assert_eq!(data.opt_string, Some("modified".to_string()));
}

#[test]
fn test_mutable_vec_type() {
    let mut data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test mutable access to Vec first element
    let vec_kp = AllWrapperTypes::vec_numbers();
    vec_kp.get_mut(&mut data).map(|v| *v = 99);
    assert_eq!(data.vec_numbers[0], 99);
}

#[test]
fn test_mutable_box_type() {
    let mut data = AllWrapperTypes {
        basic: "hello".to_string(),
        opt_string: Some("world".to_string()),
        vec_numbers: vec![1, 2, 3],
        boxed_value: Box::new(42),
        arc_value: Arc::new("arc".to_string()),
        rc_value: Rc::new("rc".to_string()),
        hash_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        hash_set: HashSet::new(),
        btree_set: BTreeSet::new(),
        vec_deque: VecDeque::new(),
        linked_list: LinkedList::new(),
        binary_heap: BinaryHeap::new(),
        result_value: Ok("success".to_string()),
        mutex_value: StdMutex::new(100),
        rwlock_value: StdRwLock::new("locked".to_string()),
    };
    
    // Test mutable access to Box inner value
    let box_kp = AllWrapperTypes::boxed_value();
    box_kp.get_mut(&mut data).map(|v| *v = 99);
    assert_eq!(*data.boxed_value, 99);
}

#[derive(Kp, Debug, PartialEq)]
enum MyEnum {
    Unit,
    Single(String),
    Tuple(i32, String),
    Named { x: i32, y: String },
}

#[test]
fn test_enum_unit_variant() {
    let e = MyEnum::Unit;
    let unit_kp = MyEnum::unit();
    assert!(unit_kp.get(&e).is_some());
    
    let e2 = MyEnum::Single("test".to_string());
    assert!(unit_kp.get(&e2).is_none());
}

#[test]
fn test_enum_single_variant() {
    let e = MyEnum::Single("hello".to_string());
    let single_kp = MyEnum::single();
    assert_eq!(single_kp.get(&e), Some(&"hello".to_string()));
    
    let e2 = MyEnum::Unit;
    assert!(single_kp.get(&e2).is_none());
}

#[test]
fn test_enum_tuple_variant() {
    let e = MyEnum::Tuple(42, "world".to_string());
    let tuple_kp = MyEnum::tuple();
    assert_eq!(tuple_kp.get(&e), Some(&e));
    
    let e2 = MyEnum::Unit;
    assert!(tuple_kp.get(&e2).is_none());
}

#[test]
fn test_enum_named_variant() {
    let e = MyEnum::Named { x: 42, y: "test".to_string() };
    let named_kp = MyEnum::named();
    assert_eq!(named_kp.get(&e), Some(&e));
    
    let e2 = MyEnum::Unit;
    assert!(named_kp.get(&e2).is_none());
}

#[derive(Kp)]
struct TupleStruct(String, i32, Option<bool>);

#[test]
fn test_tuple_struct_fields() {
    let ts = TupleStruct("hello".to_string(), 42, Some(true));
    
    // Test field 0 (String)
    let f0_kp = TupleStruct::f0();
    assert_eq!(f0_kp.get(&ts), Some(&"hello".to_string()));
    
    // Test field 1 (i32)
    let f1_kp = TupleStruct::f1();
    assert_eq!(f1_kp.get(&ts), Some(&42));
    
    // Test field 2 (Option<bool> - unwraps to bool)
    let f2_kp = TupleStruct::f2();
    assert_eq!(f2_kp.get(&ts), Some(&true));
}

#[test]
fn test_tuple_struct_mutable() {
    let mut ts = TupleStruct("hello".to_string(), 42, Some(true));
    
    // Modify field 0
    let f0_kp = TupleStruct::f0();
    f0_kp.get_mut(&mut ts).map(|v| *v = "modified".to_string());
    assert_eq!(ts.0, "modified");
    
    // Modify field 1
    let f1_kp = TupleStruct::f1();
    f1_kp.get_mut(&mut ts).map(|v| *v = 99);
    assert_eq!(ts.1, 99);
    
    // Modify field 2 (Option inner value)
    let f2_kp = TupleStruct::f2();
    f2_kp.get_mut(&mut ts).map(|v| *v = false);
    assert_eq!(ts.2, Some(false));
}
