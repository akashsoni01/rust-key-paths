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

// ========== ENUM KEYPATHS ==========

/// EnumKp - A keypath for enum variants that supports both extraction and embedding
/// Leverages the existing Kp architecture where optionals are built-in via Option<Value>
///
/// This struct serves dual purposes:
/// 1. As a concrete keypath instance for extracting and embedding enum variants
/// 2. As a namespace for static factory methods: `EnumKp::for_ok()`, `EnumKp::for_some()`, etc.
pub struct EnumKp<Enum, Variant, Root, Value, MutRoot, MutValue, G, S, E>
where
    Root: std::borrow::Borrow<Enum>,
    Value: std::borrow::Borrow<Variant>,
    MutRoot: std::borrow::BorrowMut<Enum>,
    MutValue: std::borrow::BorrowMut<Variant>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
    E: Fn(Variant) -> Enum,
{
    extractor: Kp<Enum, Variant, Root, Value, MutRoot, MutValue, G, S>,
    embedder: E,
}

impl<Enum, Variant, Root, Value, MutRoot, MutValue, G, S, E>
    EnumKp<Enum, Variant, Root, Value, MutRoot, MutValue, G, S, E>
where
    Root: std::borrow::Borrow<Enum>,
    Value: std::borrow::Borrow<Variant>,
    MutRoot: std::borrow::BorrowMut<Enum>,
    MutValue: std::borrow::BorrowMut<Variant>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
    E: Fn(Variant) -> Enum,
{
    /// Create a new EnumKp with extractor and embedder functions
    pub fn new(
        extractor: Kp<Enum, Variant, Root, Value, MutRoot, MutValue, G, S>,
        embedder: E,
    ) -> Self {
        Self {
            extractor,
            embedder,
        }
    }

    /// Extract the variant from an enum (returns None if wrong variant)
    pub fn get(&self, enum_value: Root) -> Option<Value> {
        self.extractor.get(enum_value)
    }

    /// Extract the variant mutably from an enum (returns None if wrong variant)
    pub fn get_mut(&self, enum_value: MutRoot) -> Option<MutValue> {
        self.extractor.get_mut(enum_value)
    }

    /// Embed a value into the enum variant
    pub fn embed(&self, value: Variant) -> Enum {
        (self.embedder)(value)
    }

    /// Get the underlying Kp for composition with other keypaths
    pub fn as_kp(&self) -> &Kp<Enum, Variant, Root, Value, MutRoot, MutValue, G, S> {
        &self.extractor
    }

    /// Convert to Kp (loses embedding capability but gains composition)
    pub fn into_kp(self) -> Kp<Enum, Variant, Root, Value, MutRoot, MutValue, G, S> {
        self.extractor
    }
}

// Type alias for the common case with references
pub type EnumKpType<'a, Enum, Variant> = EnumKp<
    Enum,
    Variant,
    &'a Enum,
    &'a Variant,
    &'a mut Enum,
    &'a mut Variant,
    for<'b> fn(&'b Enum) -> Option<&'b Variant>,
    for<'b> fn(&'b mut Enum) -> Option<&'b mut Variant>,
    fn(Variant) -> Enum,
>;

// Static factory functions for creating EnumKp instances
/// Create an enum keypath with both extraction and embedding for a specific variant
///
/// # Example
/// ```
/// enum MyEnum {
///     A(String),
///     B(i32),
/// }
///
/// let kp = enum_variant(
///     |e: &MyEnum| match e { MyEnum::A(s) => Some(s), _ => None },
///     |e: &mut MyEnum| match e { MyEnum::A(s) => Some(s), _ => None },
///     |s: String| MyEnum::A(s)
/// );
/// ```
pub fn enum_variant<'a, Enum, Variant>(
    getter: for<'b> fn(&'b Enum) -> Option<&'b Variant>,
    setter: for<'b> fn(&'b mut Enum) -> Option<&'b mut Variant>,
    embedder: fn(Variant) -> Enum,
) -> EnumKpType<'a, Enum, Variant> {
    EnumKp::new(Kp::new(getter, setter), embedder)
}

/// Extract from Result<T, E> - Ok variant
///
/// # Example
/// ```
/// let result: Result<String, i32> = Ok("success".to_string());
/// let ok_kp = enum_ok();
/// assert_eq!(ok_kp.get(&result), Some(&"success".to_string()));
/// ```
pub fn enum_ok<'a, T, E>() -> EnumKpType<'a, Result<T, E>, T> {
    EnumKp::new(
        Kp::new(
            |r: &Result<T, E>| r.as_ref().ok(),
            |r: &mut Result<T, E>| r.as_mut().ok(),
        ),
        |t: T| Ok(t),
    )
}

