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

    /// Map the value through a transformation function
    /// Returns a new keypath that transforms the value when accessed
    /// 
    /// # Example
    /// ```
    /// let user = User { name: "Alice".to_string() };
    /// let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
    /// let len_kp = name_kp.map(|name: &String| name.len());
    /// assert_eq!(len_kp.get(&user), Some(5));
    /// ```
    pub fn map<MappedValue, F>(
        &self,
        mapper: F,
    ) -> Kp<
        R,
        MappedValue,
        Root,
        MappedValue,
        MutRoot,
        MappedValue,
        impl Fn(Root) -> Option<MappedValue>,
        impl Fn(MutRoot) -> Option<MappedValue>,
    >
    where
        F: Fn(&V) -> MappedValue + Copy + 'static,
        V: 'static,
        MappedValue: 'static,
    {
        Kp::new(
            move |root: Root| {
                (&self.get)(root).map(|value| {
                    let v: &V = value.borrow();
                    mapper(v)
                })
            },
            move |root: MutRoot| {
                (&self.set)(root).map(|value| {
                    let v: &V = value.borrow();
                    mapper(v)
                })
            },
        )
    }

    /// Filter the value based on a predicate
    /// Returns None if the predicate returns false, otherwise returns the value
    /// 
    /// # Example
    /// ```
    /// let user = User { age: 30 };
    /// let age_kp = KpType::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));
    /// let adult_kp = age_kp.filter(|age: &i32| *age >= 18);
    /// assert_eq!(adult_kp.get(&user), Some(&30));
    /// ```
    pub fn filter<F>(
        &self,
        predicate: F,
    ) -> Kp<
        R,
        V,
        Root,
        Value,
        MutRoot,
        MutValue,
        impl Fn(Root) -> Option<Value>,
        impl Fn(MutRoot) -> Option<MutValue>,
    >
    where
        F: Fn(&V) -> bool + Copy + 'static,
        V: 'static,
    {
        Kp::new(
            move |root: Root| {
                (&self.get)(root).filter(|value| {
                    let v: &V = value.borrow();
                    predicate(v)
                })
            },
            move |root: MutRoot| {
                (&self.set)(root).filter(|value| {
                    let v: &V = value.borrow();
                    predicate(v)
                })
            },
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

    /// Map the variant value through a transformation function
    /// 
    /// # Example
    /// ```
    /// let result: Result<String, i32> = Ok("hello".to_string());
    /// let ok_kp = enum_ok();
    /// let len_kp = ok_kp.map(|s: &String| s.len());
    /// assert_eq!(len_kp.get(&result), Some(5));
    /// ```
    pub fn map<MappedValue, F>(
        &self,
        mapper: F,
    ) -> EnumKp<
        Enum,
        MappedValue,
        Root,
        MappedValue,
        MutRoot,
        MappedValue,
        impl Fn(Root) -> Option<MappedValue>,
        impl Fn(MutRoot) -> Option<MappedValue>,
        impl Fn(MappedValue) -> Enum,
    >
    where
        F: Fn(&Variant) -> MappedValue + Copy + 'static,
        Variant: 'static,
        MappedValue: 'static,
        E: Fn(Variant) -> Enum + Copy + 'static,
    {
        let mapped_extractor = self.extractor.map(mapper);
        let embedder = self.embedder;
        
        // Create a new embedder that maps back
        // Note: This is a limitation - we can't reverse the map for embedding
        // So we create a placeholder that panics
        let new_embedder = move |_value: MappedValue| -> Enum {
            panic!("Cannot embed mapped values back into enum. Use the original EnumKp for embedding.")
        };
        
        EnumKp::new(mapped_extractor, new_embedder)
    }

    /// Filter the variant value based on a predicate
    /// Returns None if the predicate fails or if wrong variant
    /// 
    /// # Example
    /// ```
    /// let result: Result<i32, String> = Ok(42);
    /// let ok_kp = enum_ok();
    /// let positive_kp = ok_kp.filter(|x: &i32| *x > 0);
    /// assert_eq!(positive_kp.get(&result), Some(&42));
    /// ```
    pub fn filter<F>(
        &self,
        predicate: F,
    ) -> EnumKp<
        Enum,
        Variant,
        Root,
        Value,
        MutRoot,
        MutValue,
        impl Fn(Root) -> Option<Value>,
        impl Fn(MutRoot) -> Option<MutValue>,
        E,
    >
    where
        F: Fn(&Variant) -> bool + Copy + 'static,
        Variant: 'static,
        E: Copy,
    {
        let filtered_extractor = self.extractor.filter(predicate);
        EnumKp::new(filtered_extractor, self.embedder)
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

// ========== PARTIAL KEYPATHS (Hide Value Type) ==========

use std::any::{Any, TypeId};
use std::rc::Rc;

/// PKp (PartialKeyPath) - Hides the Value type but keeps Root visible
/// Useful for storing keypaths in collections without knowing the exact Value type
///
/// # Why PhantomData<Root>?
///
/// `PhantomData<Root>` is needed because:
/// 1. The `Root` type parameter is not actually stored in the struct (only used in the closure)
/// 2. Rust needs to know the generic type parameter for:
///    - Type checking at compile time
///    - Ensuring correct usage (e.g., `PKp<User>` can only be used with `&User`)
///    - Preventing mixing different Root types
/// 3. Without `PhantomData`, Rust would complain that `Root` is unused
/// 4. `PhantomData` is zero-sized - it adds no runtime overhead
#[derive(Clone)]
pub struct PKp<Root> {
    getter: Rc<dyn for<'r> Fn(&'r Root) -> Option<&'r dyn Any>>,
    value_type_id: TypeId,
    _phantom: std::marker::PhantomData<Root>,
}

impl<Root> PKp<Root>
where
    Root: 'static,
{
    /// Create a new PKp from a KpType (the common reference-based keypath)
    pub fn new<'a, V>(keypath: KpType<'a, Root, V>) -> Self
    where
        V: Any + 'static,
    {
        let value_type_id = TypeId::of::<V>();
        let getter_fn = keypath.get;

        Self {
            getter: Rc::new(move |root: &Root| {
                getter_fn(root).map(|val: &V| val as &dyn Any)
            }),
            value_type_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a PKp from a KpType (alias for `new()`)
    pub fn from<'a, V>(keypath: KpType<'a, Root, V>) -> Self
    where
        V: Any + 'static,
    {
        Self::new(keypath)
    }

    /// Get the value as a trait object
    pub fn get<'r>(&self, root: &'r Root) -> Option<&'r dyn Any> {
        (self.getter)(root)
    }

    /// Get the TypeId of the Value type
    pub fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }

    /// Try to downcast the result to a specific type
    pub fn get_as<'a, Value: Any>(&self, root: &'a Root) -> Option<&'a Value> {
        if self.value_type_id == TypeId::of::<Value>() {
            self.get(root).and_then(|any| any.downcast_ref::<Value>())
        } else {
            None
        }
    }

    /// Get a human-readable name for the value type
    pub fn kind_name(&self) -> String {
        format!("{:?}", self.value_type_id)
    }

    /// Adapt this keypath to work with Arc<Root> instead of Root
    pub fn for_arc(&self) -> PKp<Arc<Root>> {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;

        PKp {
            getter: Rc::new(move |arc: &Arc<Root>| getter(arc.as_ref())),
            value_type_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Adapt this keypath to work with Box<Root> instead of Root
    pub fn for_box(&self) -> PKp<Box<Root>> {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;

        PKp {
            getter: Rc::new(move |boxed: &Box<Root>| getter(boxed.as_ref())),
            value_type_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Adapt this keypath to work with Rc<Root> instead of Root
    pub fn for_rc(&self) -> PKp<Rc<Root>> {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;

        PKp {
            getter: Rc::new(move |rc: &Rc<Root>| getter(rc.as_ref())),
            value_type_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Adapt this keypath to work with Option<Root> instead of Root
    pub fn for_option(&self) -> PKp<Option<Root>> {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;

        PKp {
            getter: Rc::new(move |opt: &Option<Root>| {
                opt.as_ref().and_then(|root| getter(root))
            }),
            value_type_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Adapt this keypath to work with Result<Root, E> instead of Root
    pub fn for_result<E>(&self) -> PKp<Result<Root, E>>
    where
        E: 'static,
    {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;

        PKp {
            getter: Rc::new(move |result: &Result<Root, E>| {
                result.as_ref().ok().and_then(|root| getter(root))
            }),
            value_type_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Map the value through a transformation function
    /// The mapped value must also implement Any for type erasure
    /// 
    /// # Example
    /// ```
    /// let user = User { name: "Alice".to_string() };
    /// let name_kp = KpType::new(|u: &User| Some(&u.name), |_| None);
    /// let name_pkp = PKp::new(name_kp);
    /// let len_pkp = name_pkp.map::<String, _, _>(|s| s.len());
    /// assert_eq!(len_pkp.get_as::<usize>(&user), Some(&5));
    /// ```
    pub fn map<OrigValue, MappedValue, F>(
        &self,
        mapper: F,
    ) -> PKp<Root>
    where
        OrigValue: Any + 'static,
        MappedValue: Any + 'static,
        F: Fn(&OrigValue) -> MappedValue + 'static,
    {
        let orig_type_id = self.value_type_id;
        let getter = self.getter.clone();
        let mapped_type_id = TypeId::of::<MappedValue>();

        PKp {
            getter: Rc::new(move |root: &Root| {
                getter(root).and_then(|any_value| {
                    // Verify the original type matches
                    if orig_type_id == TypeId::of::<OrigValue>() {
                        any_value.downcast_ref::<OrigValue>().map(|orig_val| {
                            let mapped = mapper(orig_val);
                            // Box the mapped value and return as &dyn Any
                            // Note: This creates a new allocation
                            Box::leak(Box::new(mapped)) as &dyn Any
                        })
                    } else {
                        None
                    }
                })
            }),
            value_type_id: mapped_type_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Filter the value based on a predicate with type checking
    /// Returns None if the type doesn't match or predicate fails
    /// 
    /// # Example
    /// ```
    /// let user = User { age: 30 };
    /// let age_kp = KpType::new(|u: &User| Some(&u.age), |_| None);
    /// let age_pkp = PKp::new(age_kp);
    /// let adult_pkp = age_pkp.filter::<i32, _>(|age| *age >= 18);
    /// assert_eq!(adult_pkp.get_as::<i32>(&user), Some(&30));
    /// ```
    pub fn filter<Value, F>(&self, predicate: F) -> PKp<Root>
    where
        Value: Any + 'static,
        F: Fn(&Value) -> bool + 'static,
    {
        let orig_type_id = self.value_type_id;
        let getter = self.getter.clone();

        PKp {
            getter: Rc::new(move |root: &Root| {
                getter(root).filter(|any_value| {
                    // Type check and apply predicate
                    if orig_type_id == TypeId::of::<Value>() {
                        any_value
                            .downcast_ref::<Value>()
                            .map(|val| predicate(val))
                            .unwrap_or(false)
                    } else {
                        false
                    }
                })
            }),
            value_type_id: orig_type_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

// ========== ANY KEYPATHS (Hide Both Root and Value Types) ==========

/// AKp (AnyKeyPath) - Hides both Root and Value types
/// Most flexible keypath type for heterogeneous collections
/// Uses dynamic dispatch and type checking at runtime
#[derive(Clone)]
pub struct AKp {
    getter: Rc<dyn for<'r> Fn(&'r dyn Any) -> Option<&'r dyn Any>>,
    root_type_id: TypeId,
    value_type_id: TypeId,
}

impl AKp {
    /// Create a new AKp from a KpType (the common reference-based keypath)
    pub fn new<'a, R, V>(keypath: KpType<'a, R, V>) -> Self
    where
        R: Any + 'static,
        V: Any + 'static,
    {
        let root_type_id = TypeId::of::<R>();
        let value_type_id = TypeId::of::<V>();
        let getter_fn = keypath.get;

        Self {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(root) = any.downcast_ref::<R>() {
                    getter_fn(root).map(|value: &V| value as &dyn Any)
                } else {
                    None
                }
            }),
            root_type_id,
            value_type_id,
        }
    }

    /// Create an AKp from a KpType (alias for `new()`)
    pub fn from<'a, R, V>(keypath: KpType<'a, R, V>) -> Self
    where
        R: Any + 'static,
        V: Any + 'static,
    {
        Self::new(keypath)
    }

    /// Get the value as a trait object (with root type checking)
    pub fn get<'r>(&self, root: &'r dyn Any) -> Option<&'r dyn Any> {
        (self.getter)(root)
    }

    /// Get the TypeId of the Root type
    pub fn root_type_id(&self) -> TypeId {
        self.root_type_id
    }

    /// Get the TypeId of the Value type
    pub fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }

    /// Try to get the value with full type checking
    pub fn get_as<'a, Root: Any, Value: Any>(&self, root: &'a Root) -> Option<Option<&'a Value>> {
        if self.root_type_id == TypeId::of::<Root>() && self.value_type_id == TypeId::of::<Value>()
        {
            Some(
                self.get(root as &dyn Any)
                    .and_then(|any| any.downcast_ref::<Value>()),
            )
        } else {
            None
        }
    }

    /// Get a human-readable name for the value type
    pub fn kind_name(&self) -> String {
        format!("{:?}", self.value_type_id)
    }

    /// Get a human-readable name for the root type
    pub fn root_kind_name(&self) -> String {
        format!("{:?}", self.root_type_id)
    }

    /// Adapt this keypath to work with Arc<Root> instead of Root
    pub fn for_arc<Root>(&self) -> AKp
    where
        Root: Any + 'static,
    {
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();

        AKp {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(arc) = any.downcast_ref::<Arc<Root>>() {
                    getter(arc.as_ref() as &dyn Any)
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Arc<Root>>(),
            value_type_id,
        }
    }

    /// Adapt this keypath to work with Box<Root> instead of Root
    pub fn for_box<Root>(&self) -> AKp
    where
        Root: Any + 'static,
    {
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();

        AKp {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(boxed) = any.downcast_ref::<Box<Root>>() {
                    getter(boxed.as_ref() as &dyn Any)
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Box<Root>>(),
            value_type_id,
        }
    }

    /// Adapt this keypath to work with Rc<Root> instead of Root
    pub fn for_rc<Root>(&self) -> AKp
    where
        Root: Any + 'static,
    {
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();

        AKp {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(rc) = any.downcast_ref::<Rc<Root>>() {
                    getter(rc.as_ref() as &dyn Any)
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Rc<Root>>(),
            value_type_id,
        }
    }

    /// Adapt this keypath to work with Option<Root> instead of Root
    pub fn for_option<Root>(&self) -> AKp
    where
        Root: Any + 'static,
    {
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();

        AKp {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(opt) = any.downcast_ref::<Option<Root>>() {
                    opt.as_ref().and_then(|root| getter(root as &dyn Any))
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Option<Root>>(),
            value_type_id,
        }
    }

    /// Adapt this keypath to work with Result<Root, E> instead of Root
    pub fn for_result<Root, E>(&self) -> AKp
    where
        Root: Any + 'static,
        E: Any + 'static,
    {
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();

        AKp {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(result) = any.downcast_ref::<Result<Root, E>>() {
                    result
                        .as_ref()
                        .ok()
                        .and_then(|root| getter(root as &dyn Any))
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Result<Root, E>>(),
            value_type_id,
        }
    }

    /// Map the value through a transformation function with type checking
    /// Both original and mapped values must implement Any
    /// 
    /// # Example
    /// ```
    /// let user = User { name: "Alice".to_string() };
    /// let name_kp = KpType::new(|u: &User| Some(&u.name), |_| None);
    /// let name_akp = AKp::new(name_kp);
    /// let len_akp = name_akp.map::<User, String, _, _>(|s| s.len());
    /// ```
    pub fn map<Root, OrigValue, MappedValue, F>(
        &self,
        mapper: F,
    ) -> AKp
    where
        Root: Any + 'static,
        OrigValue: Any + 'static,
        MappedValue: Any + 'static,
        F: Fn(&OrigValue) -> MappedValue + 'static,
    {
        let orig_root_type_id = self.root_type_id;
        let orig_value_type_id = self.value_type_id;
        let getter = self.getter.clone();
        let mapped_type_id = TypeId::of::<MappedValue>();

        AKp {
            getter: Rc::new(move |any_root: &dyn Any| {
                // Check root type matches
                if any_root.type_id() == orig_root_type_id {
                    getter(any_root).and_then(|any_value| {
                        // Verify the original value type matches
                        if orig_value_type_id == TypeId::of::<OrigValue>() {
                            any_value.downcast_ref::<OrigValue>().map(|orig_val| {
                                let mapped = mapper(orig_val);
                                // Box the mapped value and return as &dyn Any
                                Box::leak(Box::new(mapped)) as &dyn Any
                            })
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            }),
            root_type_id: orig_root_type_id,
            value_type_id: mapped_type_id,
        }
    }

    /// Filter the value based on a predicate with full type checking
    /// Returns None if types don't match or predicate fails
    /// 
    /// # Example
    /// ```
    /// let user = User { age: 30 };
    /// let age_kp = KpType::new(|u: &User| Some(&u.age), |_| None);
    /// let age_akp = AKp::new(age_kp);
    /// let adult_akp = age_akp.filter::<User, i32, _>(|age| *age >= 18);
    /// ```
    pub fn filter<Root, Value, F>(&self, predicate: F) -> AKp
    where
        Root: Any + 'static,
        Value: Any + 'static,
        F: Fn(&Value) -> bool + 'static,
    {
        let orig_root_type_id = self.root_type_id;
        let orig_value_type_id = self.value_type_id;
        let getter = self.getter.clone();

        AKp {
            getter: Rc::new(move |any_root: &dyn Any| {
                // Check root type matches
                if any_root.type_id() == orig_root_type_id {
                    getter(any_root).filter(|any_value| {
                        // Type check value and apply predicate
                        if orig_value_type_id == TypeId::of::<Value>() {
                            any_value
                                .downcast_ref::<Value>()
                                .map(|val| predicate(val))
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    })
                } else {
                    None
                }
            }),
            root_type_id: orig_root_type_id,
            value_type_id: orig_value_type_id,
        }
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

    #[test]
    fn test_pkp_basic() {
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32,
        }

        let user = User {
            name: "Alice".to_string(),
            age: 30,
        };

        // Create regular keypaths
        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let age_kp = KpType::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));

        // Convert to partial keypaths
        let name_pkp = PKp::new(name_kp);
        let age_pkp = PKp::new(age_kp);

        // Test get_as with correct type
        assert_eq!(name_pkp.get_as::<String>(&user), Some(&"Alice".to_string()));
        assert_eq!(age_pkp.get_as::<i32>(&user), Some(&30));

        // Test get_as with wrong type returns None
        assert_eq!(name_pkp.get_as::<i32>(&user), None);
        assert_eq!(age_pkp.get_as::<String>(&user), None);

        // Test value_type_id
        assert_eq!(name_pkp.value_type_id(), TypeId::of::<String>());
        assert_eq!(age_pkp.value_type_id(), TypeId::of::<i32>());
    }

    #[test]
    fn test_pkp_collection() {
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32,
        }

        let user = User {
            name: "Bob".to_string(),
            age: 25,
        };

        // Create a collection of partial keypaths
        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let age_kp = KpType::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));

        let keypaths: Vec<PKp<User>> = vec![PKp::new(name_kp), PKp::new(age_kp)];

        // Access values through the collection
        let name_value = keypaths[0].get_as::<String>(&user);
        let age_value = keypaths[1].get_as::<i32>(&user);

        assert_eq!(name_value, Some(&"Bob".to_string()));
        assert_eq!(age_value, Some(&25));
    }

    #[test]
    fn test_pkp_for_arc() {
        #[derive(Debug)]
        struct User {
            name: String,
        }

        let user = Arc::new(User {
            name: "Charlie".to_string(),
        });

        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let name_pkp = PKp::new(name_kp);

        // Adapt for Arc
        let arc_pkp = name_pkp.for_arc();

        assert_eq!(
            arc_pkp.get_as::<String>(&user),
            Some(&"Charlie".to_string())
        );
    }

    #[test]
    fn test_pkp_for_option() {
        #[derive(Debug)]
        struct User {
            name: String,
        }

        let some_user = Some(User {
            name: "Diana".to_string(),
        });
        let none_user: Option<User> = None;

        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let name_pkp = PKp::new(name_kp);

        // Adapt for Option
        let opt_pkp = name_pkp.for_option();

        assert_eq!(
            opt_pkp.get_as::<String>(&some_user),
            Some(&"Diana".to_string())
        );
        assert_eq!(opt_pkp.get_as::<String>(&none_user), None);
    }

    #[test]
    fn test_akp_basic() {
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32,
        }

        #[derive(Debug)]
        struct Product {
            title: String,
            price: f64,
        }

        let user = User {
            name: "Eve".to_string(),
            age: 28,
        };

        let product = Product {
            title: "Book".to_string(),
            price: 19.99,
        };

        // Create AnyKeypaths
        let user_name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let user_name_akp = AKp::new(user_name_kp);

        let product_title_kp = KpType::new(
            |p: &Product| Some(&p.title),
            |p: &mut Product| Some(&mut p.title),
        );
        let product_title_akp = AKp::new(product_title_kp);

        // Test get_as with correct types
        assert_eq!(
            user_name_akp.get_as::<User, String>(&user),
            Some(Some(&"Eve".to_string()))
        );
        assert_eq!(
            product_title_akp.get_as::<Product, String>(&product),
            Some(Some(&"Book".to_string()))
        );

        // Test get_as with wrong root type
        assert_eq!(user_name_akp.get_as::<Product, String>(&product), None);
        assert_eq!(product_title_akp.get_as::<User, String>(&user), None);

        // Test TypeIds
        assert_eq!(user_name_akp.root_type_id(), TypeId::of::<User>());
        assert_eq!(user_name_akp.value_type_id(), TypeId::of::<String>());
        assert_eq!(product_title_akp.root_type_id(), TypeId::of::<Product>());
        assert_eq!(product_title_akp.value_type_id(), TypeId::of::<String>());
    }

    #[test]
    fn test_akp_heterogeneous_collection() {
        #[derive(Debug)]
        struct User {
            name: String,
        }

        #[derive(Debug)]
        struct Product {
            title: String,
        }

        let user = User {
            name: "Frank".to_string(),
        };
        let product = Product {
            title: "Laptop".to_string(),
        };

        // Create a heterogeneous collection of AnyKeypaths
        let user_name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let product_title_kp = KpType::new(
            |p: &Product| Some(&p.title),
            |p: &mut Product| Some(&mut p.title),
        );

        let keypaths: Vec<AKp> = vec![AKp::new(user_name_kp), AKp::new(product_title_kp)];

        // Access through trait objects
        let user_any: &dyn Any = &user;
        let product_any: &dyn Any = &product;

        let user_value = keypaths[0].get(user_any);
        let product_value = keypaths[1].get(product_any);

        assert!(user_value.is_some());
        assert!(product_value.is_some());

        // Downcast to concrete types
        assert_eq!(
            user_value.and_then(|v| v.downcast_ref::<String>()),
            Some(&"Frank".to_string())
        );
        assert_eq!(
            product_value.and_then(|v| v.downcast_ref::<String>()),
            Some(&"Laptop".to_string())
        );
    }

    #[test]
    fn test_akp_for_option() {
        #[derive(Debug)]
        struct User {
            name: String,
        }

        let some_user = Some(User {
            name: "Grace".to_string(),
        });
        let none_user: Option<User> = None;

        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let name_akp = AKp::new(name_kp);

        // Adapt for Option
        let opt_akp = name_akp.for_option::<User>();

        assert_eq!(
            opt_akp.get_as::<Option<User>, String>(&some_user),
            Some(Some(&"Grace".to_string()))
        );
        assert_eq!(
            opt_akp.get_as::<Option<User>, String>(&none_user),
            Some(None)
        );
    }

    #[test]
    fn test_akp_for_result() {
        #[derive(Debug)]
        struct User {
            name: String,
        }

        let ok_user: Result<User, String> = Ok(User {
            name: "Henry".to_string(),
        });
        let err_user: Result<User, String> = Err("Not found".to_string());

        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let name_akp = AKp::new(name_kp);

        // Adapt for Result
        let result_akp = name_akp.for_result::<User, String>();

        assert_eq!(
            result_akp.get_as::<Result<User, String>, String>(&ok_user),
            Some(Some(&"Henry".to_string()))
        );
        assert_eq!(
            result_akp.get_as::<Result<User, String>, String>(&err_user),
            Some(None)
        );
    }

    // ========== MAP TESTS ==========

    #[test]
    fn test_kp_map() {
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32,
        }

        let user = User {
            name: "Alice".to_string(),
            age: 30,
        };

        // Map string to its length
        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let len_kp = name_kp.map(|name: &String| name.len());

        assert_eq!(len_kp.get(&user), Some(5));

        // Map age to double
        let age_kp = KpType::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));
        let double_age_kp = age_kp.map(|age: &i32| age * 2);

        assert_eq!(double_age_kp.get(&user), Some(60));

        // Map to boolean
        let is_adult_kp = age_kp.map(|age: &i32| *age >= 18);
        assert_eq!(is_adult_kp.get(&user), Some(true));
    }

    #[test]
    fn test_kp_filter() {
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32,
        }

        let adult = User {
            name: "Alice".to_string(),
            age: 30,
        };

        let minor = User {
            name: "Bob".to_string(),
            age: 15,
        };

        let age_kp = KpType::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));
        let adult_age_kp = age_kp.filter(|age: &i32| *age >= 18);

        assert_eq!(adult_age_kp.get(&adult), Some(&30));
        assert_eq!(adult_age_kp.get(&minor), None);

        // Filter names by length
        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let short_name_kp = name_kp.filter(|name: &String| name.len() <= 4);

        assert_eq!(short_name_kp.get(&minor), Some(&"Bob".to_string()));
        assert_eq!(short_name_kp.get(&adult), None);
    }

    #[test]
    fn test_kp_map_and_filter() {
        #[derive(Debug)]
        struct User {
            scores: Vec<i32>,
        }

        let user = User {
            scores: vec![85, 92, 78, 95],
        };

        let scores_kp = KpType::new(|u: &User| Some(&u.scores), |u: &mut User| Some(&mut u.scores));
        
        // Map to average score
        let avg_kp = scores_kp.map(|scores: &Vec<i32>| {
            scores.iter().sum::<i32>() / scores.len() as i32
        });

        // Filter for high averages
        let high_avg_kp = avg_kp.filter(|avg: &i32| *avg >= 85);

        assert_eq!(high_avg_kp.get(&user), Some(87)); // (85+92+78+95)/4 = 87.5 -> 87
    }

    #[test]
    fn test_enum_kp_map() {
        let ok_result: Result<String, i32> = Ok("hello".to_string());
        let err_result: Result<String, i32> = Err(42);

        let ok_kp = enum_ok::<String, i32>();
        let len_kp = ok_kp.map(|s: &String| s.len());

        assert_eq!(len_kp.get(&ok_result), Some(5));
        assert_eq!(len_kp.get(&err_result), None);

        // Map Option
        let some_opt = Some(vec![1, 2, 3, 4, 5]);
        let none_opt: Option<Vec<i32>> = None;

        let some_kp = enum_some::<Vec<i32>>();
        let count_kp = some_kp.map(|vec: &Vec<i32>| vec.len());

        assert_eq!(count_kp.get(&some_opt), Some(5));
        assert_eq!(count_kp.get(&none_opt), None);
    }

    #[test]
    fn test_enum_kp_filter() {
        let ok_result1: Result<i32, String> = Ok(42);
        let ok_result2: Result<i32, String> = Ok(-5);
        let err_result: Result<i32, String> = Err("error".to_string());

        let ok_kp = enum_ok::<i32, String>();
        let positive_kp = ok_kp.filter(|x: &i32| *x > 0);

        assert_eq!(positive_kp.get(&ok_result1), Some(&42));
        assert_eq!(positive_kp.get(&ok_result2), None); // Negative number filtered out
        assert_eq!(positive_kp.get(&err_result), None); // Err variant

        // Filter Option strings by length
        let long_str = Some("hello world".to_string());
        let short_str = Some("hi".to_string());

        let some_kp = enum_some::<String>();
        let long_kp = some_kp.filter(|s: &String| s.len() > 5);

        assert_eq!(long_kp.get(&long_str), Some(&"hello world".to_string()));
        assert_eq!(long_kp.get(&short_str), None);
    }

    #[test]
    fn test_pkp_filter() {
        #[derive(Debug)]
        struct User {
            name: String,
            age: i32,
        }

        let adult = User {
            name: "Alice".to_string(),
            age: 30,
        };

        let minor = User {
            name: "Bob".to_string(),
            age: 15,
        };

        let age_kp = KpType::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));
        let age_pkp = PKp::new(age_kp);

        // Filter for adults
        let adult_pkp = age_pkp.filter::<i32, _>(|age| *age >= 18);

        assert_eq!(adult_pkp.get_as::<i32>(&adult), Some(&30));
        assert_eq!(adult_pkp.get_as::<i32>(&minor), None);

        // Filter names
        let name_kp = KpType::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
        let name_pkp = PKp::new(name_kp);
        let short_name_pkp = name_pkp.filter::<String, _>(|name| name.len() <= 4);

        assert_eq!(short_name_pkp.get_as::<String>(&minor), Some(&"Bob".to_string()));
        assert_eq!(short_name_pkp.get_as::<String>(&adult), None);
    }

    #[test]
    fn test_akp_filter() {
        #[derive(Debug)]
        struct User {
            age: i32,
        }

        #[derive(Debug)]
        struct Product {
            price: f64,
        }

        let adult = User { age: 30 };
        let minor = User { age: 15 };
        let expensive = Product { price: 99.99 };
        let cheap = Product { price: 5.0 };

        // Filter user ages
        let age_kp = KpType::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));
        let age_akp = AKp::new(age_kp);
        let adult_akp = age_akp.filter::<User, i32, _>(|age| *age >= 18);

        assert_eq!(
            adult_akp.get_as::<User, i32>(&adult),
            Some(Some(&30))
        );
        assert_eq!(
            adult_akp.get_as::<User, i32>(&minor),
            Some(None)
        );

        // Filter product prices
        let price_kp = KpType::new(|p: &Product| Some(&p.price), |p: &mut Product| Some(&mut p.price));
        let price_akp = AKp::new(price_kp);
        let expensive_akp = price_akp.filter::<Product, f64, _>(|price| *price > 50.0);

        assert_eq!(
            expensive_akp.get_as::<Product, f64>(&expensive),
            Some(Some(&99.99))
        );
        assert_eq!(
            expensive_akp.get_as::<Product, f64>(&cheap),
            Some(None)
        );
    }
}
