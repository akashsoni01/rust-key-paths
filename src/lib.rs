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
        SubValue: Borrow<SV>,
        MutSubValue: BorrowMut<SV>,
        G2: Fn(Value) -> Option<SubValue>,
        S2: Fn(MutValue) -> Option<MutSubValue>,
        V: 'static,
    {
        Kp::new(
            move |root: Root| (&self.get)(root).and_then(|value| (next.get)(value)),
            move |root: MutRoot| (&self.set)(root).and_then(|value| (next.set)(value)),
        )
    }

    /// need some more work here.
    pub fn for_arc(
        &self,
    ) -> Kp<
        Arc<R>,
        V,
        Arc<R>,
        Value,
        Arc<R>,
        MutValue,
        impl Fn(Arc<R>) -> Option<Value>,
        impl Fn(Arc<R>) -> Option<MutValue>,
    >
    where
        R: Clone + 'static,
        V: 'static,
        Root: From<R>,
        MutRoot: From<R>,
    {
        Kp::new(
            move |arc_root: Arc<R>| {
                let r = (*arc_root).clone();
                (&self.get)(Root::from(r))
            },
            move |arc_root: Arc<R>| {
                // Try to unwrap Arc to get exclusive ownership
                match Arc::try_unwrap(arc_root) {
                    Ok(r) => (&self.set)(MutRoot::from(r)),
                    Err(_) => None, // Can't mutate if there are multiple references
                }
            },
        )
    }

    // pub fn for_arc<'a, NewRoot, NewMutRoot>(
    //     &self,
    // ) -> Kp<
    //     Arc<R>,
    //     V,
    //     Root,
    //     Value,
    //     MutRoot,
    //     MutValue,
    //     impl Fn(NewRoot) -> Option<Value>,
    //     impl Fn(NewMutRoot) -> Option<MutValue>,
    // > where
    //     NewRoot: Borrow<Arc<R>> + 'a,
    //     NewMutRoot: BorrowMut<Arc<R>> + 'a,
    // {
    //     todo!()
    //     // Kp::new(
    //     //     move |root: Root| (&self.get)(root).and_then(|value| (next.get)(value)),
    //     //     move |root: MutRoot| (&self.set)(root).and_then(|value| (next.set)(value)),
    //     // )
    // }
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
    fn a<'a>() -> KpType<'a, TestKP2, String> {
        Kp::new(|r: &TestKP2| Some(&r.a), |r: &mut TestKP2| Some(&mut r.a))
    }

    // example - cloning arc mutex
    fn b<'a>() -> KpType<'a, TestKP2, Arc<std::sync::Mutex<TestKP3>>> {
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
            b: Arc::new(std::sync::Mutex::new(TestKP3::new())),
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
    //     Root: Borrow<TestKP2>,
    //     MutRoot: BorrowMut<TestKP2>,
    //     G: Fn(Root) -> Option<Root>,
    //     S: Fn(MutRoot) -> Option<MutRoot>,
    // {
    //     Kp::<TestKP2, TestKP2, Root, Root, MutRoot, MutRoot, G, S>::identity()
    // }

    fn a<'a>() -> KpType<'a, TestKP2, String> {
        KpType::new(|r: &TestKP2| Some(&r.a), |r: &mut TestKP2| Some(&mut r.a))
    }

    fn b<'a>() -> KpType<'a, TestKP2, Arc<std::sync::Mutex<TestKP3>>> {
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
            b: Arc::new(std::sync::Mutex::new(String::from("b2"))),
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
    //     Root: Borrow<TestKP2>,
    //     MutRoot: BorrowMut<TestKP2>,
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
