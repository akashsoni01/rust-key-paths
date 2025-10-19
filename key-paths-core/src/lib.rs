use std::sync::{Arc, Mutex, RwLock};
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

#[cfg(feature = "tagged_core")]
use tagged_core::Tagged;

/// Trait for no-clone callback-based access to container types
/// Provides methods to execute closures with references to values inside containers
/// without requiring cloning of the values
pub trait WithContainer<Root, Value> {
    /// Execute a closure with a reference to the value inside an Arc
    /// This avoids cloning by working with references directly
    fn with_arc<F, R>(self, arc: &Arc<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a reference to the value inside a Box
    /// This avoids cloning by working with references directly
    fn with_box<F, R>(self, boxed: &Box<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside a Box
    /// This avoids cloning by working with references directly
    fn with_box_mut<F, R>(self, boxed: &mut Box<Root>, f: F) -> R
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside an Rc
    /// This avoids cloning by working with references directly
    fn with_rc<F, R>(self, rc: &Rc<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a reference to the value inside a Result
    /// This avoids cloning by working with references directly
    fn with_result<F, R, E>(self, result: &Result<Root, E>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside a Result
    /// This avoids cloning by working with references directly
    fn with_result_mut<F, R, E>(self, result: &mut Result<Root, E>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside an Option
    /// This avoids cloning by working with references directly
    fn with_option<F, R>(self, option: &Option<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside an Option
    /// This avoids cloning by working with references directly
    fn with_option_mut<F, R>(self, option: &mut Option<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside a RefCell
    /// This avoids cloning by working with references directly
    fn with_refcell<F, R>(self, refcell: &RefCell<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside a RefCell
    /// This avoids cloning by working with references directly
    fn with_refcell_mut<F, R>(self, refcell: &RefCell<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside a Tagged
    /// This avoids cloning by working with references directly
    #[cfg(feature = "tagged_core")]
    fn with_tagged<F, R, Tag>(self, tagged: &Tagged<Root, Tag>, f: F) -> R
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a reference to the value inside a Mutex
    /// This avoids cloning by working with references while the guard is alive
    fn with_mutex<F, R>(self, mutex: &Mutex<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside a Mutex
    /// This avoids cloning by working with references while the guard is alive
    fn with_mutex_mut<F, R>(self, mutex: &mut Mutex<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside an RwLock
    /// This avoids cloning by working with references while the guard is alive
    fn with_rwlock<F, R>(self, rwlock: &RwLock<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside an RwLock
    /// This avoids cloning by working with references while the guard is alive
    fn with_rwlock_mut<F, R>(self, rwlock: &mut RwLock<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside an Arc<RwLock<Root>>
    /// This avoids cloning by working with references while the guard is alive
    fn with_arc_rwlock<F, R>(self, arc_rwlock: &Arc<RwLock<Root>>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside an Arc<RwLock<Root>>
    /// This avoids cloning by working with references while the guard is alive
    fn with_arc_rwlock_mut<F, R>(self, arc_rwlock: &Arc<RwLock<Root>>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;
}

#[derive(Clone)]
/// Go to examples section to see the implementations
///
pub enum KeyPaths<Root, Value> {
    Readable(Arc<dyn for<'a> Fn(&'a Root) -> &'a Value + Send + Sync>),
    ReadableEnum {
        extract: Arc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value> + Send + Sync>,
        embed: Arc<dyn Fn(Value) -> Root + Send + Sync>,
    },
    FailableReadable(Arc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value> + Send + Sync>),

    Writable(Arc<dyn for<'a> Fn(&'a mut Root) -> &'a mut Value + Send + Sync>),
    FailableWritable(Arc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + Send + Sync>),
    WritableEnum {
        extract: Arc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value> + Send + Sync>,
        extract_mut: Arc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + Send + Sync>,
        embed: Arc<dyn Fn(Value) -> Root + Send + Sync>,
    },

    // Reference-specific writable keypath (for reference types like classes)
    ReferenceWritable(Arc<dyn for<'a> Fn(&'a mut Root) -> &'a mut Value + Send + Sync>),

    // New Owned KeyPath types (value semantics)
    Owned(Arc<dyn Fn(Root) -> Value + Send + Sync>),
    FailableOwned(Arc<dyn Fn(Root) -> Option<Value> + Send + Sync>),    
}

/// PartialKeyPath<Root> - Type-erased keypath with known Root but unknown Value
/// Equivalent to Swift's PartialKeyPath<Root>
/// Useful for collections of keypaths from the same root type but with different value types
#[derive(Clone)]
pub enum PartialKeyPath<Root> {
    Readable(Arc<dyn for<'a> Fn(&'a Root) -> &'a (dyn Any + Send + Sync) + Send + Sync>),
    ReadableEnum {
        extract: Arc<dyn for<'a> Fn(&'a Root) -> Option<&'a (dyn Any + Send + Sync)> + Send + Sync>,
        embed: Arc<dyn Fn(Box<dyn Any>) -> Root + Send + Sync>,
    },
    FailableReadable(Arc<dyn for<'a> Fn(&'a Root) -> Option<&'a (dyn Any + Send + Sync)> + Send + Sync>),

    Writable(Arc<dyn for<'a> Fn(&'a mut Root) -> &'a mut (dyn Any + Send + Sync) + Send + Sync>),
    FailableWritable(Arc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut (dyn Any + Send + Sync)> + Send + Sync>),
    WritableEnum {
        extract: Arc<dyn for<'a> Fn(&'a Root) -> Option<&'a (dyn Any + Send + Sync)> + Send + Sync>,
        extract_mut: Arc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut (dyn Any + Send + Sync)> + Send + Sync>,
        embed: Arc<dyn Fn(Box<dyn Any>) -> Root + Send + Sync>,
    },

    ReferenceWritable(Arc<dyn for<'a> Fn(&'a mut Root) -> &'a mut (dyn Any + Send + Sync) + Send + Sync>),

    Owned(Arc<dyn Fn(Root) -> Box<dyn Any> + Send + Sync>),
    FailableOwned(Arc<dyn Fn(Root) -> Option<Box<dyn Any>> + Send + Sync>),
}

/// AnyKeyPath - Fully type-erased keypath for unknown Root and Value
/// Equivalent to Swift's AnyKeyPath
/// Useful when Root and Value types are unknown or need to be hidden
#[derive(Clone)]
pub enum AnyKeyPath {
    Readable(Arc<dyn for<'a> Fn(&'a (dyn Any + Send + Sync)) -> &'a (dyn Any + Send + Sync) + Send + Sync>),
    ReadableEnum {
        extract: Arc<dyn for<'a> Fn(&'a (dyn Any + Send + Sync)) -> Option<&'a (dyn Any + Send + Sync)> + Send + Sync>,
        embed: Arc<dyn Fn(Box<dyn Any>) -> Box<dyn Any> + Send + Sync>,
    },
    FailableReadable(Arc<dyn for<'a> Fn(&'a (dyn Any + Send + Sync)) -> Option<&'a (dyn Any + Send + Sync)> + Send + Sync>),

    Writable(Arc<dyn for<'a> Fn(&'a mut (dyn Any + Send + Sync)) -> &'a mut (dyn Any + Send + Sync) + Send + Sync>),
    FailableWritable(Arc<dyn for<'a> Fn(&'a mut (dyn Any + Send + Sync)) -> Option<&'a mut (dyn Any + Send + Sync)> + Send + Sync>),
    WritableEnum {
        extract: Arc<dyn for<'a> Fn(&'a (dyn Any + Send + Sync)) -> Option<&'a (dyn Any + Send + Sync)> + Send + Sync>,
        extract_mut: Arc<dyn for<'a> Fn(&'a mut (dyn Any + Send + Sync)) -> Option<&'a mut (dyn Any + Send + Sync)> + Send + Sync>,
        embed: Arc<dyn Fn(Box<dyn Any>) -> Box<dyn Any> + Send + Sync>,
    },

    ReferenceWritable(Arc<dyn for<'a> Fn(&'a mut (dyn Any + Send + Sync)) -> &'a mut (dyn Any + Send + Sync) + Send + Sync>),

    Owned(Arc<dyn Fn(Box<dyn Any>) -> Box<dyn Any> + Send + Sync>),
    FailableOwned(Arc<dyn Fn(Box<dyn Any>) -> Option<Box<dyn Any>> + Send + Sync>),
}

impl<Root, Value> KeyPaths<Root, Value> {
    #[inline]
    pub fn readable(get: impl for<'a> Fn(&'a Root) -> &'a Value + Send + Sync + 'static) -> Self {
        Self::Readable(Arc::new(get))
    }

    #[inline]
    pub fn writable(get_mut: impl for<'a> Fn(&'a mut Root) -> &'a mut Value + Send + Sync + 'static) -> Self {
        Self::Writable(Arc::new(get_mut))
    }

    #[inline]
    pub fn failable_readable(
        get: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + Send + Sync + 'static,
    ) -> Self {
        Self::FailableReadable(Arc::new(get))
    }

    #[inline]
    pub fn failable_writable(
        get_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + Send + Sync + 'static,
    ) -> Self {
        Self::FailableWritable(Arc::new(get_mut))
    }

    #[inline]
    pub fn readable_enum(
        embed: impl Fn(Value) -> Root + Send + Sync + 'static,
        extract: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + Send + Sync + 'static,
    ) -> Self {
        Self::ReadableEnum {
            extract: Arc::new(extract),
            embed: Arc::new(embed),
        }
    }

    #[inline]
    pub fn writable_enum(
        embed: impl Fn(Value) -> Root + Send + Sync + 'static,
        extract: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + Send + Sync + 'static,
        extract_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + Send + Sync + 'static,
    ) -> Self {
        Self::WritableEnum {
            extract: Arc::new(extract),
            embed: Arc::new(embed),
            extract_mut: Arc::new(extract_mut),
        }
    }


    // New Owned KeyPath constructors
    #[inline]
    pub fn owned(get: impl Fn(Root) -> Value + Send + Sync + 'static) -> Self {
        Self::Owned(Arc::new(get))
    }

    #[inline]
    pub fn failable_owned(get: impl Fn(Root) -> Option<Value> + Send + Sync + 'static) -> Self {
        Self::FailableOwned(Arc::new(get))
    }

    #[inline]
    pub fn owned_writable(get: impl Fn(Root) -> Value + Send + Sync + 'static) -> Self {
        Self::Owned(Arc::new(get))
    }
    
    #[inline]
    pub fn failable_owned_writable(get: impl Fn(Root) -> Option<Value> + Send + Sync + 'static) -> Self {
        Self::FailableOwned(Arc::new(get))
    }

    #[inline]
    pub fn reference_writable(get_mut: impl for<'a> Fn(&'a mut Root) -> &'a mut Value + Send + Sync + 'static) -> Self {
        Self::ReferenceWritable(Arc::new(get_mut))
    }

    /// Convert this keypath to a PartialKeyPath (type-erased Value)
    /// This allows storing keypaths with different Value types in the same collection
    pub fn to_partial(self) -> PartialKeyPath<Root>
    where
        Root: 'static,
        Value: 'static + Send + Sync,
    {
        match self {
            KeyPaths::Readable(f) => PartialKeyPath::Readable(Arc::new(move |root| f(root) as &(dyn Any + Send + Sync))),
            KeyPaths::Writable(f) => PartialKeyPath::Writable(Arc::new(move |root| f(root) as &mut (dyn Any + Send + Sync))),
            KeyPaths::FailableReadable(f) => PartialKeyPath::FailableReadable(Arc::new(move |root| f(root).map(|v| v as &(dyn Any + Send + Sync)))),
            KeyPaths::FailableWritable(f) => PartialKeyPath::FailableWritable(Arc::new(move |root| f(root).map(|v| v as &mut (dyn Any + Send + Sync)))),
            KeyPaths::ReadableEnum { extract, embed } => PartialKeyPath::ReadableEnum {
                extract: Arc::new(move |root| extract(root).map(|v| v as &(dyn Any + Send + Sync))),
                embed: Arc::new(move |value| embed(*value.downcast::<Value>().unwrap())),
            },
            KeyPaths::WritableEnum { extract, extract_mut, embed } => PartialKeyPath::WritableEnum {
                extract: Arc::new(move |root| extract(root).map(|v| v as &(dyn Any + Send + Sync))),
                extract_mut: Arc::new(move |root| extract_mut(root).map(|v| v as &mut (dyn Any + Send + Sync))),
                embed: Arc::new(move |value| embed(*value.downcast::<Value>().unwrap())),
            },
            KeyPaths::ReferenceWritable(f) => PartialKeyPath::ReferenceWritable(Arc::new(move |root| f(root) as &mut (dyn Any + Send + Sync))),
            KeyPaths::Owned(f) => PartialKeyPath::Owned(Arc::new(move |root| Box::new(f(root)) as Box<dyn Any>)),
            KeyPaths::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |root| f(root).map(|v| Box::new(v) as Box<dyn Any>))),
        }
    }

    /// Convert this keypath to an AnyKeyPath (fully type-erased)
    /// This allows storing keypaths with different Root and Value types in the same collection
    pub fn to_any(self) -> AnyKeyPath
    where
        Root: 'static + Send + Sync,
        Value: 'static + Send + Sync,
    {
        match self {
            KeyPaths::Readable(f) => AnyKeyPath::Readable(Arc::new(move |root| {
                let typed_root = root.downcast_ref::<Root>().unwrap();
                f(typed_root) as &(dyn Any + Send + Sync)
            })),
            KeyPaths::Writable(f) => AnyKeyPath::Writable(Arc::new(move |root| {
                let typed_root = root.downcast_mut::<Root>().unwrap();
                f(typed_root) as &mut (dyn Any + Send + Sync)
            })),
            KeyPaths::FailableReadable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let typed_root = root.downcast_ref::<Root>().unwrap();
                f(typed_root).map(|v| v as &(dyn Any + Send + Sync))
            })),
            KeyPaths::FailableWritable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let typed_root = root.downcast_mut::<Root>().unwrap();
                f(typed_root).map(|v| v as &mut (dyn Any + Send + Sync))
            })),
            KeyPaths::ReadableEnum { extract, embed } => AnyKeyPath::ReadableEnum {
                extract: Arc::new(move |root| {
                    let typed_root = root.downcast_ref::<Root>().unwrap();
                    extract(typed_root).map(|v| v as &(dyn Any + Send + Sync))
                }),
                embed: Arc::new(move |value| {
                    let typed_value = *value.downcast::<Value>().unwrap();
                    Box::new(embed(typed_value)) as Box<dyn Any>
                }),
            },
            KeyPaths::WritableEnum { extract, extract_mut, embed } => AnyKeyPath::WritableEnum {
                extract: Arc::new(move |root| {
                    let typed_root = root.downcast_ref::<Root>().unwrap();
                    extract(typed_root).map(|v| v as &(dyn Any + Send + Sync))
                }),
                extract_mut: Arc::new(move |root| {
                    let typed_root = root.downcast_mut::<Root>().unwrap();
                    extract_mut(typed_root).map(|v| v as &mut (dyn Any + Send + Sync))
                }),
                embed: Arc::new(move |value| {
                    let typed_value = *value.downcast::<Value>().unwrap();
                    Box::new(embed(typed_value)) as Box<dyn Any>
                }),
            },
            KeyPaths::ReferenceWritable(f) => AnyKeyPath::ReferenceWritable(Arc::new(move |root| {
                let typed_root = root.downcast_mut::<Root>().unwrap();
                f(typed_root) as &mut (dyn Any + Send + Sync)
            })),
            KeyPaths::Owned(f) => AnyKeyPath::Owned(Arc::new(move |root| {
                let typed_root = *root.downcast::<Root>().unwrap();
                Box::new(f(typed_root)) as Box<dyn Any>
            })),
            KeyPaths::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let typed_root = *root.downcast::<Root>().unwrap();
                f(typed_root).map(|v| Box::new(v) as Box<dyn Any>)
            })),
        }
    }

    /// Extract values from a slice of references using this keypath
    /// This is a convenience method for working with collections of references
    /// 
    /// Example:
    /// ```rust
    /// let name_path = Person::name_r();
    /// let people = vec![&person1, &person2, &person3];
    /// let names: Vec<&String> = name_path.extract_from_ref_slice(&people);
    /// ```
    #[inline]
    pub fn extract_from_ref_slice<'a>(&self, slice: &'a [&Root]) -> Vec<&'a Value>
    where
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Readable(f) => {
                slice.iter().map(|item| f(item)).collect()
            }
            KeyPaths::FailableReadable(f) => {
                slice.iter().filter_map(|item| f(item)).collect()
            }
            KeyPaths::ReadableEnum { extract, .. } => {
                slice.iter().filter_map(|item| extract(item)).collect()
            }
            _ => panic!("extract_from_ref_slice only works with readable keypaths"),
        }
    }

    /// Extract mutable values from a slice of mutable references using this keypath
    /// This is a convenience method for working with collections of mutable references
    /// 
    /// Example:
    /// ```rust
    /// let name_path = Person::name_w();
    /// let mut people = vec![&mut person1, &mut person2, &mut person3];
    /// let names: Vec<&mut String> = name_path.extract_mut_from_ref_slice(&mut people);
    /// ```
    #[inline]
    pub fn extract_mut_from_ref_slice<'a>(&self, slice: &'a mut [&mut Root]) -> Vec<&'a mut Value>
    where
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Writable(f) => {
                slice.iter_mut().map(|item| f(item)).collect()
            }
            KeyPaths::FailableWritable(f) => {
                slice.iter_mut().filter_map(|item| f(item)).collect()
            }
            KeyPaths::WritableEnum { extract_mut, .. } => {
                slice.iter_mut().filter_map(|item| extract_mut(item)).collect()
            }
            _ => panic!("extract_mut_from_ref_slice only works with writable keypaths"),
        }
    }
}

