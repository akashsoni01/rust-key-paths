//! Basic keypath example: derive Kp, compose with then(), read and write.
//!
//! Run with: `cargo run --example basics`

use key_paths_derive::Kp;

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

    // Read: compose keypaths with then()
    {
        let width_path = Rectangle::size().then(Size::width());
        if let Some(w) = width_path.get(&rect) {
            println!("Width: {}", w);
        }
        println!("Width (direct): {:?}", width_path.get(&rect));
    }

    // Writable: get_mut and modify
    {
        let width_mut_kp = Rectangle::size().then(Size::width());
        if let Some(w) = width_mut_kp.get_mut(&mut rect) {
            *w += 50;
        }
    }
    println!("Updated rectangle: {:?}", rect);
}
