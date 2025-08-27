use key_paths_core::{FailableWritableKeyPath, FailableReadable, FailableWritable};

#[derive(Debug)]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct Rectangle {
    size: Option<Size>,
    name: String,
}

fn main() {
    let mut rect = Rectangle {
        size: Some(Size { width: 10, height: 20 }),
        name: "MyRect".into(),
    };

    // Define a keypath for width inside Rectangle.size
    let size_width = FailableWritableKeyPath::new(
        |r: &Rectangle| r.size.as_ref().map(|s| &s.width),
        |r: &mut Rectangle| r.size.as_mut().map(|s| &mut s.width),
    );

    // Try reading
    if let Some(width) = size_width.try_get(&rect) {
        println!("Width = {}", width);
    }

    // Try writing
    if let Some(width) = size_width.try_get_mut(&mut rect) {
        *width = 42;
    }

    println!("Updated Rectangle = {:?}", rect);

    // If size is None
    rect.size = None;
    println!("Missing width = {:?}", size_width.try_get(&rect));
}
