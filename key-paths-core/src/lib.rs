// pub trait Readable<Root, Value> {
//     fn get<'a>(&self, root: &'a Root) -> &'a Value;
//
//     fn iter<'a>(&self, slice: &'a [Root]) -> Box<dyn Iterator<Item = &'a Value> + 'a>
//     where
//         Self: Sized,
//     {
//         let f = self.get_fn(); // capture fn pointer
//         Box::new(slice.iter().map(move |root| f(root)))
//     }
//
//     fn get_fn(&self) -> for<'a> fn(&'a Root) -> &'a Value;
// }

/// Read-only keypath
pub struct ReadableKeyPath<Root, Value> {
    pub get: Box<dyn for<'a> Fn(&'a Root) -> &'a Value>,
}

impl<Root, Value> ReadableKeyPath<Root, Value> {
    pub fn new(get: impl for<'a> Fn(&'a Root) -> &'a Value + 'static) -> Self {
        Self { get: Box::new(get) }
    }
    pub fn try_get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.get)(root)
    }

}

// --- Compose: impl over (Root, Mid); Value is method-generic ---
// --- Compose ---
impl<Root, Mid> ReadableKeyPath<Root, Mid>
where
    Root: 'static,
    Mid: 'static,
{
    pub fn compose<Value>(
        self,
        mid: ReadableKeyPath<Mid, Value>,
    ) -> ReadableKeyPath<Root, Value> 
    where 
    Value: 'static,
    {
        ReadableKeyPath::new(move |r: &Root| {
            let mid_ref: &Mid = (self.get)(r);
            (mid.get)(mid_ref)
        })
    }
}
// impl<Root, Value> ReadableKeyPath<Root, Value> {
//     pub fn new(get: for<'a> fn(&'a Root) -> &'a Value) -> Self {
//         Self { get }
//     }
// }
// 
// impl<Root, Value> ReadableKeyPath<Root, Value> {
//     pub fn get<'a>(&self, root: &'a Root) -> &'a Value {
//         (self.get)(root)
//     }
// 
//     // pub fn get_fn(&self) -> for<'a> fn(&'a Root) -> &'a Value {
//     //     self.get
//     // }
// }
// 


pub struct WritableKeyPath<Root, Value> {
    pub get_mut: Box<dyn for<'a> Fn(&'a mut Root) -> &'a mut Value>,
}

impl<Root, Value> WritableKeyPath<Root, Value> {
    pub fn new(get: impl for<'a> Fn(&'a mut Root) -> &'a mut Value + 'static) -> Self {
        Self { get_mut: Box::new(get) }
    }
    pub fn try_get<'a>(&self, root: &'a mut Root) -> &'a mut Value {
        (self.get_mut)(root)
    }

}

// --- Compose: impl over (Root, Mid); Value is method-generic ---
// --- Compose ---
impl<Root, Mid> WritableKeyPath<Root, Mid>
where
    Root: 'static,
    Mid: 'static,
{
    pub fn compose<Value>(
        self,
        mid: WritableKeyPath<Mid, Value>,
    ) -> WritableKeyPath<Root, Value>
    where
        Value: 'static,
    {
        WritableKeyPath::new(move |r: &mut Root| {
            let mid_ref: &mut Mid = (self.get_mut)(r);
            (mid.get_mut)(mid_ref)
        })
    }
}

/// Read/write keypath
// pub struct WritableKeyPath<Root, Value> {
//     pub get_mut: for<'a> fn(&'a mut Root) -> &'a mut Value,
// }

// pub trait Writable<Root, Value>: Readable<Root, Value> {
//     fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut Value;
//
//     fn iter_mut<'a>(&self, slice: &'a mut [Root]) -> Box<dyn Iterator<Item = &'a mut Value> + 'a>
//     where
//         Self: Sized,
//     {
//         let f = self.get_mut_fn(); // capture fn pointer
//         Box::new(slice.iter_mut().map(move |root| f(root)))
//     }
//
//     fn get_mut_fn(&self) -> for<'a> fn(&'a mut Root) -> &'a mut Value;
// }

// impl<Root, Value> WritableKeyPath<Root, Value> {
//     pub fn new(
//         get_mut: for<'a> fn(&'a mut Root) -> &'a mut Value,
//     ) -> Self {
//         Self { get_mut }
//     }
// }

// impl<Root, Value> WritableKeyPath<Root, Value> {
//     pub fn get<'a>(&self, root: &'a mut Root) -> &'a mut Value {
//         (self.get_mut)(root)
//     }
// 
//     fn get_fn(&self) -> for<'a> fn(&'a Root) -> &'a Value {
//         self.get
//     }
// }

