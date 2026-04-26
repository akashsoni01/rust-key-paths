use rust_key_paths::{Kp, KpTrait};

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

// Manual keypath: Rectangle -> Size
fn rect_size_kp<'a>() -> Kp<
    Rectangle,
    Size,
    &'a Rectangle,
    &'a Size,
    &'a mut Rectangle,
    &'a mut Size,
    impl Fn(&'a Rectangle) -> Option<&'a Size>,
    impl Fn(&'a mut Rectangle) -> Option<&'a mut Size>
> {
    Kp::new(
        |x: &'a Rectangle| { Some(& x.size) },
        |x: &'a mut Rectangle| { Some(&mut x.size) }
    )
}

// Manual keypath: Size -> width
fn size_width_kp<'a>() -> Kp<
    Size,
    u32,
    &'a Size,
    &'a u32,
    &'a mut Size,
    &'a mut u32,
    impl Fn(&'a Size) -> Option<&'a u32>,
    impl Fn(&'a mut Size) -> Option<&'a mut u32>
> {
    Kp::new(
        |x: &Size| { Some(& x.width) },
        |x: &mut Size| { Some(&mut x.width) }
    )
}

#[test]
fn manual_keypath_then_read_write_works() {
    let rect = Rectangle {
        size: Size {
            width: 30,
            height: 50,
        },
        name: "MyRect".to_string(),
    };

    let width_kp = rect_size_kp().then(size_width_kp());

    println!("size of concreate kp = {:?}", size_of_val(&width_kp));
    assert_eq!(width_kp.get(&rect), Some(&30));

    // if let Some(w) = width_kp.get_mut(&mut rect) {
    //     *w += 12;
    // }

    // assert_eq!((width_kp).get(&rect), Some(&42));
    // assert_eq!(rect.size.height, 50);
    // assert_eq!(rect.name, "MyRect");
}