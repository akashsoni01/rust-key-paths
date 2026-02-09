use key_paths_derive::Kp;
use rust_key_paths::KpType;

#[derive(Debug, Kp)]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Debug, Kp)]
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
    
    // Keypaths from derive-generated methods
    // Note: size and name are NOT Option types, so they use () methods, not _fw()
    let rect_size_w = Rectangle::size();
    let rect_name_w = Rectangle::name();
    let size_width_w = Size::width();
    let size_height_w = Size::height();

    let name_readable = Rectangle::name();
    println!("Name (readable): {:?}", name_readable.get(&rect));

    let size_writable = Rectangle::size();
    if let Some(s) = size_writable.get_mut(&mut rect)
    {
        s.width += 1;
    }

    // Use them - () methods return &mut T directly (not Option)
    // For WritableKeyPath, we need to convert to OptionalKeyPath to chain, or access directly
    {
        let s = rect_size_w.get_mut(&mut rect).unwrap();
        let w = size_width_w.get_mut(s).unwrap();
        *w += 5;
        let h = size_height_w.get_mut(s).unwrap();
        *h += 10;
    }
    // () methods return &mut T directly
    let name = rect_name_w.get_mut(&mut rect).unwrap();
    name.push_str("_w");
    println!("After failable updates: {:?}", rect);
}
