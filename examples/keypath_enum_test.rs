use keypaths_proc::Keypath;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

// todo add support for more cointainers for Keypaths macro
#[derive(Debug, Keypath)]
enum Message {
    // Unit variant
    Ping,

    // Single-field tuple variants with different container types
    Text(String),
    Number(i32),
    Email(Option<String>),
    Tags(Vec<String>),
    Metadata(HashMap<String, String>),
    Recipients(HashSet<String>),
    Queue(VecDeque<String>),
    Priority(BinaryHeap<i32>),
    Content(Box<String>),
    Reference(Rc<String>),
    Shared(Arc<HashMap<String, String>>),

    // Multi-field tuple variant
    Coordinate(f64, f64),

    // Named field variant
    User { name: String, age: u32 },
}

fn main() {
    println!("=== Enum Keypaths Access ===");

    // Test unit variant
    let ping = Message::Ping;
    if let Some(msg) = Message::ping().get(&ping) {
        println!("Unit variant: {:?}", msg);
    }

    // Test single-field tuple variants
    let text_msg = Message::Text("Hello World".to_string());
    if let Some(text) = Message::text().get(&text_msg) {
        println!("Text message: {}", text);
    }

    let number_msg = Message::Number(42);
    if let Some(num) = Message::number().get(&number_msg) {
        println!("Number message: {}", num);
    }

    let email_msg = Message::Email(Some("user@example.com".to_string()));
    if let Some(email) = Message::email().get(&email_msg) {
        println!("Email message: {}", email);
    }

    let tags_msg = Message::Tags(vec!["urgent".to_string(), "important".to_string()]);
    if let Some(tag) = Message::tags().get(&tags_msg) {
        println!("First tag: {}", tag);
    }

    let metadata_msg = Message::Metadata({
        let mut map = HashMap::new();
        map.insert("sender".to_string(), "akash".to_string());
        map.insert("timestamp".to_string(), "2024-01-01".to_string());
        map
    });
    if let Some(metadata) = Message::metadata().get(&metadata_msg) {
        println!("Metadata: {:?}", metadata);
    }

    let recipients_msg = Message::Recipients({
        let mut set = HashSet::new();
        set.insert("bob".to_string());
        set.insert("charlie".to_string());
        set
    });
    if let Some(recipient) = Message::recipients().get(&recipients_msg) {
        println!("A recipient: {}", recipient);
    }

    let queue_msg = Message::Queue({
        let mut deque = VecDeque::new();
        deque.push_back("task1".to_string());
        deque.push_back("task2".to_string());
        deque
    });
    if let Some(task) = Message::queue().get(&queue_msg) {
        println!("Front task: {}", task);
    }

    let priority_msg = Message::Priority({
        let mut heap = BinaryHeap::new();
        heap.push(10);
        heap.push(5);
        heap.push(15);
        heap
    });
    if let Some(priority) = Message::priority().get(&priority_msg) {
        println!("Peek priority: {}", priority);
    }

    let content_msg = Message::Content(Box::new("Important content".to_string()));
    if let Some(content) = Message::content().get(&content_msg) {
        println!("Content: {}", content);
    }

    let reference_msg = Message::Reference(Rc::new("Shared reference".to_string()));
    if let Some(reference) = Message::reference().get(&reference_msg) {
        println!("Reference: {}", reference);
    }

    let shared_msg = Message::Shared(Arc::new({
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());
        map
    }));
    if let Some(shared) = Message::shared().get(&shared_msg) {
        println!("Shared data keys: {:?}", shared.keys().collect::<Vec<_>>());
    }

    // Test multi-field tuple variant
    let coord_msg = Message::Coordinate(10.5, 20.3);
    if let Some(coord) = Message::coordinate().get(&coord_msg) {
        println!("Coordinate message: {:?}", coord);
    }

    // Test named field variant
    let user_msg = Message::User {
        name: "Akash".to_string(),
        age: 30,
    };
    if let Some(user) = Message::user().get(&user_msg) {
        println!("User message: {:?}", user);
    }

    // Test non-matching variants
    let ping_msg = Message::Ping;
    if let Some(text) = Message::text().get(&ping_msg) {
        println!("This should not print: {}", text);
    } else {
        println!("✓ Correctly returned None for non-matching variant");
    }

    println!("\n=== Enum Keypaths Types ===");
    println!(
        "ping() returns: KeyPath<Message, Message, impl for<\'r> Fn(&\'r Message) -> &\'r Message> (readable)"
    );
    println!(
        "text() returns: KeyPath<Message, String, impl for<\'r> Fn(&\'r Message) -> &\'r String> (failable readable)"
    );
    println!(
        "number() returns: KeyPath<Message, i32, impl for<\'r> Fn(&\'r Message) -> &\'r i32> (failable readable)"
    );
    println!(
        "email() returns: KeyPath<Message, String, impl for<\'r> Fn(&\'r Message) -> &\'r String> (failable readable)"
    );
    println!(
        "tags() returns: KeyPath<Message, String, impl for<\'r> Fn(&\'r Message) -> &\'r String> (failable readable)"
    );
    println!(
        "metadata() returns: KeyPath<Message, HashMap<String, String, impl for<\'r> Fn(&\'r Message) -> &\'r HashMap<String, String>> (failable readable)"
    );
    println!(
        "coordinate() returns: KeyPath<Message, Message, impl for<\'r> Fn(&\'r Message) -> &\'r Message> (failable readable)"
    );
    println!(
        "user() returns: KeyPath<Message, Message, impl for<\'r> Fn(&\'r Message) -> &\'r Message> (failable readable)"
    );

    println!("\n=== All enum keypath tests completed successfully! ===");
}
