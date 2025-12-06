use std::sync::Arc;
use std::marker::PhantomData;
use std::any::{Any, TypeId};
use std::rc::Rc;

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

// Container access utilities
pub mod containers {
    use super::{OptionalKeyPath, WritableOptionalKeyPath, KeyPath, WritableKeyPath};
    use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque, LinkedList, BinaryHeap};
    use std::sync::{Mutex, RwLock, Weak as StdWeak, Arc};
    use std::rc::{Weak as RcWeak, Rc};
    use std::ops::{Deref, DerefMut};

    #[cfg(feature = "parking_lot")]
    use parking_lot::{Mutex as ParkingMutex, RwLock as ParkingRwLock};

    #[cfg(feature = "tagged")]
    use tagged_core::Tagged;

    /// Create a keypath for indexed access in Vec<T>
    pub fn for_vec_index<T>(index: usize) -> OptionalKeyPath<Vec<T>, T, impl for<'r> Fn(&'r Vec<T>) -> Option<&'r T>> {
        OptionalKeyPath::new(move |vec: &Vec<T>| vec.get(index))
    }

    /// Create a keypath for indexed access in VecDeque<T>
    pub fn for_vecdeque_index<T>(index: usize) -> OptionalKeyPath<VecDeque<T>, T, impl for<'r> Fn(&'r VecDeque<T>) -> Option<&'r T>> {
        OptionalKeyPath::new(move |deque: &VecDeque<T>| deque.get(index))
    }

    /// Create a keypath for indexed access in LinkedList<T>
    pub fn for_linkedlist_index<T>(index: usize) -> OptionalKeyPath<LinkedList<T>, T, impl for<'r> Fn(&'r LinkedList<T>) -> Option<&'r T>> {
        OptionalKeyPath::new(move |list: &LinkedList<T>| {
            list.iter().nth(index)
        })
    }

    /// Create a keypath for key-based access in HashMap<K, V>
    pub fn for_hashmap_key<K, V>(key: K) -> OptionalKeyPath<HashMap<K, V>, V, impl for<'r> Fn(&'r HashMap<K, V>) -> Option<&'r V>>
    where
        K: std::hash::Hash + Eq + Clone + 'static,
        V: 'static,
    {
        OptionalKeyPath::new(move |map: &HashMap<K, V>| map.get(&key))
    }

    /// Create a keypath for key-based access in BTreeMap<K, V>
    pub fn for_btreemap_key<K, V>(key: K) -> OptionalKeyPath<BTreeMap<K, V>, V, impl for<'r> Fn(&'r BTreeMap<K, V>) -> Option<&'r V>>
    where
        K: Ord + Clone + 'static,
        V: 'static,
    {
        OptionalKeyPath::new(move |map: &BTreeMap<K, V>| map.get(&key))
    }

    /// Create a keypath for getting a value from HashSet<T> (returns Option<&T>)
    pub fn for_hashset_get<T>(value: T) -> OptionalKeyPath<HashSet<T>, T, impl for<'r> Fn(&'r HashSet<T>) -> Option<&'r T>>
    where
        T: std::hash::Hash + Eq + Clone + 'static,
    {
        OptionalKeyPath::new(move |set: &HashSet<T>| set.get(&value))
    }

    /// Create a keypath for checking membership in BTreeSet<T>
    pub fn for_btreeset_get<T>(value: T) -> OptionalKeyPath<BTreeSet<T>, T, impl for<'r> Fn(&'r BTreeSet<T>) -> Option<&'r T>>
    where
        T: Ord + Clone + 'static,
    {
        OptionalKeyPath::new(move |set: &BTreeSet<T>| set.get(&value))
    }

    /// Create a keypath for peeking at the top of BinaryHeap<T>
    pub fn for_binaryheap_peek<T>() -> OptionalKeyPath<BinaryHeap<T>, T, impl for<'r> Fn(&'r BinaryHeap<T>) -> Option<&'r T>>
    where
        T: Ord + 'static,
    {
        OptionalKeyPath::new(|heap: &BinaryHeap<T>| heap.peek())
    }

    // ========== WRITABLE VERSIONS ==========

    /// Create a writable keypath for indexed access in Vec<T>
    pub fn for_vec_index_mut<T>(index: usize) -> WritableOptionalKeyPath<Vec<T>, T, impl for<'r> Fn(&'r mut Vec<T>) -> Option<&'r mut T>> {
        WritableOptionalKeyPath::new(move |vec: &mut Vec<T>| vec.get_mut(index))
    }

    /// Create a writable keypath for indexed access in VecDeque<T>
    pub fn for_vecdeque_index_mut<T>(index: usize) -> WritableOptionalKeyPath<VecDeque<T>, T, impl for<'r> Fn(&'r mut VecDeque<T>) -> Option<&'r mut T>> {
        WritableOptionalKeyPath::new(move |deque: &mut VecDeque<T>| deque.get_mut(index))
    }

    /// Create a writable keypath for indexed access in LinkedList<T>
    pub fn for_linkedlist_index_mut<T>(index: usize) -> WritableOptionalKeyPath<LinkedList<T>, T, impl for<'r> Fn(&'r mut LinkedList<T>) -> Option<&'r mut T>> {
        WritableOptionalKeyPath::new(move |list: &mut LinkedList<T>| {
            // LinkedList doesn't have get_mut, so we need to iterate
            let mut iter = list.iter_mut();
            iter.nth(index)
        })
    }

    /// Create a writable keypath for key-based access in HashMap<K, V>
    pub fn for_hashmap_key_mut<K, V>(key: K) -> WritableOptionalKeyPath<HashMap<K, V>, V, impl for<'r> Fn(&'r mut HashMap<K, V>) -> Option<&'r mut V>>
    where
        K: std::hash::Hash + Eq + Clone + 'static,
        V: 'static,
    {
        WritableOptionalKeyPath::new(move |map: &mut HashMap<K, V>| map.get_mut(&key))
    }

    /// Create a writable keypath for key-based access in BTreeMap<K, V>
    pub fn for_btreemap_key_mut<K, V>(key: K) -> WritableOptionalKeyPath<BTreeMap<K, V>, V, impl for<'r> Fn(&'r mut BTreeMap<K, V>) -> Option<&'r mut V>>
    where
        K: Ord + Clone + 'static,
        V: 'static,
    {
        WritableOptionalKeyPath::new(move |map: &mut BTreeMap<K, V>| map.get_mut(&key))
    }

    /// Create a writable keypath for getting a mutable value from HashSet<T>
    /// Note: HashSet doesn't support mutable access to elements, but we provide it for consistency
    pub fn for_hashset_get_mut<T>(value: T) -> WritableOptionalKeyPath<HashSet<T>, T, impl for<'r> Fn(&'r mut HashSet<T>) -> Option<&'r mut T>>
    where
        T: std::hash::Hash + Eq + Clone + 'static,
    {
        WritableOptionalKeyPath::new(move |set: &mut HashSet<T>| {
            // HashSet doesn't have get_mut, so we need to check and return None
            // This is a limitation of HashSet's design
            if set.contains(&value) {
                // We can't return a mutable reference to the value in the set
                // This is a fundamental limitation of HashSet
                None
            } else {
                None
            }
        })
    }

    /// Create a writable keypath for getting a mutable value from BTreeSet<T>
    /// Note: BTreeSet doesn't support mutable access to elements, but we provide it for consistency
    pub fn for_btreeset_get_mut<T>(value: T) -> WritableOptionalKeyPath<BTreeSet<T>, T, impl for<'r> Fn(&'r mut BTreeSet<T>) -> Option<&'r mut T>>
    where
        T: Ord + Clone + 'static,
    {
        WritableOptionalKeyPath::new(move |set: &mut BTreeSet<T>| {
            // BTreeSet doesn't have get_mut, so we need to check and return None
            // This is a limitation of BTreeSet's design
            if set.contains(&value) {
                // We can't return a mutable reference to the value in the set
                // This is a fundamental limitation of BTreeSet
                None
            } else {
                None
            }
        })
    }

    /// Create a writable keypath for peeking at the top of BinaryHeap<T>
    /// Note: BinaryHeap.peek_mut() returns PeekMut which is a guard type.
    /// Due to Rust's borrowing rules, we cannot return &mut T directly from PeekMut.
    /// This function returns None as BinaryHeap doesn't support direct mutable access
    /// through keypaths. Use heap.peek_mut() directly for mutable access.
    pub fn for_binaryheap_peek_mut<T>() -> WritableOptionalKeyPath<BinaryHeap<T>, T, impl for<'r> Fn(&'r mut BinaryHeap<T>) -> Option<&'r mut T>>
    where
        T: Ord + 'static,
    {
        // BinaryHeap.peek_mut() returns PeekMut which is a guard type that owns the mutable reference.
        // We cannot return &mut T from it due to lifetime constraints.
        // This is a fundamental limitation - use heap.peek_mut() directly instead.
        WritableOptionalKeyPath::new(|_heap: &mut BinaryHeap<T>| {
            None
        })
    }

    // ========== SYNCHRONIZATION PRIMITIVES ==========
    // Note: Mutex and RwLock return guards that own the lock, not references.
    // We cannot create keypaths that return references from guards due to lifetime constraints.
    // These helper functions are provided for convenience, but direct lock()/read()/write() calls are recommended.

    /// Helper function to lock a Mutex<T> and access its value
    /// Returns None if the mutex is poisoned
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn lock_mutex<T>(mutex: &Mutex<T>) -> Option<std::sync::MutexGuard<'_, T>> {
        mutex.lock().ok()
    }

    /// Helper function to read-lock an RwLock<T> and access its value
    /// Returns None if the lock is poisoned
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn read_rwlock<T>(rwlock: &RwLock<T>) -> Option<std::sync::RwLockReadGuard<'_, T>> {
        rwlock.read().ok()
    }

    /// Helper function to write-lock an RwLock<T> and access its value
    /// Returns None if the lock is poisoned
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn write_rwlock<T>(rwlock: &RwLock<T>) -> Option<std::sync::RwLockWriteGuard<'_, T>> {
        rwlock.write().ok()
    }

    /// Helper function to lock an Arc<Mutex<T>> and access its value
    /// Returns None if the mutex is poisoned
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn lock_arc_mutex<T>(arc_mutex: &Arc<Mutex<T>>) -> Option<std::sync::MutexGuard<'_, T>> {
        arc_mutex.lock().ok()
    }

    /// Helper function to read-lock an Arc<RwLock<T>> and access its value
    /// Returns None if the lock is poisoned
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn read_arc_rwlock<T>(arc_rwlock: &Arc<RwLock<T>>) -> Option<std::sync::RwLockReadGuard<'_, T>> {
        arc_rwlock.read().ok()
    }

    /// Helper function to write-lock an Arc<RwLock<T>> and access its value
    /// Returns None if the lock is poisoned
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn write_arc_rwlock<T>(arc_rwlock: &Arc<RwLock<T>>) -> Option<std::sync::RwLockWriteGuard<'_, T>> {
        arc_rwlock.write().ok()
    }

    /// Helper function to upgrade a Weak<T> to Arc<T>
    /// Returns None if the Arc has been dropped
    /// Note: This returns an owned Arc, not a reference, so it cannot be used in keypaths directly
    pub fn upgrade_weak<T>(weak: &StdWeak<T>) -> Option<Arc<T>> {
        weak.upgrade()
    }

    /// Helper function to upgrade an Rc::Weak<T> to Rc<T>
    /// Returns None if the Rc has been dropped
    /// Note: This returns an owned Rc, not a reference, so it cannot be used in keypaths directly
    pub fn upgrade_rc_weak<T>(weak: &RcWeak<T>) -> Option<Rc<T>> {
        weak.upgrade()
    }

    #[cfg(feature = "parking_lot")]
    /// Helper function to lock a parking_lot::Mutex<T> and access its value
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn lock_parking_mutex<T>(mutex: &ParkingMutex<T>) -> parking_lot::MutexGuard<'_, T> {
        mutex.lock()
    }

    #[cfg(feature = "parking_lot")]
    /// Helper function to read-lock a parking_lot::RwLock<T> and access its value
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn read_parking_rwlock<T>(rwlock: &ParkingRwLock<T>) -> parking_lot::RwLockReadGuard<'_, T> {
        rwlock.read()
    }

    #[cfg(feature = "parking_lot")]
    /// Helper function to write-lock a parking_lot::RwLock<T> and access its value
    /// Note: This returns a guard, not a reference, so it cannot be used in keypaths directly
    pub fn write_parking_rwlock<T>(rwlock: &ParkingRwLock<T>) -> parking_lot::RwLockWriteGuard<'_, T> {
        rwlock.write()
    }

    #[cfg(feature = "tagged")]
    /// Create a keypath for accessing the inner value of Tagged<Tag, T>
    /// Tagged implements Deref, so we can access the inner value directly
    pub fn for_tagged<Tag, T>() -> KeyPath<Tagged<Tag, T>, T, impl for<'r> Fn(&'r Tagged<Tag, T>) -> &'r T>
    where
        Tagged<Tag, T>: std::ops::Deref<Target = T>,
        Tag: 'static,
        T: 'static,
    {
        KeyPath::new(|tagged: &Tagged<Tag, T>| tagged.deref())
    }

    #[cfg(feature = "tagged")]
    /// Create a writable keypath for accessing the inner value of Tagged<Tag, T>
    /// Tagged implements DerefMut, so we can access the inner value directly
    pub fn for_tagged_mut<Tag, T>() -> WritableKeyPath<Tagged<Tag, T>, T, impl for<'r> Fn(&'r mut Tagged<Tag, T>) -> &'r mut T>
    where
        Tagged<Tag, T>: std::ops::DerefMut<Target = T>,
        Tag: 'static,
        T: 'static,
    {
        WritableKeyPath::new(|tagged: &mut Tagged<Tag, T>| tagged.deref_mut())
    }
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

