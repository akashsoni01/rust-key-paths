#[derive(Debug)]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct Rectangle {
    size: Size,
    name: String,
}

fn main() {
    let mut rect = Rectangle {
        size: Size {
            width: 30,
            height: 50,
        },
        name: "MyRect".into(),
    };

    let width_direct = KeyPaths::readable(|r: &Rectangle| &r.size.width);
    println!("Width: {:?}", width_direct.get(&rect));
}
