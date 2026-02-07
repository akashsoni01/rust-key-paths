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

use std::sync::{Arc, Mutex};

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

// struct A{
//     b: std::sync::Arc<std::sync::Mutex<B>>,
// }
// struct B{
//     c: C
// }
// struct C{
//     d: String
// }

// pub struct LockKp {
//     first: KpType<'static, A, B>,
//     mid: KpType<'static, std::sync::Mutex<B>, B>,
//     second: KpType<'static, B, C>,
// }
//
// impl LockKp {
//     fn then(&self, kp: KpType<'static, B, String>) {
//
//     }
//     fn then_lock() {}
// }

// New type alias for composed/transformed keypaths
pub type KpComposed<R, V> = Kp<
    R,
    V,
    &'static R,
    &'static V,
    &'static mut R,
    &'static mut V,
    Box<dyn for<'b> Fn(&'b R) -> Option<&'b V>>,
    Box<dyn for<'b> Fn(&'b mut R) -> Option<&'b mut V>>,
>;

pub struct LockKp<R, V, MV, G1, S1, G3, S3>
where
    G1: Fn(&R) -> Option<&Arc<Mutex<MV>>>,
    S1: Fn(&mut R) -> Option<&mut Arc<Mutex<MV>>>,
    G3: Fn(&MV) -> Option<&V>,
    S3: Fn(&mut MV) -> Option<&mut V>,
    R: 'static,
    V: 'static,
    MV: 'static,
{
    first: Kp<
        R,
        Arc<Mutex<MV>>,
        &'static R,
        &'static Arc<Mutex<MV>>,
        &'static mut R,
        &'static mut Arc<Mutex<MV>>,
        G1,
        S1,
    >,
    second: Kp<MV, V, &'static MV, &'static V, &'static mut MV, &'static mut V, G3, S3>,
}

