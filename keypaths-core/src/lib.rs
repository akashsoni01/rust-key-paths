use std::marker::PhantomData;
use std::any::{Any, TypeId};
use std::sync::Arc;
use std::rc::Rc;

// ========== CORE KEYPATH TYPES ==========

/// Read-only keypath (like Swift's KeyPath)
#[derive(Copy, Clone)]
pub struct ReadableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> &'a Value,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value> ReadableKeyPath<Root, Value> {
    pub const fn new(get: for<'a> fn(&'a Root) -> &'a Value) -> Self {
        Self {
            get,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.get)(root)
    }
    
    pub fn appending<SubValue>(
        &self,
        next: ReadableKeyPath<Value, SubValue>,
    ) -> ReadableKeyPath<Root, SubValue> {
        ReadableKeyPath::new(|root| next.get(self.get(root)))
    }
}

/// Writable keypath (like Swift's WritableKeyPath)
#[derive(Copy, Clone)]
pub struct WritableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> &'a Value,
    pub set: for<'a> fn(&'a mut Root, Value),
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value> WritableKeyPath<Root, Value> {
    pub const fn new(
        get: for<'a> fn(&'a Root) -> &'a Value,
        set: for<'a> fn(&'a mut Root, Value),
    ) -> Self {
        Self {
            get,
            set,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.get)(root)
    }
    
    pub fn set<'a>(&self, root: &'a mut Root, value: Value) {
        (self.set)(root, value)
    }
    
    pub fn mut_get<'a>(&self, root: &'a mut Root) -> &'a mut Value {
        unsafe {
            let ptr = root as *mut Root;
            let value_ptr = (self.get)(&*ptr) as *const Value as *mut Value;
            &mut *value_ptr
        }
    }
    
    pub fn appending<SubValue>(
        &self,
        next: WritableKeyPath<Value, SubValue>,
    ) -> WritableKeyPath<Root, SubValue> {
        WritableKeyPath::new(
            |root| next.get(self.get(root)),
            |root, value| {
                let inner = self.mut_get(root);
                next.set(inner, value);
            },
        )
    }
}

/// Reference-writable keypath (like Swift's ReferenceWritableKeyPath)
#[derive(Copy, Clone)]
pub struct ReferenceWritableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> &'a Value,
    pub set: fn(&Root, Value),
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value> ReferenceWritableKeyPath<Root, Value> {
    pub const fn new(
        get: for<'a> fn(&'a Root) -> &'a Value,
        set: fn(&Root, Value),
    ) -> Self {
        Self {
            get,
            set,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.get)(root)
    }
    
    pub fn set(&self, root: &Root, value: Value) {
        (self.set)(root, value)
    }
    
    pub fn appending<SubValue>(
        &self,
        next: ReferenceWritableKeyPath<Value, SubValue>,
    ) -> ReferenceWritableKeyPath<Root, SubValue> {
        ReferenceWritableKeyPath::new(
            |root| next.get(self.get(root)),
            |root, value| {
                let inner = self.get(root);
                next.set(inner, value);
            },
        )
    }
}

// ========== OPTIONAL KEYPATHS ==========

/// Optional/Nullable keypath (returns Option<&T>)
#[derive(Copy, Clone)]
pub struct OptionalKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> Option<&'a Value>,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value> OptionalKeyPath<Root, Value> {
    pub const fn new(get: for<'a> fn(&'a Root) -> Option<&'a Value>) -> Self {
        Self {
            get,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, root: &'a Root) -> Option<&'a Value> {
        (self.get)(root)
    }
    
    pub fn appending<SubValue>(
        &self,
        next: ReadableKeyPath<Value, SubValue>,
    ) -> OptionalKeyPath<Root, SubValue> {
        OptionalKeyPath::new(|root| self.get(root).map(|v| next.get(v)))
    }
    
    pub fn appending_optional<SubValue>(
        &self,
        next: OptionalKeyPath<Value, SubValue>,
    ) -> OptionalKeyPath<Root, SubValue> {
        OptionalKeyPath::new(|root| self.get(root).and_then(|v| next.get(v)))
    }
}

