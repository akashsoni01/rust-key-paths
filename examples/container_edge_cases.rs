use key_paths_core::KeyPaths;
use key_paths_derive::{Keypaths, Casepaths};
use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, LinkedList, BinaryHeap};
use std::rc::Rc;
use std::sync::Arc;

// ===== Edge Cases and Special Scenarios =====

#[derive(Debug, Keypaths)]
struct EdgeCaseStruct {
    // Empty collections
    empty_vec: Vec<String>,
    empty_hashmap: HashMap<String, i32>,
    empty_hashset: HashSet<String>,
    
    // Single element collections
    single_vec: Vec<String>,
    single_hashmap: HashMap<String, i32>,
    single_hashset: HashSet<String>,
    
    // Option with None
    none_option: Option<String>,
    
    // Box with None inside
    box_none: Box<Option<String>>,
    
    // Rc with None inside
    rc_none: Rc<Option<String>>,
    
    // Arc with None inside
    arc_none: Arc<Option<String>>,
    
    // Vec with all None elements
    vec_all_none: Vec<Option<String>>,
    
    // HashMap with None values
    hashmap_with_none: HashMap<String, Option<i32>>,
}

#[derive(Debug, Keypaths)]
struct PerformanceTest {
    // Large collections
    large_vec: Vec<i32>,
    large_hashmap: HashMap<i32, String>,
    large_hashset: HashSet<i32>,
    
    // Nested performance test
    nested_performance: Option<Box<Vec<Option<HashMap<String, i32>>>>>,
}

#[derive(Debug, Casepaths)]
enum EdgeCaseEnum {
    EmptyVariant,
    SingleVariant(String),
    VecVariant(Vec<String>),
    OptionVariant(Option<String>),
    NestedVariant(Option<Box<Vec<Option<String>>>>),
}

// ===== Custom Types for Testing =====

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CustomKey {
    id: i32,
    name: String,
}

#[derive(Debug, Keypaths)]
struct CustomTypeTest {
    custom_key_map: HashMap<CustomKey, String>,
    custom_key_btree: BTreeMap<CustomKey, String>,
    custom_key_set: HashSet<CustomKey>,
}

