use rust_keypaths::{WritableOptionalKeyPath};

#[derive(Debug)]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Debug)]
enum Color {
    Red,
    Green,
    Blue,
    Other(RGBU8),
}

#[derive(Debug)]
struct RGBU8(u8, u8, u8);

#[derive(Debug)]
struct ABox {
    name: String,
    size: Size,
    color: Color,
}

#[derive(Debug)]
struct Rectangle {
    size: Size,
    name: String,
}

fn main() {
    let mut a_box = ABox {
        name: String::from("A box"),
        size: Size {
            width: 10,
            height: 20,
        },
        color: Color::Other(RGBU8(10, 20, 30)),
    };

    // Create a writable keypath for the color field
    let color_kp = WritableOptionalKeyPath::new(|x: &mut ABox| Some(&mut x.color));

    // Create a writable enum keypath for the Other variant
    let case_path = WritableOptionalKeyPath::writable_enum(
        |v| Color::Other(v),
        |p: &Color| match p {
            Color::Other(rgb) => Some(rgb),
            _ => None,
        },
        |p: &mut Color| match p {
            Color::Other(rgb) => Some(rgb),
            _ => None,
        },
    );

    // Compose color with rgb
    println!("{:?}", a_box);
    let color_rgb_kp = color_kp.then(case_path);
    if let Some(value) = color_rgb_kp.get_mut(&mut a_box) {
        *value = RGBU8(0, 0, 0);
    }

    println!("{:?}", a_box);
}

/*
ABox { name: "A box", size: Size { width: 10, height: 20 }, color: Other(RGBU8(10, 20, 30)) }
ABox { name: "A box", size: Size { width: 10, height: 20 }, color: Other(RGBU8(0, 0, 0)) }
*/
