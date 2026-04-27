use key_paths_derive::Kp;
use rust_key_paths::{Kp, KpTrait};

#[derive(Kp, Debug)]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Kp, Debug)]
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
    impl for<'b> Fn(&'b mut Rectangle) -> Option<&'b mut Size>,
> {
    Kp::new(
        get_ref(|x: &Rectangle| Some(&x.size)),
        set_ref(|x: &mut Rectangle| Some(&mut x.size)),
    )
}

fn get_ref<T, U, F>(f: F) -> F
where F: Fn(&T) -> Option<&U> { f }

fn set_ref<T, U, F>(f: F) -> F
where F: Fn(&mut T) -> Option<&mut U> { f }


// Manual keypath: Size -> width
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
        get_ref(|x: &Size| Some(&x.width)),
        set_ref(|x: &mut Size| Some(&mut x.width)),
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


    let kp = rect_size_kp().then(size_width_kp());

    if let Some(res) = (kp).get(& rect) {

    }

    // if let Some(res) = kp.get(&mut rect) {
        
    // }

    // println!("size of kp = {:?}", size_of_val(&kp));
    let x = &mut rect;

    // println!("size of kp = {:?}", size_of_val(&kp));

    // let x = &rect.name;

    // let y = &mut rect.size;

    // y.height = 234;
    // rect_size_kp()
    //     .then(size_width_kp())
    //     .get(&rect)
    //     .map(|x| assert_eq!(x, &30));
    // if let Some(x) = rect_size_kp().then(size_width_kp()).get(&rect) {
    //     println!("{}", x);
    // }

    // if let Some(x) = rect_size_kp().then(size_width_kp()).get_mut(&mut rect) {
    //     println!("{}", x);
    // }

    // // with direct get field ownership getting moved while get, get_mut fn taking &self
    // let x = rect_size_kp().then(size_width_kp()).get(&rect);
    // let x = rect_size_kp().then(size_width_kp()).get_mut(&mut rect);

    // rect_size_kp()
    //     .then(size_width_kp())
    //     .get_mut(&mut rect)
    //     .map(|x| assert_eq!(x, &mut 30));
    // let width_kp = rect_size_kp().then(size_width_kp());
    // println!("size of concreate kp = {:?}", size_of_val(&width_kp));
    // assert_eq!(rect_size_kp().then(size_width_kp()).get(&rect), Some(&30));
    // assert_eq!(rect_size_kp().then(size_width_kp()).get_mut(&mut rect), Some(&mut 30));

    // if let Some(w) = width_kp.get_mut(&mut rect) {
    //     *w += 12;
    // }

    // assert_eq!((width_kp).get(&rect), Some(&42));
    // assert_eq!(rect.size.height, 50);
    // assert_eq!(rect.name, "MyRect");
}


// #[test]
// fn manual_keypath_then_read_write_works2() {
//     let mut rect = Rectangle {
//         size: Size {
//             width: 30,
//             height: 50,
//         },
//         name: "MyRect".to_string(),
//     };

//     // let width_kp = Rectangle::rect_size_kp().then(Size::size_width_kp());

//     println!("size of concreate kp = {:?}", size_of_val(&Rectangle::rect_size_kp().then(Size::size_width_kp())));
//     assert_eq!(Rectangle::rect_size_kp().then(Size::size_width_kp()).get(&rect), Some(&30));

//     if let Some(w) = Rectangle::rect_size_kp().then(Size::size_width_kp()).get_mut(&mut rect) {
//         *w += 12;
//     }

//     assert_eq!((Rectangle::rect_size_kp().then(Size::size_width_kp())).get(&rect), Some(&42));
//     // assert_eq!(rect.size.height, 50);
//     // assert_eq!(rect.name, "MyRect");
// }