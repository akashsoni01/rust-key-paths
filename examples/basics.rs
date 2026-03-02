//! Basic keypath example: derive Kp, compose with then(), read and write.
//!
//! Run with: `cargo run --example basics`

use key_paths_derive::Kp;
use rust_key_paths::{Kp, KpDynamic, KpType};

pub struct Service {
    rect_to_width_kp: KpDynamic<
        Rectangle,
        u32,
    >,

}

// impl Service {
//     pub fn new() -> Self {
//         Self {
//             rect_to_width_kp: Rectangle::test().into(),
//         }
//     }
// }


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

// Standalone fn pointers for keypath (reference: lib.rs identity_typed / Kp with fn types)

impl Rectangle {
    // /// Keypath to `size.width`, built with fn pointers (same pattern as lib.rs `identity_typed`).
    // pub const fn size() -> KpType<'static, Rectangle, Size> {
    //     const fn g(r: &Rectangle) -> Option<&Size> {
    //         Some(&r.size)
    //     }
    //     const fn s(r: &mut Rectangle) -> Option<&mut Size> {
    //         Some(&mut r.size)
    //     }

    //     Kp::new_const(g, s)
    // }
}

// const rec_to_width: KpType<'static, Rectangle, Size> = crate::Rectangle::size().then(Size::width());

fn main() {
    let mut rect = Rectangle {
        size: Size {
            width: 30,
            height: 50,
        },
        name: "MyRect".into(),
    };

    // // Read: compose keypaths with then()
    // {
    //     let width_path = Rectangle::size().then(Size::width());
    //     if let Some(w) = width_path.get(&rect) {
    //         println!("Width: {}", w);
    //     }
    //     println!("Width (direct): {:?}", width_path.get(&rect));
    // }
    //
    // // Writable: get_mut and modify
    // {
    //     let width_mut_kp = Rectangle::size().then(Size::width());
    //     if let Some(w) = width_mut_kp.get_mut(&mut rect) {
    //         *w += 50;
    //     }
    // }
    // println!("Updated rectangle: {:?}", rec_to_width.get(&rect));
}
