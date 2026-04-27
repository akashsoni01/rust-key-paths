
#[derive(Clone)]
pub struct Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    /// Getter closure: used by [Kp::get] for read-only access.
    pub get: G,
    /// Setter closure: used by [Kp::get_mut] for mutation.
    pub set: S,
    pub _p: std::marker::PhantomData<(R, V, Root, Value, MutRoot, MutValue)>,
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S> Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    pub fn new(get: G, set: S) -> Self {
        Self {
            get: get,
            set: set,
            _p: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn get(&self, root: Root) -> Option<Value> {
        (self.get)(root)
    }

    #[inline]
    pub fn get_mut(&self, root: MutRoot) -> Option<MutValue> {
        (self.set)(root)
    }

    #[inline]
    pub fn then<SV, SubValue, MutSubValue, G2, S2>(
        self,
        next: Kp<V, SV, Value, SubValue, MutValue, MutSubValue, G2, S2>,
    ) -> Kp<
        R,
        SV,
        Root,
        SubValue,
        MutRoot,
        MutSubValue,
        impl Fn(Root) -> Option<SubValue>,
        impl Fn(MutRoot) -> Option<MutSubValue>,
    >
    where
        SubValue: std::borrow::Borrow<SV>,
        MutSubValue: std::borrow::BorrowMut<SV>,
        G2: Fn(Value) -> Option<SubValue>,
        S2: Fn(MutValue) -> Option<MutSubValue>,
    {
        let first_get = self.get;
        let first_set = self.set;
        let second_get = next.get;
        let second_set = next.set;

        Kp::new(
            move |root: Root| first_get(root).and_then(|value| second_get(value)),
            move |root: MutRoot| first_set(root).and_then(|value| second_set(value)),
        )
    }
}

struct Size {
    width: u32,
    height: u32,
}

struct Rectangle {
    size: Size,
    name: String,
}

// Manual keypath: Rectangle -> Size
fn rect_size_kp() -> Kp<
    Rectangle,
    Size,
    &'static Rectangle,
    &'static Size,
    &'static mut Rectangle,
    &'static mut Size,
    impl Fn(&Rectangle) -> Option<&Size>,
    impl Fn(&mut Rectangle) -> Option<&mut Size>,
> {
    Kp::new(
        get_ref(|x: &Rectangle| Some(&x.size)),
        set_ref(|x: &mut Rectangle| Some(&mut x.size)),
    )
}

fn get_ref<T, U, F>(f: F) -> F
where
    F: Fn(&T) -> Option<&U>,
{
    f
}

fn set_ref<T, U, F>(f: F) -> F
where
    F: Fn(&mut T) -> Option<&mut U>,
{
    f
}

// Manual keypath: Size -> width
fn size_width_kp() -> Kp<
    Size,
    u32,
    &'static Size,
    &'static u32,
    &'static mut Size,
    &'static mut u32,
    impl Fn(&Size) -> Option<&u32>,
    impl Fn(&mut Size) -> Option<&mut u32>,
> {
    Kp{
        get: |x: &Size| Some(&x.width),
        set: |x: &mut Size| Some(&mut x.width),
        _p: std::marker::PhantomData
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
    {
        let kp = rect_size_kp().then(size_width_kp());
        if let Some(res) = (kp).get(&rect) {}
        if let Some(res) = (kp).get(&rect) {}
    }

    let mutable_borrowed = &mut rect;
}