impl<R, V, MV, G1, S1, G3, S3> LockKp<R, V, MV, G1, S1, G3, S3>
where
    R: 'static,
    V: 'static + Clone,
    MV: 'static,
    G1: Fn(&R) -> Option<&Arc<Mutex<MV>>>,
    S1: Fn(&mut R) -> Option<&mut Arc<Mutex<MV>>>,
    G3: Fn(&MV) -> Option<&V>,
    S3: Fn(&mut MV) -> Option<&mut V>,
{
    pub fn new(
        first: Kp<
            R,
            Arc<Mutex<MV>>,
            &'static R,
            &'static Arc<Mutex<MV>>,
            &'static mut R,
            &'static mut Arc<Mutex<MV>>,
            G1,
            S1,
        >,
        second: Kp<MV, V, &'static MV, &'static V, &'static mut MV, &'static mut V, G3, S3>,
    ) -> Self {
        Self { first, second }
    }

    pub fn get(&self, root: &R) -> Option<V> {
        (self.first.get)(root).and_then(|arc_mutex| {
            arc_mutex
                .lock()
                .ok()
                .and_then(|guard| (self.second.get)(&*guard).map(|v| v.clone()))
        })
    }

    pub fn set<F>(&self, root: &R, updater: F) -> Result<(), String>
    where
        F: FnOnce(&mut V),
    {
        (self.first.get)(root)
            .ok_or_else(|| "Failed to get Arc<Mutex>".to_string())
            .and_then(|arc_mutex| {
                arc_mutex
                    .lock()
                    .map_err(|e| format!("Failed to lock mutex: {}", e))
                    .and_then(|mut guard| {
                        (self.second.set)(&mut *guard)
                            .ok_or_else(|| "Failed to get value".to_string())
                            .map(|value| {
                                updater(value);
                            })
                    })
            })
    }

    // Compose with a regular KpType (V -> SV)
    pub fn then<SV, G4, S4>(
        self,
        next: Kp<V, SV, &'static V, &'static SV, &'static mut V, &'static mut SV, G4, S4>,
    ) -> LockKp<
        R,
        SV,
        MV,
        G1,
        S1,
        for<'a> fn(&'a MV) -> Option<&'a SV>,
        for<'a> fn(&'a mut MV) -> Option<&'a mut SV>,
    >
    where
        SV: 'static + Clone,
        G4: Fn(&V) -> Option<&SV> + 'static,
        S4: Fn(&mut V) -> Option<&mut SV> + 'static,
    {
        fn new_get<MV, V, SV, G3, G4>(
            second: Kp<
                MV,
                V,
                &'static MV,
                &'static V,
                &'static mut MV,
                &'static mut V,
                G3,
                impl Fn(&mut MV) -> Option<&mut V>,
            >,
            next: Kp<
                V,
                SV,
                &'static V,
                &'static SV,
                &'static mut V,
                &'static mut SV,
                G4,
                impl Fn(&mut V) -> Option<&mut SV>,
            >,
        ) -> impl Fn(&MV) -> Option<&SV>
        where
            G3: Fn(&MV) -> Option<&V>,
            G4: Fn(&V) -> Option<&SV>,
        {
            move |mv: &MV| (second.get)(mv).and_then(|v| (next.get)(v))
        }

        fn new_set<MV, V, SV, S3, S4>(
            second: Kp<
                MV,
                V,
                &'static MV,
                &'static V,
                &'static mut MV,
                &'static mut V,
                impl Fn(&MV) -> Option<&V>,
                S3,
            >,
            next: Kp<
                V,
                SV,
                &'static V,
                &'static SV,
                &'static mut V,
                &'static mut SV,
                impl Fn(&V) -> Option<&SV>,
                S4,
            >,
        ) -> impl Fn(&mut MV) -> Option<&mut SV>
        where
            S3: Fn(&mut MV) -> Option<&mut V>,
            S4: Fn(&mut V) -> Option<&mut SV>,
        {
            move |mv: &mut MV| (second.set)(mv).and_then(|v| (next.set)(v))
        }

        // Store the closures in static variables won't work, we need a different approach
        // We need to use function pointers, so we'll box the keypaths

        let second_box = Box::new(self.second);
        let next_box = Box::new(next);

        let get_fn: for<'a> fn(&'a MV) -> Option<&'a SV> = |mv: &MV| -> Option<&SV> {
            // This won't work because we can't access the boxes here
            None
        };

        let set_fn: for<'a> fn(&'a mut MV) -> Option<&'a mut SV> =
            |mv: &mut MV| -> Option<&mut SV> { None };

        LockKp {
            first: self.first,
            second: Kp::new(get_fn, set_fn),
        }
    }

    // Compose with another LockKp (V -> Arc<Mutex<MV2>> -> SV)
    // pub fn then_lock<SV, MV2, G4, S4, G5, S5>(
    //     self,
    //     next: LockKp<V, SV, MV2, G4, S4, G5, S5>,
    // ) -> LockKp<R, SV, MV, G1, S1, impl Fn(&MV) -> Option<&SV>, impl Fn(&mut MV) -> Option<&mut SV>>
    // where
    //     SV: 'static + Clone,
    //     MV2: 'static,
    //     G4: Fn(&V) -> Option<&Arc<Mutex<MV2>>> + 'static,
    //     S4: Fn(&mut V) -> Option<&mut Arc<Mutex<MV2>>> + 'static,
    //     G5: Fn(&MV2) -> Option<&SV> + 'static,
    //     S5: Fn(&mut MV2) -> Option<&mut SV> + 'static,
    // {
    //     let new_second = Kp::new(
    //         move |mv: &MV| {
    //             (self.second.get)(mv)
    //                 .and_then(|v| {
    //                     (next.first.get)(v).and_then(|arc_mutex| {
    //                         arc_mutex
    //                             .lock()
    //                             .ok()
    //                             .and_then(|guard| (next.second.get)(&*guard).map(|sv| sv.clone()))
    //                     })
    //                 })
    //                 .as_ref()
    //                 .map(|sv| sv as &SV)
    //         },
    //         move |mv: &mut MV| {
    //             (self.second.set)(mv).and_then(|v| {
    //                 (next.first.get)(v).and_then(|arc_mutex| {
    //                     arc_mutex
    //                         .lock()
    //                         .ok()
    //                         .and_then(|mut guard| (next.second.set)(&mut *guard))
    //                 })
    //             })
    //         },
    //     );
    //
    //     LockKp::new(self.first, new_second)
    // }
}