/// Optional-writable keypath
#[derive(Copy, Clone)]
pub struct OptionalWritableKeyPath<Root, Value> {
    pub get: for<'a> fn(&'a Root) -> Option<&'a Value>,
    pub set: for<'a> fn(&'a mut Root, Option<Value>),
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value> OptionalWritableKeyPath<Root, Value> {
    pub const fn new(
        get: for<'a> fn(&'a Root) -> Option<&'a Value>,
        set: for<'a> fn(&'a mut Root, Option<Value>),
    ) -> Self {
        Self {
            get,
            set,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, root: &'a Root) -> Option<&'a Value> {
        (self.get)(root)
    }
    
    pub fn set<'a>(&self, root: &'a mut Root, value: Option<Value>) {
        (self.set)(root, value)
    }
}

// ========== TYPE-ERASED KEYPATHS ==========

/// Partial type-erased keypath (knows root type)
pub struct PartialReadableKeyPath<Root> {
    pub get: for<'a> fn(&'a Root) -> &'a dyn Any,
    pub value_type_id: TypeId,
}

impl<Root> PartialReadableKeyPath<Root> {
    pub const fn new<Value>(keypath: ReadableKeyPath<Root, Value>) -> Self 
    where
        Value: 'static,
    {
        Self {
            get: |root| keypath.get(root) as &dyn Any,
            value_type_id: TypeId::of::<Value>(),
        }
    }
    
    pub fn get<'a>(&self, root: &'a Root) -> &'a dyn Any {
        (self.get)(root)
    }
    
    pub fn get_as<'a, Value>(&self, root: &'a Root) -> Option<&'a Value>
    where
        Value: 'static,
    {
        if self.value_type_id == TypeId::of::<Value>() {
            (self.get)(root).downcast_ref()
        } else {
            None
        }
    }
}

/// Fully type-erased keypath
pub struct AnyReadableKeyPath {
    pub get: for<'a> fn(&'a dyn Any) -> &'a dyn Any,
    pub root_type_id: TypeId,
    pub value_type_id: TypeId,
}

impl AnyReadableKeyPath {
    pub const fn new<Root, Value>(keypath: ReadableKeyPath<Root, Value>) -> Self 
    where
        Root: 'static,
        Value: 'static,
    {
        Self {
            get: |root| {
                let root = root.downcast_ref::<Root>().expect("Type mismatch");
                keypath.get(root) as &dyn Any
            },
            root_type_id: TypeId::of::<Root>(),
            value_type_id: TypeId::of::<Value>(),
        }
    }
    
    pub fn get<'a>(&self, root: &'a dyn Any) -> &'a dyn Any {
        (self.get)(root)
    }
    
    pub fn get_typed<'a, Root, Value>(&self, root: &'a Root) -> Option<&'a Value>
    where
        Root: 'static,
        Value: 'static,
    {
        if TypeId::of::<Root>() == self.root_type_id && 
           TypeId::of::<Value>() == self.value_type_id {
            let any = root as &dyn Any;
            self.get(any).downcast_ref()
        } else {
            None
        }
    }
}

// ========== CONTAINER KEYPATHS ==========

/// Keypath for smart pointers (Box, Arc, Rc)
#[derive(Copy, Clone)]
pub struct SmartPointerKeyPath<Ptr, Value> {
    pub deref: for<'a> fn(&'a Ptr) -> &'a Value,
    _phantom: PhantomData<(Ptr, Value)>,
}

impl<Ptr, Value> SmartPointerKeyPath<Ptr, Value> {
    pub const fn new(deref: for<'a> fn(&'a Ptr) -> &'a Value) -> Self {
        Self {
            deref,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, ptr: &'a Ptr) -> &'a Value {
        (self.deref)(ptr)
    }
}

// Predefined smart pointer keypaths
pub mod smart_ptr {
    use super::*;
    
    pub const fn boxed<T>() -> SmartPointerKeyPath<Box<T>, T> {
        SmartPointerKeyPath::new(|b: &Box<T>| b.as_ref())
    }
    
    pub const fn arc<T>() -> SmartPointerKeyPath<Arc<T>, T> {
        SmartPointerKeyPath::new(|a: &Arc<T>| a.as_ref())
    }
    
