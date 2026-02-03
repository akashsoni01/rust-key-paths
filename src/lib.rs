use std::borrow::{Borrow, BorrowMut};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

// use std::marker::PhantomData;
// use std::rc::Rc;
// use std::sync::{Arc, Mutex};
//
// type Getter<R, V> = for<'r> fn(&'r R) -> Option<&'r V>;
// type Setter<R, V> = for<'r> fn(&'r mut R) -> Option<&'r mut V>;
// // type LockGetter<R, V> = for<'r> fn(&'r R) -> Option<std::sync::Arc<&'r V>>;
// // type LockSetter<R, V> = for<'r> fn(&'r mut R) -> Option<std::sync::Arc<&'r mut V>>;
//
// pub type Kp<R, V> = KpType<
//     R,
//     V,
//     Getter<R, V>,
//     Setter<R, V>,
//     //  LockGetter<R, V>,
//     //  LockSetter<R, V>
// >;
//
pub struct Kp<R,V, Root, Value, MutRoot, MutValue,G, S>
    where
    Root: Borrow<R>,
    MutRoot: BorrowMut<R>,
    MutValue: BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    get: G,
    set: S,
    _p: PhantomData<(R, V, Root, Value, MutRoot, MutValue)>,
}

impl<R,V, Root, Value, MutRoot, MutValue,G, S> Kp<R,V, Root, Value, MutRoot, MutValue,G, S>
where
    Root: Borrow<R>,
    Value: Borrow<V>,
    MutRoot: BorrowMut<R>,
    MutValue: BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    pub fn new(get: G, set: S) -> Self {
        Self {
            get: get,
            set: set,
            _p: PhantomData,
        }
    }

    fn get(&self, root: Root) -> Option<Value> {
        (self.get)(root)
    }
    fn get_mut(&self, root: MutRoot) -> Option<MutValue> {
        (self.set)(root)
    }

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
        // wrong syntax - expected fn pointer, found closure
        // fn(Root) -> Option<SubValue>,
        // fn(MutRoot) -> Option<MutSubValue>,
        impl Fn(Root) -> Option<SubValue>,
        impl Fn(MutRoot) -> Option<MutSubValue>,
    >
    where
        SubValue: Borrow<SV>,
        MutSubValue: BorrowMut<SV>,
        G2: Fn(Value) -> Option<SubValue>,
        S2: Fn(MutValue) -> Option<MutSubValue>,
        V: 'static,
    {
        Kp::new(
            move |root: Root| (self.get)(root).and_then(|value| (next.get)(value)),
            move |root: MutRoot| (self.set)(root).and_then(|value| (next.set)(value)),
        )
    }

    //     pub fn for_arc(
    //         self,
    //     ) -> KpType<
    //         std::sync::Arc<R>,
    //         V,
    //         impl for<'r> Fn(&'r std::sync::Arc<R>) -> Option<&'r V>,
    //         impl for<'r> Fn(&'r mut std::sync::Arc<R>) -> Option<&'r mut V>,
    //     > {
    //         KpType {
    //             g: move |root: &std::sync::Arc<R>| {
    //                 // Dereference the Arc to get &R, then apply the original getter
    //                 (self.g)(&root)
    //             },
    //             s: move |root: &mut std::sync::Arc<R>| {
    //                 // For mutable access, we need to handle Arc's interior mutability carefully
    //                 // This assumes R: Send + Sync for thread safety with Arc
    //                 // Note: This will only work if the Arc has exactly one strong reference
    //                 // Otherwise, we cannot get a mutable reference
    //
    //                 // Try to get a mutable reference from Arc
    //                 if let Some(r) = Arc::get_mut(root) {
    //                     (self.s)(r)
    //                 } else {
    //                     None
    //                 }
    //             },
    //             _p: PhantomData,
    //         }
    //     }
    //

}

impl<R, Root, MutRoot,G, S> Kp<R, R, Root, Root, MutRoot, MutRoot, G, S>
where
    Root: Borrow<R>,
    MutRoot: BorrowMut<R>,
    G: Fn(Root) -> Option<Root>,
    S: Fn(MutRoot) -> Option<MutRoot> {
    pub fn identity() -> Kp<
    R, R, Root, Root, MutRoot, MutRoot, impl Fn(Root) -> Option<Root>, impl Fn(MutRoot) -> Option<MutRoot>
    > 
    {
        Kp::new(
            |r: Root| Some(r),
            |r: MutRoot| Some(r),
        )
    }
}