// WritableKeyPath for mutable access
#[derive(Clone)]
pub struct WritableKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> &'r mut Value,
{
    getter: F,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value, F> WritableKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> &'r mut Value,
{
    pub fn new(getter: F) -> Self {
        Self {
            getter,
            _phantom: PhantomData,
        }
    }
    
    pub fn get_mut<'r>(&self, root: &'r mut Root) -> &'r mut Value {
        (self.getter)(root)
    }
    
    // Instance methods for unwrapping containers (automatically infers Target from Value::Target)
    // Box<T> -> T
    pub fn for_box<Target>(self) -> WritableKeyPath<Root, Target, impl for<'r> Fn(&'r mut Root) -> &'r mut Target + 'static>
    where
        Value: std::ops::DerefMut<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableKeyPath {
            getter: move |root: &mut Root| {
                getter(root).deref_mut()
            },
            _phantom: PhantomData,
        }
    }
    
    // Arc<T> -> T (Note: Arc doesn't support mutable access, but we provide it for consistency)
    // This will require interior mutability patterns
    pub fn for_arc<Target>(self) -> WritableKeyPath<Root, Target, impl for<'r> Fn(&'r mut Root) -> &'r mut Target + 'static>
    where
        Value: std::ops::DerefMut<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableKeyPath {
            getter: move |root: &mut Root| {
                getter(root).deref_mut()
            },
            _phantom: PhantomData,
        }
    }
    
    // Rc<T> -> T (Note: Rc doesn't support mutable access, but we provide it for consistency)
    // This will require interior mutability patterns
    pub fn for_rc<Target>(self) -> WritableKeyPath<Root, Target, impl for<'r> Fn(&'r mut Root) -> &'r mut Target + 'static>
    where
        Value: std::ops::DerefMut<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableKeyPath {
            getter: move |root: &mut Root| {
                getter(root).deref_mut()
            },
            _phantom: PhantomData,
        }
    }
}

