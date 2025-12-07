use std::marker::PhantomData;

/// A readable keypath that can access a value from a root type
#[derive(Copy, Clone)]
pub struct ReadableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> &'a Value,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value> ReadableKeyPath<Root, Value> {
    /// Create a new keypath from a function pointer
    pub const fn new(get: for<'a> fn(&'a Root) -> &'a Value) -> Self {
        Self {
            get,
            _phantom: PhantomData,
        }
    }
    
    /// Get a reference to the value from the root
    pub fn get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.get)(root)
    }
}

#[cfg(test)]
mod tests {
    use super::ReadableKeyPath;

    struct Person {
        name: String,
        age: u32,
    }

    #[test]
    fn test_readable_keypath_single_field() {
        // Create a keypath for the name field
        let name_keypath = ReadableKeyPath::new(|p: &Person| &p.name);
        
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };
        
        // Test getting the value through the keypath
        let name = name_keypath.get(&person);
        assert_eq!(name, "Alice");
        
        // Test that we get a reference to the actual field
        assert_eq!(*name, person.name);
    }
}