// struct TestKP {
//     a: String,
//     b: String,
//     c: std::sync::Arc<String>,
//     d: Mutex<String>,
//     e: std::sync::Arc<Mutex<TestKP2>>,
//     f: Option<TestKP2>,
// }
//
// impl TestKP {
//     fn new() -> Self {
//         Self {
//             a: String::from("a"),
//             b: String::from("b"),
//             c: Arc::new(String::from("c")),
//             d: Mutex::new(String::from("d")),
//             e: Arc::new(Mutex::new(TestKP2::new())),
//             f: Some(TestKP2 {
//                 a: String::from("a3"),
//                 b: Arc::new(Mutex::new(TestKP3::new())),
//             }),
//         }
//     }
//
//     // Helper to create an identity keypath for TestKP2
//     fn identity() -> Kp<TestKP2, TestKP2> {
//         Kp {
//             g: |r: &TestKP2| Some(r),
//             s: |r: &mut TestKP2| Some(r),
//             _p: PhantomData,
//         }
//     }
// }
//
// struct TestKP2 {
//     a: String,
//     b: std::sync::Arc<Mutex<TestKP3>>,
// }
//
// impl TestKP2 {
//     fn new() -> Self {
//         TestKP2 {
//             a: String::from("a2"),
//             b: Arc::new(Mutex::new(TestKP3::new())),
//         }
//     }
//
//     fn a() -> Kp<TestKP2, String> {
//         Kp {
//             g: |r: &TestKP2| Some(&r.a),
//             s: |r: &mut TestKP2| Some(&mut r.a),
//             _p: PhantomData,
//         }
//     }
//
//     fn b() -> Kp<TestKP2, std::sync::Arc<Mutex<TestKP3>>> {
//         Kp {
//             g: |r: &TestKP2| Some(&r.b),
//             s: |r: &mut TestKP2| Some(&mut r.b),
//             _p: PhantomData,
//         }
//     }
//
//     // fn identity() -> Kp<Self, Self> {
//     //     Kp::identity()
//     // }
// }
//
// #[derive(Debug)]
// struct TestKP3 {
//     a: String,
//     b: std::sync::Arc<Mutex<String>>,
// }
//
// impl TestKP3 {
//     fn new() -> Self {
//         TestKP3 {
//             a: String::from("a2"),
//             b: Arc::new(Mutex::new(String::from("b2"))),
//         }
//     }
//
//     fn a() -> Kp<TestKP3, String> {
//         Kp {
//             g: |r: &TestKP3| Some(&r.a),
//             s: |r: &mut TestKP3| Some(&mut r.a),
//             _p: PhantomData,
//         }
//     }
//
//     // fn b_lock() -> LKp<
//     //     Kp<TestKP2, TestKP3>, // Root
//     //     Kp<TestKP3, String>,  // MutexValue
//     //     TestKP3,              // InnerValue
//     //     String,               // SubValue
//     //     fn(&TestKP3) -> Option<&String>,
//     //     fn(&mut TestKP3) -> Option<&mut String>,
//     // > {
//     //     todo!()
//     // }
//
//     // fn b() -> Kp<std::sync::Arc<TestKP3>, std::sync::Arc<Mutex<String>>> {
//     //         let k = Kp {
//     //             g: |r: &TestKP3| Some(&r.b),
//     //             s: |r: &mut TestKP3| Some(&mut r.b),
//     //             _p: PhantomData,
//     //         };
//     //         k.for_arc_mutex()
//     //     }
//
//     // fn identity() -> Kp<Self, Self> {
//     //     Kp::identity()
//     // }
// }
//
// impl TestKP3 {
//     fn b() -> KpType<
//         // std::sync::Arc<TestKP3>,
//         TestKP3,
//         std::sync::Arc<Mutex<String>>,
//         // impl for<'r> Fn(&'r std::sync::Arc<TestKP3>) -> Option<&'r std::sync::Arc<Mutex<String>>>,
//         impl for<'r> Fn(&'r TestKP3) -> Option<&'r std::sync::Arc<Mutex<String>>>,
//         impl for<'r> Fn(&'r mut TestKP3) -> Option<&'r mut std::sync::Arc<Mutex<String>>>,
//     > {
//         Kp {
//             g: |r: &TestKP3| Some(&r.b),
//             s: |r: &mut TestKP3| Some(&mut r.b),
//             _p: PhantomData,
//         }
//         // k.for_arc()
//     }
// }
//
// impl TestKP {
//     fn a() -> Kp<TestKP, String> {
//         Kp {
//             g: |r: &TestKP| Some(&r.a),
//             s: |r: &mut TestKP| Some(&mut r.a),
//             _p: PhantomData,
//         }
//     }
//
//     fn b() -> Kp<TestKP, String> {
//         Kp {
//             g: |r: &TestKP| Some(&r.b),
//             s: |r: &mut TestKP| Some(&mut r.b),
//             _p: PhantomData,
//         }
//     }
//
//     fn c() -> Kp<TestKP, String> {
//         Kp {
//             g: |r: &TestKP| Some(r.c.as_ref()),
//             s: |r: &mut TestKP| None,
//             _p: PhantomData,
//         }
//     }
//
//     fn d() -> Kp<TestKP, Mutex<String>> {
//         Kp {
//             g: |r: &TestKP| Some(&r.d),
//             s: |r: &mut TestKP| Some(&mut r.d),
//             _p: PhantomData,
//         }
//     }
//
//     fn e() -> Kp<TestKP, std::sync::Arc<Mutex<TestKP2>>> {
//         Kp {
//             g: |r: &TestKP| Some(&r.e),
//             s: |r: &mut TestKP| Some(&mut r.e),
//             _p: PhantomData,
//         }
//     }
//
//     fn f() -> Kp<TestKP, TestKP2> {
//         Kp {
//             g: |r: &TestKP| r.f.as_ref(),
//             s: |r: &mut TestKP| r.f.as_mut(),
//             _p: PhantomData,
//         }
//     }
// }