// Usage example
#[derive(Debug, Clone)]
struct A {
    b: Arc<Mutex<B>>,
}

#[derive(Debug, Clone)]
struct B {
    c: C,
    e: Arc<Mutex<E>>,
}

#[derive(Debug, Clone)]
struct C {
    d: String,
}

#[derive(Debug, Clone)]
struct E {
    f: String,
}

impl A {
    fn b() -> KpType<'static, A, Arc<Mutex<B>>> {
        Kp::new(|a: &A| Some(&a.b), |a: &mut A| Some(&mut a.b))
    }
}

impl B {
    fn c() -> KpType<'static, B, C> {
        Kp::new(|b: &B| Some(&b.c), |b: &mut B| Some(&mut b.c))
    }

    fn e() -> KpType<'static, B, Arc<Mutex<E>>> {
        Kp::new(|b: &B| Some(&b.e), |b: &mut B| Some(&mut b.e))
    }
}

impl C {
    fn d() -> KpType<'static, C, String> {
        Kp::new(|c: &C| Some(&c.d), |c: &mut C| Some(&mut c.d))
    }
}

impl E {
    fn f() -> KpType<'static, E, String> {
        Kp::new(|e: &E| Some(&e.f), |e: &mut E| Some(&mut e.f))
    }
}

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
                std::sync::Arc::get_mut(&mut arc_root)
                    .and_then(|r_mut| (&self.set)(MutRoot::from(r_mut)))
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
    pub fn identity_typed() -> Kp<
        R,
        R,
        Root,
        Root,
        MutRoot,
        MutRoot,
        fn(Root) -> Option<Root>,
        fn(MutRoot) -> Option<MutRoot>,
    > {
        Kp::new(|r: Root| Some(r), |r: MutRoot| Some(r))
    }

    pub fn identity<'a>() -> KpType<'a, R, R> {
        KpType::new(|r| Some(r), |r| Some(r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
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

        fn c<'a>() -> KpType<'a, TestKP, String> {
            KpType::new(
                |r: &TestKP| Some(r.c.as_ref()),
                |r: &mut TestKP| match std::sync::Arc::get_mut(&mut r.c) {
                    Some(arc_str) => Some(arc_str),
                    None => None,
                },
            )
        }

        fn a<'a>() -> KpType<'a, TestKP, String> {
            KpType::new(|r: &TestKP| Some(&r.a), |r: &mut TestKP| Some(&mut r.a))
        }

        fn f<'a>() -> KpType<'a, TestKP, TestKP2> {
            KpType::new(|r: &TestKP| r.f.as_ref(), |r: &mut TestKP| r.f.as_mut())
        }

        fn identity<'a>() -> KpType<'a, TestKP, TestKP> {
            KpType::identity()
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
    #[test]
    fn test_a() {
        let instance2 = TestKP2::new();
        let mut instance = TestKP::new();
        let kp = TestKP::identity();
        let kp_a = TestKP::a();
        // TestKP::a().for_arc();
        let kp_f = TestKP::f();
        let wres = kp_f.then(TestKP2::a()).get_mut(&mut instance).unwrap();
        *wres = String::from("a3 changed successfully");
        let res = kp_f.then(TestKP2::a()).get(&instance);
        println!("{:?}", res);
        let res = kp_f.then(TestKP2::identity()).get(&instance);
        println!("{:?}", res);
        let res = kp.get(&instance);
        println!("{:?}", res);
    }

    #[test]
    fn test_lock() {
        let lock_kp = LockKp::new(A::b(), kp_arc_mutex::<B>(), B::c());

        let mut a = A {
            b: Arc::new(Mutex::new(B {
                c: C {
                    d: String::from("hello"),
                },
            })),
        };

        // Get value
        if let Some(value) = lock_kp.get(&a) {
            println!("Got: {:?}", value);
            assert_eq!(value.d, "hello");
        } else {
            panic!("Value not found");
        }

        // Set value using closure
        let result = lock_kp.set(&a, |d| {
            d.d.push_str(" world");
        });

        if result.is_ok() {
            if let Some(value) = lock_kp.get(&a) {
                println!("After set: {:?}", value);
                assert_eq!(value.d, "hello");
            } else {
                panic!("Value not found");
            }
        }
    }
}
