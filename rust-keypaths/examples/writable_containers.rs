use rust_keypaths::{WritableKeyPath, WritableOptionalKeyPath, containers};
use std::collections::{HashMap, BTreeMap, VecDeque};

#[derive(Debug)]
struct ContainerTest {
    vec_field: Option<Vec<String>>,
    vecdeque_field: VecDeque<String>,
    hashmap_field: HashMap<String, i32>,
    btreemap_field: BTreeMap<String, i32>,
    name: String,
}

fn main() {
    let mut test = ContainerTest {
        vec_field: Some(vec!["one".to_string(), "two".to_string(), "three".to_string()]),
        vecdeque_field: VecDeque::from(vec!["a".to_string(), "b".to_string()]),
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
        name: "Original".to_string(),
    };

    println!("Before mutations:");
    println!("  vec_field[1]: {:?}", test.vec_field.as_ref().and_then(|v| v.get(1)));
    println!("  vecdeque_field[0]: {:?}", test.vecdeque_field.get(0));
    println!("  hashmap_field[\"key1\"]: {:?}", test.hashmap_field.get("key1"));
    println!("  btreemap_field[\"key2\"]: {:?}", test.btreemap_field.get("key2"));
    println!("  name: {}", test.name);

    // Mutate Vec element at index
    let vec_kp = WritableOptionalKeyPath::new(|c: &mut ContainerTest| c.vec_field.as_mut());
    let vec_index_kp = containers::for_vec_index_mut::<String>(1);
    let chained_vec = vec_kp.then(vec_index_kp);
    
    if let Some(value) = chained_vec.get_mut(&mut test) {
        *value = "TWO".to_string();
        println!("\n✅ Mutated vec_field[1] to: {}", value);
    }

    // Mutate VecDeque element at index
    let vecdeque_index_kp = containers::for_vecdeque_index_mut::<String>(0);
    let chained_vecdeque = WritableOptionalKeyPath::new(|c: &mut ContainerTest| Some(&mut c.vecdeque_field))
        .then(vecdeque_index_kp);
    
    if let Some(value) = chained_vecdeque.get_mut(&mut test) {
        *value = "A".to_string();
        println!("✅ Mutated vecdeque_field[0] to: {}", value);
    }

    // Mutate HashMap value by key
    let hashmap_key_kp = containers::for_hashmap_key_mut("key1".to_string());
    let chained_hashmap = WritableOptionalKeyPath::new(|c: &mut ContainerTest| Some(&mut c.hashmap_field))
        .then(hashmap_key_kp);
    
    if let Some(value) = chained_hashmap.get_mut(&mut test) {
        *value = 999;
        println!("✅ Mutated hashmap_field[\"key1\"] to: {}", value);
    }

    // Mutate BTreeMap value by key
    let btreemap_key_kp = containers::for_btreemap_key_mut("key2".to_string());
    let chained_btreemap = WritableOptionalKeyPath::new(|c: &mut ContainerTest| Some(&mut c.btreemap_field))
        .then(btreemap_key_kp);
    
    if let Some(value) = chained_btreemap.get_mut(&mut test) {
        *value = 888;
        println!("✅ Mutated btreemap_field[\"key2\"] to: {}", value);
    }

    // Mutate name field directly
    let name_kp = WritableKeyPath::new(|c: &mut ContainerTest| &mut c.name);
    *name_kp.get_mut(&mut test) = "Updated".to_string();
    println!("✅ Mutated name to: {}", test.name);

    println!("\nAfter mutations:");
    println!("  vec_field[1]: {:?}", test.vec_field.as_ref().and_then(|v| v.get(1)));
    println!("  vecdeque_field[0]: {:?}", test.vecdeque_field.get(0));
    println!("  hashmap_field[\"key1\"]: {:?}", test.hashmap_field.get("key1"));
    println!("  btreemap_field[\"key2\"]: {:?}", test.btreemap_field.get("key2"));
    println!("  name: {}", test.name);

    println!("\n✅ All writable container access methods work!");
}

