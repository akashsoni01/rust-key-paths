use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
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

#[derive(Debug, Keypaths)]
struct ABox {
    name: String,
    size: Size,
    color: Color,
}

fn main() {
    let mut a_box = ABox {
        name: String::from("A box"),
        size: Size { width: 10, height: 20 },
        color: Color::Other(RGBU8(10, 20, 30)),
    };

    let color_kp = ABox::color_w();
    let case_path = KeyPaths::writable_enum(
        |v| Color::Other(v),
        |c: &Color| match c { Color::Other(rgb) => Some(rgb), _ => None },
        |c: &mut Color| match c { Color::Other(rgb) => Some(rgb), _ => None },
    );

    let color_rgb_kp = color_kp.compose(case_path);
    if let Some(value) = color_rgb_kp.get_mut(&mut a_box) {
        *value = RGBU8(0, 0, 0);
    }

    println!("{:?}", a_box);
}


