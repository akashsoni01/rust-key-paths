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
    impl for<'b> Fn(&'b Rectangle) -> Option<&'b Size>,
    impl for<'b> Fn(&'b mut Rectangle) -> Option<&'b mut Size>
> {
    Kp::new(
        hrtb_ref(|x: &Rectangle| { Some(& x.size) }),
        hrtb_mut(|x: &mut Rectangle| { Some(&mut x.size) })
    )
}
fn hrtb_ref<T, U, F>(f: F) -> F
where F: for<'a> Fn(&'a T) -> Option<&'a U> { f }

fn hrtb_mut<T, U, F>(f: F) -> F
where F: for<'a> Fn(&'a mut T) -> Option<&'a mut U> { f }

// Manual keypath: Size -> width
fn size_width_kp<'a>() -> Kp<
    Size,
    u32,
    &'a Size,
    &'a u32,
    &'a mut Size,
    &'a mut u32,
    impl for<'b> Fn(&'b Size) -> Option<&'b u32>,
    impl for<'b> Fn(&'b mut Size) -> Option<&'b mut u32>
> {
    Kp::new(
        hrtb_ref(|x: &Size| { Some(& x.width) }),
        hrtb_mut(|x: &mut Size| { Some(&mut x.width) })
    )
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

    // let width_kp = rect_size_kp().then(size_width_kp());

    println!("size of concreate kp = {:?}", size_of_val(&rect_size_kp().then(size_width_kp())));
    assert_eq!(rect_size_kp().then(size_width_kp()).get(&rect), Some(&30));

    if let Some(w) = rect_size_kp().then(size_width_kp()).get_mut(&mut rect) {
        *w += 12;
    }

    assert_eq!((rect_size_kp().then(size_width_kp())).get(&rect), Some(&42));
    // assert_eq!(rect.size.height, 50);
    // assert_eq!(rect.name, "MyRect");
}