impl<Root, Value> KeyPaths<Root, Value> {
    /// Get an immutable reference if possible
    #[inline]
    pub fn get<'a>(&'a self, root: &'a Root) -> Option<&'a Value> {
        match self {
            KeyPaths::Readable(f) => Some(f(root)),
            KeyPaths::Writable(_) => None, // Writable requires mut
            KeyPaths::FailableReadable(f) => f(root),
            KeyPaths::FailableWritable(_) => None, // needs mut
            KeyPaths::ReadableEnum { extract, .. } => extract(root),
            KeyPaths::WritableEnum { extract, .. } => extract(root),
            KeyPaths::ReferenceWritable(_) => None, // ReferenceWritable requires mut
            // New owned keypath types (don't work with references)
            KeyPaths::Owned(_) => None, // Owned keypaths don't work with references
            KeyPaths::FailableOwned(_) => None, // Owned keypaths don't work with references
        }
    }

    /// Get an immutable reference when Root is itself a reference (&T)
    /// This enables using keypaths with collections of references like Vec<&T>
    #[inline]
    pub fn get_ref<'a, 'b>(&'a self, root: &'b &Root) -> Option<&'b Value> 
    where
        'a: 'b,
    {
        match self {
            KeyPaths::Readable(f) => Some(f(*root)),
            KeyPaths::Writable(_) => None, // Writable requires mut
            KeyPaths::FailableReadable(f) => f(*root),
            KeyPaths::FailableWritable(_) => None, // needs mut
            KeyPaths::ReadableEnum { extract, .. } => extract(*root),
            KeyPaths::WritableEnum { extract, .. } => extract(*root),
            KeyPaths::ReferenceWritable(_) => None, // ReferenceWritable requires mut
            // New owned keypath types (don't work with references)
            KeyPaths::Owned(_) => None, // Owned keypaths don't work with references
            KeyPaths::FailableOwned(_) => None, // Owned keypaths don't work with references
        }
    }

    /// Get a mutable reference if possible
    #[inline]
    pub fn get_mut<'a>(&'a self, root: &'a mut Root) -> Option<&'a mut Value> {
        match self {
            KeyPaths::Readable(_) => None, // immutable only
            KeyPaths::Writable(f) => Some(f(root)),
            KeyPaths::FailableReadable(_) => None, // immutable only
            KeyPaths::FailableWritable(f) => f(root),
            KeyPaths::ReadableEnum { .. } => None, // immutable only
            KeyPaths::WritableEnum { extract_mut, .. } => extract_mut(root),
            KeyPaths::ReferenceWritable(f) => Some(f(root)),
            // New owned keypath types (don't work with references)
            KeyPaths::Owned(_) => None, // Owned keypaths don't work with references
            KeyPaths::FailableOwned(_) => None, // Owned keypaths don't work with references
        }
    }

    /// Get a mutable reference when Root is itself a mutable reference (&mut T)
    /// This enables using writable keypaths with collections of mutable references
    #[inline]
    pub fn get_mut_ref<'a, 'b>(&'a self, root: &'b mut &mut Root) -> Option<&'b mut Value> 
    where
        'a: 'b,
    {
        match self {
            KeyPaths::Readable(_) => None, // immutable only
            KeyPaths::Writable(f) => Some(f(*root)),
            KeyPaths::FailableReadable(_) => None, // immutable only
            KeyPaths::FailableWritable(f) => f(*root),
            KeyPaths::ReadableEnum { .. } => None, // immutable only
            KeyPaths::WritableEnum { extract_mut, .. } => extract_mut(*root),
            KeyPaths::ReferenceWritable(f) => Some(f(*root)),
            // New owned keypath types (don't work with references)
            KeyPaths::Owned(_) => None, // Owned keypaths don't work with references
            KeyPaths::FailableOwned(_) => None, // Owned keypaths don't work with references
        }
    }

    // ===== Smart Pointer / Container Adapter Methods =====
    // These methods create new keypaths that work with wrapped types
    // Enables using KeyPaths<T, V> with Vec<Arc<T>>, Vec<Box<T>>, etc.

