use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct MinimalTest {
    box_option_field: Box<Option<String>>,
}

fn main() {
    println!("Minimal test");
}
