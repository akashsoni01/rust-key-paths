#[derive(Clone, Copy)]
pub struct Kp<R, V, G, S>
where
    G: for<'a> Fn(&'a R) -> Option<&'a V>,
    S: for<'a> Fn(&'a mut R) -> Option<&'a mut V>,
{
    pub get: G,
    pub set: S,
    pub _p: std::marker::PhantomData<(R, V)>,
}

impl<R, V, G, S> Kp<R, V, G, S>
where
    G: for<'a> Fn(&'a R) -> Option<&'a V>,
    S: for<'a> Fn(&'a mut R) -> Option<&'a mut V>,
{
    pub fn new(get: G, set: S) -> Self {
        Self { get, set, _p: std::marker::PhantomData }
    }

    #[inline]
    pub fn get<'a>(&self, root: &'a R) -> Option<&'a V> {
        (self.get)(root)
    }

    #[inline]
    pub fn get_mut<'a>(&self, root: &'a mut R) -> Option<&'a mut V> {
        (self.set)(root)
    }

    pub fn then<W, G2, S2>(
        self,
        next: Kp<V, W, G2, S2>,
    ) -> Kp<
        R,
        W,
        impl for<'a> Fn(&'a R) -> Option<&'a W>,
        impl for<'a> Fn(&'a mut R) -> Option<&'a mut W>,
    >
    where
        G2: for<'a> Fn(&'a V) -> Option<&'a W>,
        S2: for<'a> Fn(&'a mut V) -> Option<&'a mut W>,
        V: 'static
    {
        let first_get = self.get;
        let first_set = self.set;
        let second_get = next.get;
        let second_set = next.set;

        Kp::new(
            move |root: &R| first_get(root).and_then(|v| second_get(v)),
            move |root: &mut R| first_set(root).and_then(|v| second_set(v)),
        )
    }
}

// --- No more get_ref/set_ref/funnel helpers needed ---

struct Size {
    width: u32,
    height: u32,
}

struct Rectangle {
    size: Size,
    name: String,
}

fn rect_size_kp() -> Kp<
    Rectangle,
    Size,
    impl for<'a> Fn(&'a Rectangle) -> Option<&'a Size>,
    impl for<'a> Fn(&'a mut Rectangle) -> Option<&'a mut Size>,
> {
    Kp::new(
        |x: &Rectangle| Some(&x.size),
        |x: &mut Rectangle| Some(&mut x.size),
    )
}

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

#[test]
fn manual_keypath_then_read_write_works() {
    let mut rect = Rectangle {
        size: Size { width: 30, height: 50 },
        name: "MyRect".to_string(),
    };

    let kp = rect_size_kp().then(size_width_kp());

    // Reusable: both calls work, no lifetime consumed
    if let Some(w) = kp.get(&rect) {
        assert_eq!(*w, 30);
    }
    if let Some(w) = kp.get(&rect) {
        assert_eq!(*w, 30);
    }

    // Mutation still works
    if let Some(w) = kp.get_mut(&mut rect) {
        *w = 99;
    }
    assert_eq!(rect.size.width, 99);

    // Can borrow mutably after keypath is done — no conflict
    let _mutable_borrowed = &mut rect;
}