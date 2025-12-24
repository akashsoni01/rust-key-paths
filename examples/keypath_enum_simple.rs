use keypaths_proc::Keypaths;

#[derive(Debug, Keypaths)]
enum Status {
    // Unit variant
    Loading,
    
    // Single-field tuple variants
    Success(String),
    // not supported 
    Error(String),
    // Error(Option<String>),
    // Data(Vec<i32>),
    
    // Multi-field tuple variant
    Position(f64, f64),
    
    // Named field variant
    User { name: String, age: Option<u32> },
}

fn main() {
    println!("=== Enum Keypaths Access ===");
    
    // Test unit variant
    let loading = Status::Loading;
    if let Some(status) = Status::loading_case_r().get(&loading) {
        println!("Status: {:?}", status);
    }
    
    // Test single-field tuple variants
    let success = Status::Success("Operation completed".to_string());
    if let Some(message) = Status::success_case_r().get(&success) {
        println!("Success message: {}", message);
    }
    
    let error = Status::Error("Something went wrong".to_string());
    if let Some(error_msg) = Status::error_case_r().get(&error) {
        println!("Error message: {}", error_msg);
    }
    
    // let data = Status::Data(vec![1, 2, 3, 4, 5]);
    // if let Some(first_value) = Status::data().get(&data) {
    //     println!("First data value: {}", first_value);
    // }
    
    // Test multi-field tuple variant
    let position = Status::Position(10.5, 20.3);
    if let Some(pos) = Status::position_f0_r().get(&position) {
        println!("Position: {:?}", pos);
    }
    
    // Test named field variant
    let user = Status::User { 
        name: "Alice".to_string(), 
        age: Some(30) 
    };
    if let Some(user_status_name) = Status::user_name_r().get(&user) {
        println!("User status name : {:?}", user_status_name);
    }
    
    // Test non-matching variants
    let loading_status = Status::Loading;
    if let Some(message) = Status::success_case_r().get(&loading_status) {
        println!("This should not print: {}", message);
    } else {
        println!("✓ Correctly returned None for non-matching variant");
    }
    
    println!("\n=== Keypaths Types ===");
    println!("loading() returns: KeyPath<Status, Status, impl for<\'r> Fn(&\'r Status) -> &\'r Status> (readable)");
    println!("success() returns: KeyPath<Status, String, impl for<\'r> Fn(&\'r Status) -> &\'r String> (failable readable)");
    println!("error() returns: KeyPath<Status, String, impl for<\'r> Fn(&\'r Status) -> &\'r String> (failable readable)");
    println!("data() returns: KeyPath<Status, i32, impl for<\'r> Fn(&\'r Status) -> &\'r i32> (failable readable)");
    println!("position() returns: KeyPath<Status, Status, impl for<\'r> Fn(&\'r Status) -> &\'r Status> (failable readable)");
    println!("user() returns: KeyPath<Status, Status, impl for<\'r> Fn(&\'r Status) -> &\'r Status> (failable readable)");
}
