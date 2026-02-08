use keypaths_proc::Kp;
use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

#[derive(Debug, Kp)]
#[All]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Debug, Kp)]
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
    let size_kp = KeyPath::new(|r: &Rectangle| &r.size);
    let width_kp = KeyPath::new(|s: &Size| &s.width);

    // Compose nested paths (assuming composition is supported).
    // e.g., rect[&size_kp.then(&width_kp)] — hypothetical chaining

    // Alternatively, define them directly:
    let width_direct = KeyPath::new(|r: &Rectangle| &r.size.width);
    println!("Width: {:?}", width_direct.get(&rect));

    // Writable keypath for modifying fields:
    let width_mut = WritableKeyPath::new(|r: &mut Rectangle| &mut r.size.width);
    // Mutable
    let hp_mut = width_mut.get_mut(&mut rect);
    {
        *hp_mut += 50;
    }
    println!("Updated rectangle: {:?}", rect);

    // Keypaths from derive-generated methods
    // Note: size and name are NOT Option types, so they use _w() methods, not _fw()
    let rect_size_w = Rectangle::size_w();
    let rect_name_w = Rectangle::name_w();
    let size_width_w = Size::width_w();
    let size_height_w = Size::height_w();

    let name_readable = Rectangle::name_r();
    println!("Name (readable): {:?}", name_readable.get(&rect));

    let size_writable = Rectangle::size_w();
    let s = size_writable.get_mut(&mut rect);
    {
        s.width += 1;
    }

    // Use them - _w() methods return &mut T directly (not Option)
    // For WritableKeyPath, we need to convert to OptionalKeyPath to chain, or access directly
    {
        let s = rect_size_w.get_mut(&mut rect);
        let w = size_width_w.get_mut(s);
        *w += 5;
        let h = size_height_w.get_mut(s);
        *h += 10;
    }
    // _w() methods return &mut T directly
    let name = rect_name_w.get_mut(&mut rect);
    name.push_str("_w");
    println!("After failable updates: {:?}", rect);
}