/// Extract from Result<T, E> - Err variant
///
/// # Example
/// ```
/// let result: Result<String, i32> = Err(42);
/// let err_kp = enum_err();
/// assert_eq!(err_kp.get(&result), Some(&42));
/// ```
pub fn enum_err<'a, T, E>() -> EnumKpType<'a, Result<T, E>, E> {
    EnumKp::new(
        Kp::new(
            |r: &Result<T, E>| r.as_ref().err(),
            |r: &mut Result<T, E>| r.as_mut().err(),
        ),
        |e: E| Err(e),
    )
}

/// Extract from Option<T> - Some variant
///
/// # Example
/// ```
/// let opt = Some("value".to_string());
/// let some_kp = enum_some();
/// assert_eq!(some_kp.get(&opt), Some(&"value".to_string()));
/// ```
pub fn enum_some<'a, T>() -> EnumKpType<'a, Option<T>, T> {
    EnumKp::new(
        Kp::new(|o: &Option<T>| o.as_ref(), |o: &mut Option<T>| o.as_mut()),
        |t: T| Some(t),
    )
}

// Helper functions for creating enum keypaths with type inference
/// Create an enum keypath for a specific variant with type inference
///
/// # Example
/// ```
/// enum MyEnum {
///     A(String),
///     B(i32),
/// }
///
/// let kp_a = variant_of(
///     |e: &MyEnum| match e { MyEnum::A(s) => Some(s), _ => None },
///     |e: &mut MyEnum| match e { MyEnum::A(s) => Some(s), _ => None },
///     |s: String| MyEnum::A(s)
/// );
/// ```
pub fn variant_of<'a, Enum, Variant>(
    getter: for<'b> fn(&'b Enum) -> Option<&'b Variant>,
    setter: for<'b> fn(&'b mut Enum) -> Option<&'b mut Variant>,
    embedder: fn(Variant) -> Enum,
) -> EnumKpType<'a, Enum, Variant> {
    enum_variant(getter, setter, embedder)
}

// ========== CONTAINER KEYPATHS ==========

// Helper functions for working with standard containers (Box, Arc, Rc)
/// Create a keypath for unwrapping Box<T> -> T
///
/// # Example
/// ```
/// let boxed = Box::new("value".to_string());
/// let kp = kp_box();
/// assert_eq!(kp.get(&boxed), Some(&"value".to_string()));
/// ```
pub fn kp_box<'a, T>() -> KpType<'a, Box<T>, T> {
    Kp::new(
        |b: &Box<T>| Some(b.as_ref()),
        |b: &mut Box<T>| Some(b.as_mut()),
    )
}

/// Create a keypath for unwrapping Arc<T> -> T (read-only)
///
/// # Example
/// ```
/// let arc = Arc::new("value".to_string());
/// let kp = kp_arc();
/// assert_eq!(kp.get(&arc), Some(&"value".to_string()));
/// ```
pub fn kp_arc<'a, T>() -> Kp<
    Arc<T>,
    T,
    &'a Arc<T>,
    &'a T,
    &'a mut Arc<T>,
    &'a mut T,
    for<'b> fn(&'b Arc<T>) -> Option<&'b T>,
    for<'b> fn(&'b mut Arc<T>) -> Option<&'b mut T>,
> {
    Kp::new(
        |arc: &Arc<T>| Some(arc.as_ref()),
        |arc: &mut Arc<T>| Arc::get_mut(arc),
    )
}

/// Create a keypath for unwrapping Rc<T> -> T (read-only)
///
/// # Example
/// ```
/// let rc = Rc::new("value".to_string());
/// let kp = kp_rc();
/// assert_eq!(kp.get(&rc), Some(&"value".to_string()));
/// ```
pub fn kp_rc<'a, T>() -> Kp<
    std::rc::Rc<T>,
    T,
    &'a std::rc::Rc<T>,
    &'a T,
    &'a mut std::rc::Rc<T>,
    &'a mut T,
    for<'b> fn(&'b std::rc::Rc<T>) -> Option<&'b T>,
    for<'b> fn(&'b mut std::rc::Rc<T>) -> Option<&'b mut T>,
