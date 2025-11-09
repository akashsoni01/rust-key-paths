use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct Point(
    u32,
    #[Writable]
    Option<u32>,
    #[Writable]
    String,
);

fn main() {
    let mut p = Point(10, Some(20), "name".into());

    // Non-Option fields
    let x_path = Point::f0();
    let name_path = Point::f2();
    println!("x = {:?}", x_path.get(&p));
    if let Some(n) = name_path.get_mut(&mut p) {
        n.push_str("_edited");
    }

    // Option field with failable
    let y_path = Point::f1();
    println!("y (fr) = {:?}", y_path.get(&p));

    if let Some(y) = y_path.get_mut(&mut p) {
        *y += 1;
    }

    println!("updated p = {:?}", p);
}
