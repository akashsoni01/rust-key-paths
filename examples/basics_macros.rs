use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::Keypaths;

#[derive(Debug, Keypaths)]
#[All]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Debug, Keypaths)]
#[All]
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
    let size_kp: KeyPath<Rectangle, Size, impl for<\'r> Fn(&\'r Rectangle) -> &\'r Size> = KeyPath::new(|r: &Rectangle| &r.size);
    let width_kp: KeyPath<Size, u32, impl for<\'r> Fn(&\'r Size) -> &\'r u32> = KeyPath::new(|s: &Size| &s.width);

    // Compose nested paths (assuming composition is supported).
    // e.g., rect[&size_kp.then(&width_kp)] — hypothetical chaining

    // Alternatively, define them directly:
    let width_direct: KeyPath<Rectangle, u32, impl for<\'r> Fn(&\'r Rectangle) -> &\'r u32> = KeyPath::new(|r: &Rectangle| &r.size.width);
    println!("Width: {:?}", width_direct.get(&rect));

    // Writable keypath for modifying fields:
    let width_mut: KeyPath<Rectangle, u32, impl for<\'r> Fn(&\'r Rectangle) -> &\'r u32> = WritableKeyPath::new(
        // |r: &Rectangle| &r.size.width,
        |r: &mut Rectangle| &mut r.size.width,
    );
    // Mutable
    let hp_mut = width_mut.get_mut(&mut rect);
    {
        *hp_mut += 50;
    }
    println!("Updated rectangle: {:?}", rect);

    // Keypaths from derive-generated methods
    let rect_size_fw = Rectangle::size_fw();
    let rect_name_fw = Rectangle::name_fw();
    let size_width_fw = Size::width_fw();
    let size_height_fw = Size::height_fw();

    let name_readable = Rectangle::name_r();
    println!("Name (readable): {:?}", name_readable.get(&rect));

    let size_writable = Rectangle::size_w();
    let s = size_writable.get_mut(&mut rect);
    {
        s.width += 1;
    }

    // Use them
    let s = rect_size_fw.get_mut(&mut rect);
    {
        if let Some(w) = size_width_fw.get_mut(s) {
            *w += 5;
        }
        if let Some(h) = size_height_fw.get_mut(s) {
            *h += 10;
        }
    }
    let name = rect_name_fw.get_mut(&mut rect);
    {
        name.push_str("_fw");
    }
    println!("After failable updates: {:?}", rect);
}
