use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct SimpleTest {
    // Basic types that should work
    string_field: String,
    int_field: i32,
    bool_field: bool,

    // Simple containers
    option_string: Option<String>,
    vec_string: Vec<String>,
    box_string: Box<String>,
}

fn main() {
    println!("Simple working test");

    // Test basic types
    let _string_path = SimpleTest::string_field();
    let _int_path = SimpleTest::int_field();
    let _bool_path = SimpleTest::bool_field();

    // Test simple containers
    let _option_path = SimpleTest::option_string_fr();
    let _vec_path = SimpleTest::vec_string();
    let _box_path = SimpleTest::box_string();

    println!("All simple keypaths generated successfully!");
}
