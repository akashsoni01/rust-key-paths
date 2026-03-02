//! Basic keypath example: derive Kp, compose with then_const(), read and write.
//!
//! Run with: `cargo run --example basics`

use key_paths_derive::Kp;
use rust_key_paths::KpDynamic;

pub struct Service {
    rect_to_width_kp: KpDynamic<Rectangle, u32>,
}

impl Service {
    pub fn new() -> Self {
        Self {
            rect_to_width_kp: Rectangle::size().then_const(Size::width()).into(),
        }
    }
}

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

// Const composition: Rectangle -> Size -> u32 (width)
const RECT_TO_WIDTH: rust_key_paths::ThenKp<Rectangle, Size, u32> =
    Rectangle::size().then_const(Size::width());

fn main() {
    let mut rect = Rectangle {
        size: Size {
            width: 30,
            height: 50,
        },
        name: "MyRect".into(),
    };

    // Read: compose keypaths with then_const()
    {
        let width_path = Rectangle::size().then_const(Size::width());
        if let Some(w) = width_path.get(&rect) {
            println!("Width: {}", w);
        }
        println!("Width (direct): {:?}", width_path.get(&rect));
    }

    // Const keypath works the same
    if let Some(w) = RECT_TO_WIDTH.get(&rect) {
        println!("Width (const): {}", w);
    }

    // Writable: get_mut and modify
    {
        let width_mut_kp = Rectangle::size().then_const(Size::width());
        if let Some(w) = width_mut_kp.get_mut(&mut rect) {
            *w += 50;
        }
    }
    println!("Updated rectangle: {:?}", rect);
    println!("Width after update: {:?}", RECT_TO_WIDTH.get(&rect));
}
