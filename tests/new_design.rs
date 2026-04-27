// use rust_key_paths::{Kp, KpTrait, hlp_get, hlp_set};
struct Kp<Root, Value, G, S> {
    getter: G,
    setter: S,
    _phantom: std::marker::PhantomData<fn() -> (Root, Value)>,
}

impl<Root, Value, G, S> Kp<Root, Value, G, S>
where
    G: for<'a> Fn(&'a Root) -> Option<&'a Value>,
    S: for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>,
{
    fn new(getter: G, setter: S) -> Self {
        Self { getter, setter, _phantom: std::marker::PhantomData }
    }

    fn get<'a>(&self, root: &'a Root) -> Option<&'a Value> {
        (self.getter)(root)
    }

    fn get_mut<'a>(&self, root: &'a mut Root) -> Option<&'a mut Value> {
        (self.setter)(root)
    }

    fn then<Value2, G2, S2>(
        self,
        next: Kp<Value, Value2, G2, S2>,
    ) -> Kp<
        Root,
        Value2,
        impl for<'a> Fn(&'a Root) -> Option<&'a Value2>,
        impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value2>,
    >
    where
        G2: for<'a> Fn(&'a Value) -> Option<&'a Value2>,
        S2: for<'a> Fn(&'a mut Value) -> Option<&'a mut Value2>,
        Value: 'static
    {
        Kp::new(
            move |root: &Root| {
                (self.getter)(root).and_then(|v| (next.getter)(v))
            },
            move |root: &mut Root| {
                (self.setter)(root).and_then(|v| (next.setter)(v))
            },
        )
    }
}

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
    fn rect_size_kp() -> Kp<
    Rectangle,
    Size,
    impl for<'a> Fn(&'a Rectangle) -> Option<&'a Size>,
    impl for<'a> Fn(&'a mut Rectangle) -> Option<&'a mut Size>
> {
    Kp::new(
        |x: & Rectangle| { Some(& x.size) },
          |x: & mut Rectangle| { Some(&mut x.size) }
    )
}

}
// Manual keypath: Rectangle -> Size

impl Size {
    // Manual keypath: Size -> width
fn size_width_kp() -> Kp<
    Size,
    u32,
    impl for<'a> Fn(&'a Size) -> Option<&'a u32>,
    impl for<'a> Fn(&'a mut Size) -> Option<&'a mut u32>,
> {
    Kp::new(
        |x: &Size| Some(&x.width),
        |x: &mut Size| Some(&mut x.width),
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
    assert_eq!(width_kp.get(&rect), Some(&30));

    if let Some(w) = width_kp.get_mut(&mut rect) {
        *w += 12;
    }

    // assert_eq!((width_kp).get(&rect), Some(&42));
    // assert_eq!(rect.size.height, 50);
    // assert_eq!(rect.name, "MyRect");
}