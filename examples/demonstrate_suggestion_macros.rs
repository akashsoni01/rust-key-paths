use key_paths_derive::{Keypaths, keypath_suggestion, keypath_help};

#[derive(Keypaths)]
struct SomeStruct {
    value: u64,
}

fn main() {
    println!("🔧 Demonstrating KeyPaths Suggestion Macros");
    println!("===========================================");
    
    // These macros will cause compile errors with helpful suggestions
    // Uncomment any of these lines to see the helpful error messages:
    
    // 1. General help message
    // keypath_help!();
    
    // 2. Help for specific container type
    // keypath_help!(Arc);
    
    // 3. Specific suggestion for Arc type mismatch
    // keypath_suggestion!(Arc<SomeStruct> KeyPaths<SomeStruct, u64>);
    
    // 8. Specific suggestion for Mutex type mismatch
    // keypath_suggestion!(Mutex<SomeStruct> KeyPaths<SomeStruct, u64>);
    
    // 9. Specific suggestion for RwLock type mismatch
    // keypath_suggestion!(RwLock<SomeStruct> KeyPaths<SomeStruct, u64>);
    
    // 1. General help message
    keypath_help!();
    
    // 4. Specific suggestion for Box type mismatch
    // keypath_suggestion!(Box<SomeStruct> KeyPaths<SomeStruct, u64>);
    
    // 5. Specific suggestion for Rc type mismatch
    // keypath_suggestion!(Rc<SomeStruct> KeyPaths<SomeStruct, u64>);
    
    // 6. Specific suggestion for Option type mismatch
    // keypath_suggestion!(Option<SomeStruct> KeyPaths<SomeStruct, u64>);
    
    // 7. Specific suggestion for Result type mismatch
    // keypath_suggestion!(Result<SomeStruct, String> KeyPaths<SomeStruct, u64>);
    
    // 8. Specific suggestion for Mutex type mismatch
    // keypath_suggestion!(Mutex<SomeStruct> KeyPaths<SomeStruct, u64>);
    
    // 9. Specific suggestion for RwLock type mismatch
    // keypath_suggestion!(RwLock<SomeStruct> KeyPaths<SomeStruct, u64>);
    
    println!("💡 Uncomment any of the lines above to see helpful error messages!");
    println!("   These macros provide suggestions when you have type mismatches with container types.");
    println!("   Use adapter methods like .for_arc(), .for_box(), .for_rc(), .for_option(), .for_result()");
    println!("   For Mutex/RwLock, use .with_mutex() and .with_rwlock() from WithContainer trait (no cloning)");
}