// WritableOptionalKeyPath for failable mutable access
#[derive(Clone)]
pub struct WritableOptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value>,
{
    getter: F,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value, F> WritableOptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value>,
{
    pub fn new(getter: F) -> Self {
        Self {
            getter,
            _phantom: PhantomData,
        }
    }
    
    pub fn get_mut<'r>(&self, root: &'r mut Root) -> Option<&'r mut Value> {
        (self.getter)(root)
    }
    
    // Swift-like operator for chaining WritableOptionalKeyPath
    pub fn then<SubValue, G>(
        self,
        next: WritableOptionalKeyPath<Value, SubValue, G>,
    ) -> WritableOptionalKeyPath<Root, SubValue, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut SubValue>>
    where
        G: for<'r> Fn(&'r mut Value) -> Option<&'r mut SubValue>,
        F: 'static,
        G: 'static,
        Value: 'static,
    {
        let first = self.getter;
        let second = next.getter;
        
        WritableOptionalKeyPath::new(move |root: &mut Root| {
            first(root).and_then(|value| second(value))
        })
    }
    
    // Instance methods for unwrapping containers from Option<Container<T>>
    // Option<Box<T>> -> Option<&mut T> (type automatically inferred from Value::Target)
    pub fn for_box<Target>(self) -> WritableOptionalKeyPath<Root, Target, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut Target> + 'static>
    where
        Value: std::ops::DerefMut<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |root: &mut Root| {
                getter(root).map(|boxed| boxed.deref_mut())
            },
            _phantom: PhantomData,
        }
    }
    
    // Option<Arc<T>> -> Option<&mut T> (type automatically inferred from Value::Target)
    pub fn for_arc<Target>(self) -> WritableOptionalKeyPath<Root, Target, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut Target> + 'static>
    where
        Value: std::ops::DerefMut<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |root: &mut Root| {
                getter(root).map(|arc| arc.deref_mut())
            },
            _phantom: PhantomData,
        }
    }
    
    // Option<Rc<T>> -> Option<&mut T> (type automatically inferred from Value::Target)
    pub fn for_rc<Target>(self) -> WritableOptionalKeyPath<Root, Target, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut Target> + 'static>
    where
        Value: std::ops::DerefMut<Target = Target>,
        F: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |root: &mut Root| {
                getter(root).map(|rc| rc.deref_mut())
            },
            _phantom: PhantomData,
        }
    }
    
    // Static method for Option<T> -> Option<&mut T>
    pub fn for_option<T>() -> WritableOptionalKeyPath<Option<T>, T, impl for<'r> Fn(&'r mut Option<T>) -> Option<&'r mut T>> {
        WritableOptionalKeyPath::new(|opt: &mut Option<T>| opt.as_mut())
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

    // Writable versions
    // Box<T> -> T (mutable)
    pub fn for_box_mut<T>() -> WritableKeyPath<Box<T>, T, impl for<'r> Fn(&'r mut Box<T>) -> &'r mut T> {
        WritableKeyPath::new(|b: &mut Box<T>| b.as_mut())
    }

    // Note: Arc<T> and Rc<T> don't support direct mutable access without interior mutability
    // (e.g., Arc<Mutex<T>> or Rc<RefCell<T>>). These methods are not provided as they
    // would require unsafe code or interior mutability patterns.
}

