
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
            get,
            set,
            _p: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn get<'a>(&self, root: &'a R) -> Option<&'a V>
    where
        G: for<'b> Fn(&'b R) -> Option<&'b V>,  // ← HRTB only here
    {
        (self.get)(root)
    }

    #[inline]
    pub fn get_mut<'a>(&self, root: &'a mut R) -> Option<&'a mut V>
    where
        S: for<'b> Fn(&'b mut R) -> Option<&'b mut V>,  // ← HRTB only here
    {
        (self.set)(root)
    }

pub fn then<SV, G2, S2>(
    self,
    next: Kp<
        V,
        SV,
        &'static V,        // ← concrete ref types, not free Value/SubValue/MutSubValue
        &'static SV,
        &'static mut V,
        &'static mut SV,
        G2,
        S2,
    >,
) -> Kp<
    R,
    SV,
    &'static R,
    &'static SV,
    &'static mut R,
    &'static mut SV,
    impl for<'b> Fn(&'b R) -> Option<&'b SV>,
    impl for<'b> Fn(&'b mut R) -> Option<&'b mut SV>,
>
where
    G: for<'b> Fn(&'b R) -> Option<&'b V>,
    S: for<'b> Fn(&'b mut R) -> Option<&'b mut V>,
    G2: for<'b> Fn(&'b V) -> Option<&'b SV>,
    S2: for<'b> Fn(&'b mut V) -> Option<&'b mut SV>,
{
    let first_get = self.get;
    let first_set = self.set;
    let second_get = next.get;
    let second_set = next.set;

Kp::new(
    constrain_get(move |root: &R| first_get(root).and_then(|value| second_get(value))),
    constrain_set(move |root: &mut R| first_set(root).and_then(|value| second_set(value))),
)}
}


// Helper that forces the compiler to accept a closure as for<'b> Fn
fn constrain_get<R, V, F>(f: F) -> F
where
    F: for<'b> Fn(&'b R) -> Option<&'b V>,
{
    f
}

fn constrain_set<R, V, F>(f: F) -> F
where
    F: for<'b> Fn(&'b mut R) -> Option<&'b mut V>,
{
    f
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
        constrain_get(|x: &Rectangle| Some(&x.size)),
        constrain_set(|x: &mut Rectangle| Some(&mut x.size)),
    )
}

// Manual keypath: Size -> width
fn size_width_kp() -> Kp<
    Size,
    u32,
    &'static Size,
    &'static u32,
    &'static mut Size,
    &'static mut u32,
    impl for<'b> Fn(&'b Size) -> Option<&'b u32>,
    impl for<'b> Fn(&'b mut Size) -> Option<&'b mut u32>,
> {
    Kp{
        get:constrain_get( |x: &Size| Some(&x.width)),
        set: constrain_set(|x: &mut Size| Some(&mut x.width)),
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
    
    let kp = rect_size_kp().then(size_width_kp());
    if let Some(res) = (kp).get(&rect) {}
    if let Some(res) = (kp).get(&rect) {}
    if let Some(res) = (kp).get_mut(&mut rect) {}



    let mutable_borrowed = &mut rect;
    mutable_borrowed.name = String::from("value");

    println!("size = {:?}", size_of_val(&kp));
}