    pub const fn rc<T>() -> SmartPointerKeyPath<Rc<T>, T> {
        SmartPointerKeyPath::new(|r: &Rc<T>| r.as_ref())
    }
}

/// Keypath for collections (index-based access)
#[derive(Copy, Clone)]
pub struct CollectionKeyPath<Collection, Item> {
    pub get: for<'a> fn(&'a Collection, usize) -> Option<&'a Item>,
    _phantom: PhantomData<(Collection, Item)>,
}

impl<Collection, Item> CollectionKeyPath<Collection, Item> {
    pub const fn new(get: for<'a> fn(&'a Collection, usize) -> Option<&'a Item>) -> Self {
        Self {
            get,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, collection: &'a Collection, index: usize) -> Option<&'a Item> {
        (self.get)(collection, index)
    }
}

/// Keypath for tuples
#[derive(Copy, Clone)]
pub struct TupleKeyPath<Tuple, Element, const N: usize> {
    pub get: for<'a> fn(&'a Tuple) -> &'a Element,
    _phantom: PhantomData<(Tuple, Element)>,
}

impl<Tuple, Element, const N: usize> TupleKeyPath<Tuple, Element, N> {
    pub const fn new(get: for<'a> fn(&'a Tuple) -> &'a Element) -> Self {
        Self {
            get,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, tuple: &'a Tuple) -> &'a Element {
        (self.get)(tuple)
    }
}

// ========== ENUM KEYPATHS ==========

/// Keypath for enum variants
#[derive(Copy, Clone)]
pub struct EnumVariantKeyPath<Enum, Variant> {
    pub extract: for<'a> fn(&'a Enum) -> Option<&'a Variant>,
    _phantom: PhantomData<(Enum, Variant)>,
}

impl<Enum, Variant> EnumVariantKeyPath<Enum, Variant> {
    pub const fn new(extract: for<'a> fn(&'a Enum) -> Option<&'a Variant>) -> Self {
        Self {
            extract,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'a>(&self, enm: &'a Enum) -> Option<&'a Variant> {
        (self.extract)(enm)
    }
}

/// Keypath for Result types
pub mod result {
    use super::*;
    
    pub const fn ok<T, E>() -> EnumVariantKeyPath<Result<T, E>, T> {
        EnumVariantKeyPath::new(|r: &Result<T, E>| r.as_ref().ok())
    }
    
    pub const fn err<T, E>() -> EnumVariantKeyPath<Result<T, E>, E> {
        EnumVariantKeyPath::new(|r: &Result<T, E>| r.as_ref().err())
    }
}

/// Keypath for Option types
pub mod option {
    use super::*;
    
    pub const fn some<T>() -> EnumVariantKeyPath<Option<T>, T> {
        EnumVariantKeyPath::new(|o: &Option<T>| o.as_ref())
    }
}

// ========== CONVERSION TRAITS ==========

pub trait IntoReadable<Root, Value> {
    fn into_readable(self) -> ReadableKeyPath<Root, Value>;
}

pub trait IntoWritable<Root, Value> {
    fn into_writable(self) -> WritableKeyPath<Root, Value>;
}

pub trait IntoOptional<Root, Value> {
    fn into_optional(self) -> OptionalKeyPath<Root, Value>;
}

// ========== EXAMPLE TYPES ==========

#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u32,
    address: Box<Address>,
    metadata: Option<Metadata>,
    scores: Vec<u32>,
}

#[derive(Debug, Clone)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Debug, Clone)]
struct Metadata {
    created_at: String,
    tags: Vec<String>,
}

#[derive(Debug)]
enum PaymentMethod {
    CreditCard { number: String, expiry: String },
    PayPal(String),
    Cash,
}

// ========== MACROS FOR EASY CREATION ==========

#[macro_export]
macro_rules! readable {
    ($root:ty => $value:ty : |$param:ident| $expr:expr) => {
        ReadableKeyPath::new(|$param: &$root| {
            let value: &$value = $expr;
            value
        })
    };
}

