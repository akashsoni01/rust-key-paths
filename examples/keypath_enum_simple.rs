use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
enum Status {
    // Unit variant
    Loading,
    
    // Single-field tuple variants
    Success(String),
    Error(Option<String>),
    Data(Vec<i32>),
    
    // Multi-field tuple variant
    Position(f64, f64),
    
    // Named field variant
    User { name: String, age: u32 },
}

fn main() {
    println!("=== Enum Keypaths Access ===");
    
    // Test unit variant
    let loading = Status::Loading;
    if let Some(status) = Status::loading().get(&loading) {
        println!("Status: {:?}", status);
    }
    
    // Test single-field tuple variants
    let success = Status::Success("Operation completed".to_string());
    if let Some(message) = Status::success().get(&success) {
        println!("Success message: {}", message);
    }
    
    let error = Status::Error(Some("Something went wrong".to_string()));
    if let Some(error_msg) = Status::error().get(&error) {
        println!("Error message: {}", error_msg);
    }
    
    let data = Status::Data(vec![1, 2, 3, 4, 5]);
    if let Some(first_value) = Status::data().get(&data) {
        println!("First data value: {}", first_value);
    }
    
    // Test multi-field tuple variant
    let position = Status::Position(10.5, 20.3);
    if let Some(pos) = Status::position().get(&position) {
        println!("Position: {:?}", pos);
    }
    
    // Test named field variant
    let user = Status::User { 
        name: "Alice".to_string(), 
        age: 30 
    };
    if let Some(user_status) = Status::user().get(&user) {
        println!("User status: {:?}", user_status);
    }
    
    // Test non-matching variants
    let loading_status = Status::Loading;
    if let Some(message) = Status::success().get(&loading_status) {
        println!("This should not print: {}", message);
    } else {
        println!("✓ Correctly returned None for non-matching variant");
    }
    
    println!("\n=== Keypaths Types ===");
    println!("loading() returns: KeyPaths<Status, Status> (readable)");
    println!("success() returns: KeyPaths<Status, String> (failable readable)");
    println!("error() returns: KeyPaths<Status, String> (failable readable)");
    println!("data() returns: KeyPaths<Status, i32> (failable readable)");
    println!("position() returns: KeyPaths<Status, Status> (failable readable)");
    println!("user() returns: KeyPaths<Status, Status> (failable readable)");
}
