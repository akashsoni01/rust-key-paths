/// Read-only keypath
pub struct ReadableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> &'a Value,
}

/// Read/write keypath
pub struct WritableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> &'a Value,
    pub get_mut: for<'a> fn(&'a mut Root) -> &'a mut Value,
}

