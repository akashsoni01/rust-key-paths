use std::sync::Arc;
use std::marker::PhantomData;

// Base KeyPath
#[derive(Clone)]
pub struct KeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Value,
{
    getter: F,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value, F> KeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Value,
{
    pub fn new(getter: F) -> Self {
        Self {
            getter,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'r>(&self, root: &'r Root) -> &'r Value {
        (self.getter)(root)
    }
    
    // Static methods for container unwrapping (creating new keypaths)
    // Box<T> -> T
    pub fn for_box_static<T>() -> KeyPath<Box<T>, T, impl for<'r> Fn(&'r Box<T>) -> &'r T> {
        KeyPath::new(|b: &Box<T>| b.as_ref())
    }
    
    // Arc<T> -> T
    pub fn for_arc_static<T>() -> KeyPath<Arc<T>, T, impl for<'r> Fn(&'r Arc<T>) -> &'r T> {
        KeyPath::new(|arc: &Arc<T>| arc.as_ref())
    }
    
    // Rc<T> -> T
    pub fn for_rc_static<T>() -> KeyPath<std::rc::Rc<T>, T, impl for<'r> Fn(&'r std::rc::Rc<T>) -> &'r T> {
        KeyPath::new(|rc: &std::rc::Rc<T>| rc.as_ref())
    }
    
    // Instance methods for unwrapping containers (automatically infers Target from Value::Target)
    // Box<T> -> T
    pub fn for_box<Target>(self) -> KeyPath<Root, Target, impl for<'r> Fn(&'r Root) -> &'r Target + 'static>
    where
        Value: std::ops::Deref<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        KeyPath {
            getter: move |root: &Root| {
                getter(root).deref()
            },
            _phantom: PhantomData,
        }
    }
    
    // Arc<T> -> T
    pub fn for_arc<Target>(self) -> KeyPath<Root, Target, impl for<'r> Fn(&'r Root) -> &'r Target + 'static>
    where
        Value: std::ops::Deref<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        KeyPath {
            getter: move |root: &Root| {
                getter(root).deref()
            },
            _phantom: PhantomData,
        }
    }
    
    // Rc<T> -> T
    pub fn for_rc<Target>(self) -> KeyPath<Root, Target, impl for<'r> Fn(&'r Root) -> &'r Target + 'static>
    where
        Value: std::ops::Deref<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        KeyPath {
            getter: move |root: &Root| {
                getter(root).deref()
            },
            _phantom: PhantomData,
        }
    }
}

// Utility function for slice access (kept as standalone function)
pub fn for_slice<T>() -> impl for<'r> Fn(&'r [T], usize) -> Option<&'r T> {
    |slice: &[T], index: usize| slice.get(index)
}

// OptionalKeyPath for Option<T>
#[derive(Clone)]
pub struct OptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Value>,
{
    getter: F,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value, F> OptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Value>,
{
    pub fn new(getter: F) -> Self {
        Self {
            getter,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'r>(&self, root: &'r Root) -> Option<&'r Value> {
        (self.getter)(root)
    }
    
    // Swift-like operator for chaining OptionalKeyPath
    pub fn then<SubValue, G>(
        self,
        next: OptionalKeyPath<Value, SubValue, G>,
    ) -> OptionalKeyPath<Root, SubValue, impl for<'r> Fn(&'r Root) -> Option<&'r SubValue>>
    where
        G: for<'r> Fn(&'r Value) -> Option<&'r SubValue>,
        F: 'static,
        G: 'static,
        Value: 'static,
    {
        let first = self.getter;
        let second = next.getter;
        
        OptionalKeyPath::new(move |root: &Root| {
            first(root).and_then(|value| second(value))
        })
    }
    
    // Instance methods for unwrapping containers from Option<Container<T>>
    // Option<Box<T>> -> Option<&T> (type automatically inferred from Value::Target)
    pub fn for_box<Target>(self) -> OptionalKeyPath<Root, Target, impl for<'r> Fn(&'r Root) -> Option<&'r Target> + 'static>
    where
        Value: std::ops::Deref<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |root: &Root| {
                getter(root).map(|boxed| boxed.deref())
            },
            _phantom: PhantomData,
        }
    }
    
    // Option<Arc<T>> -> Option<&T> (type automatically inferred from Value::Target)
    pub fn for_arc<Target>(self) -> OptionalKeyPath<Root, Target, impl for<'r> Fn(&'r Root) -> Option<&'r Target> + 'static>
    where
        Value: std::ops::Deref<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |root: &Root| {
                getter(root).map(|arc| arc.deref())
            },
            _phantom: PhantomData,
        }
    }
    
    // Option<Rc<T>> -> Option<&T> (type automatically inferred from Value::Target)
    pub fn for_rc<Target>(self) -> OptionalKeyPath<Root, Target, impl for<'r> Fn(&'r Root) -> Option<&'r Target> + 'static>
    where
        Value: std::ops::Deref<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |root: &Root| {
                getter(root).map(|rc| rc.deref())
            },
            _phantom: PhantomData,
        }
    }
    
    // Static method for Option<T> -> Option<&T>
    pub fn for_option<T>() -> OptionalKeyPath<Option<T>, T, impl for<'r> Fn(&'r Option<T>) -> Option<&'r T>> {
        OptionalKeyPath::new(|opt: &Option<T>| opt.as_ref())
    }
}

