
pub trait Readable<Root, Value> {
    fn get<'a>(&self, root: &'a Root) -> &'a Value;

    fn iter<'a>(&self, slice: &'a [Root]) -> Box<dyn Iterator<Item = &'a Value> + 'a>
    where
        Self: Sized,
    {
        let f = self.get_fn(); // capture fn pointer
        Box::new(slice.iter().map(move |root| f(root)))
    }

    fn get_fn(&self) -> for<'a> fn(&'a Root) -> &'a Value;
}

/// Read-only keypath
pub struct ReadableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> &'a Value,
}

impl<Root, Value> ReadableKeyPath<Root, Value> {
    pub fn new(get: for<'a> fn(&'a Root) -> &'a Value) -> Self {
        Self { get }
    }
}

impl<Root, Value> Readable<Root, Value> for ReadableKeyPath<Root, Value> {
    fn get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.get)(root)
    }

    fn get_fn(&self) -> for<'a> fn(&'a Root) -> &'a Value {
        self.get
    }
}


/// Read/write keypath
pub struct WritableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> &'a Value,
    pub get_mut: for<'a> fn(&'a mut Root) -> &'a mut Value,
}

pub trait Writable<Root, Value>: Readable<Root, Value> {
    fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut Value;

    fn iter_mut<'a>(&self, slice: &'a mut [Root]) -> Box<dyn Iterator<Item = &'a mut Value> + 'a>
    where
        Self: Sized,
    {
        let f = self.get_mut_fn(); // capture fn pointer
        Box::new(slice.iter_mut().map(move |root| f(root)))
    }

    fn get_mut_fn(&self) -> for<'a> fn(&'a mut Root) -> &'a mut Value;
}

impl<Root, Value> WritableKeyPath<Root, Value> {
    pub fn new(
        get: for<'a> fn(&'a Root) -> &'a Value,
        get_mut: for<'a> fn(&'a mut Root) -> &'a mut Value,
    ) -> Self {
        Self { get, get_mut }
    }
}

impl<Root, Value> Readable<Root, Value> for WritableKeyPath<Root, Value> {
    fn get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.get)(root)
    }

    fn get_fn(&self) -> for<'a> fn(&'a Root) -> &'a Value {
        self.get
    }
}

impl<Root, Value> Writable<Root, Value> for WritableKeyPath<Root, Value> {
    fn get_mut<'a>(&self, root: &'a mut Root) -> &'a mut Value {
        (self.get_mut)(root)
    }

    fn get_mut_fn(&self) -> for<'a> fn(&'a mut Root) -> &'a mut Value {
        self.get_mut
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

