use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
enum TestEnum {
    // Labeled enum variant
    User { name: String, age: u32 },
}

fn main() {
    let user = TestEnum::User { 
        name: "Alice".to_string(), 
        age: 30 
    };
    
    // Test the generated methods
    let name_kp = TestEnum::user_name_r();
    let age_kp = TestEnum::user_age_r();
    
    if let Some(name) = name_kp.get(&user) {
        println!("Name: {}", name);
    }
    
    if let Some(age) = age_kp.get(&user) {
        println!("Age: {}", age);
    }
}