// Enum-specific keypaths
pub struct EnumKeyPaths;

impl EnumKeyPaths {
    // Extract from a specific enum variant
    pub fn for_variant<Enum, Variant, ExtractFn>(
        extractor: ExtractFn
    ) -> OptionalKeyPath<Enum, Variant, impl for<'r> Fn(&'r Enum) -> Option<&'r Variant>>
    where
        ExtractFn: Fn(&Enum) -> Option<&Variant>,
    {
        OptionalKeyPath::new(extractor)
    }
    
    // Match against multiple variants (returns a tagged union)
    pub fn for_match<Enum, Output, MatchFn>(
        matcher: MatchFn
    ) -> KeyPath<Enum, Output, impl for<'r> Fn(&'r Enum) -> &'r Output>
    where
        MatchFn: Fn(&Enum) -> &Output,
    {
        KeyPath::new(matcher)
    }
    
    // Extract from Result<T, E>
    pub fn for_ok<T, E>() -> OptionalKeyPath<Result<T, E>, T, impl for<'r> Fn(&'r Result<T, E>) -> Option<&'r T>> {
        OptionalKeyPath::new(|result: &Result<T, E>| result.as_ref().ok())
    }
    
    pub fn for_err<T, E>() -> OptionalKeyPath<Result<T, E>, E, impl for<'r> Fn(&'r Result<T, E>) -> Option<&'r E>> {
        OptionalKeyPath::new(|result: &Result<T, E>| result.as_ref().err())
    }
    
    // Extract from Option<T>
    pub fn for_some<T>() -> OptionalKeyPath<Option<T>, T, impl for<'r> Fn(&'r Option<T>) -> Option<&'r T>> {
        OptionalKeyPath::new(|opt: &Option<T>| opt.as_ref())
    }
    
    // Static methods for container unwrapping (returns KeyPath)
    // Box<T> -> T
    pub fn for_box<T>() -> KeyPath<Box<T>, T, impl for<'r> Fn(&'r Box<T>) -> &'r T> {
        KeyPath::new(|b: &Box<T>| b.as_ref())
    }
    
    // Arc<T> -> T
    pub fn for_arc<T>() -> KeyPath<Arc<T>, T, impl for<'r> Fn(&'r Arc<T>) -> &'r T> {
        KeyPath::new(|arc: &Arc<T>| arc.as_ref())
    }
    
    // Rc<T> -> T
    pub fn for_rc<T>() -> KeyPath<std::rc::Rc<T>, T, impl for<'r> Fn(&'r std::rc::Rc<T>) -> &'r T> {
        KeyPath::new(|rc: &std::rc::Rc<T>| rc.as_ref())
    }
}

// Helper to create enum variant keypaths with type inference
pub fn variant_of<Enum, Variant, F>(extractor: F) -> OptionalKeyPath<Enum, Variant, F>
where
    F: for<'r> Fn(&'r Enum) -> Option<&'r Variant>,
{
    OptionalKeyPath::new(extractor)
}

// Usage example
#[derive(Debug)]
struct User {
    name: String,
    metadata: Option<Box<UserMetadata>>,
    friends: Vec<Arc<User>>,
}

#[derive(Debug)]
struct UserMetadata {
    created_at: String,
}

fn some_fn() {
    let alice = User {
        name: "Alice".to_string(),
        metadata: Some(Box::new(UserMetadata {
            created_at: "2024-01-01".to_string(),
        })),
        friends: vec![
            Arc::new(User {
                name: "Bob".to_string(),
                metadata: None,
                friends: vec![],
            }),
        ],
    };
    
    // Create keypaths
    let name_kp = KeyPath::new(|u: &User| &u.name);
    let metadata_kp = OptionalKeyPath::new(|u: &User| u.metadata.as_ref());
    let friends_kp = KeyPath::new(|u: &User| &u.friends);
    
    // Use them
    println!("Name: {}", name_kp.get(&alice));
    
    if let Some(metadata) = metadata_kp.get(&alice) {
        println!("Has metadata: {:?}", metadata);
    }
    
    // Access first friend's name
    if let Some(first_friend) = alice.friends.get(0) {
        println!("First friend: {}", name_kp.get(first_friend));
    }
    
    // Access metadata through Box using for_box()
    let created_at_kp = KeyPath::new(|m: &UserMetadata| &m.created_at);
    
    if let Some(metadata) = alice.metadata.as_ref() {
        // Use for_box() to unwrap Box<UserMetadata> to &UserMetadata
        let boxed_metadata: &Box<UserMetadata> = metadata;
        let unwrapped = boxed_metadata.as_ref();
        println!("Created at: {:?}", created_at_kp.get(unwrapped));
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        some_fn();
    }
}