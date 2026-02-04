// pub type KpType<R, V, Root, Value, MutRoot, MutValue, G, S>
// where
//     Root: ,
//     Value:    Borrow<V>,
//     MutRoot:  BorrowMut<R>,
//     MutValue: std::borrow::BorrowMut<V>,
//     G:        Fn(Root) -> Option<Value>,
//     S:        Fn(MutRoot) -> Option<MutValue> = Kp<R, V, Root, Value, MutRoot, MutValue, G, S>;

// type Getter<R, V, Root, Value> where Root: std::borrow::Borrow<R>, Value: std::borrow::Borrow<V> = fn(Root) -> Option<Value>;
// type Setter<R, V> = fn(&'r mut R) -> Option<&'r mut V>;

pub type KpType<'a, R, V> = Kp<
    R,
    V,
    &'a R,
    &'a V,
    &'a mut R,
    &'a mut V,
    for<'b> fn(&'b R) -> Option<&'b V>,
    for<'b> fn(&'b mut R) -> Option<&'b mut V>,
>;

// pub type KpType<R, V> = Kp<
//     R,
//     V,
//     &'static R,
//     &'static V,
//     &'static mut R,
//     &'static mut V,
//     for<'a> fn(&'a R) -> Option<&'a V>,
//     for<'a> fn(&'a mut R) -> Option<&'a mut V>,
// >;

pub struct Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    get: G,
    set: S,
    _p: std::marker::PhantomData<(R, V, Root, Value, MutRoot, MutValue)>,
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

    fn get(&self, root: Root) -> Option<Value> {
        (self.get)(root)
    }
    fn get_mut(&self, root: MutRoot) -> Option<MutValue> {
        (self.set)(root)
    }

    pub fn then<SV, SubValue, MutSubValue, G2, S2>(
        &self,
        next: Kp<V, SV, Value, SubValue, MutValue, MutSubValue, G2, S2>,
    ) -> Kp<
        R,
        SV,
        Root,
        SubValue,
        MutRoot,
        MutSubValue,
        // wrong syntax - expected fn pointer, found closure
        // fn(Root) -> Option<SubValue>,
        // fn(MutRoot) -> Option<MutSubValue>,
        impl Fn(Root) -> Option<SubValue>,
        impl Fn(MutRoot) -> Option<MutSubValue>,
    >
    where
        SubValue: std::borrow::Borrow<SV>,
        MutSubValue: std::borrow::BorrowMut<SV>,
        G2: Fn(Value) -> Option<SubValue>,
        S2: Fn(MutValue) -> Option<MutSubValue>,
        V: 'static,
    {
        Kp::new(
            move |root: Root| (&self.get)(root).and_then(|value| (next.get)(value)),
            move |root: MutRoot| (&self.set)(root).and_then(|value| (next.set)(value)),
        )
    }

    pub fn for_arc<'b>(
        &self,
    ) -> Kp<
        std::sync::Arc<R>,
        V,
        std::sync::Arc<R>,
        Value,
        std::sync::Arc<R>,
        MutValue,
        impl Fn(std::sync::Arc<R>) -> Option<Value>,
        impl Fn(std::sync::Arc<R>) -> Option<MutValue>,
    >
    where
        R: 'b,
        V: 'b,
        Root: for<'a> From<&'a R>,
        MutRoot: for<'a> From<&'a mut R>,
    {
        Kp::new(
            move |arc_root: std::sync::Arc<R>| {
                let r_ref: &R = &*arc_root;
                (&self.get)(Root::from(r_ref))
            },
            move |mut arc_root: std::sync::Arc<R>| {
                // Get mutable reference only if we have exclusive ownership
                std::sync::Arc::get_mut(&mut arc_root).and_then(|r_mut| (&self.set)(MutRoot::from(r_mut)))
            },
        )
    }

    pub fn for_box<'a>(
        &self,
    ) -> Kp<
        Box<R>,
        V,
        Box<R>,
        Value,
        Box<R>,
        MutValue,
        impl Fn(Box<R>) -> Option<Value>,
        impl Fn(Box<R>) -> Option<MutValue>,
    >
    where
        R: 'a,
        V: 'a,
        Root: for<'b> From<&'b R>,
        MutRoot: for<'b> From<&'b mut R>,
    {
        Kp::new(
            move |r: Box<R>| {
                let r_ref: &R = r.as_ref();
                (&self.get)(Root::from(r_ref))
            },
            move |mut r: Box<R>| {
                // Get mutable reference only if we have exclusive ownership
                (self.set)(MutRoot::from(r.as_mut()))
            },
        )
    }

}

