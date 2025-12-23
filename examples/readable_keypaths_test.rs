use key_paths_derive::ReadableKeypaths;
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque, LinkedList, BinaryHeap};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, ReadableKeypaths)]
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

#[derive(Debug, ReadableKeypaths)]
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

    // Test basic readable keypaths
    println!("=== Basic Readable Keypaths ===");
    let name_path = User::name_r();
    println!("Name: {:?}", name_path.get(&user));
    
    let age_path = User::age_r();
    println!("Age: {:?}", age_path.get(&user));

    // Test failable readable keypaths for Option
    println!("\n=== Failable Readable Keypaths (Option) ===");
    let email_path = User::email_fr();
    if let Some(email) = email_path.get(&user) {
        println!("Email: {}", email);
    } else {
        println!("No email found");
    }

    // Test failable readable keypaths for Vec
    println!("\n=== Failable Readable Keypaths (Vec) ===");
    let first_tag_path = User::tags_fr();
    if let Some(tag) = first_tag_path.get(&user) {
        println!("First tag: {}", tag);
    }
    
    let tag_at_1_path = User::tags_fr_at(1);
    if let Some(tag) = tag_at_1_path.get(&user) {
        println!("Tag at index 1: {}", tag);
    }

    // Test failable readable keypaths for HashMap
    println!("\n=== Failable Readable Keypaths (HashMap) ===");
    let theme_path = User::preferences_fr("theme".to_string());
    if let Some(theme) = theme_path.get(&user) {
        println!("Theme preference: {}", theme);
    }

    // Test failable readable keypaths for HashSet
    println!("\n=== Failable Readable Keypaths (HashSet) ===");
    let first_friend_path = User::friends_fr();
    if let Some(friend) = first_friend_path.get(&user) {
        println!("First friend: {}", friend);
    }

    // Test failable readable keypaths for BTreeMap
    println!("\n=== Failable Readable Keypaths (BTreeMap) ===");
    let math_score_path = User::scores_fr("math".to_string());
    if let Some(score) = math_score_path.get(&user) {
        println!("Math score: {}", score);
    }

    // Test failable readable keypaths for VecDeque
    println!("\n=== Failable Readable Keypaths (VecDeque) ===");
    let front_history_path = User::history_fr();
    if let Some(history) = front_history_path.get(&user) {
        println!("Front history: {}", history);
    }

    // Test failable readable keypaths for LinkedList
    println!("\n=== Failable Readable Keypaths (LinkedList) ===");
    let front_note_path = User::notes_fr();
    if let Some(note) = front_note_path.get(&user) {
        println!("Front note: {}", note);
    }

    // Test failable readable keypaths for BinaryHeap
    println!("\n=== Failable Readable Keypaths (BinaryHeap) ===");
    let peek_priority_path = User::priority_queue_fr();
    if let Some(priority) = peek_priority_path.get(&user) {
        println!("Peek priority: {}", priority);
    }

    // Test Box dereferencing
    println!("\n=== Box Dereferencing ===");
    let bio_path = User::profile_r();
    if let Some(profile) = bio_path.get(&user) {
        println!("Profile bio: {}", profile.bio);
    }

    // Test Rc dereferencing
    println!("\n=== Rc Dereferencing ===");
    let avatar_path = User::avatar_r();
    if let Some(avatar) = avatar_path.get(&user) {
        println!("Avatar: {}", avatar);
    }

    // Test Arc dereferencing
    println!("\n=== Arc Dereferencing ===");
    let metadata_path = User::metadata_r();
    if let Some(metadata) = metadata_path.get(&user) {
        println!("Metadata keys: {:?}", metadata.keys().collect::<Vec<_>>());
    }

    // Test tuple struct
    println!("\n=== Tuple Struct ===");
    let tuple = TupleStruct("test".to_string(), Some(42), vec![1.0, 2.0, 3.0]);
    
    let f0_path = TupleStruct::f0_r();
    println!("Tuple f0: {:?}", f0_path.get(&tuple));
    
    let f1_fr_path = TupleStruct::f1_fr();
    if let Some(value) = f1_fr_path.get(&tuple) {
        println!("Tuple f1 (Option): {}", value);
    }
    
    let f2_fr_path = TupleStruct::f2_fr();
    if let Some(value) = f2_fr_path.get(&tuple) {
        println!("Tuple f2 (Vec first): {}", value);
    }

    println!("\n=== All tests completed successfully! ===");
}
