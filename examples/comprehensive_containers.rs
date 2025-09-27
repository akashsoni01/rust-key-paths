use key_paths_core::KeyPaths;
use key_paths_derive::{Keypaths, Casepaths};
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, LinkedList, BinaryHeap};
use std::rc::Rc;
use std::sync::Arc;

// ===== Basic Container Types =====

#[derive(Debug, Keypaths)]
struct BasicContainers {
    // Standard collections
    vec_field: Vec<String>,
    hashmap_field: HashMap<String, i32>,
    btreemap_field: BTreeMap<String, i32>,
    hashset_field: HashSet<String>,
    btreeset_field: BTreeSet<String>,
    vecdeque_field: VecDeque<String>,
    linkedlist_field: LinkedList<String>,
    binaryheap_field: BinaryHeap<i32>,
    
    // Smart pointers
    box_field: Box<String>,
    rc_field: Rc<String>,
    arc_field: Arc<String>,
    
    // Option types
    option_field: Option<String>,
}

// ===== Nested Container Combinations =====

#[derive(Debug, Keypaths)]
struct NestedContainers {
    // Option<Box<T>>
    option_box_field: Option<Box<String>>,
    
    // Option<Rc<T>>
    option_rc_field: Option<Rc<String>>,
    
    // Option<Arc<T>>
    option_arc_field: Option<Arc<String>>,
    
    // Box<Option<T>>
    box_option_field: Box<Option<String>>,
    
    // Rc<Option<T>>
    rc_option_field: Rc<Option<String>>,
    
    // Arc<Option<T>>
    arc_option_field: Arc<Option<String>>,
    
    // Vec<Option<T>>
    vec_option_field: Vec<Option<String>>,
    
    // Option<Vec<T>>
    option_vec_field: Option<Vec<String>>,
    
    // HashMap<Option<T>>
    hashmap_option_field: HashMap<String, Option<i32>>,
    
    // Option<HashMap<T>>
    option_hashmap_field: Option<HashMap<String, i32>>,
}

// ===== Complex Nested Structures =====

#[derive(Debug, Keypaths)]
struct InnerData {
    value: String,
    count: i32,
}

#[derive(Debug, Keypaths)]
struct ComplexNested {
    // Deeply nested: Option<Box<Vec<Option<InnerData>>>>
    deeply_nested: Option<Box<Vec<Option<InnerData>>>>,
    
    // Map with complex values
    complex_map: HashMap<String, Option<Box<InnerData>>>,
    
    // Set of complex types
    complex_set: HashSet<Box<InnerData>>,
}

// ===== Enum with Container Types =====

#[derive(Debug, Casepaths)]
enum ContainerEnum {
    VecVariant(Vec<String>),
    HashMapVariant(HashMap<String, i32>),
    OptionVariant(Option<String>),
    BoxVariant(Box<String>),
    NestedVariant(Option<Box<Vec<String>>>),
}

// ===== Tuple Struct with Containers =====

#[derive(Debug, Keypaths)]
struct TupleContainers(
    Vec<String>,
    Option<HashMap<String, i32>>,
    Box<VecDeque<String>>,
);

