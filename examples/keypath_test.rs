use key_paths_derive::Keypaths;
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque, LinkedList, BinaryHeap};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Keypaths)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
    tags: Vec<String>,
    preferences: HashMap<String, String>,
    friends: HashSet<String>,
    scores: BTreeMap<String, u32>,
    history: VecDeque<String>,
    notes: LinkedList<String>,
    priority_queue: BinaryHeap<i32>,
    profile: Box<UserProfile>,
    avatar: Rc<String>,
    metadata: Arc<HashMap<String, String>>,
}

#[derive(Debug)]
struct UserProfile {
    bio: String,
    location: String,
}

#[derive(Debug, Keypaths)]
struct TupleStruct(String, Option<i32>, Vec<f64>);

fn main() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: Some("alice@example.com".to_string()),
        tags: vec!["developer".to_string(), "rust".to_string()],
        preferences: {
            let mut map = HashMap::new();
            map.insert("theme".to_string(), "dark".to_string());
            map.insert("language".to_string(), "en".to_string());
            map
        },
        friends: {
            let mut set = HashSet::new();
            set.insert("bob".to_string());
            set.insert("charlie".to_string());
            set
        },
        scores: {
            let mut map = BTreeMap::new();
            map.insert("math".to_string(), 95);
            map.insert("science".to_string(), 88);
            map
        },
        history: {
            let mut deque = VecDeque::new();
            deque.push_back("login".to_string());
            deque.push_back("view_profile".to_string());
            deque
        },
        notes: {
            let mut list = LinkedList::new();
            list.push_back("Important note".to_string());
            list.push_back("Another note".to_string());
            list
        },
        priority_queue: {
            let mut heap = BinaryHeap::new();
            heap.push(10);
            heap.push(5);
            heap.push(15);
            heap
        },
        profile: Box::new(UserProfile {
            bio: "Software developer".to_string(),
            location: "San Francisco".to_string(),
        }),
        avatar: Rc::new("avatar.png".to_string()),
        metadata: Arc::new({
            let mut map = HashMap::new();
            map.insert("created_at".to_string(), "2024-01-01".to_string());
            map
        }),
    };

    println!("=== Smart Keypaths Access ===");
    
    // Basic types - readable keypath
    println!("Name: {:?}", User::name().get(&user));
    println!("Age: {:?}", User::age().get(&user));

    // Option<T> - failable readable keypath to inner type
    if let Some(email) = User::email().get(&user) {
        println!("Email: {}", email);
    } else {
        println!("No email found");
    }

    // Vec<T> - failable readable keypath to first element
    if let Some(tag) = User::tags().get(&user) {
        println!("First tag: {}", tag);
    }

    // HashMap<K,V> - readable keypath to container
    if let Some(preferences) = User::preferences().get(&user) {
        println!("Preferences: {:?}", preferences);
    }

    // HashSet<T> - failable readable keypath to any element
    if let Some(friend) = User::friends().get(&user) {
        println!("A friend: {}", friend);
    }

    // BTreeMap<K,V> - readable keypath to container
    if let Some(scores) = User::scores().get(&user) {
        println!("Scores: {:?}", scores);
    }

    // VecDeque<T> - failable readable keypath to front element
    if let Some(history) = User::history().get(&user) {
        println!("Front history: {}", history);
    }

    // LinkedList<T> - failable readable keypath to front element
    if let Some(note) = User::notes().get(&user) {
        println!("Front note: {}", note);
    }

    // BinaryHeap<T> - failable readable keypath to peek element
    if let Some(priority) = User::priority_queue().get(&user) {
        println!("Peek priority: {}", priority);
    }

    // Box<T> - readable keypath to inner type
    if let Some(profile) = User::profile().get(&user) {
        println!("Profile bio: {}", profile.bio);
    }

    // Rc<T> - readable keypath to inner type
    if let Some(avatar) = User::avatar().get(&user) {
        println!("Avatar: {}", avatar);
    }

    // Arc<T> - readable keypath to inner type
    if let Some(metadata) = User::metadata().get(&user) {
        println!("Metadata keys: {:?}", metadata.keys().collect::<Vec<_>>());
    }

    // Test tuple struct
    println!("\n=== Tuple Struct ===");
    let tuple = TupleStruct("test".to_string(), Some(42), vec![1.0, 2.0, 3.0]);
    
    if let Some(f0) = TupleStruct::f0().get(&tuple) {
        println!("Tuple f0: {}", f0);
    }
    
    if let Some(f1) = TupleStruct::f1().get(&tuple) {
        println!("Tuple f1 (Option): {}", f1);
    }
    
    if let Some(f2) = TupleStruct::f2().get(&tuple) {
        println!("Tuple f2 (Vec first): {}", f2);
    }

    println!("\n=== All smart keypath tests completed successfully! ===");
}
