use key_paths_core::{KeyPaths, WithContainer};
use key_paths_derive::{Keypaths, keypath_suggestion, keypath_help};
use std::sync::Arc;

#[derive(Keypaths)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    println!("🔧 KeyPaths Suggestion Macros Example");
    println!("=====================================");
    
    // Create a regular keypath
    let name_keypath = User::name_r();
    
    // This would normally cause a type mismatch error:
    // let arc_user = Arc::new(User { name: "Alice".to_string(), age: 30 });
    // let name = name_keypath.get_ref(&arc_user); // ❌ Type mismatch!
    
    // Instead, use the adapter method:
    let arc_user = Arc::new(User { name: "Alice".to_string(), age: 30 });
    let arc_name_keypath = name_keypath.for_arc();
    
    // Now it works correctly:
    if let Some(name) = arc_name_keypath.get_ref(&&arc_user) {
        println!("✅ Name from Arc<User>: {}", name);
    }
    
    // Demonstrate the suggestion macros (these will cause compile errors with helpful messages)
    println!("\n📝 To see helpful error messages, uncomment one of these lines:");
    println!("   keypath_suggestion!(Arc<SomeStruct> KeyPaths<SomeStruct, u64>);");
    println!("   keypath_help!();");
    println!("   keypath_help!(Arc);");
    
    // Uncomment any of these to see the helpful error messages:
    // keypath_suggestion!(Arc<SomeStruct> KeyPaths<SomeStruct, u64>);
    // keypath_help!();
    // keypath_help!(Arc);
    
    println!("\n💡 The suggestion macros provide helpful error messages when you have type mismatches!");
    println!("   Use .for_arc(), .for_box(), .for_rc(), .for_option(), .for_result() adapter methods.");
}