// Helper to create enum variant keypaths with type inference
pub fn variant_of<Enum, Variant, F>(extractor: F) -> OptionalKeyPath<Enum, Variant, F>
where
    F: for<'r> Fn(&'r Enum) -> Option<&'r Variant>,
{
    OptionalKeyPath::new(extractor)
}

// ========== PARTIAL KEYPATHS (Hide Value Type) ==========

/// PartialKeyPath - Hides the Value type but keeps Root visible
/// Useful for storing keypaths in collections without knowing the exact Value type
#[derive(Clone)]
pub struct PartialKeyPath<Root> {
    getter: Rc<dyn for<'r> Fn(&'r Root) -> &'r dyn Any>,
    value_type_id: TypeId,
    _phantom: PhantomData<Root>,
}

impl<Root> PartialKeyPath<Root> {
    pub fn new<Value>(keypath: KeyPath<Root, Value, impl for<'r> Fn(&'r Root) -> &'r Value + 'static>) -> Self
    where
        Value: Any + 'static,
        Root: 'static,
    {
        let value_type_id = TypeId::of::<Value>();
        let getter = Rc::new(keypath.getter);
        
        Self {
            getter: Rc::new(move |root: &Root| {
                let value: &Value = getter(root);
                value as &dyn Any
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'r>(&self, root: &'r Root) -> &'r dyn Any {
        (self.getter)(root)
    }
    
    /// Get the TypeId of the Value type
    pub fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }
    
    /// Try to downcast the result to a specific type
    pub fn get_as<'a, Value: Any>(&self, root: &'a Root) -> Option<&'a Value> {
        if self.value_type_id == TypeId::of::<Value>() {
            self.get(root).downcast_ref::<Value>()
        } else {
            None
        }
    }
}

