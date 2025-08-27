/// Read-only keypath
pub struct ReadableKeyPath<Root, Value> {
    pub get: Box<dyn for<'a> Fn(&'a Root) -> &'a Value>,
}

impl<Root, Value> ReadableKeyPath<Root, Value> {
    pub fn new(get: impl for<'a> Fn(&'a Root) -> &'a Value + 'static) -> Self {
        Self { get: Box::new(get) }
    }
    pub fn get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.get)(root)
    }

    /// Iterate a slice of `Root` and yield references to `Value`
    pub fn iter<'a>(
        &'a self,
        slice: &'a [Root],
    ) -> impl Iterator<Item = &'a Value> + 'a {
        slice.iter().map(move |root| (self.get)(root))
    }
}

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


// 
// /// Macro for readable keypaths
// #[macro_export]
// macro_rules! readable_keypath {
//     ($Root:ty, $field:ident) => {
//         ReadableKeyPath::new(|root: &$Root| &root.$field)
//     };
// }
// 
// /// Macro for writable keypaths
// #[macro_export]
// macro_rules! writable_keypath {
//     ($Root:ty, $field:ident) => {
//         WritableKeyPath::new(
//             |root: &$Root| &root.$field,
//             |root: &mut $Root| &mut root.$field,
//         )
//     };
// }
