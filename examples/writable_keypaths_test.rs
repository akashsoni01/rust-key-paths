use keypaths_proc::WritableKeypaths;
use std::collections::{HashMap, HashSet, BTreeMap, VecDeque, LinkedList, BinaryHeap};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, WritableKeypaths)]
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

#[derive(Debug, WritableKeypaths)]
struct TupleStruct(String, Option<i32>, Vec<f64>);

fn main() {
    let mut user = User {
        name: "Alice".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
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

    println!("=== Initial User State ===");
    println!("Name: {}", user.name);
    println!("Age: {}", user.age);
    println!("Email: {:?}", user.email);
    println!("Tags: {:?}", user.tags);
    println!("Preferences: {:?}", user.preferences);
    println!("Scores: {:?}", user.scores);

    // Test basic writable keypaths
    println!("\n=== Basic Writable Keypaths ===");
    let name_path = User::name_w();
    let name_ref = name_path.get_mut(&mut user);
    {
        *name_ref = "Alice Updated".to_string();
        println!("Updated name to: {}", name_ref);
    }

    let age_path = User::age_w();
    let age_ref = age_path.get_mut(&mut user);
    {
        *age_ref = 31;
        println!("Updated age to: {}", age_ref);
    }

    // Test failable writable keypaths for Option
    println!("\n=== Failable Writable Keypaths (Option) ===");
    let email_path = User::email_fw();
    if let Some(email_ref) = email_path.get_mut(&mut user) {
        *email_ref = "akash.updated@example.com".to_string();
        println!("Updated email to: {}", email_ref);
    }

    // Test failable writable keypaths for Vec
    println!("\n=== Failable Writable Keypaths (Vec) ===");
    let first_tag_path = User::tags_fw();
    if let Some(tag_ref) = first_tag_path.get_mut(&mut user) {
        *tag_ref = "senior_developer".to_string();
        println!("Updated first tag to: {}", tag_ref);
    }
    
    let tag_at_1_path = User::tags_fw_at(1);
    if let Some(tag_ref) = tag_at_1_path.get_mut(&mut user) {
        *tag_ref = "rustacean".to_string();
        println!("Updated tag at index 1 to: {}", tag_ref);
    }

    // Test failable writable keypaths for HashMap
    println!("\n=== Failable Writable Keypaths (HashMap) ===");
    let theme_path = User::preferences_fw("theme".to_string());
    let theme_ref = theme_path.get_mut(&mut user);
    {
        *theme_ref = "light".to_string();
        println!("Updated theme preference to: {}", theme_ref);
    }

    // Test failable writable keypaths for BTreeMap
    println!("\n=== Failable Writable Keypaths (BTreeMap) ===");
    let math_score_path = User::scores_fw("math".to_string());
    let score_ref = math_score_path.get_mut(&mut user);
    {
        *score_ref = 98;
        println!("Updated math score to: {}", score_ref);
    }

    // Test failable writable keypaths for VecDeque
    println!("\n=== Failable Writable Keypaths (VecDeque) ===");
    let front_history_path = User::history_fw();
    let history_ref = front_history_path.get_mut(&mut user);
    {
        *history_ref = "updated_login".to_string();
        println!("Updated front history to: {}", history_ref);
    }

    // Test failable writable keypaths for LinkedList
    println!("\n=== Failable Writable Keypaths (LinkedList) ===");
    let front_note_path = User::notes_fw();
    let note_ref = front_note_path.get_mut(&mut user);
    {
        *note_ref = "Updated important note".to_string();
        println!("Updated front note to: {}", note_ref);
    }

    // Test writable keypaths for BinaryHeap (container-level only)
    println!("\n=== Writable Keypaths (BinaryHeap) ===");
    let priority_queue_path = User::priority_queue_w();
    let queue_ref = priority_queue_path.get_mut(&mut user);
    {
        queue_ref.push(20);
        println!("Added new priority to queue: 20");
    }

    // Test Box dereferencing
    println!("\n=== Box Dereferencing ===");
    let bio_path = User::profile_w();
    let profile_ref = bio_path.get_mut(&mut user);
    {
        profile_ref.bio = "Senior Software Developer".to_string();
        println!("Updated profile bio to: {}", profile_ref.bio);
    }

    // Test tuple struct
    println!("\n=== Tuple Struct ===");
    let mut tuple = TupleStruct("test".to_string(), Some(42), vec![1.0, 2.0, 3.0]);
    
    let f0_path = TupleStruct::f0_w();
    if let Some(f0_ref) = f0_path.get_mut(&mut tuple) {
        *f0_ref = "updated_test".to_string();
        println!("Updated tuple f0 to: {}", f0_ref);
    }
    
    let f1_fw_path = TupleStruct::f1_fw();
    if let Some(f1_ref) = f1_fw_path.get_mut(&mut tuple) {
        *f1_ref = 100;
        println!("Updated tuple f1 (Option) to: {}", f1_ref);
    }
    
    let f2_fw_path = TupleStruct::f2_fw();
    if let Some(f2_ref) = f2_fw_path.get_mut(&mut tuple) {
        *f2_ref = 99.9;
        println!("Updated tuple f2 (Vec first) to: {}", f2_ref);
    }

    println!("\n=== Final User State ===");
    println!("Name: {}", user.name);
    println!("Age: {}", user.age);
    println!("Email: {:?}", user.email);
    println!("Tags: {:?}", user.tags);
    println!("Preferences: {:?}", user.preferences);
    println!("Scores: {:?}", user.scores);
    println!("Profile bio: {}", user.profile.bio);
    println!("Tuple: {:?}", tuple);

    println!("\n=== All writable keypath tests completed successfully! ===");
}