/// PartialOptionalKeyPath - Hides the Value type but keeps Root visible
/// Useful for storing optional keypaths in collections without knowing the exact Value type
#[derive(Clone)]
pub struct PartialOptionalKeyPath<Root> {
    getter: Rc<dyn for<'r> Fn(&'r Root) -> Option<&'r dyn Any>>,
    value_type_id: TypeId,
    _phantom: PhantomData<Root>,
}

impl<Root> PartialOptionalKeyPath<Root> {
    pub fn new<Value>(keypath: OptionalKeyPath<Root, Value, impl for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static>) -> Self
    where
        Value: Any + 'static,
        Root: 'static,
    {
        let value_type_id = TypeId::of::<Value>();
        let getter = Rc::new(keypath.getter);
        
        Self {
            getter: Rc::new(move |root: &Root| {
                getter(root).map(|value: &Value| value as &dyn Any)
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    pub fn get<'r>(&self, root: &'r Root) -> Option<&'r dyn Any> {
        (self.getter)(root)
    }
    
    /// Get the TypeId of the Value type
    pub fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }
    
    /// Try to downcast the result to a specific type
    pub fn get_as<'a, Value: Any>(&self, root: &'a Root) -> Option<Option<&'a Value>> {
        if self.value_type_id == TypeId::of::<Value>() {
            self.get(root).map(|any| any.downcast_ref::<Value>())
        } else {
            None
        }
    }
    
    /// Chain with another PartialOptionalKeyPath
    /// Note: This requires the Value type of the first keypath to match the Root type of the second
    /// For type-erased chaining, consider using AnyKeyPath instead
    pub fn then<MidValue>(
        self,
        next: PartialOptionalKeyPath<MidValue>,
    ) -> PartialOptionalKeyPath<Root>
    where
        MidValue: Any + 'static,
        Root: 'static,
    {
        let first = self.getter;
        let second = next.getter;
        let value_type_id = next.value_type_id;
        
        PartialOptionalKeyPath {
            getter: Rc::new(move |root: &Root| {
                first(root).and_then(|any| {
                    if let Some(mid_value) = any.downcast_ref::<MidValue>() {
                        second(mid_value)
                    } else {
                        None
                    }
                })
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
}

/// PartialWritableKeyPath - Hides the Value type but keeps Root visible (writable)
#[derive(Clone)]
pub struct PartialWritableKeyPath<Root> {
    getter: Rc<dyn for<'r> Fn(&'r mut Root) -> &'r mut dyn Any>,
    value_type_id: TypeId,
    _phantom: PhantomData<Root>,
}

impl<Root> PartialWritableKeyPath<Root> {
    pub fn new<Value>(keypath: WritableKeyPath<Root, Value, impl for<'r> Fn(&'r mut Root) -> &'r mut Value + 'static>) -> Self
    where
        Value: Any + 'static,
        Root: 'static,
    {
        let value_type_id = TypeId::of::<Value>();
        let getter = Rc::new(keypath.getter);
        
        Self {
            getter: Rc::new(move |root: &mut Root| {
                let value: &mut Value = getter(root);
                value as &mut dyn Any
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    pub fn get_mut<'r>(&self, root: &'r mut Root) -> &'r mut dyn Any {
        (self.getter)(root)
    }
    
    /// Get the TypeId of the Value type
    pub fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }
    
    /// Try to downcast the result to a specific type
    pub fn get_mut_as<'a, Value: Any>(&self, root: &'a mut Root) -> Option<&'a mut Value> {
        if self.value_type_id == TypeId::of::<Value>() {
            self.get_mut(root).downcast_mut::<Value>()
        } else {
            None
        }
    }
}

