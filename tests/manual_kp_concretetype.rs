use rust_key_paths::{Kp, KpTrait, hlp_get, hlp_set};

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

impl Rectangle {
    fn rect_size_kp<'a>() -> Kp<
        Rectangle,
        Size,
        &'a Rectangle,
        &'a Size,
        &'a mut Rectangle,
        &'a mut Size,
        impl for<'b> Fn(&'b Rectangle) -> Option<&'b Size>,
        impl for<'b> Fn(&'b mut Rectangle) -> Option<&'b mut Size>,
    > {
        Kp::new(
            hlp_get(|x: &Rectangle| Some(&x.size)),
            hlp_set(|x: &mut Rectangle| Some(&mut x.size)),
        )
    }
}

impl Size {
    fn size_width_kp<'a>() -> Kp<
        Size,
        u32,
        &'a Size,
        &'a u32,
        &'a mut Size,
        &'a mut u32,
        impl for<'b> Fn(&'b Size) -> Option<&'b u32>,
        impl for<'b> Fn(&'b mut Size) -> Option<&'b mut u32>,
    > {
        Kp::new(
            hlp_get(|x: &Size| Some(&x.height)),
            hlp_set(|x: &mut Size| Some(&mut x.height)),
        )
    }
}


#[test]
fn manual_keypath_then_read_write_works() {
    let mut rect = Rectangle {
        size: Size {
            width: 30,
            height: 50,
        },
        name: "MyRect".to_string(),
    };

    let width_kp = Rectangle::rect_size_kp().then(Size::size_width_kp());

    println!("size of concreate kp = {:?}", size_of_val(&width_kp));
    // assert_eq!(width_kp.get(&rect), Some(&30));

    if let Some(w) = width_kp.get_mut(&mut rect) {
        *w += 12;
    }

    // assert_eq!((width_kp).get(&rect), Some(&42));
    // assert_eq!(rect.size.height, 50);
    // assert_eq!(rect.name, "MyRect");
}