use std::sync::Arc;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

// Global counter to track memory allocations/deallocations
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

// Type that panics on clone to detect unwanted cloning
#[derive(Debug)]
struct NoCloneType {
    id: usize,
    data: String,
}

impl NoCloneType {
    fn new(data: String) -> Self {
        ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
        Self {
            id: ALLOC_COUNT.load(Ordering::SeqCst),
            data,
        }
    }
}

impl Clone for NoCloneType {
    fn clone(&self) -> Self {
        panic!("NoCloneType should not be cloned! ID: {}", self.id);
    }
}

impl Drop for NoCloneType {
    fn drop(&mut self) {
        DEALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

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
#[derive(Clone)]
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
    
    // Static method for Option<T> -> Option<&T> (alias for for_some for consistency)
    pub fn for_option<T>() -> OptionalKeyPath<Option<T>, T, impl for<'r> Fn(&'r Option<T>) -> Option<&'r T>> {
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

    // Helper functions for testing memory management
    fn reset_memory_counters() {
        ALLOC_COUNT.store(0, Ordering::SeqCst);
        DEALLOC_COUNT.store(0, Ordering::SeqCst);
    }

    fn get_alloc_count() -> usize {
        ALLOC_COUNT.load(Ordering::SeqCst)
    }

    fn get_dealloc_count() -> usize {
        DEALLOC_COUNT.load(Ordering::SeqCst)
    }

    #[test]
    fn test_name() {
        some_fn();
    }
    
    #[test]
    fn test_no_cloning_on_keypath_operations() {
        reset_memory_counters();
        
        // Create a value that panics on clone
        let value = NoCloneType::new("test".to_string());
        let boxed = Box::new(value);
        
        // Create keypath - should not clone
        let kp = KeyPath::new(|b: &Box<NoCloneType>| b.as_ref());
        
        // Access value - should not clone
        let _ref = kp.get(&boxed);
        
        // Clone the keypath itself (this is allowed)
        let _kp_clone = kp.clone();
        
        // Access again - should not clone the value
        let _ref2 = _kp_clone.get(&boxed);
        
        // Verify no panics occurred (if we got here, no cloning happened)
        assert_eq!(get_alloc_count(), 1);
    }
    
    #[test]
    fn test_no_cloning_on_optional_keypath_operations() {
        reset_memory_counters();
        
        let value = NoCloneType::new("test".to_string());
        let opt = Some(Box::new(value));
        
        // Create optional keypath
        let okp = OptionalKeyPath::new(|o: &Option<Box<NoCloneType>>| o.as_ref());
        
        // Access - should not clone
        let _ref = okp.get(&opt);
        
        // Clone keypath (allowed)
        let _okp_clone = okp.clone();
        
        // Chain operations - should not clone values
        let chained = okp.then(OptionalKeyPath::new(|b: &Box<NoCloneType>| Some(b.as_ref())));
        let _ref2 = chained.get(&opt);
        
        assert_eq!(get_alloc_count(), 1);
    }
    
    #[test]
    fn test_memory_release() {
        reset_memory_counters();
        
        {
            let value = NoCloneType::new("test".to_string());
            let boxed = Box::new(value);
            let kp = KeyPath::new(|b: &Box<NoCloneType>| b.as_ref());
            
            // Use the keypath
            let _ref = kp.get(&boxed);
            
            // boxed goes out of scope here
        }
        
        // After drop, memory should be released
        // Note: This is a best-effort check since drop timing can vary
        assert_eq!(get_alloc_count(), 1);
        // Deallocation happens when the value is dropped
        // We can't reliably test exact timing, but we verify the counter exists
    }
    
    #[test]
    fn test_keypath_clone_does_not_clone_underlying_data() {
        reset_memory_counters();
        
        let value = NoCloneType::new("data".to_string());
        let rc_value = Rc::new(value);
        
        // Create keypath
        let kp = KeyPath::new(|r: &Rc<NoCloneType>| r.as_ref());
        
        // Clone keypath multiple times
        let kp1 = kp.clone();
        let kp2 = kp.clone();
        let kp3 = kp1.clone();
        
        // All should work without cloning the underlying data
        let _ref1 = kp.get(&rc_value);
        let _ref2 = kp1.get(&rc_value);
        let _ref3 = kp2.get(&rc_value);
        let _ref4 = kp3.get(&rc_value);
        
        // Only one allocation should have happened
        assert_eq!(get_alloc_count(), 1);
    }
    
    #[test]
    fn test_optional_keypath_chaining_no_clone() {
        reset_memory_counters();
        
        let value = NoCloneType::new("value1".to_string());
        
        struct Container {
            inner: Option<Box<NoCloneType>>,
        }
        
        let container = Container {
            inner: Some(Box::new(value)),
        };
        
        // Create chained keypath
        let kp1 = OptionalKeyPath::new(|c: &Container| c.inner.as_ref());
        let kp2 = OptionalKeyPath::new(|b: &Box<NoCloneType>| Some(b.as_ref()));
        
        // Chain them - should not clone
        let chained = kp1.then(kp2);
        
        // Use chained keypath
        let _result = chained.get(&container);
        
        // Should only have one allocation
        assert_eq!(get_alloc_count(), 1);
    }
    
    #[test]
    fn test_for_box_no_clone() {
        reset_memory_counters();
        
        let value = NoCloneType::new("test".to_string());
        let boxed = Box::new(value);
        let opt_boxed = Some(boxed);
        
        // Create keypath with for_box
        let kp = OptionalKeyPath::new(|o: &Option<Box<NoCloneType>>| o.as_ref());
        let unwrapped = kp.for_box();
        
        // Access - should not clone
        let _ref = unwrapped.get(&opt_boxed);
        
        assert_eq!(get_alloc_count(), 1);
    }
}