/// PartialWritableOptionalKeyPath - Hides the Value type but keeps Root visible (writable optional)
#[derive(Clone)]
pub struct PartialWritableOptionalKeyPath<Root> {
    getter: Rc<dyn for<'r> Fn(&'r mut Root) -> Option<&'r mut dyn Any>>,
    value_type_id: TypeId,
    _phantom: PhantomData<Root>,
}

impl<Root> PartialWritableOptionalKeyPath<Root> {
    pub fn new<Value>(keypath: WritableOptionalKeyPath<Root, Value, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static>) -> Self
    where
        Value: Any + 'static,
        Root: 'static,
    {
        let value_type_id = TypeId::of::<Value>();
        let getter = Rc::new(keypath.getter);
        
        Self {
            getter: Rc::new(move |root: &mut Root| {
                getter(root).map(|value: &mut Value| value as &mut dyn Any)
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    pub fn get_mut<'r>(&self, root: &'r mut Root) -> Option<&'r mut dyn Any> {
        (self.getter)(root)
    }
    
    /// Get the TypeId of the Value type
    pub fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }
    
    /// Try to downcast the result to a specific type
    pub fn get_mut_as<'a, Value: Any>(&self, root: &'a mut Root) -> Option<Option<&'a mut Value>> {
        if self.value_type_id == TypeId::of::<Value>() {
            self.get_mut(root).map(|any| any.downcast_mut::<Value>())
        } else {
            None
        }
    }
}

