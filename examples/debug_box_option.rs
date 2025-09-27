use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct DebugTest {
    box_option_field: Box<Option<String>>,
}

fn main() {
    println!("Debug Box<Option<String>> test");
    
    // Try to use the generated methods
    let _box_option_r = DebugTest::box_option_field_r();
    let _box_option_w = DebugTest::box_option_field_w();
    let _box_option_fr = DebugTest::box_option_field_fr();
    let _box_option_fw = DebugTest::box_option_field_fw();
    
    println!("BoxOption methods generated successfully!");
}