    /// Adapt this keypath to work with Arc<Root>
    /// Enables using KeyPaths<T, V> with collections like Vec<Arc<T>>
    #[inline]
    pub fn for_arc(self) -> KeyPaths<Arc<Root>, Value>
    where
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::Readable(Arc::new(move |root: &Arc<Root>| {
                f(&**root)
            })),
            KeyPaths::Writable(_) => {
                // Writable doesn't work with Arc (no mutable access)
                panic!("Cannot create writable keypath for Arc (Arc is immutable)")
            }
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableReadable(Arc::new(move |root: &Arc<Root>| f(&**root)))
            }
            KeyPaths::ReadableEnum { extract, embed } => KeyPaths::ReadableEnum {
                extract: Arc::new(move |root: &Arc<Root>| extract(&**root)),
                embed: Arc::new(move |value| Arc::new(embed(value))),
            },
            other => panic!("Unsupported keypath variant for Arc adapter: {:?}", kind_name(&other)),
        }
    }

    /// Helper function to extract values from a slice using this keypath
    /// This is useful when you have a slice &[T] and want to access fields via references
    /// 
    /// Example:
    /// ```rust
    /// let people = vec![Person { name: "Alice".to_string(), age: 30 }];
    /// let names: Vec<&String> = Person::name_r().extract_from_slice(&people);
    /// ```
    #[inline]
    pub fn extract_from_slice<'a>(&self, slice: &'a [Root]) -> Vec<&'a Value>
    where
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Readable(f) => {
                slice.iter().map(|item| f(item)).collect()
            }
            KeyPaths::FailableReadable(f) => {
                slice.iter().filter_map(|item| f(item)).collect()
            }
            KeyPaths::ReadableEnum { extract, .. } => {
                slice.iter().filter_map(|item| extract(item)).collect()
            }
            _ => panic!("extract_from_slice only works with readable keypaths"),
        }
    }

    /// Helper function to extract values from an iterator using this keypath
    /// This is useful when you have an iterator over &T and want to access fields
    /// 
    /// Example:
    /// ```rust
    /// let people = vec![Person { name: "Alice".to_string(), age: 30 }];
    /// let names: Vec<&String> = Person::name_r().extract_from_iter(people.iter());
    /// ```
    #[inline]
    pub fn extract_from_iter<'a, I>(&self, iter: I) -> Vec<&'a Value>
    where
        I: Iterator<Item = &'a Root>,
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Readable(f) => {
                iter.map(|item| f(item)).collect()
            }
            KeyPaths::FailableReadable(f) => {
                iter.filter_map(|item| f(item)).collect()
            }
            KeyPaths::ReadableEnum { extract, .. } => {
                iter.filter_map(|item| extract(item)).collect()
            }
            _ => panic!("extract_from_iter only works with readable keypaths"),
        }
    }

    /// Helper function to extract mutable values from a mutable slice using this keypath
    /// This is useful when you have a mutable slice &mut [T] and want to access fields via mutable references
    /// 
    /// Example:
    /// ```rust
    /// let mut people = vec![Person { name: "Alice".to_string(), age: 30 }];
    /// let names: Vec<&mut String> = Person::name_w().extract_mut_from_slice(&mut people);
    /// ```
    #[inline]
    pub fn extract_mut_from_slice<'a>(&self, slice: &'a mut [Root]) -> Vec<&'a mut Value>
    where
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Writable(f) => {
                slice.iter_mut().map(|item| f(item)).collect()
            }
            KeyPaths::FailableWritable(f) => {
                slice.iter_mut().filter_map(|item| f(item)).collect()
            }
            KeyPaths::WritableEnum { extract_mut, .. } => {
                slice.iter_mut().filter_map(|item| extract_mut(item)).collect()
            }
            _ => panic!("extract_mut_from_slice only works with writable keypaths"),
        }
    }

    /// Adapt this keypath to work with Box<Root>
    /// Enables using KeyPaths<T, V> with collections like Vec<Box<T>>
    #[inline]
    pub fn for_box(self) -> KeyPaths<Box<Root>, Value>
    where
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::Readable(Arc::new(move |root: &Box<Root>| {
                f(&**root)
            })),
            KeyPaths::Writable(f) => KeyPaths::Writable(Arc::new(move |root: &mut Box<Root>| {
                f(&mut **root)
            })),
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableReadable(Arc::new(move |root: &Box<Root>| f(&**root)))
            }
            KeyPaths::FailableWritable(f) => {
                KeyPaths::FailableWritable(Arc::new(move |root: &mut Box<Root>| f(&mut **root)))
            }
            KeyPaths::ReadableEnum { extract, embed } => KeyPaths::ReadableEnum {
                extract: Arc::new(move |root: &Box<Root>| extract(&**root)),
                embed: Arc::new(move |value| Box::new(embed(value))),
            },
            KeyPaths::WritableEnum { extract, extract_mut, embed } => KeyPaths::WritableEnum {
                extract: Arc::new(move |root: &Box<Root>| extract(&**root)),
                extract_mut: Arc::new(move |root: &mut Box<Root>| extract_mut(&mut **root)),
                embed: Arc::new(move |value| Box::new(embed(value))),
            },
            other => panic!("Unsupported keypath variant for Box adapter: {:?}", kind_name(&other)),
        }
    }

    /// Adapt this keypath to work with Rc<Root>
    /// Enables using KeyPaths<T, V> with collections like Vec<Rc<T>>
    #[inline]
    pub fn for_rc(self) -> KeyPaths<Rc<Root>, Value>
    where
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::Readable(Arc::new(move |root: &Rc<Root>| {
                f(&**root)
            })),
            KeyPaths::Writable(_) => {
                // Writable doesn't work with Rc (no mutable access)
                panic!("Cannot create writable keypath for Rc (Rc is immutable)")
            }
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableReadable(Arc::new(move |root: &Rc<Root>| f(&**root)))
            }
            KeyPaths::ReadableEnum { extract, embed } => KeyPaths::ReadableEnum {
                extract: Arc::new(move |root: &Rc<Root>| extract(&**root)),
                embed: Arc::new(move |value| Rc::new(embed(value))),
            },
            other => panic!("Unsupported keypath variant for Rc adapter: {:?}", kind_name(&other)),
        }
    }

    /// Adapt this keypath to work with Result<Root, E>
    /// Enables using KeyPaths<T, V> with Result types
    /// Note: This creates a FailableReadable keypath since Result can be Err
    #[inline]
    pub fn for_result<E>(self) -> KeyPaths<Result<Root, E>, Value>
    where
        Root: 'static,
        Value: 'static,
        E: 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::FailableReadable(Arc::new(move |root: &Result<Root, E>| {
                root.as_ref().ok().map(|r| f(r))
            })),
            KeyPaths::Writable(f) => KeyPaths::FailableWritable(Arc::new(move |root: &mut Result<Root, E>| {
                root.as_mut().ok().map(|r| f(r))
            })),
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableReadable(Arc::new(move |root: &Result<Root, E>| {
                    root.as_ref().ok().and_then(|r| f(r))
                }))
            }
            KeyPaths::FailableWritable(f) => {
                KeyPaths::FailableWritable(Arc::new(move |root: &mut Result<Root, E>| {
                    root.as_mut().ok().and_then(|r| f(r))
                }))
            }
            KeyPaths::ReadableEnum { extract, embed } => KeyPaths::ReadableEnum {
                extract: Arc::new(move |root: &Result<Root, E>| {
                    root.as_ref().ok().and_then(|r| extract(r))
                }),
                embed: Arc::new(move |value| Ok(embed(value))),
            },
            KeyPaths::WritableEnum { extract, extract_mut, embed } => KeyPaths::WritableEnum {
                extract: Arc::new(move |root: &Result<Root, E>| {
                    root.as_ref().ok().and_then(|r| extract(r))
                }),
                extract_mut: Arc::new(move |root: &mut Result<Root, E>| {
                    root.as_mut().ok().and_then(|r| extract_mut(r))
                }),
                embed: Arc::new(move |value| Ok(embed(value))),
            },
            other => panic!("Unsupported keypath variant for Result adapter: {:?}", kind_name(&other)),
        }
    }

    /// Adapt this keypath to work with Option<Root>
    /// Enables using KeyPaths<T, V> with Option types
    /// Note: This creates a FailableReadable/FailableWritable keypath since Option can be None
    #[inline]
    pub fn for_option(self) -> KeyPaths<Option<Root>, Value>
    where
        Root: 'static,
        Value: 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::FailableReadable(Arc::new(move |root: &Option<Root>| {
                root.as_ref().map(|r| f(r))
            })),
            KeyPaths::Writable(f) => KeyPaths::FailableWritable(Arc::new(move |root: &mut Option<Root>| {
                root.as_mut().map(|r| f(r))
            })),
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableReadable(Arc::new(move |root: &Option<Root>| {
                    root.as_ref().and_then(|r| f(r))
                }))
            }
            KeyPaths::FailableWritable(f) => {
                KeyPaths::FailableWritable(Arc::new(move |root: &mut Option<Root>| {
                    root.as_mut().and_then(|r| f(r))
                }))
            }
            KeyPaths::ReadableEnum { extract, embed } => KeyPaths::ReadableEnum {
                extract: Arc::new(move |root: &Option<Root>| {
                    root.as_ref().and_then(|r| extract(r))
                }),
                embed: Arc::new(move |value| Some(embed(value))),
            },
            KeyPaths::WritableEnum { extract, extract_mut, embed } => KeyPaths::WritableEnum {
                extract: Arc::new(move |root: &Option<Root>| {
                    root.as_ref().and_then(|r| extract(r))
                }),
                extract_mut: Arc::new(move |root: &mut Option<Root>| {
                    root.as_mut().and_then(|r| extract_mut(r))
                }),
                embed: Arc::new(move |value| Some(embed(value))),
            },
            other => panic!("Unsupported keypath variant for Option adapter: {:?}", kind_name(&other)),
        }
    }

    /// Adapt this keypath to work with Arc<RwLock<Root>>
    /// Enables using KeyPaths<T, V> with Arc<RwLock<T>> containers
    /// Note: This creates a FailableOwned keypath since RwLock access can fail and we need to clone values
    #[inline]
    pub fn for_arc_rwlock(self) -> KeyPaths<Arc<RwLock<Root>>, Value>
    where
        Root: 'static,
        Value: Clone + 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::FailableOwned(Arc::new(move |root: Arc<RwLock<Root>>| {
                let guard = root.read().ok()?;
                Some(f(&*guard).clone())
            })),
            KeyPaths::Writable(_) => {
                // Writable doesn't work with Arc<RwLock> (Arc is immutable, need write guard)
                panic!("Cannot create writable keypath for Arc<RwLock> (use with_arc_rwlock_mut instead)")
            }
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableOwned(Arc::new(move |root: Arc<RwLock<Root>>| {
                    let guard = root.read().ok()?;
                    f(&*guard).map(|v| v.clone())
                }))
            }
            KeyPaths::ReadableEnum { extract, embed: _ } => KeyPaths::FailableOwned(Arc::new(move |root: Arc<RwLock<Root>>| {
                let guard = root.read().ok()?;
                extract(&*guard).map(|v| v.clone())
            })),
            other => panic!("Unsupported keypath variant for Arc<RwLock> adapter: {:?}", kind_name(&other)),
        }
    }

    /// Adapt this keypath to work with Arc<Mutex<Root>>
    /// Enables using KeyPaths<T, V> with Arc<Mutex<T>> containers
    /// Note: This creates a FailableOwned keypath since Mutex access can fail and we need to clone values
    #[inline]
    pub fn for_arc_mutex(self) -> KeyPaths<Arc<Mutex<Root>>, Value>
    where
        Root: 'static,
        Value: Clone + 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::FailableOwned(Arc::new(move |root: Arc<Mutex<Root>>| {
                let guard = root.lock().ok()?;
                Some(f(&*guard).clone())
            })),
            KeyPaths::Writable(_) => {
                // Writable doesn't work with Arc<Mutex> (Arc is immutable, need write guard)
                panic!("Cannot create writable keypath for Arc<Mutex> (use with_arc_mutex_mut instead)")
            }
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableOwned(Arc::new(move |root: Arc<Mutex<Root>>| {
                    let guard = root.lock().ok()?;
                    f(&*guard).map(|v| v.clone())
                }))
            }
            KeyPaths::ReadableEnum { extract, embed: _ } => KeyPaths::FailableOwned(Arc::new(move |root: Arc<Mutex<Root>>| {
                let guard = root.lock().ok()?;
                extract(&*guard).map(|v| v.clone())
            })),
            other => panic!("Unsupported keypath variant for Arc<Mutex> adapter: {:?}", kind_name(&other)),
        }
    }

    /// Adapt this keypath to work with Arc<parking_lot::Mutex<Root>>
    /// Enables using KeyPaths<T, V> with Arc<parking_lot::Mutex<T>> containers
    /// Note: This creates a FailableOwned keypath since Mutex access can fail and we need to clone values
    /// Requires the "parking_lot" feature to be enabled
    #[cfg(feature = "parking_lot")]
    #[inline]
    pub fn for_arc_parking_mutex(self) -> KeyPaths<Arc<parking_lot::Mutex<Root>>, Value>
    where
        Root: 'static,
        Value: Clone + 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::FailableOwned(Arc::new(move |root: Arc<parking_lot::Mutex<Root>>| {
                let guard = root.lock();
                Some(f(&*guard).clone())
            })),
            KeyPaths::Writable(_) => {
                // Writable doesn't work with Arc<parking_lot::Mutex> (Arc is immutable, need write guard)
                panic!("Cannot create writable keypath for Arc<parking_lot::Mutex> (use with_arc_parking_mutex_mut instead)")
            }
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableOwned(Arc::new(move |root: Arc<parking_lot::Mutex<Root>>| {
                    let guard = root.lock();
                    f(&*guard).map(|v| v.clone())
                }))
            }
            KeyPaths::ReadableEnum { extract, embed: _ } => KeyPaths::FailableOwned(Arc::new(move |root: Arc<parking_lot::Mutex<Root>>| {
                let guard = root.lock();
                extract(&*guard).map(|v| v.clone())
            })),
            other => panic!("Unsupported keypath variant for Arc<parking_lot::Mutex> adapter: {:?}", kind_name(&other)),
        }
    }

    /// Adapt this keypath to work with Arc<parking_lot::RwLock<Root>>
    /// Enables using KeyPaths<T, V> with Arc<parking_lot::RwLock<T>> containers
    /// Note: This creates a FailableOwned keypath since RwLock access can fail and we need to clone values
    /// Requires the "parking_lot" feature to be enabled
    #[cfg(feature = "parking_lot")]
    #[inline]
    pub fn for_arc_parking_rwlock(self) -> KeyPaths<Arc<parking_lot::RwLock<Root>>, Value>
    where
        Root: 'static,
        Value: Clone + 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::FailableOwned(Arc::new(move |root: Arc<parking_lot::RwLock<Root>>| {
                let guard = root.read();
                Some(f(&*guard).clone())
            })),
            KeyPaths::Writable(_) => {
                // Writable doesn't work with Arc<parking_lot::RwLock> (Arc is immutable, need write guard)
                panic!("Cannot create writable keypath for Arc<parking_lot::RwLock> (use with_arc_parking_rwlock_mut instead)")
            }
            KeyPaths::FailableReadable(f) => {
                KeyPaths::FailableOwned(Arc::new(move |root: Arc<parking_lot::RwLock<Root>>| {
                    let guard = root.read();
                    f(&*guard).map(|v| v.clone())
                }))
            }
            KeyPaths::ReadableEnum { extract, embed: _ } => KeyPaths::FailableOwned(Arc::new(move |root: Arc<parking_lot::RwLock<Root>>| {
                let guard = root.read();
                extract(&*guard).map(|v| v.clone())
            })),
            other => panic!("Unsupported keypath variant for Arc<parking_lot::RwLock> adapter: {:?}", kind_name(&other)),
        }
    }

    /// Adapt a keypath to work with Tagged<Tag, Root>
    /// Returns a new KeyPaths instance that can work with Tagged values
    /// Note: Tagged<T, Tag> where T is the actual value and Tag is a zero-sized phantom type
    /// Tagged only implements Deref, not DerefMut, so writable keypaths are not supported
    #[cfg(feature = "tagged_core")]
    #[inline]
    pub fn for_tagged<Tag>(self) -> KeyPaths<Tagged<Root, Tag>, Value>
    where
        Root: Clone + 'static,
        Value: 'static,
        Tag: 'static,
    {
        match self {
            KeyPaths::Readable(f) => KeyPaths::Readable(Arc::new(move |root: &Tagged<Root, Tag>| {
                f(&**root)
            })),
            KeyPaths::Writable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            KeyPaths::FailableReadable(f) => KeyPaths::FailableReadable(Arc::new(move |root: &Tagged<Root, Tag>| {
                f(&**root)
            })),
            KeyPaths::FailableWritable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            KeyPaths::ReadableEnum { extract, embed } => KeyPaths::ReadableEnum {
                extract: Arc::new(move |root: &Tagged<Root, Tag>| {
                    extract(&**root)
                }),
                embed: Arc::new(move |value: Value| {
                    Tagged::new(embed(value))
                }),
            },
            KeyPaths::WritableEnum { .. } => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            KeyPaths::ReferenceWritable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            KeyPaths::Owned(f) => KeyPaths::Owned(Arc::new(move |root: Tagged<Root, Tag>| {
                // Tagged consumes itself and returns the inner value by cloning
                f((*root).clone())
            })),
            KeyPaths::FailableOwned(f) => KeyPaths::FailableOwned(Arc::new(move |root: Tagged<Root, Tag>| {
                f((*root).clone())
            })),
        }
    }

    // ===== WithContainer Trait Implementation =====
    // All with_* methods are now implemented via the WithContainer trait

    pub fn embed(&self, value: Value) -> Option<Root>
    where
        Value: Clone,
    {
        match self {
            KeyPaths::ReadableEnum { embed, .. } => Some(embed(value)),
            _ => None,
        }
    }

    pub fn embed_mut(&self, value: Value) -> Option<Root>
    where
        Value: Clone,
    {
        match self {
            KeyPaths::WritableEnum { embed, .. } => Some(embed(value)),
            _ => None,
        }
    }


    // ===== Owned KeyPath Accessor Methods =====

    /// Get an owned value (primary method for owned keypaths)
    #[inline]
    pub fn get_owned(self, root: Root) -> Value {
        match self {
            KeyPaths::Owned(f) => f(root),
            _ => panic!("get_owned only works with owned keypaths"),
        }
    }

    /// Get an owned value with failable access
    #[inline]
    pub fn get_failable_owned(self, root: Root) -> Option<Value> {
        match self {
            KeyPaths::FailableOwned(f) => f(root),
            _ => panic!("get_failable_owned only works with failable owned keypaths"),
        }
    }

    /// Iter over immutable references if `Value: IntoIterator`
    pub fn iter<'a, T>(&'a self, root: &'a Root) -> Option<<&'a Value as IntoIterator>::IntoIter>
    where
        &'a Value: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        self.get(root).map(|v| v.into_iter())
    }

    /// Iter over mutable references if `&mut Value: IntoIterator`
    pub fn iter_mut<'a, T>(
        &'a self,
        root: &'a mut Root,
    ) -> Option<<&'a mut Value as IntoIterator>::IntoIter>
    where
        &'a mut Value: IntoIterator<Item = &'a mut T>,
        T: 'a,
    {
        self.get_mut(root).map(|v| v.into_iter())
    }

    /// Consume root and iterate if `Value: IntoIterator`
    #[inline]
    pub fn into_iter<T>(self, root: Root) -> Option<<Value as IntoIterator>::IntoIter>
    where
        Value: IntoIterator<Item = T> + Clone,
    {
        match self {
            KeyPaths::Readable(f) => Some(f(&root).clone().into_iter()), // requires Clone
            KeyPaths::Writable(_) => None,
            KeyPaths::FailableReadable(f) => f(&root).map(|v| v.clone().into_iter()),
            KeyPaths::FailableWritable(_) => None,
            KeyPaths::ReadableEnum { extract, .. } => extract(&root).map(|v| v.clone().into_iter()),
            KeyPaths::WritableEnum { extract, .. } => extract(&root).map(|v| v.clone().into_iter()),
            KeyPaths::ReferenceWritable(_) => None, // ReferenceWritable doesn't work with owned iteration
            // New owned keypath types
            KeyPaths::Owned(f) => Some(f(root).into_iter()),
            KeyPaths::FailableOwned(f) => f(root).map(|v| v.into_iter()),
        }
    }
}

