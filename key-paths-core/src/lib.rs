
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