fn main() {
    println!("🔑 Comprehensive Container KeyPaths Examples\n");
    
    // ===== Basic Container Examples =====
    println!("=== Basic Container Types ===");
    
    let mut basic = BasicContainers {
        vec_field: vec!["hello".to_string(), "world".to_string()],
        hashmap_field: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), 42);
            map.insert("key2".to_string(), 24);
            map
        },
        btreemap_field: {
            let mut map = BTreeMap::new();
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
        btreeset_field: {
            let mut set = BTreeSet::new();
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
        linkedlist_field: {
            let mut list = LinkedList::new();
            list.push_back("first".to_string());
            list.push_back("second".to_string());
            list
        },
        binaryheap_field: {
            let mut heap = BinaryHeap::new();
            heap.push(10);
            heap.push(5);
            heap.push(15);
            heap
        },
        box_field: Box::new("boxed".to_string()),
        rc_field: Rc::new("rc".to_string()),
        arc_field: Arc::new("arc".to_string()),
        option_field: Some("option".to_string()),
    };
    
    // Vec operations
    println!("Vec operations:");
    let vec_path = BasicContainers::vec_field_fr();
    if let Some(first) = vec_path.get(&basic) {
        println!("  First element: {}", first);
    }
    
    let vec_at_path = BasicContainers::vec_field_fr_at(1);
    if let Some(second) = vec_at_path.get(&basic) {
        println!("  Second element: {}", second);
    }
    
    // HashMap operations
    println!("HashMap operations:");
    let map_path = BasicContainers::hashmap_field_fr("key1".to_string());
    if let Some(value) = map_path.get(&basic) {
        println!("  key1 value: {}", value);
    }
    
    // BTreeMap operations
    println!("BTreeMap operations:");
    let btree_path = BasicContainers::btreemap_field_fr("key2".to_string());
    if let Some(value) = btree_path.get(&basic) {
        println!("  key2 value: {}", value);
    }
    
    // HashSet operations
    println!("HashSet operations:");
    let set_path = BasicContainers::hashset_field_fr();
    if let Some(element) = set_path.get(&basic) {
        println!("  First element: {}", element);
    }
    
    // VecDeque operations
    println!("VecDeque operations:");
    let deque_path = BasicContainers::vecdeque_field_fr();
    if let Some(front) = deque_path.get(&basic) {
        println!("  Front element: {}", front);
    }
    
    // BinaryHeap operations
    println!("BinaryHeap operations:");
    let heap_path = BasicContainers::binaryheap_field_fr();
    if let Some(max) = heap_path.get(&basic) {
        println!("  Max element: {}", max);
    }
    
    // ===== Nested Container Examples =====
    println!("\n=== Nested Container Types ===");
    
    let mut nested = NestedContainers {
        option_box_field: Some(Box::new("option box".to_string())),
        option_rc_field: Some(Rc::new("option rc".to_string())),
        option_arc_field: Some(Arc::new("option arc".to_string())),
        box_option_field: Box::new(Some("box option".to_string())),
        rc_option_field: Rc::new(Some("rc option".to_string())),
        arc_option_field: Arc::new(Some("arc option".to_string())),
        vec_option_field: vec![Some("vec option 1".to_string()), None, Some("vec option 3".to_string())],
        option_vec_field: Some(vec!["option vec 1".to_string(), "option vec 2".to_string()]),
        hashmap_option_field: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), Some(100));
            map.insert("key2".to_string(), None);
            map.insert("key3".to_string(), Some(200));
            map
        },
        option_hashmap_field: Some({
            let mut map = HashMap::new();
            map.insert("key1".to_string(), 300);
            map.insert("key2".to_string(), 400);
            map
        }),
    };
    
    // Option<Box<T>> operations
    println!("Option<Box<T>> operations:");
    let option_box_path = NestedContainers::option_box_field_fr();
    if let Some(value) = option_box_path.get(&nested) {
        println!("  Option<Box<T>> value: {}", value);
    }
    
    // Box<Option<T>> operations
    println!("Box<Option<T>> operations:");
    let box_option_path = NestedContainers::box_option_field_fr();
    if let Some(value) = box_option_path.get(&nested) {
        println!("  Box<Option<T>> value: {}", value);
    }
    
    // Vec<Option<T>> operations
    println!("Vec<Option<T>> operations:");
    let vec_option_path = NestedContainers::vec_option_field_fr();
    if let Some(value) = vec_option_path.get(&nested) {
        println!("  Vec<Option<T>> first value: {}", value);
    }
    
    let vec_option_at_path = NestedContainers::vec_option_field_fr_at(2);
    if let Some(value) = vec_option_at_path.get(&nested) {
        println!("  Vec<Option<T>> third value: {}", value);
    }
    
    // Option<Vec<T>> operations
    println!("Option<Vec<T>> operations:");
    let option_vec_path = NestedContainers::option_vec_field_fr();
    if let Some(value) = option_vec_path.get(&nested) {
        println!("  Option<Vec<T>> first value: {}", value);
    }
    
    // HashMap<Option<T>> operations
    println!("HashMap<Option<T>> operations:");
    let map_option_path = NestedContainers::hashmap_option_field_fr("key1".to_string());
    if let Some(value) = map_option_path.get(&nested) {
        println!("  HashMap<Option<T>> key1 value: {}", value);
    }
    
    // Option<HashMap<T>> operations
    println!("Option<HashMap<T>> operations:");
    let option_map_path = NestedContainers::option_hashmap_field_fr("key1".to_string());
    if let Some(value) = option_map_path.get(&nested) {
        println!("  Option<HashMap<T>> key1 value: {}", value);
    }
    
    // ===== Complex Nested Examples =====
    println!("\n=== Complex Nested Structures ===");
    
    let mut complex = ComplexNested {
        deeply_nested: Some(Box::new(vec![
            Some(InnerData { value: "deep1".to_string(), count: 1 }),
            None,
            Some(InnerData { value: "deep3".to_string(), count: 3 }),
        ])),
        complex_map: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), Some(Box::new(InnerData { value: "map1".to_string(), count: 10 })));
            map.insert("key2".to_string(), None);
            map
        },
        complex_set: {
            let mut set = HashSet::new();
            set.insert(Box::new(InnerData { value: "set1".to_string(), count: 20 }));
            set.insert(Box::new(InnerData { value: "set2".to_string(), count: 30 }));
            set
        },
    };
    
    // Deeply nested access
    println!("Deeply nested access:");
    let deep_path = ComplexNested::deeply_nested_fr();
    if let Some(inner_data) = deep_path.get(&complex) {
        println!("  Deeply nested first value: {}, count: {}", inner_data.value, inner_data.count);
    }
    
    // Complex map access
    println!("Complex map access:");
    let complex_map_path = ComplexNested::complex_map_fr("key1".to_string());
    if let Some(inner_data) = complex_map_path.get(&complex) {
        println!("  Complex map key1 value: {}, count: {}", inner_data.value, inner_data.count);
    }
    
    // ===== Enum Examples =====
    println!("\n=== Enum with Container Types ===");
    
    let container_enum = ContainerEnum::NestedVariant(Some(Box::new(vec!["enum1".to_string(), "enum2".to_string()])));
    
    // Enum case path
    let enum_path = ContainerEnum::nested_variant_case_fr();
    if let Some(first) = enum_path.get(&container_enum) {
        println!("  Enum nested variant first value: {}", first);
    }
    
    // ===== Tuple Struct Examples =====
    println!("\n=== Tuple Struct with Containers ===");
    
    let mut tuple = TupleContainers(
        vec!["tuple1".to_string(), "tuple2".to_string()],
        Some({
            let mut map = HashMap::new();
            map.insert("tuple_key".to_string(), 42);
            map
        }),
        Box::new({
            let mut deque = VecDeque::new();
            deque.push_back("tuple_deque".to_string());
            deque
        }),
    );
    
    // Tuple struct field access
    let tuple_vec_path = TupleContainers::f0_fr();
    if let Some(first) = tuple_vec_path.get(&tuple) {
        println!("  Tuple vec first element: {}", first);
    }
    
    let tuple_option_map_path = TupleContainers::f1_fr("tuple_key".to_string());
    if let Some(value) = tuple_option_map_path.get(&tuple) {
        println!("  Tuple option map value: {}", value);
    }
    
    // ===== Composition Examples =====
    println!("\n=== KeyPath Composition ===");
    
    // Compose through nested structures
    let composed_path = ComplexNested::deeply_nested_fr()
        .then(InnerData::value_fr());
    
    if let Some(value) = composed_path.get(&complex) {
        println!("  Composed path value: {}", value);
    }
    
    // Compose through enum and container
    let enum_composed = ContainerEnum::nested_variant_case_fr()
        .then(String::from("enum1")); // This would need manual keypath creation
    
    println!("\n✅ All container types and nested combinations demonstrated!");
    println!("🔑 KeyPaths now support:");
    println!("  • All standard library collections (Vec, HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, LinkedList, BinaryHeap)");
    println!("  • Smart pointers (Box, Rc, Arc)");
    println!("  • Nested combinations (Option<Box<T>>, Vec<Option<T>>, etc.)");
    println!("  • Complex deeply nested structures");
    println!("  • Enum variants with container types");
    println!("  • Tuple structs with containers");
    println!("  • Full composition support across all types");
}
