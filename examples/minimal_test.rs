use keypaths_proc::Kp;

#[derive(Debug, Kp)]
struct MinimalTest {
    box_option_field: Box<Option<String>>,
}

fn main() {
    println!("Minimal test");
}
