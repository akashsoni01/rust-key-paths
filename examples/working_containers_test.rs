use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct WorkingTest {
    // Basic types
    string_field: String,
    int_field: i32,
    bool_field: bool,
    
    // Basic containers that should work
    option_string: Option<String>,
    vec_string: Vec<String>,
    box_string: Box<String>,
}

fn main() {
    println!("Working containers test");
    
    // Test basic types
    let _string_path = WorkingTest::string_field_r();
    let _int_path = WorkingTest::int_field_r();
    let _bool_path = WorkingTest::bool_field_r();
    
    // Test basic containers
    let _option_path = WorkingTest::option_string_fr();
    let _vec_path = WorkingTest::vec_string_r();
    let _box_path = WorkingTest::box_string_r();
    
    println!("All working containers generated successfully!");
}