// ===== PartialKeyPath Implementation =====
impl<Root> PartialKeyPath<Root> {
    /// Get an immutable reference if possible
    #[inline]
    pub fn get<'a>(&'a self, root: &'a Root) -> Option<&'a (dyn Any + Send + Sync)> {
        match self {
            PartialKeyPath::Readable(f) => Some(f(root)),
            PartialKeyPath::Writable(_) => None, // Writable requires mut
            PartialKeyPath::FailableReadable(f) => f(root),
            PartialKeyPath::FailableWritable(_) => None, // needs mut
            PartialKeyPath::ReadableEnum { extract, .. } => extract(root),
            PartialKeyPath::WritableEnum { extract, .. } => extract(root),
            PartialKeyPath::ReferenceWritable(_) => None, // ReferenceWritable requires mut
            PartialKeyPath::Owned(_) => None, // Owned keypaths don't work with references
            PartialKeyPath::FailableOwned(_) => None, // Owned keypaths don't work with references
        }
    }

    /// Get a mutable reference if possible
    #[inline]
    pub fn get_mut<'a>(&'a self, root: &'a mut Root) -> Option<&'a mut (dyn Any + Send + Sync)> {
        match self {
            PartialKeyPath::Readable(_) => None, // immutable only
            PartialKeyPath::Writable(f) => Some(f(root)),
            PartialKeyPath::FailableReadable(_) => None, // immutable only
            PartialKeyPath::FailableWritable(f) => f(root),
            PartialKeyPath::ReadableEnum { .. } => None, // immutable only
            PartialKeyPath::WritableEnum { extract_mut, .. } => extract_mut(root),
            PartialKeyPath::ReferenceWritable(f) => Some(f(root)),
            PartialKeyPath::Owned(_) => None, // Owned keypaths don't work with references
            PartialKeyPath::FailableOwned(_) => None, // Owned keypaths don't work with references
        }
    }

    /// Get an owned value (primary method for owned keypaths)
    #[inline]
    pub fn get_owned(self, root: Root) -> Box<dyn Any> {
        match self {
            PartialKeyPath::Owned(f) => f(root),
            _ => panic!("get_owned only works with owned keypaths"),
        }
    }

    /// Get an owned value with failable access
    #[inline]
    pub fn get_failable_owned(self, root: Root) -> Option<Box<dyn Any>> {
        match self {
            PartialKeyPath::FailableOwned(f) => f(root),
            _ => panic!("get_failable_owned only works with failable owned keypaths"),
        }
    }

