use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct RGBU8(u8, u8, u8);

#[derive(Debug)]
enum Color {
    Red,
    Green,
    Blue,
    Other(RGBU8),
}

fn main() {
    let mut color = Color::Other(RGBU8(10, 20, 30));

    let other_rgb = KeyPaths::writable_enum(
        |v| Color::Other(v),
        |c: &Color| match c { Color::Other(rgb) => Some(rgb), _ => None },
        |c: &mut Color| match c { Color::Other(rgb) => Some(rgb), _ => None },
    );

    if let Some(rgb) = other_rgb.get_mut(&mut color) {
        *rgb = RGBU8(0, 0, 0);
    }

    println!("{:?}", color);
}