#[macro_export]
macro_rules! writable {
    ($root:ty => $value:ty : 
        get |$get_param:ident| $get_expr:expr,
        set |$set_param:ident, $value_param:ident| $set_expr:expr
    ) => {
        WritableKeyPath::new(
            |$get_param: &$root| {
                let value: &$value = $get_expr;
                value
            },
            |$set_param: &mut $root, $value_param: $value| $set_expr,
        )
    };
}

#[macro_export]
macro_rules! optional {
    ($root:ty => $value:ty : |$param:ident| $expr:expr) => {
        OptionalKeyPath::new(|$param: &$root| {
            let opt: Option<&$value> = $expr;
            opt
        })
    };
}

// ========== USAGE EXAMPLES ==========

fn main() {
    println!("=== Example 1: Basic ReadableKeyPath ===");
    
    let alice = Person {
        name: "Alice".to_string(),
        age: 30,
        address: Box::new(Address {
            street: "123 Main St".to_string(),
            city: "San Francisco".to_string(),
            zip: "94107".to_string(),
        }),
        metadata: Some(Metadata {
            created_at: "2024-01-01".to_string(),
            tags: vec!["premium".to_string()],
        }),
        scores: vec![95, 88, 92],
    };
    
    // Create readable keypaths
    let name_kp = ReadableKeyPath::new(|p: &Person| &p.name);
    let age_kp = ReadableKeyPath::new(|p: &Person| &p.age);
    
    println!("Name: {}", name_kp.get(&alice));
    println!("Age: {}", age_kp.get(&alice));
    
    // Using macro
    let address_kp = readable!(Person => Box<Address> : |p| &p.address);
    println!("Address: {:?}", address_kp.get(&alice));
    
    println!("\n=== Example 2: Chaining KeyPaths ===");
    
    // Chain: Person -> Box<Address> -> Address -> city
    let city_kp = ReadableKeyPath::new(|a: &Address| &a.city);
    let person_city_kp = address_kp.appending(smart_ptr::boxed::<Address>()).appending(city_kp);
    
    println!("City: {}", person_city_kp.get(&alice));
    
    println!("\n=== Example 3: WritableKeyPath ===");
    
    let mut bob = Person {
        name: "Bob".to_string(),
        age: 25,
        address: Box::new(Address {
            street: "456 Oak St".to_string(),
            city: "New York".to_string(),
            zip: "10001".to_string(),
        }),
        metadata: None,
        scores: vec![],
    };
    
    let name_writable = WritableKeyPath::new(
        |p: &Person| &p.name,
        |p: &mut Person, value: String| p.name = value,
    );
    
    // Using macro
    let age_writable = writable!(Person => u32 : 
        get |p| &p.age,
        set |p, value| p.age = value
    );
    
    println!("Before - Name: {}, Age: {}", name_writable.get(&bob), age_writable.get(&bob));
    
    name_writable.set(&mut bob, "Robert".to_string());
    age_writable.set(&mut bob, 26);
    
    println!("After - Name: {}, Age: {}", name_writable.get(&bob), age_writable.get(&bob));
    
    // Mutable get
    let age_ref = age_writable.mut_get(&mut bob);
    *age_ref = 27;
    println!("After mut_get - Age: {}", age_writable.get(&bob));
    
    println!("\n=== Example 4: OptionalKeyPath ===");
    
    let metadata_kp = OptionalKeyPath::new(|p: &Person| p.metadata.as_ref());
    let tags_kp = ReadableKeyPath::new(|m: &Metadata| &m.tags);
    
    let person_tags_kp = metadata_kp.appending(tags_kp);
    
    if let Some(tags) = person_tags_kp.get(&alice) {
        println!("Tags: {:?}", tags);
    }
    
    if let Some(tags) = person_tags_kp.get(&bob) {
        println!("Bob's tags: {:?}", tags);
    } else {
        println!("Bob has no metadata");
    }
    
    println!("\n=== Example 5: Enum KeyPaths ===");
    
    let payment = PaymentMethod::CreditCard {
        number: "4111111111111111".to_string(),
        expiry: "12/25".to_string(),
    };
    
    // Create enum variant extractor manually
    let credit_card_kp = EnumVariantKeyPath::new(|p: &PaymentMethod| {
        match p {
            PaymentMethod::CreditCard { number, expiry } => {
                // Return a tuple or create a struct
                // For simplicity, we'll just return the number reference
                Some(number)
            }
            _ => None,
        }
    });
    
    if let Some(number) = credit_card_kp.get(&payment) {
        println!("Credit card number: {}", number);
    }
    
    println!("\n=== Example 6: Type-Erased KeyPaths ===");
    
    // Convert to partial keypath
    let name_partial = PartialReadableKeyPath::new(name_kp);
    let age_partial = PartialReadableKeyPath::new(age_kp);
    
    // Store in homogeneous collection
    let partials: [&PartialReadableKeyPath<Person>; 2] = [&name_partial, &age_partial];
    
    for partial in &partials {
        if let Some(name) = partial.get_as::<String>(&alice) {
            println!("String value: {}", name);
        }
        if let Some(age) = partial.get_as::<u32>(&alice) {
            println!("u32 value: {}", age);
        }
    }
    
    // Convert to any keypath
    let name_any = AnyReadableKeyPath::new(name_kp);
    let age_any = AnyReadableKeyPath::new(age_kp);
    
    if let Some(name) = name_any.get_typed::<Person, String>(&alice) {
        println!("Via AnyKeyPath - Name: {}", name);
    }
    
    println!("\n=== Example 7: Collection KeyPaths ===");
    
    let scores_kp = CollectionKeyPath::new(|p: &Person, idx| p.scores.get(idx));
    
    if let Some(score) = scores_kp.get(&alice, 0) {
        println!("First score: {}", score);
    }
    
    println!("\n=== Example 8: Tuple KeyPaths ===");
    
    let point = (10, 20, "origin");
    let x_kp = TupleKeyPath::new(|p: &(i32, i32, &str)| &p.0);
    let z_kp = TupleKeyPath::new(|p: &(i32, i32, &str)| &p.2);
    
    println!("X coordinate: {}", x_kp.get(&point));
    println!("Label: {}", z_kp.get(&point));
}

