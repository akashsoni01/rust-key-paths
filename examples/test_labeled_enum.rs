use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::Casepaths;

#[derive(Debug, Clone, Casepaths)]
enum TestEnum {
    // Unit variant
    Unit,
    
    // Single-field tuple variant
    Single(f32),
    
    // Multi-field tuple variant
    Color(f32, f32, f32, f32),
    
    // Labeled variant
    User { 
        name: String, 
        age: u32 
    },
    
    // Complex labeled variant
    Person { 
        name: String, 
        age: u32, 
        address: Option<String> 
    },
}

fn main() {
    println!("Testing Casepaths derive macro with complex enum variants");
    
    // Test unit variant
    let unit_enum = TestEnum::Unit;
    let unit_case = TestEnum::unit_case_r();
    if let Some(()) = unit_case.get(&unit_enum) {
        println!("Unit variant works!");
    }
    
    // Test single-field tuple variant
    let single_enum = TestEnum::Single(42.0);
    let single_case = TestEnum::single_case_r();
    if let Some(value) = single_case.get(&single_enum) {
        println!("Single variant: {}", value);
    }
    
    // Test multi-field tuple variant
    let color_enum = TestEnum::Color(1.0, 0.5, 0.0, 1.0);
    let color_case = TestEnum::color_case_r();
    if let Some((r, g, b, a)) = color_case.get(&color_enum) {
        println!("Color: RGBA({}, {}, {}, {})", r, g, b, a);
    }
    
    // Test writable variants for complex enum variants
    // Note: Complex enum variants (multi-field tuples and labeled structs) 
    // cannot support traditional get_mut() because we can't return mutable 
    // references to individual fields in a tuple. Use the failable_owned 
    // approach to extract values, modify them, and reconstruct the enum.
    
    let user_enum = TestEnum::User { 
        name: "Charlie".to_string(), 
        age: 35 
    };
    let user_case_w = TestEnum::user_case_w();
    
    // Extract the fields, modify them, and reconstruct
    if let Some((name, age)) = user_case_w.get_failable_owned(user_enum) {
        let updated_user = TestEnum::User { name, age: 36 };
        println!("Updated user: {:?}", updated_user);
    }
    
    // Alternative: Use the readable keypath for the same functionality
    let user_enum2 = TestEnum::User { 
        name: "David".to_string(), 
        age: 40 
    };
    let user_case_r = TestEnum::user_case_r();
    if let Some((name, age)) = user_case_r.get_failable_owned(user_enum2) {
        let updated_user = TestEnum::User { name, age: 41 };
        println!("Updated user via readable keypath: {:?}", updated_user);
    }
}