    /// Convert this PartialKeyPath to an AnyKeyPath (fully type-erased)
    pub fn to_any(self) -> AnyKeyPath
    where
        Root: 'static,
    {
        match self {
            PartialKeyPath::Readable(f) => AnyKeyPath::Readable(Arc::new(move |root| {
                let typed_root = root.downcast_ref::<Root>().unwrap();
                f(typed_root)
            })),
            PartialKeyPath::Writable(f) => AnyKeyPath::Writable(Arc::new(move |root| {
                let typed_root = root.downcast_mut::<Root>().unwrap();
                f(typed_root)
            })),
            PartialKeyPath::FailableReadable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let typed_root = root.downcast_ref::<Root>().unwrap();
                f(typed_root)
            })),
            PartialKeyPath::FailableWritable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let typed_root = root.downcast_mut::<Root>().unwrap();
                f(typed_root)
            })),
            PartialKeyPath::ReadableEnum { extract, embed } => AnyKeyPath::ReadableEnum {
                extract: Arc::new(move |root| {
                    let typed_root = root.downcast_ref::<Root>().unwrap();
                    extract(typed_root)
                }),
                embed: Arc::new(move |value| {
                    let typed_value = *value.downcast::<Root>().unwrap();
                    Box::new(embed(Box::new(typed_value))) as Box<dyn Any>
                }),
            },
            PartialKeyPath::WritableEnum { extract, extract_mut, embed } => AnyKeyPath::WritableEnum {
                extract: Arc::new(move |root| {
                    let typed_root = root.downcast_ref::<Root>().unwrap();
                    extract(typed_root)
                }),
                extract_mut: Arc::new(move |root| {
                    let typed_root = root.downcast_mut::<Root>().unwrap();
                    extract_mut(typed_root)
                }),
                embed: Arc::new(move |value| {
                    let typed_value = *value.downcast::<Root>().unwrap();
                    Box::new(embed(Box::new(typed_value))) as Box<dyn Any>
                }),
            },
            PartialKeyPath::ReferenceWritable(f) => AnyKeyPath::ReferenceWritable(Arc::new(move |root| {
                let typed_root = root.downcast_mut::<Root>().unwrap();
                f(typed_root)
            })),
            PartialKeyPath::Owned(f) => AnyKeyPath::Owned(Arc::new(move |root| {
                let typed_root = *root.downcast::<Root>().unwrap();
                f(typed_root)
            })),
            PartialKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let typed_root = *root.downcast::<Root>().unwrap();
                f(typed_root)
            })),
        }
    }

    /// Get the kind name of this keypath
    #[inline]
    pub fn kind_name(&self) -> &'static str {
        match self {
            PartialKeyPath::Readable(_) => "PartialKeyPath::Readable",
            PartialKeyPath::Writable(_) => "PartialKeyPath::Writable",
            PartialKeyPath::FailableReadable(_) => "PartialKeyPath::FailableReadable",
            PartialKeyPath::FailableWritable(_) => "PartialKeyPath::FailableWritable",
            PartialKeyPath::ReadableEnum { .. } => "PartialKeyPath::ReadableEnum",
            PartialKeyPath::WritableEnum { .. } => "PartialKeyPath::WritableEnum",
            PartialKeyPath::ReferenceWritable(_) => "PartialKeyPath::ReferenceWritable",
            PartialKeyPath::Owned(_) => "PartialKeyPath::Owned",
            PartialKeyPath::FailableOwned(_) => "PartialKeyPath::FailableOwned",
        }
    }

    // ===== Aggregator Functions for PartialKeyPath =====

    /// Adapt this PartialKeyPath to work with Arc<Root>
    pub fn for_arc(self) -> PartialKeyPath<Arc<Root>>
    where
        Root: 'static + Clone,
    {
        match self {
            PartialKeyPath::Readable(f) => PartialKeyPath::Readable(Arc::new(move |arc: &Arc<Root>| f(&**arc))),
            PartialKeyPath::Writable(_) => {
                panic!("Arc does not support writable keypaths (Arc only implements Deref, not DerefMut)")
            }
            PartialKeyPath::FailableReadable(f) => PartialKeyPath::FailableReadable(Arc::new(move |arc: &Arc<Root>| f(&**arc))),
            PartialKeyPath::FailableWritable(_) => {
                panic!("Arc does not support writable keypaths (Arc only implements Deref, not DerefMut)")
            }
            PartialKeyPath::ReadableEnum { extract, embed } => PartialKeyPath::ReadableEnum {
                extract: Arc::new(move |arc: &Arc<Root>| extract(&**arc)),
                embed: Arc::new(move |value| Arc::new(embed(value))),
            },
            PartialKeyPath::WritableEnum { .. } => {
                panic!("Arc does not support writable keypaths (Arc only implements Deref, not DerefMut)")
            }
            PartialKeyPath::ReferenceWritable(_) => {
                panic!("Arc does not support writable keypaths (Arc only implements Deref, not DerefMut)")
            }
            PartialKeyPath::Owned(f) => PartialKeyPath::Owned(Arc::new(move |arc: Arc<Root>| f((*arc).clone()))),
            PartialKeyPath::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |arc: Arc<Root>| f((*arc).clone()))),
        }
    }

    /// Adapt this PartialKeyPath to work with Box<Root>
    pub fn for_box(self) -> PartialKeyPath<Box<Root>>
    where
        Root: 'static,
    {
        match self {
            PartialKeyPath::Readable(f) => PartialKeyPath::Readable(Arc::new(move |boxed: &Box<Root>| f(&**boxed))),
            PartialKeyPath::Writable(f) => PartialKeyPath::Writable(Arc::new(move |boxed: &mut Box<Root>| f(&mut **boxed))),
            PartialKeyPath::FailableReadable(f) => PartialKeyPath::FailableReadable(Arc::new(move |boxed: &Box<Root>| f(&**boxed))),
            PartialKeyPath::FailableWritable(f) => PartialKeyPath::FailableWritable(Arc::new(move |boxed: &mut Box<Root>| f(&mut **boxed))),
            PartialKeyPath::ReadableEnum { extract, embed } => PartialKeyPath::ReadableEnum {
                extract: Arc::new(move |boxed: &Box<Root>| extract(&**boxed)),
                embed: Arc::new(move |value| Box::new(embed(value))),
            },
            PartialKeyPath::WritableEnum { extract, extract_mut, embed } => PartialKeyPath::WritableEnum {
                extract: Arc::new(move |boxed: &Box<Root>| extract(&**boxed)),
                extract_mut: Arc::new(move |boxed: &mut Box<Root>| extract_mut(&mut **boxed)),
                embed: Arc::new(move |value| Box::new(embed(value))),
            },
            PartialKeyPath::ReferenceWritable(f) => PartialKeyPath::ReferenceWritable(Arc::new(move |boxed: &mut Box<Root>| f(&mut **boxed))),
            PartialKeyPath::Owned(f) => PartialKeyPath::Owned(Arc::new(move |boxed: Box<Root>| f(*boxed))),
            PartialKeyPath::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |boxed: Box<Root>| f(*boxed))),
        }
    }

    /// Adapt this PartialKeyPath to work with Rc<Root>
    pub fn for_rc(self) -> PartialKeyPath<Rc<Root>>
    where
        Root: 'static + Clone,
    {
        match self {
            PartialKeyPath::Readable(f) => PartialKeyPath::Readable(Arc::new(move |rc: &Rc<Root>| f(&**rc))),
            PartialKeyPath::Writable(_) => {
                panic!("Rc does not support writable keypaths (Rc only implements Deref, not DerefMut)")
            }
            PartialKeyPath::FailableReadable(f) => PartialKeyPath::FailableReadable(Arc::new(move |rc: &Rc<Root>| f(&**rc))),
            PartialKeyPath::FailableWritable(_) => {
                panic!("Rc does not support writable keypaths (Rc only implements Deref, not DerefMut)")
            }
            PartialKeyPath::ReadableEnum { extract, embed } => PartialKeyPath::ReadableEnum {
                extract: Arc::new(move |rc: &Rc<Root>| extract(&**rc)),
                embed: Arc::new(move |value| Rc::new(embed(value))),
            },
            PartialKeyPath::WritableEnum { .. } => {
                panic!("Rc does not support writable keypaths (Rc only implements Deref, not DerefMut)")
            }
            PartialKeyPath::ReferenceWritable(_) => {
                panic!("Rc does not support writable keypaths (Rc only implements Deref, not DerefMut)")
            }
            PartialKeyPath::Owned(f) => PartialKeyPath::Owned(Arc::new(move |rc: Rc<Root>| f((*rc).clone()))),
            PartialKeyPath::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |rc: Rc<Root>| f((*rc).clone()))),
        }
    }

    /// Adapt this PartialKeyPath to work with Result<Root, E>
    pub fn for_result<E>(self) -> PartialKeyPath<Result<Root, E>>
    where
        Root: 'static,
    {
        match self {
            PartialKeyPath::Readable(f) => PartialKeyPath::FailableReadable(Arc::new(move |result: &Result<Root, E>| {
                result.as_ref().ok().map(|root| f(root) as &(dyn Any + Send + Sync))
            })),
            PartialKeyPath::Writable(f) => PartialKeyPath::FailableWritable(Arc::new(move |result: &mut Result<Root, E>| {
                result.as_mut().ok().map(|root| f(root) as &mut (dyn Any + Send + Sync))
            })),
            PartialKeyPath::FailableReadable(f) => PartialKeyPath::FailableReadable(Arc::new(move |result: &Result<Root, E>| {
                result.as_ref().ok().and_then(|root| f(root))
            })),
            PartialKeyPath::FailableWritable(f) => PartialKeyPath::FailableWritable(Arc::new(move |result: &mut Result<Root, E>| {
                result.as_mut().ok().and_then(|root| f(root))
            })),
            PartialKeyPath::ReadableEnum { extract, embed } => PartialKeyPath::ReadableEnum {
                extract: Arc::new(move |result: &Result<Root, E>| {
                    result.as_ref().ok().and_then(|root| extract(root))
                }),
                embed: Arc::new(move |value| Ok(embed(value))),
            },
            PartialKeyPath::WritableEnum { extract, extract_mut, embed } => PartialKeyPath::WritableEnum {
                extract: Arc::new(move |result: &Result<Root, E>| {
                    result.as_ref().ok().and_then(|root| extract(root))
                }),
                extract_mut: Arc::new(move |result: &mut Result<Root, E>| {
                    result.as_mut().ok().and_then(|root| extract_mut(root))
                }),
                embed: Arc::new(move |value| Ok(embed(value))),
            },
            PartialKeyPath::ReferenceWritable(f) => PartialKeyPath::FailableWritable(Arc::new(move |result: &mut Result<Root, E>| {
                result.as_mut().ok().map(|root| f(root) as &mut (dyn Any + Send + Sync))
            })),
            PartialKeyPath::Owned(f) => PartialKeyPath::FailableOwned(Arc::new(move |result: Result<Root, E>| {
                result.ok().map(|root| f(root))
            })),
            PartialKeyPath::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |result: Result<Root, E>| {
                result.ok().and_then(|root| f(root))
            })),
        }
    }

    /// Adapt this PartialKeyPath to work with Option<Root>
    pub fn for_option(self) -> PartialKeyPath<Option<Root>>
    where
        Root: 'static,
    {
        match self {
            PartialKeyPath::Readable(f) => PartialKeyPath::FailableReadable(Arc::new(move |option: &Option<Root>| {
                option.as_ref().map(|root| f(root) as &(dyn Any + Send + Sync))
            })),
            PartialKeyPath::Writable(f) => PartialKeyPath::FailableWritable(Arc::new(move |option: &mut Option<Root>| {
                option.as_mut().map(|root| f(root) as &mut (dyn Any + Send + Sync))
            })),
            PartialKeyPath::FailableReadable(f) => PartialKeyPath::FailableReadable(Arc::new(move |option: &Option<Root>| {
                option.as_ref().and_then(|root| f(root))
            })),
            PartialKeyPath::FailableWritable(f) => PartialKeyPath::FailableWritable(Arc::new(move |option: &mut Option<Root>| {
                option.as_mut().and_then(|root| f(root))
            })),
            PartialKeyPath::ReadableEnum { extract, embed } => PartialKeyPath::ReadableEnum {
                extract: Arc::new(move |option: &Option<Root>| {
                    option.as_ref().and_then(|root| extract(root))
                }),
                embed: Arc::new(move |value| Some(embed(value))),
            },
            PartialKeyPath::WritableEnum { extract, extract_mut, embed } => PartialKeyPath::WritableEnum {
                extract: Arc::new(move |option: &Option<Root>| {
                    option.as_ref().and_then(|root| extract(root))
                }),
                extract_mut: Arc::new(move |option: &mut Option<Root>| {
                    option.as_mut().and_then(|root| extract_mut(root))
                }),
                embed: Arc::new(move |value| Some(embed(value))),
            },
            PartialKeyPath::ReferenceWritable(f) => PartialKeyPath::FailableWritable(Arc::new(move |option: &mut Option<Root>| {
                option.as_mut().map(|root| f(root) as &mut (dyn Any + Send + Sync))
            })),
            PartialKeyPath::Owned(f) => PartialKeyPath::FailableOwned(Arc::new(move |option: Option<Root>| {
                option.map(|root| f(root))
            })),
            PartialKeyPath::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |option: Option<Root>| {
                option.and_then(|root| f(root))
            })),
        }
    }

    /// Adapt this PartialKeyPath to work with Arc<RwLock<Root>>
    /// Note: This only supports owned keypaths due to guard lifetime constraints
    pub fn for_arc_rwlock(self) -> PartialKeyPath<Arc<RwLock<Root>>>
    where
        Root: 'static + Clone,
    {
        match self {
            PartialKeyPath::Readable(_) => {
                panic!("Arc<RwLock> does not support readable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::Writable(_) => {
                panic!("Arc<RwLock> does not support writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::FailableReadable(_) => {
                panic!("Arc<RwLock> does not support failable readable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::FailableWritable(_) => {
                panic!("Arc<RwLock> does not support failable writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::ReadableEnum { .. } => {
                panic!("Arc<RwLock> does not support readable enum keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::WritableEnum { .. } => {
                panic!("Arc<RwLock> does not support writable enum keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::ReferenceWritable(_) => {
                panic!("Arc<RwLock> does not support reference writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::Owned(f) => PartialKeyPath::Owned(Arc::new(move |arc_rwlock: Arc<RwLock<Root>>| {
                let guard = arc_rwlock.read().unwrap();
                let value = f((*guard).clone());
                drop(guard); // Ensure guard is dropped before returning
                value
            })),
            PartialKeyPath::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |arc_rwlock: Arc<RwLock<Root>>| {
                let guard = arc_rwlock.read().unwrap();
                let value = f((*guard).clone());
                drop(guard); // Ensure guard is dropped before returning
                value
            })),
        }
    }

    /// Adapt this PartialKeyPath to work with Arc<Mutex<Root>>
    /// Note: This only supports owned keypaths due to guard lifetime constraints
    pub fn for_arc_mutex(self) -> PartialKeyPath<Arc<Mutex<Root>>>
    where
        Root: 'static + Clone,
    {
        match self {
            PartialKeyPath::Readable(_) => {
                panic!("Arc<Mutex> does not support readable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::Writable(_) => {
                panic!("Arc<Mutex> does not support writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::FailableReadable(_) => {
                panic!("Arc<Mutex> does not support failable readable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::FailableWritable(_) => {
                panic!("Arc<Mutex> does not support failable writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::ReadableEnum { .. } => {
                panic!("Arc<Mutex> does not support readable enum keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::WritableEnum { .. } => {
                panic!("Arc<Mutex> does not support writable enum keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::ReferenceWritable(_) => {
                panic!("Arc<Mutex> does not support reference writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            PartialKeyPath::Owned(f) => PartialKeyPath::Owned(Arc::new(move |arc_mutex: Arc<Mutex<Root>>| {
                let guard = arc_mutex.lock().unwrap();
                let value = f((*guard).clone());
                drop(guard); // Ensure guard is dropped before returning
                value
            })),
            PartialKeyPath::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |arc_mutex: Arc<Mutex<Root>>| {
                let guard = arc_mutex.lock().unwrap();
                let value = f((*guard).clone());
                drop(guard); // Ensure guard is dropped before returning
                value
            })),
        }
    }

    /// Adapt this PartialKeyPath to work with Tagged<Root, Tag>
    pub fn for_tagged<Tag>(self) -> PartialKeyPath<Tagged<Root, Tag>>
    where
        Root: Clone + 'static,
    {
        match self {
            PartialKeyPath::Readable(f) => PartialKeyPath::Readable(Arc::new(move |tagged: &Tagged<Root, Tag>| {
                f(&*tagged) as &(dyn Any + Send + Sync)
            })),
            PartialKeyPath::Writable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            PartialKeyPath::FailableReadable(f) => PartialKeyPath::FailableReadable(Arc::new(move |tagged: &Tagged<Root, Tag>| {
                f(&*tagged)
            })),
            PartialKeyPath::FailableWritable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            PartialKeyPath::ReadableEnum { extract, embed } => PartialKeyPath::ReadableEnum {
                extract: Arc::new(move |tagged: &Tagged<Root, Tag>| {
                    extract(&*tagged)
                }),
                embed: Arc::new(move |value| embed(value).into()),
            },
            PartialKeyPath::WritableEnum { .. } => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            PartialKeyPath::ReferenceWritable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            PartialKeyPath::Owned(f) => PartialKeyPath::Owned(Arc::new(move |tagged: Tagged<Root, Tag>| {
                f((*tagged).clone())
            })),
            PartialKeyPath::FailableOwned(f) => PartialKeyPath::FailableOwned(Arc::new(move |tagged: Tagged<Root, Tag>| {
                f((*tagged).clone())
            })),
        }
    }
}

// ===== AnyKeyPath Implementation =====
impl AnyKeyPath {
    /// Get an immutable reference if possible
    #[inline]
    pub fn get<'a>(&'a self, root: &'a (dyn Any + Send + Sync)) -> Option<&'a (dyn Any + Send + Sync)> {
        match self {
            AnyKeyPath::Readable(f) => Some(f(root)),
            AnyKeyPath::Writable(_) => None, // Writable requires mut
            AnyKeyPath::FailableReadable(f) => f(root),
            AnyKeyPath::FailableWritable(_) => None, // needs mut
            AnyKeyPath::ReadableEnum { extract, .. } => extract(root),
            AnyKeyPath::WritableEnum { extract, .. } => extract(root),
            AnyKeyPath::ReferenceWritable(_) => None, // ReferenceWritable requires mut
            AnyKeyPath::Owned(_) => None, // Owned keypaths don't work with references
            AnyKeyPath::FailableOwned(_) => None, // Owned keypaths don't work with references
        }
    }

    /// Get a mutable reference if possible
    #[inline]
    pub fn get_mut<'a>(&'a self, root: &'a mut (dyn Any + Send + Sync)) -> Option<&'a mut (dyn Any + Send + Sync)> {
        match self {
            AnyKeyPath::Readable(_) => None, // immutable only
            AnyKeyPath::Writable(f) => Some(f(root)),
            AnyKeyPath::FailableReadable(_) => None, // immutable only
            AnyKeyPath::FailableWritable(f) => f(root),
            AnyKeyPath::ReadableEnum { .. } => None, // immutable only
            AnyKeyPath::WritableEnum { extract_mut, .. } => extract_mut(root),
            AnyKeyPath::ReferenceWritable(f) => Some(f(root)),
            AnyKeyPath::Owned(_) => None, // Owned keypaths don't work with references
            AnyKeyPath::FailableOwned(_) => None, // Owned keypaths don't work with references
        }
    }

    /// Get an owned value (primary method for owned keypaths)
    #[inline]
    pub fn get_owned(self, root: Box<dyn Any>) -> Box<dyn Any> {
        match self {
            AnyKeyPath::Owned(f) => f(root),
            _ => panic!("get_owned only works with owned keypaths"),
        }
    }

    /// Get an owned value with failable access
    #[inline]
    pub fn get_failable_owned(self, root: Box<dyn Any>) -> Option<Box<dyn Any>> {
        match self {
            AnyKeyPath::FailableOwned(f) => f(root),
            _ => panic!("get_failable_owned only works with failable owned keypaths"),
        }
    }

    /// Get the kind name of this keypath
    #[inline]
    pub fn kind_name(&self) -> &'static str {
        match self {
            AnyKeyPath::Readable(_) => "AnyKeyPath::Readable",
            AnyKeyPath::Writable(_) => "AnyKeyPath::Writable",
            AnyKeyPath::FailableReadable(_) => "AnyKeyPath::FailableReadable",
            AnyKeyPath::FailableWritable(_) => "AnyKeyPath::FailableWritable",
            AnyKeyPath::ReadableEnum { .. } => "AnyKeyPath::ReadableEnum",
            AnyKeyPath::WritableEnum { .. } => "AnyKeyPath::WritableEnum",
            AnyKeyPath::ReferenceWritable(_) => "AnyKeyPath::ReferenceWritable",
            AnyKeyPath::Owned(_) => "AnyKeyPath::Owned",
            AnyKeyPath::FailableOwned(_) => "AnyKeyPath::FailableOwned",
        }
    }

    // ===== Aggregator Functions for AnyKeyPath =====

    /// Adapt this AnyKeyPath to work with Arc<Root>
    pub fn for_arc<Root>(self) -> AnyKeyPath
    where
        Root: 'static + Send + Sync + Clone,
    {
        match self {
            AnyKeyPath::Readable(f) => AnyKeyPath::Readable(Arc::new(move |root| {
                let arc = root.downcast_ref::<Arc<Root>>().unwrap();
                f(&**arc as &(dyn Any + Send + Sync))
            })),
            AnyKeyPath::Writable(_) => {
                panic!("Arc does not support writable keypaths (Arc only implements Deref, not DerefMut)")
            }
            AnyKeyPath::FailableReadable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let arc = root.downcast_ref::<Arc<Root>>().unwrap();
                f(&**arc as &(dyn Any + Send + Sync))
            })),
            AnyKeyPath::FailableWritable(_) => {
                panic!("Arc does not support writable keypaths (Arc only implements Deref, not DerefMut)")
            }
            AnyKeyPath::ReadableEnum { extract, embed } => AnyKeyPath::ReadableEnum {
                extract: Arc::new(move |root| {
                    let arc = root.downcast_ref::<Arc<Root>>().unwrap();
                    extract(&**arc as &(dyn Any + Send + Sync))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Arc::new(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::WritableEnum { .. } => {
                panic!("Arc does not support writable keypaths (Arc only implements Deref, not DerefMut)")
            }
            AnyKeyPath::ReferenceWritable(_) => {
                panic!("Arc does not support writable keypaths (Arc only implements Deref, not DerefMut)")
            }
            AnyKeyPath::Owned(f) => AnyKeyPath::Owned(Arc::new(move |root| {
                let arc = *root.downcast::<Arc<Root>>().unwrap();
                f(Box::new((*arc).clone()) as Box<dyn Any>)
            })),
            AnyKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let arc = *root.downcast::<Arc<Root>>().unwrap();
                f(Box::new((*arc).clone()) as Box<dyn Any>)
            })),
        }
    }

    /// Adapt this AnyKeyPath to work with Box<Root>
    pub fn for_box<Root>(self) -> AnyKeyPath
    where
        Root: 'static + Send + Sync,
    {
        match self {
            AnyKeyPath::Readable(f) => AnyKeyPath::Readable(Arc::new(move |root| {
                let boxed = root.downcast_ref::<Box<Root>>().unwrap();
                f(&**boxed as &(dyn Any + Send + Sync))
            })),
            AnyKeyPath::Writable(f) => AnyKeyPath::Writable(Arc::new(move |root| {
                let boxed = root.downcast_mut::<Box<Root>>().unwrap();
                f(&mut **boxed as &mut (dyn Any + Send + Sync))
            })),
            AnyKeyPath::FailableReadable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let boxed = root.downcast_ref::<Box<Root>>().unwrap();
                f(&**boxed as &(dyn Any + Send + Sync))
            })),
            AnyKeyPath::FailableWritable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let boxed = root.downcast_mut::<Box<Root>>().unwrap();
                f(&mut **boxed as &mut (dyn Any + Send + Sync))
            })),
            AnyKeyPath::ReadableEnum { extract, embed } => AnyKeyPath::ReadableEnum {
                extract: Arc::new(move |root| {
                    let boxed = root.downcast_ref::<Box<Root>>().unwrap();
                    extract(&**boxed as &(dyn Any + Send + Sync))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Box::new(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::WritableEnum { extract, extract_mut, embed } => AnyKeyPath::WritableEnum {
                extract: Arc::new(move |root| {
                    let boxed = root.downcast_ref::<Box<Root>>().unwrap();
                    extract(&**boxed as &(dyn Any + Send + Sync))
                }),
                extract_mut: Arc::new(move |root| {
                    let boxed = root.downcast_mut::<Box<Root>>().unwrap();
                    extract_mut(&mut **boxed as &mut (dyn Any + Send + Sync))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Box::new(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::ReferenceWritable(f) => AnyKeyPath::ReferenceWritable(Arc::new(move |root| {
                let boxed = root.downcast_mut::<Box<Root>>().unwrap();
                f(&mut **boxed as &mut (dyn Any + Send + Sync))
            })),
            AnyKeyPath::Owned(f) => AnyKeyPath::Owned(Arc::new(move |root| {
                let boxed = *root.downcast::<Box<Root>>().unwrap();
                f(Box::new(*boxed) as Box<dyn Any>)
            })),
            AnyKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let boxed = *root.downcast::<Box<Root>>().unwrap();
                f(Box::new(*boxed) as Box<dyn Any>)
            })),
        }
    }

    /// Adapt this AnyKeyPath to work with Rc<Root>
    pub fn for_rc<Root>(self) -> AnyKeyPath
    where
        Root: 'static + Send + Sync + Clone,
    {
        match self {
            AnyKeyPath::Readable(f) => AnyKeyPath::Readable(Arc::new(move |root| {
                let rc = root.downcast_ref::<Rc<Root>>().unwrap();
                f(&**rc as &(dyn Any + Send + Sync))
            })),
            AnyKeyPath::Writable(_) => {
                panic!("Rc does not support writable keypaths (Rc only implements Deref, not DerefMut)")
            }
            AnyKeyPath::FailableReadable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let rc = root.downcast_ref::<Rc<Root>>().unwrap();
                f(&**rc as &(dyn Any + Send + Sync))
            })),
            AnyKeyPath::FailableWritable(_) => {
                panic!("Rc does not support writable keypaths (Rc only implements Deref, not DerefMut)")
            }
            AnyKeyPath::ReadableEnum { extract, embed } => AnyKeyPath::ReadableEnum {
                extract: Arc::new(move |root| {
                    let rc = root.downcast_ref::<Rc<Root>>().unwrap();
                    extract(&**rc as &(dyn Any + Send + Sync))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Rc::new(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::WritableEnum { .. } => {
                panic!("Rc does not support writable keypaths (Rc only implements Deref, not DerefMut)")
            }
            AnyKeyPath::ReferenceWritable(_) => {
                panic!("Rc does not support writable keypaths (Rc only implements Deref, not DerefMut)")
            }
            AnyKeyPath::Owned(f) => AnyKeyPath::Owned(Arc::new(move |root| {
                let rc = *root.downcast::<Rc<Root>>().unwrap();
                f(Box::new((*rc).clone()) as Box<dyn Any>)
            })),
            AnyKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let rc = *root.downcast::<Rc<Root>>().unwrap();
                f(Box::new((*rc).clone()) as Box<dyn Any>)
            })),
        }
    }

    /// Adapt this AnyKeyPath to work with Result<Root, E>
    pub fn for_result<Root, E>(self) -> AnyKeyPath
    where
        Root: 'static + Send + Sync,
        E: 'static,
    {
        match self {
            AnyKeyPath::Readable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let result = root.downcast_ref::<Result<Root, E>>().unwrap();
                result.as_ref().ok().map(|inner| f(inner as &(dyn Any + Send + Sync)))
            })),
            AnyKeyPath::Writable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let result = root.downcast_mut::<Result<Root, E>>().unwrap();
                result.as_mut().ok().map(|inner| f(inner as &mut (dyn Any + Send + Sync)))
            })),
            AnyKeyPath::FailableReadable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let result = root.downcast_ref::<Result<Root, E>>().unwrap();
                result.as_ref().ok().and_then(|inner| f(inner as &(dyn Any + Send + Sync)))
            })),
            AnyKeyPath::FailableWritable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let result = root.downcast_mut::<Result<Root, E>>().unwrap();
                result.as_mut().ok().and_then(|inner| f(inner as &mut (dyn Any + Send + Sync)))
            })),
            AnyKeyPath::ReadableEnum { extract, embed } => AnyKeyPath::ReadableEnum {
                extract: Arc::new(move |root| {
                    let result = root.downcast_ref::<Result<Root, E>>().unwrap();
                    result.as_ref().ok().and_then(|inner| extract(inner as &(dyn Any + Send + Sync)))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Ok::<Root, E>(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::WritableEnum { extract, extract_mut, embed } => AnyKeyPath::WritableEnum {
                extract: Arc::new(move |root| {
                    let result = root.downcast_ref::<Result<Root, E>>().unwrap();
                    result.as_ref().ok().and_then(|inner| extract(inner as &(dyn Any + Send + Sync)))
                }),
                extract_mut: Arc::new(move |root| {
                    let result = root.downcast_mut::<Result<Root, E>>().unwrap();
                    result.as_mut().ok().and_then(|inner| extract_mut(inner as &mut (dyn Any + Send + Sync)))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Ok::<Root, E>(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::ReferenceWritable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let result = root.downcast_mut::<Result<Root, E>>().unwrap();
                result.as_mut().ok().map(|inner| f(inner as &mut (dyn Any + Send + Sync)))
            })),
            AnyKeyPath::Owned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let result = *root.downcast::<Result<Root, E>>().unwrap();
                result.ok().map(|inner| f(Box::new(inner) as Box<dyn Any>))
            })),
            AnyKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let result = *root.downcast::<Result<Root, E>>().unwrap();
                result.ok().and_then(|inner| f(Box::new(inner) as Box<dyn Any>))
            })),
        }
    }

    /// Adapt this AnyKeyPath to work with Option<Root>
    pub fn for_option<Root>(self) -> AnyKeyPath
    where
        Root: 'static + Send + Sync,
    {
        match self {
            AnyKeyPath::Readable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let option = root.downcast_ref::<Option<Root>>().unwrap();
                option.as_ref().map(|inner| f(inner as &(dyn Any + Send + Sync)))
            })),
            AnyKeyPath::Writable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let option = root.downcast_mut::<Option<Root>>().unwrap();
                option.as_mut().map(|inner| f(inner as &mut (dyn Any + Send + Sync)))
            })),
            AnyKeyPath::FailableReadable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let option = root.downcast_ref::<Option<Root>>().unwrap();
                option.as_ref().and_then(|inner| f(inner as &(dyn Any + Send + Sync)))
            })),
            AnyKeyPath::FailableWritable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let option = root.downcast_mut::<Option<Root>>().unwrap();
                option.as_mut().and_then(|inner| f(inner as &mut (dyn Any + Send + Sync)))
            })),
            AnyKeyPath::ReadableEnum { extract, embed } => AnyKeyPath::ReadableEnum {
                extract: Arc::new(move |root| {
                    let option = root.downcast_ref::<Option<Root>>().unwrap();
                    option.as_ref().and_then(|inner| extract(inner as &(dyn Any + Send + Sync)))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Some(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::WritableEnum { extract, extract_mut, embed } => AnyKeyPath::WritableEnum {
                extract: Arc::new(move |root| {
                    let option = root.downcast_ref::<Option<Root>>().unwrap();
                    option.as_ref().and_then(|inner| extract(inner as &(dyn Any + Send + Sync)))
                }),
                extract_mut: Arc::new(move |root| {
                    let option = root.downcast_mut::<Option<Root>>().unwrap();
                    option.as_mut().and_then(|inner| extract_mut(inner as &mut (dyn Any + Send + Sync)))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Some(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::ReferenceWritable(f) => AnyKeyPath::FailableWritable(Arc::new(move |root| {
                let option = root.downcast_mut::<Option<Root>>().unwrap();
                option.as_mut().map(|inner| f(inner as &mut (dyn Any + Send + Sync)))
            })),
            AnyKeyPath::Owned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let option = *root.downcast::<Option<Root>>().unwrap();
                option.map(|inner| f(Box::new(inner) as Box<dyn Any>))
            })),
            AnyKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let option = *root.downcast::<Option<Root>>().unwrap();
                option.and_then(|inner| f(Box::new(inner) as Box<dyn Any>))
            })),
        }
    }

    /// Adapt this AnyKeyPath to work with Arc<RwLock<Root>>
    /// Note: This only supports owned keypaths due to guard lifetime constraints
    pub fn for_arc_rwlock<Root>(self) -> AnyKeyPath
    where
        Root: 'static + Send + Sync + Clone,
    {
        match self {
            AnyKeyPath::Readable(_) => {
                panic!("Arc<RwLock> does not support readable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::Writable(_) => {
                panic!("Arc<RwLock> does not support writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::FailableReadable(_) => {
                panic!("Arc<RwLock> does not support failable readable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::FailableWritable(_) => {
                panic!("Arc<RwLock> does not support failable writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::ReadableEnum { .. } => {
                panic!("Arc<RwLock> does not support readable enum keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::WritableEnum { .. } => {
                panic!("Arc<RwLock> does not support writable enum keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::ReferenceWritable(_) => {
                panic!("Arc<RwLock> does not support reference writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::Owned(f) => AnyKeyPath::Owned(Arc::new(move |root| {
                let arc_rwlock = *root.downcast::<Arc<RwLock<Root>>>().unwrap();
                let guard = arc_rwlock.read().unwrap();
                let value = f(Box::new((*guard).clone()) as Box<dyn Any>);
                drop(guard); // Ensure guard is dropped before returning
                value
            })),
            AnyKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let arc_rwlock = *root.downcast::<Arc<RwLock<Root>>>().unwrap();
                let guard = arc_rwlock.read().unwrap();
                let value = f(Box::new((*guard).clone()) as Box<dyn Any>);
                drop(guard); // Ensure guard is dropped before returning
                value
            })),
        }
    }

    /// Adapt this AnyKeyPath to work with Arc<Mutex<Root>>
    /// Note: This only supports owned keypaths due to guard lifetime constraints
    pub fn for_arc_mutex<Root>(self) -> AnyKeyPath
    where
        Root: 'static + Send + Sync + Clone,
    {
        match self {
            AnyKeyPath::Readable(_) => {
                panic!("Arc<Mutex> does not support readable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::Writable(_) => {
                panic!("Arc<Mutex> does not support writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::FailableReadable(_) => {
                panic!("Arc<Mutex> does not support failable readable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::FailableWritable(_) => {
                panic!("Arc<Mutex> does not support failable writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::ReadableEnum { .. } => {
                panic!("Arc<Mutex> does not support readable enum keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::WritableEnum { .. } => {
                panic!("Arc<Mutex> does not support writable enum keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::ReferenceWritable(_) => {
                panic!("Arc<Mutex> does not support reference writable keypaths due to guard lifetime constraints. Use owned keypaths instead.")
            }
            AnyKeyPath::Owned(f) => AnyKeyPath::Owned(Arc::new(move |root| {
                let arc_mutex = *root.downcast::<Arc<Mutex<Root>>>().unwrap();
                let guard = arc_mutex.lock().unwrap();
                let value = f(Box::new((*guard).clone()) as Box<dyn Any>);
                drop(guard); // Ensure guard is dropped before returning
                value
            })),
            AnyKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let arc_mutex = *root.downcast::<Arc<Mutex<Root>>>().unwrap();
                let guard = arc_mutex.lock().unwrap();
                let value = f(Box::new((*guard).clone()) as Box<dyn Any>);
                drop(guard); // Ensure guard is dropped before returning
                value
            })),
        }
    }

    /// Adapt this AnyKeyPath to work with Tagged<Root, Tag>
    pub fn for_tagged<Root, Tag>(self) -> AnyKeyPath
    where
        Root: Clone + 'static + Send + Sync,
        Tag: Send + Sync + 'static,
    {
        match self {
            AnyKeyPath::Readable(f) => AnyKeyPath::Readable(Arc::new(move |root| {
                let tagged = root.downcast_ref::<Tagged<Root, Tag>>().unwrap();
                f(&*tagged as &(dyn Any + Send + Sync))
            })),
            AnyKeyPath::Writable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            AnyKeyPath::FailableReadable(f) => AnyKeyPath::FailableReadable(Arc::new(move |root| {
                let tagged = root.downcast_ref::<Tagged<Root, Tag>>().unwrap();
                f(&*tagged as &(dyn Any + Send + Sync))
            })),
            AnyKeyPath::FailableWritable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            AnyKeyPath::ReadableEnum { extract, embed } => AnyKeyPath::ReadableEnum {
                extract: Arc::new(move |root| {
                    let tagged = root.downcast_ref::<Tagged<Root, Tag>>().unwrap();
                    extract(&*tagged as &(dyn Any + Send + Sync))
                }),
                embed: Arc::new(move |value| {
                    let inner = embed(value);
                    Box::new(Tagged::<Root, Tag>::new(*inner.downcast::<Root>().unwrap())) as Box<dyn Any>
                }),
            },
            AnyKeyPath::WritableEnum { .. } => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            AnyKeyPath::ReferenceWritable(_) => {
                panic!("Tagged does not support writable keypaths (Tagged only implements Deref, not DerefMut)")
            }
            AnyKeyPath::Owned(f) => AnyKeyPath::Owned(Arc::new(move |root| {
                let tagged = *root.downcast::<Tagged<Root, Tag>>().unwrap();
                f(Box::new((*tagged).clone()) as Box<dyn Any>)
            })),
            AnyKeyPath::FailableOwned(f) => AnyKeyPath::FailableOwned(Arc::new(move |root| {
                let tagged = *root.downcast::<Tagged<Root, Tag>>().unwrap();
                f(Box::new((*tagged).clone()) as Box<dyn Any>)
            })),
        }
    }
}

// ===== WithContainer Trait Implementation =====
impl<Root, Value> WithContainer<Root, Value> for KeyPaths<Root, Value> {
    /// Execute a closure with a reference to the value inside an Arc
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_arc<F, R>(self, arc: &Arc<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => f(get(&**arc)),
            KeyPaths::FailableReadable(get) => {
                if let Some(value) = get(&**arc) {
                    f(value)
                } else {
                    panic!("FailableReadable keypath returned None for Arc")
                }
            }
            _ => panic!("with_arc only works with readable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside a Box
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_box<F, R>(self, boxed: &Box<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => f(get(&**boxed)),
            KeyPaths::FailableReadable(get) => {
                if let Some(value) = get(&**boxed) {
                    f(value)
                } else {
                    panic!("FailableReadable keypath returned None for Box")
                }
            }
            _ => panic!("with_box only works with readable keypaths"),
        }
    }

    /// Execute a closure with a mutable reference to the value inside a Box
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_box_mut<F, R>(self, boxed: &mut Box<Root>, f: F) -> R
    where
        F: FnOnce(&mut Value) -> R,
    {
        match self {
            KeyPaths::Writable(get) => f(get(&mut **boxed)),
            KeyPaths::FailableWritable(get) => {
                if let Some(value) = get(&mut **boxed) {
                    f(value)
                } else {
                    panic!("FailableWritable keypath returned None for Box")
                }
            }
            _ => panic!("with_box_mut only works with writable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside an Rc
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_rc<F, R>(self, rc: &Rc<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => f(get(&**rc)),
            KeyPaths::FailableReadable(get) => {
                if let Some(value) = get(&**rc) {
                    f(value)
                } else {
                    panic!("FailableReadable keypath returned None for Rc")
                }
            }
            _ => panic!("with_rc only works with readable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside a Result
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_result<F, R, E>(self, result: &Result<Root, E>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => {
                result.as_ref().ok().map(|root| f(get(root)))
            }
            KeyPaths::FailableReadable(get) => {
                result.as_ref().ok().and_then(|root| get(root).map(|v| f(v)))
            }
            _ => panic!("with_result only works with readable keypaths"),
        }
    }

    /// Execute a closure with a mutable reference to the value inside a Result
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_result_mut<F, R, E>(self, result: &mut Result<Root, E>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R,
    {
        match self {
            KeyPaths::Writable(get) => {
                result.as_mut().ok().map(|root| f(get(root)))
            }
            KeyPaths::FailableWritable(get) => {
                result.as_mut().ok().and_then(|root| get(root).map(|v| f(v)))
            }
            _ => panic!("with_result_mut only works with writable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside an Option
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_option<F, R>(self, option: &Option<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => {
                option.as_ref().map(|root| f(get(root)))
            }
            KeyPaths::FailableReadable(get) => {
                option.as_ref().and_then(|root| get(root).map(|v| f(v)))
            }
            _ => panic!("with_option only works with readable keypaths"),
        }
    }

    /// Execute a closure with a mutable reference to the value inside an Option
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_option_mut<F, R>(self, option: &mut Option<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R,
    {
        match self {
            KeyPaths::Writable(get) => {
                option.as_mut().map(|root| f(get(root)))
            }
            KeyPaths::FailableWritable(get) => {
                option.as_mut().and_then(|root| get(root).map(|v| f(v)))
            }
            _ => panic!("with_option_mut only works with writable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside a RefCell
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_refcell<F, R>(self, refcell: &RefCell<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => {
                refcell.try_borrow().ok().map(|borrow| f(get(&*borrow)))
            }
            KeyPaths::FailableReadable(get) => {
                refcell.try_borrow().ok().and_then(|borrow| get(&*borrow).map(|v| f(v)))
            }
            _ => panic!("with_refcell only works with readable keypaths"),
        }
    }

    /// Execute a closure with a mutable reference to the value inside a RefCell
    /// This avoids cloning by working with references directly
    #[inline]
    fn with_refcell_mut<F, R>(self, refcell: &RefCell<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R,
    {
        match self {
            KeyPaths::Writable(get) => {
                refcell.try_borrow_mut().ok().map(|mut borrow| f(get(&mut *borrow)))
            }
            KeyPaths::FailableWritable(get) => {
                refcell.try_borrow_mut().ok().and_then(|mut borrow| get(&mut *borrow).map(|v| f(v)))
            }
            _ => panic!("with_refcell_mut only works with writable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside a Tagged
    /// This avoids cloning by working with references directly
    #[cfg(feature = "tagged_core")]
    #[inline]
    fn with_tagged<F, R, Tag>(self, tagged: &Tagged<Root, Tag>, f: F) -> R
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => f(get(&**tagged)),
            KeyPaths::FailableReadable(get) => {
                get(&**tagged).map_or_else(|| panic!("Tagged value is None"), f)
            }
            KeyPaths::ReadableEnum { extract, .. } => {
                extract(&**tagged).map_or_else(|| panic!("Tagged value is None"), f)
            }
            _ => panic!("with_tagged only works with readable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside a Mutex
    /// This avoids cloning by working with references while the guard is alive
    #[inline]
    fn with_mutex<F, R>(self, mutex: &Mutex<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => {
                mutex.try_lock().ok().map(|guard| f(get(&*guard)))
            }
            KeyPaths::FailableReadable(get) => {
                mutex.try_lock().ok().and_then(|guard| get(&*guard).map(|v| f(v)))
            }
            _ => panic!("with_mutex only works with readable keypaths"),
        }
    }

    /// Execute a closure with a mutable reference to the value inside a Mutex
    /// This avoids cloning by working with references while the guard is alive
    #[inline]
    fn with_mutex_mut<F, R>(self, mutex: &mut Mutex<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R,
    {
        match self {
            KeyPaths::Writable(get) => {
                mutex.try_lock().ok().map(|mut guard| f(get(&mut *guard)))
            }
            KeyPaths::FailableWritable(get) => {
                mutex.try_lock().ok().and_then(|mut guard| get(&mut *guard).map(|v| f(v)))
            }
            _ => panic!("with_mutex_mut only works with writable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside an RwLock
    /// This avoids cloning by working with references while the guard is alive
    #[inline]
    fn with_rwlock<F, R>(self, rwlock: &RwLock<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => {
                rwlock.try_read().ok().map(|guard| f(get(&*guard)))
            }
            KeyPaths::FailableReadable(get) => {
                rwlock.try_read().ok().and_then(|guard| get(&*guard).map(|v| f(v)))
            }
            _ => panic!("with_rwlock only works with readable keypaths"),
        }
    }

    /// Execute a closure with a mutable reference to the value inside an RwLock
    /// This avoids cloning by working with references while the guard is alive
    #[inline]
    fn with_rwlock_mut<F, R>(self, rwlock: &mut RwLock<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R,
    {
        match self {
            KeyPaths::Writable(get) => {
                rwlock.try_write().ok().map(|mut guard| f(get(&mut *guard)))
            }
            KeyPaths::FailableWritable(get) => {
                rwlock.try_write().ok().and_then(|mut guard| get(&mut *guard).map(|v| f(v)))
            }
            _ => panic!("with_rwlock_mut only works with writable keypaths"),
        }
    }

    /// Execute a closure with a reference to the value inside an Arc<RwLock<Root>>
    /// This avoids cloning by working with references while the guard is alive
    fn with_arc_rwlock<F, R>(self, arc_rwlock: &Arc<RwLock<Root>>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R,
    {
        match self {
            KeyPaths::Readable(get) => {
                arc_rwlock.try_read().ok().map(|guard| f(get(&*guard)))
            }
            KeyPaths::FailableReadable(get) => {
                arc_rwlock.try_read().ok().and_then(|guard| get(&*guard).map(|v| f(v)))
            }
            _ => panic!("with_arc_rwlock only works with readable keypaths"),
        }
    }

    /// Execute a closure with a mutable reference to the value inside an Arc<RwLock<Root>>
    /// This avoids cloning by working with references while the guard is alive
    fn with_arc_rwlock_mut<F, R>(self, arc_rwlock: &Arc<RwLock<Root>>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R,
    {
        match self {
            KeyPaths::Writable(get) => {
                arc_rwlock.try_write().ok().map(|mut guard| f(get(&mut *guard)))
            }
            KeyPaths::FailableWritable(get) => {
                arc_rwlock.try_write().ok().and_then(|mut guard| get(&mut *guard).map(|v| f(v)))
            }
            _ => panic!("with_arc_rwlock_mut only works with writable keypaths"),
        }
    }
}

impl<Root, Mid> KeyPaths<Root, Mid>
where
    Root: 'static,
    Mid: 'static,
{
    /// Alias for `compose` for ergonomic chaining.
    #[inline]
    pub fn then<Value>(self, mid: KeyPaths<Mid, Value>) -> KeyPaths<Root, Value>
    where
        Value: 'static,
    {
        self.compose(mid)
    }

    pub fn compose<Value>(self, mid: KeyPaths<Mid, Value>) -> KeyPaths<Root, Value>
    where
        Value: 'static,
    {
        use KeyPaths::*;

        match (self, mid) {
            (Readable(f1), Readable(f2)) => Readable(Arc::new(move |r| f2(f1(r)))),

            (Writable(f1), Writable(f2)) => Writable(Arc::new(move |r| f2(f1(r)))),

            (FailableReadable(f1), Readable(f2)) => {
                FailableReadable(Arc::new(move |r| f1(r).map(|m| f2(m))))
            }

            (Readable(f1), FailableReadable(f2)) => FailableReadable(Arc::new(move |r| f2(f1(r)))),

            (FailableReadable(f1), FailableReadable(f2)) => {
                FailableReadable(Arc::new(move |r| f1(r).and_then(|m| f2(m))))
            }

            (FailableWritable(f1), Writable(f2)) => {
                FailableWritable(Arc::new(move |r| f1(r).map(|m| f2(m))))
            }

            (Writable(f1), FailableWritable(f2)) => FailableWritable(Arc::new(move |r| f2(f1(r)))),

            (FailableWritable(f1), FailableWritable(f2)) => {
                FailableWritable(Arc::new(move |r| f1(r).and_then(|m| f2(m))))
            }
            (FailableReadable(f1), ReadableEnum { extract, .. }) => {
                FailableReadable(Arc::new(move |r| f1(r).and_then(|m| extract(m))))
            }
            // (ReadableEnum { extract, .. }, FailableReadable(f2)) => {
            //     FailableReadable(Arc::new(move |r| extract(r).map(|m| f2(m).unwrap())))
            // }
            (ReadableEnum { extract, .. }, Readable(f2)) => {
                FailableReadable(Arc::new(move |r| extract(r).map(|m| f2(m))))
            }

            (ReadableEnum { extract, .. }, FailableReadable(f2)) => {
                FailableReadable(Arc::new(move |r| extract(r).and_then(|m| f2(m))))
            }

            (WritableEnum { extract, .. }, Readable(f2)) => {
                FailableReadable(Arc::new(move |r| extract(r).map(|m| f2(m))))
            }

            (WritableEnum { extract, .. }, FailableReadable(f2)) => {
                FailableReadable(Arc::new(move |r| extract(r).and_then(|m| f2(m))))
            }

            (WritableEnum { extract_mut, .. }, Writable(f2)) => {
                FailableWritable(Arc::new(move |r| extract_mut(r).map(|m| f2(m))))
            }

            (
                FailableWritable(f_root_mid),
                WritableEnum {
                    extract_mut: exm_mid_val,
                    ..
                },
            ) => {
                FailableWritable(Arc::new(move |r: &mut Root| {
                    // First, apply the function that operates on Root.
                    // This will give us `Option<&mut Mid>`.
                    let intermediate_mid_ref = f_root_mid(r);

                    // Then, apply the function that operates on Mid.
                    // This will give us `Option<&mut Value>`.
                    intermediate_mid_ref.and_then(|intermediate_mid| exm_mid_val(intermediate_mid))
                }))
            }

            (WritableEnum { extract_mut, .. }, FailableWritable(f2)) => {
                FailableWritable(Arc::new(move |r| extract_mut(r).and_then(|m| f2(m))))
            }

            // New: Writable then WritableEnum => FailableWritable
            (Writable(f1), WritableEnum { extract_mut, .. }) => {
                FailableWritable(Arc::new(move |r: &mut Root| {
                    let mid: &mut Mid = f1(r);
                    extract_mut(mid)
                }))
            }

            (
                ReadableEnum {
                    extract: ex1,
                    embed: em1,
                },
                ReadableEnum {
                    extract: ex2,
                    embed: em2,
                },
            ) => ReadableEnum {
                extract: Arc::new(move |r| ex1(r).and_then(|m| ex2(m))),
                embed: Arc::new(move |v| em1(em2(v))),
            },

            (
                WritableEnum {
                    extract: ex1,
                    extract_mut: _,
                    embed: em1,
                },
                ReadableEnum {
                    extract: ex2,
                    embed: em2,
                },
            ) => ReadableEnum {
                extract: Arc::new(move |r| ex1(r).and_then(|m| ex2(m))),
                embed: Arc::new(move |v| em1(em2(v))),
            },

            (
                WritableEnum {
                    extract: ex1,
                    extract_mut: exm1,
                    embed: em1,
                },
                WritableEnum {
                    extract: ex2,
                    extract_mut: exm2,
                    embed: em2,
                },
            ) => WritableEnum {
                extract: Arc::new(move |r| ex1(r).and_then(|m| ex2(m))),
                extract_mut: Arc::new(move |r| exm1(r).and_then(|m| exm2(m))),
                embed: Arc::new(move |v| em1(em2(v))),
            },


            // New owned keypath compositions
            (Owned(f1), Owned(f2)) => {
                Owned(Arc::new(move |r| f2(f1(r))))
            }
            (FailableOwned(f1), Owned(f2)) => {
                FailableOwned(Arc::new(move |r| f1(r).map(|m| f2(m))))
            }
            (Owned(f1), FailableOwned(f2)) => {
                FailableOwned(Arc::new(move |r| f2(f1(r))))
            }
            (FailableOwned(f1), FailableOwned(f2)) => {
                FailableOwned(Arc::new(move |r| f1(r).and_then(|m| f2(m))))
            }

            // Cross-composition between owned and regular keypaths
            // Note: These compositions require Clone bounds which may not always be available
            // For now, we'll skip these complex compositions

            (a, b) => panic!(
                "Unsupported composition: {:?} then {:?}",
                kind_name(&a),
                kind_name(&b)
            ),
        }
    }

    /// Get the kind name of this keypath
    #[inline]
    pub fn kind_name(&self) -> &'static str {
        kind_name(self)
    }
}

fn kind_name<Root, Value>(k: &KeyPaths<Root, Value>) -> &'static str {
    use KeyPaths::*;
    match k {
        Readable(_) => "Readable",
        Writable(_) => "Writable",
        FailableReadable(_) => "FailableReadable",
        FailableWritable(_) => "FailableWritable",
        ReadableEnum { .. } => "ReadableEnum",
        WritableEnum { .. } => "WritableEnum",
        ReferenceWritable(_) => "ReferenceWritable",
        // New owned keypath types
        Owned(_) => "Owned",
        FailableOwned(_) => "FailableOwned",
    }
}

// ===== Helper functions for creating reusable getter functions =====
// Note: These helper functions have lifetime constraints that make them
// difficult to implement in Rust's current type system. The keypath
// instances themselves can be used directly for access.

// ===== Global compose function =====

/// Global compose function that combines two compatible key paths
pub fn compose<Root, Mid, Value>(
    kp1: KeyPaths<Root, Mid>,
    kp2: KeyPaths<Mid, Value>,
) -> KeyPaths<Root, Value>
where
    Root: 'static,
    Mid: 'static,
    Value: 'static,
{
    kp1.compose(kp2)
}

// ===== Helper macros for enum case keypaths =====

#[macro_export]
macro_rules! readable_enum_macro {
    // Unit variant: Enum::Variant
    ($enum:path, $variant:ident) => {{
        $crate::KeyPaths::readable_enum(
            |_| <$enum>::$variant,
            |e: &$enum| match e {
                <$enum>::$variant => Some(&()),
                _ => None,
            },
        )
    }};
    // Single-field tuple variant: Enum::Variant(Inner)
    ($enum:path, $variant:ident($inner:ty)) => {{
        $crate::KeyPaths::readable_enum(
            |v: $inner| <$enum>::$variant(v),
            |e: &$enum| match e {
                <$enum>::$variant(v) => Some(v),
                _ => None,
            },
        )
    }};
}

#[macro_export]
macro_rules! writable_enum_macro {
    // Unit variant: Enum::Variant (creates prism to and from ())
    ($enum:path, $variant:ident) => {{
        $crate::KeyPaths::writable_enum(
            |_| <$enum>::$variant,
            |e: &$enum| match e {
                <$enum>::$variant => Some(&()),
                _ => None,
            },
            |e: &mut $enum| match e {
                <$enum>::$variant => Some(&mut ()),
                _ => None,
            },
        )
    }};
    // Single-field tuple variant: Enum::Variant(Inner)
    ($enum:path, $variant:ident($inner:ty)) => {{
        $crate::KeyPaths::writable_enum(
            |v: $inner| <$enum>::$variant(v),
            |e: &$enum| match e {
                <$enum>::$variant(v) => Some(v),
                _ => None,
            },
            |e: &mut $enum| match e {
                <$enum>::$variant(v) => Some(v),
                _ => None,
            },
        )
    }};
}