// ========== BENCHMARK COMPARISON ==========

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn benchmark_static_vs_dynamic() {
        let person = Person {
            name: "Test".to_string(),
            age: 30,
            address: Box::new(Address {
                street: "Test".to_string(),
                city: "Test".to_string(),
                zip: "00000".to_string(),
            }),
            metadata: None,
            scores: vec![],
        };
        
        // Static dispatch (function pointer)
        let name_static = ReadableKeyPath::new(|p: &Person| &p.name);
        
        // Dynamic dispatch (trait object)
        let name_dynamic: Box<dyn for<'a> Fn(&'a Person) -> &'a String> = 
            Box::new(|p: &Person| &p.name);
        
        const ITERATIONS: usize = 10_000_000;
        
        // Benchmark static
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = name_static.get(&person);
        }
        let static_time = start.elapsed();
        
        // Benchmark dynamic
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = name_dynamic(&person);
        }
        let dynamic_time = start.elapsed();
        
        println!("Static dispatch: {:?}", static_time);
        println!("Dynamic dispatch: {:?}", dynamic_time);
        println!("Static is {:.2}x faster", 
            dynamic_time.as_nanos() as f64 / static_time.as_nanos() as f64);
        
        // Static should be significantly faster
        assert!(static_time < dynamic_time);
    }
    
    #[test]
    fn test_keypath_composition() {
        let alice = Person {
            name: "Alice".to_string(),
            age: 30,
            address: Box::new(Address {
                street: "123 Main".to_string(),
                city: "SF".to_string(),
                zip: "94107".to_string(),
            }),
            metadata: Some(Metadata {
                created_at: "now".to_string(),
                tags: vec!["test".to_string()],
            }),
            scores: vec![100],
        };
        
        // Complex chain
        let tags_kp = OptionalKeyPath::new(|p: &Person| p.metadata.as_ref())
            .appending(ReadableKeyPath::new(|m: &Metadata| &m.tags))
            .appending(CollectionKeyPath::new(|v: &Vec<String>, i| v.get(i)))
            .appending(ReadableKeyPath::new(|s: &String| s));
        
        if let Some(tag) = tags_kp.get(&alice) {
            assert_eq!(tag, "test");
        } else {
            panic!("Should have found tag");
        }
    }
}