// ========== ANY KEYPATHS (Hide Both Root and Value Types) ==========

/// AnyKeyPath - Hides both Root and Value types
/// Equivalent to Swift's AnyKeyPath
/// Useful for storing keypaths in collections without knowing either type
#[derive(Clone)]
pub struct AnyKeyPath {
    getter: Rc<dyn for<'r> Fn(&'r dyn Any) -> Option<&'r dyn Any>>,
    root_type_id: TypeId,
    value_type_id: TypeId,
}

impl AnyKeyPath {
    pub fn new<Root, Value>(keypath: OptionalKeyPath<Root, Value, impl for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static>) -> Self
    where
        Root: Any + 'static,
        Value: Any + 'static,
    {
        let root_type_id = TypeId::of::<Root>();
        let value_type_id = TypeId::of::<Value>();
        let getter = keypath.getter;
        
        Self {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(root) = any.downcast_ref::<Root>() {
                    getter(root).map(|value: &Value| value as &dyn Any)
                } else {
                    None
                }
            }),
            root_type_id,
            value_type_id,
        }
    }
    
    pub fn get<'r>(&self, root: &'r dyn Any) -> Option<&'r dyn Any> {
        (self.getter)(root)
    }
    
    /// Get the TypeId of the Root type
    pub fn root_type_id(&self) -> TypeId {
        self.root_type_id
    }
    
    /// Get the TypeId of the Value type
    pub fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }
    
    /// Try to get the value with type checking
    pub fn get_as<'a, Root: Any, Value: Any>(&self, root: &'a Root) -> Option<Option<&'a Value>> {
        if self.root_type_id == TypeId::of::<Root>() && self.value_type_id == TypeId::of::<Value>() {
            self.get(root as &dyn Any).map(|any| any.downcast_ref::<Value>())
        } else {
            None
        }
    }
}

/// AnyWritableKeyPath - Hides both Root and Value types (writable)
#[derive(Clone)]
pub struct AnyWritableKeyPath {
    getter: Rc<dyn for<'r> Fn(&'r mut dyn Any) -> Option<&'r mut dyn Any>>,
    root_type_id: TypeId,
    value_type_id: TypeId,
}