> {
    Kp::new(
        |rc: &std::rc::Rc<T>| Some(rc.as_ref()),
        |rc: &mut std::rc::Rc<T>| std::rc::Rc::get_mut(rc),
    )
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
        fn a_typed<Root, MutRoot, Value, MutValue>() -> Kp<
            TestKP2,
            String,
            Root,
            Value,
            MutRoot,
            MutValue,
            impl Fn(Root) -> Option<Value>,
            impl Fn(MutRoot) -> Option<MutValue>,
        >
        where
            Root: std::borrow::Borrow<TestKP2>,
            MutRoot: std::borrow::BorrowMut<TestKP2>,
            Value: std::borrow::Borrow<String> + From<String>,
            MutValue: std::borrow::BorrowMut<String> + From<String>,
        {
            Kp::new(
                |r: Root| Some(Value::from(r.borrow().a.clone())),
                |mut r: MutRoot| Some(MutValue::from(r.borrow_mut().a.clone())),
            )
        }

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

        fn identity_typed<Root, MutRoot, G, S>() -> Kp<
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
            Root: std::borrow::Borrow<TestKP2>,
            MutRoot: std::borrow::BorrowMut<TestKP2>,
            G: Fn(Root) -> Option<Root>,
            S: Fn(MutRoot) -> Option<MutRoot>,
        {
            Kp::<TestKP2, TestKP2, Root, Root, MutRoot, MutRoot, G, S>::identity_typed()
        }

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

        fn identity_typed<Root, MutRoot, G, S>() -> Kp<
            TestKP3, // R
            TestKP3, // V
            Root,    // Root
            Root,    // Value
            MutRoot, // MutRoot
            MutRoot, // MutValue
            fn(Root) -> Option<Root>,
            fn(MutRoot) -> Option<MutRoot>,
        >
        where
            Root: std::borrow::Borrow<TestKP3>,
            MutRoot: std::borrow::BorrowMut<TestKP3>,
            G: Fn(Root) -> Option<Root>,
            S: Fn(MutRoot) -> Option<MutRoot>,
        {
            Kp::<TestKP3, TestKP3, Root, Root, MutRoot, MutRoot, G, S>::identity_typed()
        }

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

    // #[test]
    // fn test_lock() {
    //     let lock_kp = LockKp::new(A::b(), kp_arc_mutex::<B>(), B::c());
    //
    //     let mut a = A {
    //         b: Arc::new(Mutex::new(B {
    //             c: C {
    //                 d: String::from("hello"),
    //             },
    //         })),
    //     };
    //
    //     // Get value
    //     if let Some(value) = lock_kp.get(&a) {
    //         println!("Got: {:?}", value);
    //         assert_eq!(value.d, "hello");
    //     } else {
    //         panic!("Value not found");
    //     }
    //
    //     // Set value using closure
    //     let result = lock_kp.set(&a, |d| {
    //         d.d.push_str(" world");
    //     });
    //
    //     if result.is_ok() {
    //         if let Some(value) = lock_kp.get(&a) {
    //             println!("After set: {:?}", value);
    //             assert_eq!(value.d, "hello");
    //         } else {
    //             panic!("Value not found");
    //         }
    //     }
    // }

    #[test]
    fn test_enum_kp_result_ok() {
        let ok_result: Result<String, i32> = Ok("success".to_string());
        let mut err_result: Result<String, i32> = Err(42);

        let ok_kp = enum_ok();

        // Test extraction
        assert_eq!(ok_kp.get(&ok_result), Some(&"success".to_string()));
        assert_eq!(ok_kp.get(&err_result), None);

        // Test embedding
        let embedded = ok_kp.embed("embedded".to_string());
        assert_eq!(embedded, Ok("embedded".to_string()));

        // Test mutable access
        if let Some(val) = ok_kp.get_mut(&mut err_result) {
            *val = "modified".to_string();
        }
        assert_eq!(err_result, Err(42)); // Should still be Err

        let mut ok_result2 = Ok("original".to_string());
        if let Some(val) = ok_kp.get_mut(&mut ok_result2) {
            *val = "modified".to_string();
        }
        assert_eq!(ok_result2, Ok("modified".to_string()));
    }

    #[test]
    fn test_enum_kp_result_err() {
        let ok_result: Result<String, i32> = Ok("success".to_string());
        let mut err_result: Result<String, i32> = Err(42);

        let err_kp = enum_err();

        // Test extraction
        assert_eq!(err_kp.get(&err_result), Some(&42));
        assert_eq!(err_kp.get(&ok_result), None);

        // Test embedding
        let embedded = err_kp.embed(99);
        assert_eq!(embedded, Err(99));

        // Test mutable access
        if let Some(val) = err_kp.get_mut(&mut err_result) {
            *val = 100;
        }
        assert_eq!(err_result, Err(100));
    }

    #[test]
    fn test_enum_kp_option_some() {
        let some_opt = Some("value".to_string());
        let mut none_opt: Option<String> = None;

        let some_kp = enum_some();

        // Test extraction
        assert_eq!(some_kp.get(&some_opt), Some(&"value".to_string()));
        assert_eq!(some_kp.get(&none_opt), None);

        // Test embedding
        let embedded = some_kp.embed("embedded".to_string());
        assert_eq!(embedded, Some("embedded".to_string()));

        // Test mutable access
        let mut some_opt2 = Some("original".to_string());
        if let Some(val) = some_kp.get_mut(&mut some_opt2) {
            *val = "modified".to_string();
        }
        assert_eq!(some_opt2, Some("modified".to_string()));
    }

    #[test]
    fn test_enum_kp_custom_enum() {
        #[derive(Debug, PartialEq)]
        enum MyEnum {
            A(String),
            B(i32),
            C,
        }

        let mut enum_a = MyEnum::A("hello".to_string());
        let enum_b = MyEnum::B(42);
        let enum_c = MyEnum::C;

        // Create keypath for variant A
        let kp_a = enum_variant(
            |e: &MyEnum| match e {
                MyEnum::A(s) => Some(s),
                _ => None,
            },
            |e: &mut MyEnum| match e {
                MyEnum::A(s) => Some(s),
                _ => None,
            },
            |s: String| MyEnum::A(s),
        );

        // Test extraction
        assert_eq!(kp_a.get(&enum_a), Some(&"hello".to_string()));
        assert_eq!(kp_a.get(&enum_b), None);
        assert_eq!(kp_a.get(&enum_c), None);

        // Test embedding
        let embedded = kp_a.embed("world".to_string());
        assert_eq!(embedded, MyEnum::A("world".to_string()));

        // Test mutable access
        if let Some(val) = kp_a.get_mut(&mut enum_a) {
            *val = "modified".to_string();
        }
        assert_eq!(enum_a, MyEnum::A("modified".to_string()));
    }

    #[test]
    fn test_container_kp_box() {
        let boxed = Box::new("value".to_string());
        let mut boxed_mut = Box::new("original".to_string());

        let box_kp = kp_box();

        // Test get
        assert_eq!(box_kp.get(&boxed), Some(&"value".to_string()));

        // Test get_mut
        if let Some(val) = box_kp.get_mut(&mut boxed_mut) {
            *val = "modified".to_string();
        }
        assert_eq!(*boxed_mut, "modified".to_string());
    }

    #[test]
    fn test_container_kp_arc() {
        let arc = Arc::new("value".to_string());
        let mut arc_mut = Arc::new("original".to_string());

        let arc_kp = kp_arc();

        // Test get
        assert_eq!(arc_kp.get(&arc), Some(&"value".to_string()));

        // Test get_mut (only works if Arc has no other references)
        if let Some(val) = arc_kp.get_mut(&mut arc_mut) {
            *val = "modified".to_string();
        }
        assert_eq!(*arc_mut, "modified".to_string());

        // Test with multiple references (should return None for mutable access)
        let arc_shared = Arc::new("shared".to_string());
        let arc_shared2 = Arc::clone(&arc_shared);
        let mut arc_shared_mut = arc_shared;
        assert_eq!(arc_kp.get_mut(&mut arc_shared_mut), None);
    }

    #[test]
    fn test_enum_kp_composition() {
        // Test composing enum keypath with other keypaths
        #[derive(Debug, PartialEq)]
        struct Inner {
            value: String,
        }

        let result: Result<Inner, i32> = Ok(Inner {
            value: "nested".to_string(),
        });

        // Create keypath to Inner.value
        let inner_kp = KpType::new(
            |i: &Inner| Some(&i.value),
            |i: &mut Inner| Some(&mut i.value),
        );

        // Get the Ok keypath and convert to Kp for composition
        let ok_kp = enum_ok::<Inner, i32>();
        let ok_kp_base = ok_kp.into_kp();
        let composed = ok_kp_base.then(inner_kp);

        assert_eq!(composed.get(&result), Some(&"nested".to_string()));
    }
}