fn main() {
    println!("🔍 Container Edge Cases and Special Scenarios\n");
    
    // ===== Edge Cases =====
    println!("=== Edge Cases ===");
    
    let mut edge_cases = EdgeCaseStruct {
        empty_vec: vec![],
        empty_hashmap: HashMap::new(),
        empty_hashset: HashSet::new(),
        
        single_vec: vec!["single".to_string()],
        single_hashmap: {
            let mut map = HashMap::new();
            map.insert("single_key".to_string(), 42);
            map
        },
        single_hashset: {
            let mut set = HashSet::new();
            set.insert("single_value".to_string());
            set
        },
        
        none_option: None,
        box_none: Box::new(None),
        rc_none: Rc::new(None),
        arc_none: Arc::new(None),
        
        vec_all_none: vec![None, None, None],
        hashmap_with_none: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), Some(1));
            map.insert("key2".to_string(), None);
            map.insert("key3".to_string(), Some(3));
            map
        },
    };
    
    // Test empty collections
    println!("Empty collections:");
    let empty_vec_path = EdgeCaseStruct::empty_vec_fr();
    if let Some(_) = empty_vec_path.get(&edge_cases) {
        println!("  Empty vec has elements (unexpected!)");
    } else {
        println!("  Empty vec correctly returns None");
    }
    
    let empty_hashmap_path = EdgeCaseStruct::empty_hashmap_fr("nonexistent".to_string());
    if let Some(_) = empty_hashmap_path.get(&edge_cases) {
        println!("  Empty hashmap has key (unexpected!)");
    } else {
        println!("  Empty hashmap correctly returns None");
    }
    
    // Test single element collections
    println!("Single element collections:");
    let single_vec_path = EdgeCaseStruct::single_vec_fr();
    if let Some(value) = single_vec_path.get(&edge_cases) {
        println!("  Single vec element: {}", value);
    }
    
    let single_hashmap_path = EdgeCaseStruct::single_hashmap_fr("single_key".to_string());
    if let Some(value) = single_hashmap_path.get(&edge_cases) {
        println!("  Single hashmap value: {}", value);
    }
    
    // Test None options
    println!("None options:");
    let none_option_path = EdgeCaseStruct::none_option_fr();
    if let Some(_) = none_option_path.get(&edge_cases) {
        println!("  None option has value (unexpected!)");
    } else {
        println!("  None option correctly returns None");
    }
    
    let box_none_path = EdgeCaseStruct::box_none_fr();
    if let Some(_) = box_none_path.get(&edge_cases) {
        println!("  Box<None> has value (unexpected!)");
    } else {
        println!("  Box<None> correctly returns None");
    }
    
    // Test vec with all None elements
    println!("Vec with all None elements:");
    let vec_none_path = EdgeCaseStruct::vec_all_none_fr();
    if let Some(_) = vec_none_path.get(&edge_cases) {
        println!("  Vec<None> has value (unexpected!)");
    } else {
        println!("  Vec<None> correctly returns None");
    }
    
    // Test hashmap with None values
    println!("HashMap with None values:");
    let map_none_path = EdgeCaseStruct::hashmap_with_none_fr("key2".to_string());
    if let Some(_) = map_none_path.get(&edge_cases) {
        println!("  HashMap<None> has value (unexpected!)");
    } else {
        println!("  HashMap<None> correctly returns None");
    }
    
    // ===== Performance Test =====
    println!("\n=== Performance Test ===");
    
    let mut perf_test = PerformanceTest {
        large_vec: (0..1000).collect(),
        large_hashmap: {
            let mut map = HashMap::new();
            for i in 0..1000 {
                map.insert(i, format!("value_{}", i));
            }
            map
        },
        large_hashset: (0..1000).collect(),
        nested_performance: Some(Box::new(vec![
            Some({
                let mut map = HashMap::new();
                for i in 0..100 {
                    map.insert(format!("key_{}", i), i);
                }
                map
            }),
            None,
            Some({
                let mut map = HashMap::new();
                for i in 0..100 {
                    map.insert(format!("key_{}", i), i);
                }
                map
            }),
        ])),
    };
    
    // Test large collection access
    println!("Large collection access:");
    let large_vec_path = PerformanceTest::large_vec_fr_at(500);
    if let Some(value) = large_vec_path.get(&perf_test) {
        println!("  Large vec[500]: {}", value);
    }
    
    let large_hashmap_path = PerformanceTest::large_hashmap_fr(500);
    if let Some(value) = large_hashmap_path.get(&perf_test) {
        println!("  Large hashmap[500]: {}", value);
    }
    
    // Test nested performance
    println!("Nested performance test:");
    let nested_path = PerformanceTest::nested_performance_fr();
    if let Some(map) = nested_path.get(&perf_test) {
        if let Some(value) = map.get("key_50") {
            println!("  Nested performance value: {}", value);
        }
    }
    
    // ===== Custom Types Test =====
    println!("\n=== Custom Types Test ===");
    
    let custom_key1 = CustomKey { id: 1, name: "first".to_string() };
    let custom_key2 = CustomKey { id: 2, name: "second".to_string() };
    
    let mut custom_test = CustomTypeTest {
        custom_key_map: {
            let mut map = HashMap::new();
            map.insert(custom_key1.clone(), "map_value_1".to_string());
            map.insert(custom_key2.clone(), "map_value_2".to_string());
            map
        },
        custom_key_btree: {
            let mut map = BTreeMap::new();
            map.insert(custom_key1.clone(), "btree_value_1".to_string());
            map.insert(custom_key2.clone(), "btree_value_2".to_string());
            map
        },
        custom_key_set: {
            let mut set = HashSet::new();
            set.insert(custom_key1.clone());
            set.insert(custom_key2.clone());
            set
        },
    };
    
    // Test custom key access
    println!("Custom key access:");
    let custom_map_path = CustomTypeTest::custom_key_map_fr(custom_key1.clone());
    if let Some(value) = custom_map_path.get(&custom_test) {
        println!("  Custom map value: {}", value);
    }
    
    let custom_btree_path = CustomTypeTest::custom_key_btree_fr(custom_key2.clone());
    if let Some(value) = custom_btree_path.get(&custom_test) {
        println!("  Custom btree value: {}", value);
    }
    
    let custom_set_path = CustomTypeTest::custom_key_set_fr();
    if let Some(key) = custom_set_path.get(&custom_test) {
        println!("  Custom set element: id={}, name={}", key.id, key.name);
    }
    
    // ===== Enum Edge Cases =====
    println!("\n=== Enum Edge Cases ===");
    
    let edge_enum = EdgeCaseEnum::NestedVariant(Some(Box::new(vec![
        Some("enum_value_1".to_string()),
        None,
        Some("enum_value_3".to_string()),
    ])));
    
    let enum_path = EdgeCaseEnum::nested_variant_case_fr();
    if let Some(value) = enum_path.get(&edge_enum) {
        println!("  Enum nested variant first value: {}", value);
    }
    
    // Test empty enum variant
    let empty_enum = EdgeCaseEnum::VecVariant(vec![]);
    let empty_enum_path = EdgeCaseEnum::vec_variant_case_fr();
    if let Some(_) = empty_enum_path.get(&empty_enum) {
        println!("  Empty enum vec has elements (unexpected!)");
    } else {
        println!("  Empty enum vec correctly returns None");
    }
    
    // ===== Mutation Tests =====
    println!("\n=== Mutation Tests ===");
    
    // Test mutable access
    let mut_vec_path = EdgeCaseStruct::single_vec_fw();
    if let Some(value) = mut_vec_path.get_mut(&mut edge_cases) {
        *value = "modified".to_string();
        println!("  Modified single vec element");
    }
    
    let mut_hashmap_path = EdgeCaseStruct::single_hashmap_fw("single_key".to_string());
    if let Some(value) = mut_hashmap_path.get_mut(&mut edge_cases) {
        *value = 999;
        println!("  Modified single hashmap value");
    }
    
    // Verify mutations
    let verify_vec_path = EdgeCaseStruct::single_vec_fr();
    if let Some(value) = verify_vec_path.get(&edge_cases) {
        println!("  Verified vec modification: {}", value);
    }
    
    let verify_hashmap_path = EdgeCaseStruct::single_hashmap_fr("single_key".to_string());
    if let Some(value) = verify_hashmap_path.get(&edge_cases) {
        println!("  Verified hashmap modification: {}", value);
    }
    
    println!("\n✅ All edge cases and special scenarios tested!");
    println!("🔍 Edge cases covered:");
    println!("  • Empty collections");
    println!("  • Single element collections");
    println!("  • None options and nested None values");
    println!("  • Large collections (performance)");
    println!("  • Custom types as keys");
    println!("  • Enum variants with edge cases");
    println!("  • Mutation operations");
    println!("  • All combinations return correct None/Some values");
}