impl<R, Root, MutRoot, G, S> Kp<R, R, Root, Root, MutRoot, MutRoot, G, S>
where
    Root: std::borrow::Borrow<R>,
    MutRoot: std::borrow::BorrowMut<R>,
    G: Fn(Root) -> Option<Root>,
    S: Fn(MutRoot) -> Option<MutRoot>,
{
    // pub fn identity() -> Kp<
    //     R,
    //     R,
    //     Root,
    //     Root,
    //     MutRoot,
    //     MutRoot,
    //     fn(Root) -> Option<Root>,
    //     fn(MutRoot) -> Option<MutRoot>,
    // > {
    //     Kp::new(|r: Root| Some(r), |r: MutRoot| Some(r))
    // }

    pub fn identity<'a>() -> KpType<'a, R, R> {
        KpType::new(|r| Some(r), |r| Some(r))
    }
}

struct TestKP {
    a: String,
    b: String,
    c: std::sync::Arc<String>,
    d: std::sync::Mutex<String>,
    e: std::sync::Arc<std::sync::Mutex<TestKP2>>,
    f: Option<TestKP2>,
}

impl TestKP {
    fn new() -> Self {
        Self {
            a: String::from("a"),
            b: String::from("b"),
            c: std::sync::Arc::new(String::from("c")),
            d: std::sync::Mutex::new(String::from("d")),
            e: std::sync::Arc::new(std::sync::Mutex::new(TestKP2::new())),
            f: Some(TestKP2 {
                a: String::from("a3"),
                b: std::sync::Arc::new(std::sync::Mutex::new(TestKP3::new())),
            }),
        }
    }

    // Example for - Clone ref sharing
    // Keypath for field 'a' (String)
    // fn a<Root, MutRoot, Value, MutValue>() -> Kp<
    //     TestKP2,
    //     String,
    //     Root,
    //     Value,
    //     MutRoot,
    //     MutValue,
    //     impl Fn(Root) -> Option<Value>,
    //     impl Fn(MutRoot) -> Option<MutValue>,
    // >
    // where
    //     Root: std::borrow::Borrow<TestKP2>,
    //     MutRoot: std::borrow::BorrowMut<TestKP2>,
    //     Value: std::borrow::Borrow<String> + From<String>,
    //     MutValue: std::borrow::BorrowMut<String> + From<String>,
    // {
    //     Kp::new(
    //         |r: Root| Some(Value::from(r.borrow().a.clone())),
    //         |mut r: MutRoot| Some(MutValue::from(r.borrow_mut().a.clone())),
    //     )
    // }

    // Example for taking ref
    fn a<'a>() -> KpType<'a, TestKP2, String> {
        Kp::new(|r: &TestKP2| Some(&r.a), |r: &mut TestKP2| Some(&mut r.a))
    }

    // example - cloning arc mutex
    fn b<'a>() -> KpType<'a, TestKP2, std::sync::Arc<std::sync::Mutex<TestKP3>>> {
        Kp::new(|r: &TestKP2| Some(&r.b), |r: &mut TestKP2| Some(&mut r.b))
    }

    // Helper to create an identity keypath for TestKP2
    fn identity<'a>() -> KpType<'a, TestKP2, TestKP2> {
        KpType::identity()
    }

    fn f<'a>() -> KpType<'a, TestKP, TestKP2> {
        KpType::new(|r: &TestKP| r.f.as_ref(), |r: &mut TestKP| r.f.as_mut())
    }
}

