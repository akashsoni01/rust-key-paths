use key_paths_derive::Casepaths;

#[derive(Debug, Clone, Casepaths)]
enum TestEnum {
    // Unit variant
    Unit,
    
    // Single-field tuple variant
    Single(String),
    
    // Multi-field tuple variant
    Point(f32, f32),
    
    // Labeled variant
    User { name: String, age: u32 },
    
    // Complex labeled variant
    Person { 
        name: String, 
        age: u32, 
        address: Option<String> 
    },
    
    // Mixed tuple variant
    Color(u8, u8, u8, u8), // RGBA
}

fn main() {
    // Test unit variant
    let unit_enum = TestEnum::Unit;
    let unit_case = TestEnum::unit_case_r();
    if let Some(()) = unit_case.get(&unit_enum) {
        println!("Unit variant works!");
    }
    
    // Test single-field tuple variant
    let single_enum = TestEnum::Single("Hello".to_string());
    let single_case = TestEnum::single_case_r();
    if let Some(value) = single_case.get(&single_enum) {
        println!("Single variant: {}", value);
    }
    
    // Test multi-field tuple variant
    let point_enum = TestEnum::Point(1.0, 2.0);
    let point_case = TestEnum::point_case_r();
    if let Some((x, y)) = point_case.get(&point_enum) {
        println!("Point: ({}, {})", x, y);
    }
    
    // Test labeled variant
    let user_enum = TestEnum::User { 
        name: "Alice".to_string(), 
        age: 30 
    };
    let user_case = TestEnum::user_case_r();
    if let Some((name, age)) = user_case.get(&user_enum) {
        println!("User: {} is {} years old", name, age);
    }
    
    // Test complex labeled variant
    let person_enum = TestEnum::Person { 
        name: "Bob".to_string(), 
        age: 25, 
        address: Some("123 Main St".to_string()) 
    };
    let person_case = TestEnum::person_case_r();
    if let Some((name, age, address)) = person_case.get(&person_enum) {
        println!("Person: {} is {} years old, lives at {:?}", name, age, address);
    }
    
    // Test mixed tuple variant
    let color_enum = TestEnum::Color(255, 128, 64, 255);
    let color_case = TestEnum::color_case_r();
    if let Some((r, g, b, a)) = color_case.get(&color_enum) {
        println!("Color: RGBA({}, {}, {}, {})", r, g, b, a);
    }
    
    // Test writable variants
    let mut user_enum = TestEnum::User { 
        name: "Charlie".to_string(), 
        age: 35 
    };
    let user_case_w = TestEnum::user_case_w();
    if let Some((name, age)) = user_case_w.get_mut(&mut user_enum) {
        *age = 36;
        println!("Updated user age to: {}", age);
    }
    
    println!("Final user: {:?}", user_enum);
}
