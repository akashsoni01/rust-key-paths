use rust_keypaths::{KeyPath, OptionalKeyPath, containers};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

#[derive(Debug)]
struct ContainerTest {
    vec_field: Option<Vec<String>>,
    vecdeque_field: VecDeque<String>,
    linkedlist_field: LinkedList<String>,
    hashmap_field: HashMap<String, i32>,
    btreemap_field: BTreeMap<String, i32>,
    hashset_field: HashSet<String>,
    btreeset_field: BTreeSet<String>,
    binaryheap_field: BinaryHeap<String>,
}

fn main() {
    let test = ContainerTest {
        vec_field: Some(vec![
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
        ]),
        vecdeque_field: VecDeque::from(vec!["a".to_string(), "b".to_string()]),
        linkedlist_field: LinkedList::from(["x".to_string(), "y".to_string()]),
        hashmap_field: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), 100);
            map.insert("key2".to_string(), 200);
            map
        },
        btreemap_field: {
            let mut map = BTreeMap::new();
            map.insert("key1".to_string(), 100);
            map.insert("key2".to_string(), 200);
            map
        },
        hashset_field: {
            let mut set = HashSet::new();
            set.insert("value1".to_string());
            set.insert("value2".to_string());
            set
        },
        btreeset_field: {
            let mut set = BTreeSet::new();
            set.insert("value1".to_string());
            set.insert("value2".to_string());
            set
        },
        binaryheap_field: {
            let mut heap = BinaryHeap::new();
            heap.push("priority1".to_string());
            heap.push("priority2".to_string());
            heap
        },
    };

    // Access Vec element at index (vec_field is Option<Vec<String>>, so unwrap Option first)
    let vec_index_kp = containers::for_vec_index::<String>(1);
    let chained_vec =
        OptionalKeyPath::new(|c: &ContainerTest| c.vec_field.as_ref()).then(vec_index_kp);

    if let Some(value) = chained_vec.get(&test) {
        println!("Vec[1]: {}", value);
    }

    // Access VecDeque element at index
    let vecdeque_kp = OptionalKeyPath::new(|c: &ContainerTest| Some(&c.vecdeque_field));
    let vecdeque_index_kp = containers::for_vecdeque_index::<String>(0);
    let chained_vecdeque = vecdeque_kp.then(vecdeque_index_kp);

    if let Some(value) = chained_vecdeque.get(&test) {
        println!("VecDeque[0]: {}", value);
    }

    // Access LinkedList element at index
    let linkedlist_kp = OptionalKeyPath::new(|c: &ContainerTest| Some(&c.linkedlist_field));
    let linkedlist_index_kp = containers::for_linkedlist_index::<String>(1);
    let chained_linkedlist = linkedlist_kp.then(linkedlist_index_kp);

    if let Some(value) = chained_linkedlist.get(&test) {
        println!("LinkedList[1]: {}", value);
    }

    // Access HashMap value by key
    let hashmap_kp = OptionalKeyPath::new(|c: &ContainerTest| Some(&c.hashmap_field));
    let hashmap_key_kp = containers::for_hashmap_key("key1".to_string());
    let chained_hashmap = hashmap_kp.then(hashmap_key_kp);

    if let Some(value) = chained_hashmap.get(&test) {
        println!("HashMap[\"key1\"]: {}", value);
    }

    // Access BTreeMap value by key
    let btreemap_kp = OptionalKeyPath::new(|c: &ContainerTest| Some(&c.btreemap_field));
    let btreemap_key_kp = containers::for_btreemap_key("key2".to_string());
    let chained_btreemap = btreemap_kp.then(btreemap_key_kp);

    if let Some(value) = chained_btreemap.get(&test) {
        println!("BTreeMap[\"key2\"]: {}", value);
    }

    // Access HashSet element
    let hashset_kp = OptionalKeyPath::new(|c: &ContainerTest| Some(&c.hashset_field));
    let hashset_get_kp = containers::for_hashset_get("value1".to_string());
    let chained_hashset = hashset_kp.then(hashset_get_kp);

    if let Some(value) = chained_hashset.get(&test) {
        println!("HashSet contains: {}", value);
    }

    // Access BTreeSet element
    let btreeset_kp = OptionalKeyPath::new(|c: &ContainerTest| Some(&c.btreeset_field));
    let btreeset_get_kp = containers::for_btreeset_get("value2".to_string());
    let chained_btreeset = btreeset_kp.then(btreeset_get_kp);

    if let Some(value) = chained_btreeset.get(&test) {
        println!("BTreeSet contains: {}", value);
    }

    // Access BinaryHeap peek
    let binaryheap_kp = OptionalKeyPath::new(|c: &ContainerTest| Some(&c.binaryheap_field));
    let binaryheap_peek_kp = containers::for_binaryheap_peek::<String>();
    let chained_binaryheap = binaryheap_kp.then(binaryheap_peek_kp);

    if let Some(value) = chained_binaryheap.get(&test) {
        println!("BinaryHeap peek: {}", value);
    }

    println!("\n✅ All container access methods work!");
}