#[derive(Debug)]
struct TestKP2 {
    a: String,
    b: std::sync::Arc<std::sync::Mutex<TestKP3>>,
}

impl TestKP2 {
    fn new() -> Self {
        TestKP2 {
            a: String::from("a2"),
            b: std::sync::Arc::new(std::sync::Mutex::new(TestKP3::new())),
        }
    }

    // fn identity<Root, MutRoot, G, S>() -> Kp<
    //     TestKP2, // R
    //     TestKP2, // V
    //     Root,    // Root
    //     Root,    // Value
    //     MutRoot, // MutRoot
    //     MutRoot, // MutValue
    //     fn(Root) -> Option<Root>,
    //     fn(MutRoot) -> Option<MutRoot>,
    // >
    // where
    //     Root: std::borrow::Borrow<TestKP2>,
    //     MutRoot: std::borrow::BorrowMut<TestKP2>,
    //     G: Fn(Root) -> Option<Root>,
    //     S: Fn(MutRoot) -> Option<MutRoot>,
    // {
    //     Kp::<TestKP2, TestKP2, Root, Root, MutRoot, MutRoot, G, S>::identity()
    // }

    fn a<'a>() -> KpType<'a, TestKP2, String> {
        KpType::new(|r: &TestKP2| Some(&r.a), |r: &mut TestKP2| Some(&mut r.a))
    }

    fn b<'a>() -> KpType<'a, TestKP2, std::sync::Arc<std::sync::Mutex<TestKP3>>> {
        KpType::new(|r: &TestKP2| Some(&r.b), |r: &mut TestKP2| Some(&mut r.b))
    }

    // fn b_lock<'a, V>(kp: KpType<'a, TestKP2, V>) -> KpType<'a, TestKP2, std::sync::MutexGuard<'a, TestKP3>> {
    //     KpType::new(|r: &TestKP2| Some(r.b.lock().unwrap()), |r: &mut TestKP2| Some(r.b.lock().unwrap()))
    // }

    fn identity<'a>() -> KpType<'a, TestKP2, TestKP2> {
        KpType::identity()
    }
}

#[derive(Debug)]
struct TestKP3 {
    a: String,
    b: std::sync::Arc<std::sync::Mutex<String>>,
}

impl TestKP3 {
    fn new() -> Self {
        TestKP3 {
            a: String::from("a2"),
            b: std::sync::Arc::new(std::sync::Mutex::new(String::from("b2"))),
        }
    }
    //
    // fn identity<Root, MutRoot, G, S>() -> Kp<
    //     TestKP2, // R
    //     TestKP2, // V
    //     Root,    // Root
    //     Root,    // Value
    //     MutRoot, // MutRoot
    //     MutRoot, // MutValue
    //     fn(Root) -> Option<Root>,
    //     fn(MutRoot) -> Option<MutRoot>,
    // >
    // where
    //     Root: std::borrow::Borrow<TestKP2>,
    //     MutRoot: std::borrow::BorrowMut<TestKP2>,
    //     G: Fn(Root) -> Option<Root>,
    //     S: Fn(MutRoot) -> Option<MutRoot>,
    // {
    //     Kp::<TestKP2, TestKP2, Root, Root, MutRoot, MutRoot, G, S>::identity()
    // }

    fn identity<'a>() -> KpType<'a, TestKP3, TestKP3> {
        KpType::identity()
    }
}

impl TestKP3 {}

impl TestKP {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a() {
        let instance2 = TestKP2::new();
        let mut instance = TestKP::new();
        let kp = TestKP::identity();
        let kp_a = crate::TestKP::a();
        // TestKP::a().for_arc();
        let kp_f = TestKP::f();
        let wres = kp_f.then(TestKP2::a()).get_mut(&mut instance).unwrap();
        *wres = String::from("a3 changed successfully");
        let res = kp_f.then(TestKP2::a()).get(&instance);
        println!("{:?}", res);
        let res = kp_f.then(TestKP2::identity()).get(&instance);
        println!("{:?}", res);
        let res = kp.get(&instance2);
        println!("{:?}", res);

    }
}