// impl<Root, Value> WritableKeyPath<Root, Value> {
//     pub fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut Value {
//         (self.get_mut)(root)
//     }
// 
//     // pub fn get_mut_fn(&self) -> for<'a> fn(&'a mut Root) -> &'a mut Value {
//     //     self.get_mut
//     // }
// }

// pub trait FailableReadable<Root, Value> {
//     fn try_get<'a>(&self, root: &'a Root) -> Option<&'a Value>;
// }

// --- Core type ---
pub struct FailableReadableKeyPath<Root, Value> {
    pub get: Box<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>,
}

impl<Root, Value> FailableReadableKeyPath<Root, Value> {
    pub fn new(get: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static) -> Self {
        Self { get: Box::new(get) }
    }
    pub fn try_get<'a>(&self, root: &'a Root) -> Option<&'a Value> {
        (self.get)(root)
    }

}

// --- Compose: impl over (Root, Mid); Value is method-generic ---
impl<Root, Mid> FailableReadableKeyPath<Root, Mid>
where
    Root: 'static,
    Mid: 'static,
{
    pub fn compose<Value>(
        self,
        mid: FailableReadableKeyPath<Mid, Value>,
    ) -> FailableReadableKeyPath<Root, Value>
    where
        Value: 'static,
    {
        FailableReadableKeyPath::new(move |r: &Root| (self.get)(r).and_then(|m: &Mid| (mid.get)(m)))
    }
}

pub struct FailableWritableKeyPath<Root, Value> {
    pub get_mut: Box<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>,
}

impl<Root, Value> FailableWritableKeyPath<Root, Value> {
    pub fn new(get_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + 'static) -> Self {
        Self {
            get_mut: Box::new(get_mut),
        }
    }

    pub fn try_get_mut<'a>(&self, root: &'a mut Root) -> Option<&'a mut Value> {
        (self.get_mut)(root)
    }
}

// ---------------- COMPOSE ----------------
impl<Root, Mid> FailableWritableKeyPath<Root, Mid>
where
    Root: 'static,
    Mid: 'static,
{
    pub fn compose<Value>(
        self,
        mid: FailableWritableKeyPath<Mid, Value>,
    ) -> FailableWritableKeyPath<Root, Value>
    where
        Value: 'static,
    {
        FailableWritableKeyPath::new(move |r: &mut Root| {
            (self.get_mut)(r).and_then(|m: &mut Mid| (mid.get_mut)(m))
        })
    }
}



pub struct EnumKeyPath<Enum, Inner> {
    pub extract: fn(&Enum) -> Option<&Inner>,
    pub embed: fn(Inner) -> Enum,
}

impl<Enum, Inner> EnumKeyPath<Enum, Inner> {
    pub fn new(extract: fn(&Enum) -> Option<&Inner>, embed: fn(Inner) -> Enum) -> Self {
        Self { extract, embed }
    }

    pub fn extract<'a>(&self, e: &'a Enum) -> Option<&'a Inner> {
        (self.extract)(e)
    }

    pub fn embed(&self, inner: Inner) -> Enum {
        (self.embed)(inner)
    }
}


#[macro_export]
macro_rules! enum_keypath {
    // Case with payload
    ($enum:ident :: $variant:ident ( $inner:ty )) => {{
        EnumKeyPath::<$enum, $inner>::new(
            |root: &$enum| {
                if let $enum::$variant(inner) = root {
                    Some(inner)
                } else {
                    None
                }
            },
            |inner: $inner| $enum::$variant(inner),
        )
    }};
    // Case without payload
    ($enum:ident :: $variant:ident) => {{
        EnumKeyPath::<$enum, ()>::new(
            |root: &$enum| {
                if let $enum::$variant = root {
                    Some(&())
                } else {
                    None
                }
            },
            |_| $enum::$variant,
        )
    }};
}

/// Macro for readable keypaths
#[macro_export]
macro_rules! readable_keypath {
    ($Root:ty, $field:ident) => {
        ReadableKeyPath::new(|root: &$Root| &root.$field)
    };
}

/// Macro for writable keypaths
#[macro_export]
macro_rules! writable_keypath {
    ($Root:ty, $field:ident) => {
        WritableKeyPath::new(
            |root: &$Root| &root.$field,
            |root: &mut $Root| &mut root.$field,
        )
    };
}
