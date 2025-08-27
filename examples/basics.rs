use key_paths_core::*;

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

    // Define readable and writable keypaths.
    let size_kp: ReadableKeyPath<Rectangle, Size> = ReadableKeyPath::new(|r: &Rectangle| &r.size);
    let width_kp: ReadableKeyPath<Size, u32> = ReadableKeyPath::new(|s: &Size| &s.width);

    // Compose nested paths (assuming composition is supported).
    // e.g., rect[&size_kp.then(&width_kp)] — hypothetical chaining

    // Alternatively, define them directly:
    let width_direct: ReadableKeyPath<Rectangle, u32> =
        ReadableKeyPath::new(|r: &Rectangle| &r.size.width);
    println!("Width: {}", width_direct.get(&rect));

    // Writable keypath for modifying fields:
    let width_mut = WritableKeyPath::new(
        |r: &Rectangle| &r.size.width,
        |r: &mut Rectangle| &mut r.size.width,
    );
    *width_mut.get_mut(&mut rect) = 100;
    println!("Updated rectangle: {:?}", rect);
}