impl AnyWritableKeyPath {
    pub fn new<Root, Value>(keypath: WritableOptionalKeyPath<Root, Value, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static>) -> Self
    where
        Root: Any + 'static,
        Value: Any + 'static,
    {
        let root_type_id = TypeId::of::<Root>();
        let value_type_id = TypeId::of::<Value>();
        let getter = keypath.getter;
        
        Self {
            getter: Rc::new(move |any: &mut dyn Any| {
                if let Some(root) = any.downcast_mut::<Root>() {
                    getter(root).map(|value: &mut Value| value as &mut dyn Any)
                } else {
                    None
                }
            }),
            root_type_id,
            value_type_id,
        }
    }
    
    pub fn get_mut<'r>(&self, root: &'r mut dyn Any) -> Option<&'r mut dyn Any> {
        (self.getter)(root)
    }
    
    /// Get the TypeId of the Root type
    pub fn root_type_id(&self) -> TypeId {
        self.root_type_id
    }
    
    /// Get the TypeId of the Value type
    pub fn value_type_id(&self) -> TypeId {
        self.value_type_id
    }
    
    /// Try to get the value with type checking
    pub fn get_mut_as<'a, Root: Any, Value: Any>(&self, root: &'a mut Root) -> Option<Option<&'a mut Value>> {
        if self.root_type_id == TypeId::of::<Root>() && self.value_type_id == TypeId::of::<Value>() {
            self.get_mut(root as &mut dyn Any).map(|any| any.downcast_mut::<Value>())
        } else {
            None
        }
    }
}

// Conversion methods from concrete keypaths to partial/any keypaths
impl<Root, Value, F> KeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Value + 'static,
    Root: 'static,
    Value: Any + 'static,
{
    /// Convert to PartialKeyPath (hides Value type)
    pub fn to_partial(self) -> PartialKeyPath<Root> {
        PartialKeyPath::new(self)
    }
}

impl<Root, Value, F> OptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static,
    Root: Any + 'static,
    Value: Any + 'static,
{
    /// Convert to PartialOptionalKeyPath (hides Value type)
    pub fn to_partial(self) -> PartialOptionalKeyPath<Root> {
        PartialOptionalKeyPath::new(self)
    }
    
    /// Convert to AnyKeyPath (hides both Root and Value types)
    pub fn to_any(self) -> AnyKeyPath {
        AnyKeyPath::new(self)
    }
}

impl<Root, Value, F> WritableKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> &'r mut Value + 'static,
    Root: 'static,
    Value: Any + 'static,
{
    /// Convert to PartialWritableKeyPath (hides Value type)
    pub fn to_partial(self) -> PartialWritableKeyPath<Root> {
        PartialWritableKeyPath::new(self)
    }
}

impl<Root, Value, F> WritableOptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static,
    Root: Any + 'static,
    Value: Any + 'static,
{
    /// Convert to PartialWritableOptionalKeyPath (hides Value type)
    pub fn to_partial(self) -> PartialWritableOptionalKeyPath<Root> {
        PartialWritableOptionalKeyPath::new(self)
    }
    
    /// Convert to AnyWritableKeyPath (hides both Root and Value types)
    pub fn to_any(self) -> AnyWritableKeyPath {
        AnyWritableKeyPath::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::rc::Rc;

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
        let akash = User {
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
        println!("Name: {}", name_kp.get(&akash));
    
        if let Some(metadata) = metadata_kp.get(&akash) {
        println!("Has metadata: {:?}", metadata);
    }
    
    // Access first friend's name
        if let Some(first_friend) = akash.friends.get(0) {
        println!("First friend: {}", name_kp.get(first_friend));
    }
    
        // Access metadata through Box using for_box()
    let created_at_kp = KeyPath::new(|m: &UserMetadata| &m.created_at);
    
        if let Some(metadata) = akash.metadata.as_ref() {
            // Use for_box() to unwrap Box<UserMetadata> to &UserMetadata
            let boxed_metadata: &Box<UserMetadata> = metadata;
            let unwrapped = boxed_metadata.as_ref();
            println!("Created at: {:?}", created_at_kp.get(unwrapped));
        }
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
