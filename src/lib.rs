use std::borrow::{Borrow, BorrowMut};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

// pub type KpType<R, V, Root, Value, MutRoot, MutValue, G, S>
// where
//     Root: ,
//     Value:    Borrow<V>,
//     MutRoot:  BorrowMut<R>,
//     MutValue: BorrowMut<V>,
//     G:        Fn(Root) -> Option<Value>,
//     S:        Fn(MutRoot) -> Option<MutValue> = Kp<R, V, Root, Value, MutRoot, MutValue, G, S>;

// type Getter<R, V, Root, Value> where Root: Borrow<R>, Value: Borrow<V> = fn(Root) -> Option<Value>;
// type Setter<R, V> = fn(&'r mut R) -> Option<&'r mut V>;

pub struct Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
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

impl<R, V, Root, Value, MutRoot, MutValue, G, S> Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
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

impl<R, Root, MutRoot, G, S> Kp<R, R, Root, Root, MutRoot, MutRoot, G, S>
where
    Root: Borrow<R>,
    MutRoot: BorrowMut<R>,
    G: Fn(Root) -> Option<Root>,
    S: Fn(MutRoot) -> Option<MutRoot>,
{
    pub fn identity() -> Kp<
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
            c: Arc::new(String::from("c")),
            d: std::sync::Mutex::new(String::from("d")),
            e: Arc::new(std::sync::Mutex::new(TestKP2::new())),
            f: Some(TestKP2 {
                a: String::from("a3"),
                b: Arc::new(std::sync::Mutex::new(TestKP3::new())),
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
    //     Root: Borrow<TestKP2>,
    //     MutRoot: BorrowMut<TestKP2>,
    //     Value: Borrow<String> + From<String>,
    //     MutValue: BorrowMut<String> + From<String>,
    // {
    //     Kp::new(
    //         |r: Root| Some(Value::from(r.borrow().a.clone())),
    //         |mut r: MutRoot| Some(MutValue::from(r.borrow_mut().a.clone())),
    //     )
    // }

    // Example for taking ref
    fn a<'a>() -> Kp<
        TestKP2,
        String,
        &'a TestKP2,
        &'a String,
        &'a mut TestKP2,
        &'a mut String,
        for<'b> fn(&'b TestKP2) -> Option<&'b String>,
        for<'b> fn(&'b mut TestKP2) -> Option<&'b mut String>,
    > {
        Kp::new(|r: &TestKP2| Some(&r.a), |r: &mut TestKP2| Some(&mut r.a))
    }

    // example - cloning arc mutex
    fn b<Root, MutRoot, Value, MutValue>() -> Kp<
        TestKP2,
        Arc<std::sync::Mutex<TestKP3>>,
        Root,
        Value,
        MutRoot,
        MutValue,
        impl Fn(Root) -> Option<Value>,
        impl Fn(MutRoot) -> Option<MutValue>,
    >
    where
        Root: Borrow<TestKP2>,
        MutRoot: BorrowMut<TestKP2>,
        Value: Borrow<Arc<std::sync::Mutex<TestKP3>>> + From<Arc<std::sync::Mutex<TestKP3>>>,
        MutValue: BorrowMut<Arc<std::sync::Mutex<TestKP3>>> + From<Arc<std::sync::Mutex<TestKP3>>>,
    {
        Kp::new(
            |r: Root| Some(Value::from(r.borrow().b.clone())),
            |mut r: MutRoot| Some(MutValue::from(r.borrow_mut().b.clone())),
        )
    }
    // Helper to create an identity keypath for TestKP2
    fn identity<Root, MutRoot, G, S>() -> Kp<
        TestKP2, // R
        TestKP2, // V
        Root,    // Root
        Root,    // Value
        MutRoot, // MutRoot
        MutRoot, // MutValue
        fn(Root) -> Option<Root>,
        fn(MutRoot) -> Option<MutRoot>,
    >
    where
        Root: Borrow<TestKP2>,
        MutRoot: BorrowMut<TestKP2>,
        G: Fn(Root) -> Option<Root>,
        S: Fn(MutRoot) -> Option<MutRoot>,
    {
        Kp::<TestKP2, TestKP2, Root, Root, MutRoot, MutRoot, G, S>::identity()
    }
}

struct TestKP2 {
    a: String,
    b: std::sync::Arc<std::sync::Mutex<TestKP3>>,
}

impl TestKP2 {
    fn new() -> Self {
        TestKP2 {
            a: String::from("a2"),
            b: Arc::new(std::sync::Mutex::new(TestKP3::new())),
        }
    }

    fn identity<Root, MutRoot, G, S>() -> Kp<
        TestKP2, // R
        TestKP2, // V
        Root,    // Root
        Root,    // Value
        MutRoot, // MutRoot
        MutRoot, // MutValue
        fn(Root) -> Option<Root>,
        fn(MutRoot) -> Option<MutRoot>,
    >
    where
        Root: Borrow<TestKP2>,
        MutRoot: BorrowMut<TestKP2>,
        G: Fn(Root) -> Option<Root>,
        S: Fn(MutRoot) -> Option<MutRoot>,
    {
        Kp::<TestKP2, TestKP2, Root, Root, MutRoot, MutRoot, G, S>::identity()
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
            b: Arc::new(std::sync::Mutex::new(String::from("b2"))),
        }
    }

    fn identity<Root, MutRoot, G, S>() -> Kp<
        TestKP2, // R
        TestKP2, // V
        Root,    // Root
        Root,    // Value
        MutRoot, // MutRoot
        MutRoot, // MutValue
        fn(Root) -> Option<Root>,
        fn(MutRoot) -> Option<MutRoot>,
    >
    where
        Root: Borrow<TestKP2>,
        MutRoot: BorrowMut<TestKP2>,
        G: Fn(Root) -> Option<Root>,
        S: Fn(MutRoot) -> Option<MutRoot>,
    {
        Kp::<TestKP2, TestKP2, Root, Root, MutRoot, MutRoot, G, S>::identity()
    }
}

impl TestKP3 {}

impl TestKP {}
