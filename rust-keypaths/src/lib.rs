// Enable feature gate when nightly feature is enabled
// NOTE: This will only work with nightly Rust toolchain
// #![cfg_attr(feature = "nightly", feature(impl_trait_in_assoc_type))]

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use std::marker::PhantomData;
use std::any::{Any, TypeId};
use std::rc::Rc;
use std::cell::RefCell;
// use std::ops::Shr;
use std::fmt;

// ========== FUNCTIONAL KEYPATH CHAIN (Compose first, apply container at get) ==========

#[derive(Debug)]
pub struct LKp<Root, MutexValue, InnerValue, SubValue, G, S>
where
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    S: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    o: Kp<Root, MutexValue>,
    i: KpType<InnerValue, SubValue, G, S>,
}

// Helper function to create LKp from simple keypaths
impl<Root, MutexValue, InnerValue, SubValue, G, S> LKp<Root, MutexValue, InnerValue, SubValue, G, S>
where
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    S: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Create an LKp from two simple keypaths
    /// - outer: keypath from Root to Arc<Mutex<InnerValue>>
    /// - inner: keypath from InnerValue to SubValue
    pub fn new(outer: Kp<Root, MutexValue>, inner: KpType<InnerValue, SubValue, G, S>) -> Self {
        Self { o: outer, i: inner }
    }

    pub fn then<NextValue, G2, S2>(
        self,
        next: KpType<SubValue, NextValue, G2, S2>,
    ) -> LKp<
        Root,
        MutexValue,
        InnerValue,
        NextValue,
        impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue>,
        impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue>,
    >
    where
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
        G2: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        S2: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
    {        
        LKp {
            o: self.o,
            i: self.i.then(next),
        }
    }

    /// Consider using the fn only when value is of type Rc or Arc else it will clone.
    pub fn get_cloned(&self, root: &Root) -> Option<SubValue> 
    where
        SubValue: Clone,
    {
        self.o.get(root)
            .and_then(|mutex_value| {
                let arc_mutex = mutex_value.borrow();
                let guard = arc_mutex.lock().ok()?;
                let sub_value = self.i.get(&*guard)?;
                Some(sub_value.clone())
            })
    }

    pub fn get_mut<F, R>(&self, root: &mut Root, f: F) -> Option<R>
    where
        F: FnOnce(&mut SubValue) -> R,
    {
        self.o.get_mut(root)
            .and_then(|mutex_value| {
                let arc_mutex = mutex_value.borrow();
                let mut guard = arc_mutex.lock().ok()?;
                let sub_value = self.i.get_mut(&mut *guard)?;
                Some(f(sub_value))
            })
    }

    pub fn get<F, R>(&self, root: &Root, f: F) -> Option<R>
    where
        F: FnOnce(&SubValue) -> R,
    {
        self.o.get(root)
            .and_then(|mutex_value| {
                let arc_mutex = mutex_value.borrow();
                let mut guard = arc_mutex.lock().ok()?;
                let sub_value = self.i.get(&mut *guard)?;
                Some(f(sub_value))
            })
    }



}

/// A composed keypath chain through Arc<Mutex<T>> - functional style
/// Build the chain first, then apply container at get() time
/// 
/// # Example
/// ```rust
/// // Functional style: compose first, then apply container at get()
/// ContainerTest::mutex_data_r()
///     .chain_arc_mutex_at_kp(SomeStruct::data_r())
///     .get(&container, |value| println!("Value: {}", value));
/// ```
pub struct ArcMutexKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: KeyPath<Root, MutexValue, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

impl<Root, MutexValue, InnerValue, SubValue, F, G> ArcMutexKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container, executing callback with the value (read)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_mutex_ref = self.outer_keypath.get(container);
        arc_mutex_ref.borrow().lock().ok().map(|guard| {
            let value = self.inner_keypath.get(&*guard);
            callback(value)
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcMutexKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcMutexKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== WRITABLE MUTEX KEYPATH CHAINS ==========

/// A composed writable keypath chain through Arc<Mutex<T>> - functional style
/// Build the chain first, then apply container at get_mut() time
pub struct ArcMutexWritableKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: KeyPath<Root, MutexValue, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

impl<Root, MutexValue, InnerValue, SubValue, F, G> ArcMutexWritableKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container with mutable access
    /// Consumes self - functional style (compose once, apply once)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_mutex_ref = self.outer_keypath.get(container);
        arc_mutex_ref.borrow().lock().ok().map(|mut guard| {
            let value_ref = self.inner_keypath.get_mut(&mut *guard);
            callback(value_ref)
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcMutexWritableKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcMutexWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional keypath chain through Arc<Mutex<T>> - functional style
pub struct ArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: KeyPath<Root, MutexValue, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

impl<Root, MutexValue, InnerValue, SubValue, F, G> ArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container with mutable access (if value exists)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_mutex_ref = self.outer_keypath.get(container);
        arc_mutex_ref.borrow().lock().ok().and_then(|mut guard| {
            self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        ArcMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        ArcMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional keypath chain through Arc<Mutex<T>> - functional style
pub struct ArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: KeyPath<Root, MutexValue, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

impl<Root, MutexValue, InnerValue, SubValue, F, G> ArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container, executing callback with the value
    /// Consumes self - functional style (compose once, apply once)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_mutex_ref = self.outer_keypath.get(container);
        arc_mutex_ref.borrow().lock().ok().and_then(|guard| {
            self.inner_keypath.get(&*guard).map(|value| callback(value))
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        ArcMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        ArcMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed keypath chain from OptionalKeyPath through Arc<Mutex<T>> - functional style
pub struct OptionalArcMutexKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: OptionalKeyPath<Root, MutexValue, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

impl<Root, MutexValue, InnerValue, SubValue, F, G> OptionalArcMutexKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container, executing callback with the value
    /// Consumes self - functional style (compose once, apply once)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        self.outer_keypath.get(container).and_then(|arc_mutex_ref| {
            arc_mutex_ref.borrow().lock().ok().map(|guard| {
                let value = self.inner_keypath.get(&*guard);
                callback(value)
            })
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcMutexKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcMutexKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional keypath chain from OptionalKeyPath through Arc<Mutex<T>> - functional style
pub struct OptionalArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, MutexValue, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

impl<Root, MutexValue, InnerValue, SubValue, F, G> OptionalArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container, executing callback with the value
    /// Consumes self - functional style (compose once, apply once)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        self.outer_keypath.get(container).and_then(|arc_mutex_ref| {
            arc_mutex_ref.borrow().lock().ok().and_then(|guard| {
                self.inner_keypath.get(&*guard).map(|value| callback(value))
            })
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        OptionalArcMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        OptionalArcMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== FUNCTIONAL RWLOCK KEYPATH CHAINS ==========

/// A composed keypath chain through Arc<RwLock<T>> - functional style
/// Build the chain first, then apply container at get() time
/// 
/// # Example
/// ```rust
/// // Functional style: compose first, then apply container at get()
/// ContainerTest::rwlock_data_r()
///     .chain_arc_rwlock_at_kp(SomeStruct::data_r())
///     .get(&container, |value| println!("Value: {}", value));
/// ```
pub struct ArcRwLockKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: KeyPath<Root, RwLockValue, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

impl<Root, RwLockValue, InnerValue, SubValue, F, G> ArcRwLockKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    RwLockValue: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container, executing callback with the value (read lock)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_rwlock_ref = self.outer_keypath.get(container);
        arc_rwlock_ref.borrow().read().ok().map(|guard| {
            let value = self.inner_keypath.get(&*guard);
            callback(value)
        })
    }

    /// Chain with another readable keypath through another level
    /// This allows composing deeper keypaths through the same Arc<RwLock<T>> structure
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcRwLockKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcRwLockKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional keypath chain through Arc<RwLock<T>> - functional style
pub struct ArcRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: KeyPath<Root, RwLockValue, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

impl<Root, RwLockValue, InnerValue, SubValue, F, G> ArcRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    RwLockValue: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container, executing callback with the value (read lock)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_rwlock_ref = self.outer_keypath.get(container);
        arc_rwlock_ref.borrow().read().ok().and_then(|guard| {
            self.inner_keypath.get(&*guard).map(|value| callback(value))
        })
    }
}

/// A composed keypath chain from OptionalKeyPath through Arc<RwLock<T>> - functional style
pub struct OptionalArcRwLockKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: OptionalKeyPath<Root, RwLockValue, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

impl<Root, RwLockValue, InnerValue, SubValue, F, G> OptionalArcRwLockKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    RwLockValue: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container, executing callback with the value (read lock)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        self.outer_keypath.get(container).and_then(|arc_rwlock_ref| {
            arc_rwlock_ref.borrow().read().ok().map(|guard| {
                let value = self.inner_keypath.get(&*guard);
                callback(value)
            })
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcRwLockKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcRwLockKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional keypath chain from OptionalKeyPath through Arc<RwLock<T>> - functional style
pub struct OptionalArcRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, RwLockValue, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

impl<Root, RwLockValue, InnerValue, SubValue, F, G> OptionalArcRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    RwLockValue: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container, executing callback with the value (read lock)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        self.outer_keypath.get(container).and_then(|arc_rwlock_ref| {
            arc_rwlock_ref.borrow().read().ok().and_then(|guard| {
                self.inner_keypath.get(&*guard).map(|value| callback(value))
            })
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        OptionalArcRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        OptionalArcRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== WRITABLE MUTEX KEYPATH CHAINS FROM OPTIONAL KEYPATHS ==========

/// A composed writable keypath chain from optional keypath through Arc<Mutex<T>> - functional style
/// Build the chain first, then apply container at get_mut() time
pub struct OptionalArcMutexWritableKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: OptionalKeyPath<Root, MutexValue, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

impl<Root, MutexValue, InnerValue, SubValue, F, G> OptionalArcMutexWritableKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container with mutable access
    /// Consumes self - functional style (compose once, apply once)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        self.outer_keypath.get(container).and_then(|arc_mutex_ref| {
            arc_mutex_ref.borrow().lock().ok().map(|mut guard| {
                let value_ref = self.inner_keypath.get_mut(&mut *guard);
                callback(value_ref)
            })
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcMutexWritableKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcMutexWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional keypath chain from optional keypath through Arc<Mutex<T>> - functional style
/// Build the chain first, then apply container at get_mut() time
pub struct OptionalArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, MutexValue, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

impl<Root, MutexValue, InnerValue, SubValue, F, G> OptionalArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    MutexValue: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container with mutable access (if value exists)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        self.outer_keypath.get(container).and_then(|arc_mutex_ref| {
            arc_mutex_ref.borrow().lock().ok().and_then(|mut guard| {
                self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
            })
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        OptionalArcMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        OptionalArcMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== WRITABLE RWLOCK KEYPATH CHAINS FROM OPTIONAL KEYPATHS ==========

/// A composed writable keypath chain from optional keypath through Arc<RwLock<T>> - functional style
/// Build the chain first, then apply container at get_mut() time (uses write lock)
pub struct OptionalArcRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: OptionalKeyPath<Root, RwLockValue, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

impl<Root, RwLockValue, InnerValue, SubValue, F, G> OptionalArcRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    RwLockValue: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        self.outer_keypath.get(container).and_then(|arc_rwlock_ref| {
            arc_rwlock_ref.borrow().write().ok().map(|mut guard| {
                let value_ref = self.inner_keypath.get_mut(&mut *guard);
                callback(value_ref)
            })
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcRwLockWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional keypath chain from optional keypath through Arc<RwLock<T>> - functional style
/// Build the chain first, then apply container at get_mut() time (uses write lock)
pub struct OptionalArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, RwLockValue, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

impl<Root, RwLockValue, InnerValue, SubValue, F, G> OptionalArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    RwLockValue: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock, if value exists)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        self.outer_keypath.get(container).and_then(|arc_rwlock_ref| {
            arc_rwlock_ref.borrow().write().ok().and_then(|mut guard| {
                self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
            })
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        OptionalArcRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        OptionalArcRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== WRITABLE RWLOCK KEYPATH CHAINS ==========

/// A composed writable keypath chain through Arc<RwLock<T>> - functional style
/// Build the chain first, then apply container at get_mut() time (uses write lock)
pub struct ArcRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: KeyPath<Root, RwLockValue, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

impl<Root, RwLockValue, InnerValue, SubValue, F, G> ArcRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    RwLockValue: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_rwlock_ref = self.outer_keypath.get(container);
        arc_rwlock_ref.borrow().write().ok().map(|mut guard| {
            let value_ref = self.inner_keypath.get_mut(&mut *guard);
            callback(value_ref)
        })
    }

    /// Monadic composition: chain with another writable keypath
    /// This allows composing deeper keypaths through the same Arc<RwLock<T>> structure
    /// 
    /// # Example
    /// ```rust,ignore
    /// // Compose: Root -> Arc<RwLock<InnerValue>> -> InnerValue -> SubValue -> NextValue
    /// let chain = root_keypath
    ///     .chain_arc_rwlock_writable_at_kp(inner_keypath)
    ///     .then(next_keypath);
    /// ```
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        // Create a new composed writable keypath
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcRwLockWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Monadic composition: chain with a writable optional keypath (for Option fields)
    /// This allows composing through Option types within the same Arc<RwLock<T>> structure
    /// 
    /// # Example
    /// ```rust,ignore
    /// // Compose: Root -> Arc<RwLock<InnerValue>> -> InnerValue -> Option<SubValue> -> NextValue
    /// let chain = root_keypath
    ///     .chain_arc_rwlock_writable_at_kp(inner_keypath)
    ///     .chain_optional(optional_keypath);
    /// ```
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        // Create a new composed writable optional keypath
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional keypath chain through Arc<RwLock<T>> - functional style
/// Build the chain first, then apply container at get_mut() time (uses write lock)
pub struct ArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: KeyPath<Root, RwLockValue, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

impl<Root, RwLockValue, InnerValue, SubValue, F, G> ArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    RwLockValue: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock, if value exists)
    /// Consumes self - functional style (compose once, apply once)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_rwlock_ref = self.outer_keypath.get(container);
        arc_rwlock_ref.borrow().write().ok().and_then(|mut guard| {
            self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
        })
    }

    /// Monadic composition: chain with another writable keypath
    /// This allows composing deeper keypaths through the same Arc<RwLock<T>> structure
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        // Create a new composed writable optional keypath
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            if let Some(sub) = first.get_mut(inner) {
                Some(second.get_mut(sub))
            } else {
                None
            }
        });
        
        ArcRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Monadic composition: chain with another writable optional keypath (for Option fields)
    /// This allows composing through Option types within the same Arc<RwLock<T>> structure
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        // Create a new composed writable optional keypath
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        ArcRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== PARKING_LOT MUTEX/RWLOCK CHAIN TYPES ==========

#[cfg(feature = "parking_lot")]
use parking_lot::{Mutex as ParkingMutex, RwLock as ParkingRwLock};

/// A composed keypath chain through Arc<parking_lot::Mutex<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct ArcParkingMutexKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: KeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> ArcParkingMutexKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    /// Apply the composed keypath chain to a container (read)
    pub fn get<Callback>(self, container: &Root, callback: Callback)
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_mutex_ref = self.outer_keypath.get(container);
        let guard = arc_mutex_ref.lock();
        let value = self.inner_keypath.get(&*guard);
        callback(value);
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingMutexKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcParkingMutexKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingMutexOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcParkingMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional keypath chain through Arc<parking_lot::Mutex<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct ArcParkingMutexOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: KeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> ArcParkingMutexOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    /// Apply the composed keypath chain to a container (read, if value exists)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_mutex_ref = self.outer_keypath.get(container);
        let guard = arc_mutex_ref.lock();
        self.inner_keypath.get(&*guard).map(|value| callback(value))
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingMutexOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        ArcParkingMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingMutexOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        ArcParkingMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable keypath chain through Arc<parking_lot::Mutex<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct ArcParkingMutexWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: KeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> ArcParkingMutexWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    /// Apply the composed keypath chain to a container with mutable access
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> R
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_mutex_ref = self.outer_keypath.get(container);
        let mut guard = arc_mutex_ref.lock();
        let value_ref = self.inner_keypath.get_mut(&mut *guard);
        callback(value_ref)
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingMutexWritableKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcParkingMutexWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcParkingMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional keypath chain through Arc<parking_lot::Mutex<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct ArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: KeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> ArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Apply the composed keypath chain to a container with mutable access (if value exists)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_mutex_ref = self.outer_keypath.get(container);
        let mut guard = arc_mutex_ref.lock();
        self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        ArcParkingMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        ArcParkingMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed keypath chain through Arc<parking_lot::RwLock<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct ArcParkingRwLockKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: KeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> ArcParkingRwLockKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    /// Apply the composed keypath chain to a container (read lock)
    pub fn get<Callback>(self, container: &Root, callback: Callback)
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_rwlock_ref = self.outer_keypath.get(container);
        let guard = arc_rwlock_ref.read();
        let value = self.inner_keypath.get(&*guard);
        callback(value);
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingRwLockKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcParkingRwLockKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcParkingRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional keypath chain through Arc<parking_lot::RwLock<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct ArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: KeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> ArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    /// Apply the composed keypath chain to a container (read lock, if value exists)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_rwlock_ref = self.outer_keypath.get(container);
        let guard = arc_rwlock_ref.read();
        self.inner_keypath.get(&*guard).map(|value| callback(value))
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        ArcParkingRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        ArcParkingRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable keypath chain through Arc<parking_lot::RwLock<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct ArcParkingRwLockWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: KeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> ArcParkingRwLockWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> R
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_rwlock_ref = self.outer_keypath.get(container);
        let mut guard = arc_rwlock_ref.write();
        let value_ref = self.inner_keypath.get_mut(&mut *guard);
        callback(value_ref)
    }

    /// Monadic composition: chain with another writable keypath
    /// This allows composing deeper keypaths through the same Arc<parking_lot::RwLock<T>> structure
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingRwLockWritableKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        // Create a new composed writable keypath
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcParkingRwLockWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Monadic composition: chain with a writable optional keypath (for Option fields)
    /// This allows composing through Option types within the same Arc<parking_lot::RwLock<T>> structure
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        // Create a new composed writable optional keypath
        // first.get_mut returns &mut SubValue (not Option), second.get_mut returns Option<&mut NextValue>
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcParkingRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional keypath chain through Arc<parking_lot::RwLock<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct ArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: KeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> ArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock, if value exists)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_rwlock_ref = self.outer_keypath.get(container);
        let mut guard = arc_rwlock_ref.write();
        self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
    }

    /// Monadic composition: chain with another writable keypath
    /// This allows composing deeper keypaths through the same Arc<parking_lot::RwLock<T>> structure
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first_keypath = self.inner_keypath;
        let second_keypath = next;
        
        // Create a new composed writable optional keypath
        // first_keypath.get_mut returns Option<&mut SubValue>, second_keypath.get_mut returns &mut NextValue
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first_keypath.get_mut(inner).map(|sub| second_keypath.get_mut(sub))
        });
        
        ArcParkingRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Monadic composition: chain with another writable optional keypath (for Option fields)
    /// This allows composing through Option types within the same Arc<parking_lot::RwLock<T>> structure
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        // Create a new composed writable optional keypath
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        ArcParkingRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== OPTIONAL PARKING_LOT MUTEX KEYPATH CHAINS (from OptionalKeyPath) ==========

/// A composed keypath chain from optional keypath through Arc<parking_lot::Mutex<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct OptionalArcParkingMutexKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: OptionalKeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> OptionalArcParkingMutexKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    /// Apply the composed keypath chain to a container
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        self.outer_keypath.get(container).map(|arc_mutex_ref| {
            let guard = arc_mutex_ref.lock();
            let value = self.inner_keypath.get(&*guard);
            callback(value)
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingMutexKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcParkingMutexKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingMutexOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcParkingMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional keypath chain from optional keypath through Arc<parking_lot::Mutex<T>>
#[cfg(feature = "parking_lot")]
pub struct OptionalArcParkingMutexOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> OptionalArcParkingMutexOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    /// Apply the composed keypath chain to a container
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        self.outer_keypath.get(container).and_then(|arc_mutex_ref| {
            let guard = arc_mutex_ref.lock();
            self.inner_keypath.get(&*guard).map(|value| callback(value))
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingMutexOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        OptionalArcParkingMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingMutexOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        OptionalArcParkingMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable keypath chain from optional keypath through Arc<parking_lot::Mutex<T>>
#[cfg(feature = "parking_lot")]
pub struct OptionalArcParkingMutexWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: OptionalKeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> OptionalArcParkingMutexWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    /// Apply the composed keypath chain to a container with mutable access
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        self.outer_keypath.get(container).map(|arc_mutex_ref| {
            let mut guard = arc_mutex_ref.lock();
            let value_ref = self.inner_keypath.get_mut(&mut *guard);
            callback(value_ref)
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingMutexWritableKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcParkingMutexWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcParkingMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional keypath chain from optional keypath through Arc<parking_lot::Mutex<T>>
#[cfg(feature = "parking_lot")]
pub struct OptionalArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> OptionalArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Apply the composed keypath chain to a container with mutable access (if value exists)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        self.outer_keypath.get(container).and_then(|arc_mutex_ref| {
            let mut guard = arc_mutex_ref.lock();
            self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        OptionalArcParkingMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        OptionalArcParkingMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== OPTIONAL PARKING_LOT RWLOCK KEYPATH CHAINS (from OptionalKeyPath) ==========

/// A composed keypath chain from optional keypath through Arc<parking_lot::RwLock<T>> - functional style
#[cfg(feature = "parking_lot")]
pub struct OptionalArcParkingRwLockKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: OptionalKeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> OptionalArcParkingRwLockKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    /// Apply the composed keypath chain to a container (read lock)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        self.outer_keypath.get(container).map(|arc_rwlock_ref| {
            let guard = arc_rwlock_ref.read();
            let value = self.inner_keypath.get(&*guard);
            callback(value)
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingRwLockKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcParkingRwLockKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcParkingRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional keypath chain from optional keypath through Arc<parking_lot::RwLock<T>>
#[cfg(feature = "parking_lot")]
pub struct OptionalArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> OptionalArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    /// Apply the composed keypath chain to a container (read lock)
    pub fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        self.outer_keypath.get(container).and_then(|arc_rwlock_ref| {
            let guard = arc_rwlock_ref.read();
            self.inner_keypath.get(&*guard).map(|value| callback(value))
        })
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        OptionalArcParkingRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        OptionalArcParkingRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable keypath chain from optional keypath through Arc<parking_lot::RwLock<T>>
#[cfg(feature = "parking_lot")]
pub struct OptionalArcParkingRwLockWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: OptionalKeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> OptionalArcParkingRwLockWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        self.outer_keypath.get(container).map(|arc_rwlock_ref| {
            let mut guard = arc_rwlock_ref.write();
            let value_ref = self.inner_keypath.get_mut(&mut *guard);
            callback(value_ref)
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingRwLockWritableKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcParkingRwLockWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcParkingRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional keypath chain from optional keypath through Arc<parking_lot::RwLock<T>>
#[cfg(feature = "parking_lot")]
pub struct OptionalArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, SubValue, F, G> OptionalArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock, if value exists)
    pub fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        self.outer_keypath.get(container).and_then(|arc_rwlock_ref| {
            let mut guard = arc_rwlock_ref.write();
            self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
        })
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        OptionalArcParkingRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        OptionalArcParkingRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== TOKIO MUTEX/RWLOCK CHAIN TYPES ==========

#[cfg(feature = "tokio")]
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};

/// A composed async keypath chain through Arc<tokio::sync::Mutex<T>> - functional style
#[cfg(feature = "tokio")]
pub struct ArcTokioMutexKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: KeyPath<Root, MutexValue, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, MutexValue, InnerValue, SubValue, F, G> ArcTokioMutexKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    /// Apply the composed keypath chain to a container (read, async)
    pub async fn get<Callback>(self, container: &Root, callback: Callback)
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_mutex_ref = self.outer_keypath.get(container).borrow();
        let guard = arc_mutex_ref.lock().await;
        let value = self.inner_keypath.get(&*guard);
        callback(value);
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioMutexKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcTokioMutexKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcTokioMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional async keypath chain through Arc<tokio::sync::Mutex<T>> - functional style
#[cfg(feature = "tokio")]
pub struct ArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: KeyPath<Root, MutexValue, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, MutexValue, InnerValue, SubValue, F, G> ArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    /// Apply the composed keypath chain to a container (read, if value exists, async)
    pub async fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_mutex_ref = self.outer_keypath.get(container).borrow();
        let guard = arc_mutex_ref.lock().await;
        self.inner_keypath.get(&*guard).map(|value| callback(value))
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        ArcTokioMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        ArcTokioMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable async keypath chain through Arc<tokio::sync::Mutex<T>> - functional style
#[cfg(feature = "tokio")]
pub struct ArcTokioMutexWritableKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: KeyPath<Root, MutexValue, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, MutexValue, InnerValue, SubValue, F, G> ArcTokioMutexWritableKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    /// Apply the composed keypath chain to a container with mutable access (async)
    pub async fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> R
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_mutex_ref = self.outer_keypath.get(container).borrow();
        let mut guard = arc_mutex_ref.lock().await;
        let value_ref = self.inner_keypath.get_mut(&mut *guard);
        callback(value_ref)
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioMutexWritableKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcTokioMutexWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcTokioMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional async keypath chain through Arc<tokio::sync::Mutex<T>> - functional style
#[cfg(feature = "tokio")]
pub struct ArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: KeyPath<Root, MutexValue, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, MutexValue, InnerValue, SubValue, F, G> ArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r MutexValue,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Apply the composed keypath chain to a container with mutable access (if value exists, async)
    pub async fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_mutex_ref = self.outer_keypath.get(container).borrow();
        let mut guard = arc_mutex_ref.lock().await;
        self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        ArcTokioMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        ArcTokioMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed async keypath chain through Arc<tokio::sync::RwLock<T>> - functional style
#[cfg(feature = "tokio")]
pub struct ArcTokioRwLockKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: KeyPath<Root, RwLockValue, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, RwLockValue, InnerValue, SubValue, F, G> ArcTokioRwLockKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    /// Apply the composed keypath chain to a container (read lock, async)
    pub async fn get<Callback>(self, container: &Root, callback: Callback)
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_rwlock_ref = self.outer_keypath.get(container).borrow();
        let guard = arc_rwlock_ref.read().await;
        let value = self.inner_keypath.get(&*guard);
        callback(value);
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioRwLockKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcTokioRwLockKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        ArcTokioRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional async keypath chain through Arc<tokio::sync::RwLock<T>> - functional style
#[cfg(feature = "tokio")]
pub struct ArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: KeyPath<Root, RwLockValue, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, RwLockValue, InnerValue, SubValue, F, G> ArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    /// Apply the composed keypath chain to a container (read lock, if value exists, async)
    pub async fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        let arc_rwlock_ref = self.outer_keypath.get(container).borrow();
        let guard = arc_rwlock_ref.read().await;
        self.inner_keypath.get(&*guard).map(|value| callback(value))
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        ArcTokioRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        ArcTokioRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable async keypath chain through Arc<tokio::sync::RwLock<T>> - functional style
#[cfg(feature = "tokio")]
pub struct ArcTokioRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: KeyPath<Root, RwLockValue, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, RwLockValue, InnerValue, SubValue, F, G> ArcTokioRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock, async)
    pub async fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> R
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_rwlock_ref = self.outer_keypath.get(container).borrow();
        let mut guard = arc_rwlock_ref.write().await;
        let value_ref = self.inner_keypath.get_mut(&mut *guard);
        callback(value_ref)
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcTokioRwLockWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        ArcTokioRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional async keypath chain through Arc<tokio::sync::RwLock<T>> - functional style
#[cfg(feature = "tokio")]
pub struct ArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: KeyPath<Root, RwLockValue, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, RwLockValue, InnerValue, SubValue, F, G> ArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> &'r RwLockValue,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock, if value exists, async)
    pub async fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        let arc_rwlock_ref = self.outer_keypath.get(container).borrow();
        let mut guard = arc_rwlock_ref.write().await;
        self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        ArcTokioRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> ArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        ArcTokioRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== OPTIONAL TOKIO MUTEX KEYPATH CHAINS (from OptionalKeyPath) ==========

/// A composed async keypath chain from optional keypath through Arc<tokio::sync::Mutex<T>> - functional style
#[cfg(feature = "tokio")]
pub struct OptionalArcTokioMutexKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: OptionalKeyPath<Root, MutexValue, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, MutexValue, InnerValue, SubValue, F, G> OptionalArcTokioMutexKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    /// Apply the composed keypath chain to a container (async)
    pub async fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        if let Some(arc_mutex_ref) = self.outer_keypath.get(container) { let arc_mutex_ref = arc_mutex_ref.borrow();
            let guard = arc_mutex_ref.lock().await;
            let value = self.inner_keypath.get(&*guard);
            callback(value);
            Some(())
        } else {
            None
        }
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioMutexKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcTokioMutexKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcTokioMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional async keypath chain from optional keypath through Arc<tokio::sync::Mutex<T>>
#[cfg(feature = "tokio")]
pub struct OptionalArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, MutexValue, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, MutexValue, InnerValue, SubValue, F, G> OptionalArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    /// Apply the composed keypath chain to a container (async)
    pub async fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        if let Some(arc_mutex_ref) = self.outer_keypath.get(container) { let arc_mutex_ref = arc_mutex_ref.borrow();
            let guard = arc_mutex_ref.lock().await;
            self.inner_keypath.get(&*guard).map(|value| callback(value))
        } else {
            None
        }
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        OptionalArcTokioMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioMutexOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        OptionalArcTokioMutexOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable async keypath chain from optional keypath through Arc<tokio::sync::Mutex<T>>
#[cfg(feature = "tokio")]
pub struct OptionalArcTokioMutexWritableKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: OptionalKeyPath<Root, MutexValue, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, MutexValue, InnerValue, SubValue, F, G> OptionalArcTokioMutexWritableKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    /// Apply the composed keypath chain to a container with mutable access (async)
    pub async fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        if let Some(arc_mutex_ref) = self.outer_keypath.get(container) { let arc_mutex_ref = arc_mutex_ref.borrow();
            let mut guard = arc_mutex_ref.lock().await;
            let value_ref = self.inner_keypath.get_mut(&mut *guard);
            Some(callback(value_ref))
        } else {
            None
        }
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioMutexWritableKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcTokioMutexWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcTokioMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional async keypath chain from optional keypath through Arc<tokio::sync::Mutex<T>>
#[cfg(feature = "tokio")]
pub struct OptionalArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, MutexValue, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, MutexValue, InnerValue, SubValue, F, G> OptionalArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, SubValue, F, G>
where
    MutexValue: std::borrow::Borrow<Arc<TokioMutex<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r MutexValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Apply the composed keypath chain to a container with mutable access (if value exists, async)
    pub async fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        if let Some(arc_mutex_ref) = self.outer_keypath.get(container) { let arc_mutex_ref = arc_mutex_ref.borrow();
            let mut guard = arc_mutex_ref.lock().await;
            self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
        } else {
            None
        }
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        OptionalArcTokioMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioMutexWritableOptionalKeyPathChain<Root, MutexValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        OptionalArcTokioMutexWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

// ========== OPTIONAL TOKIO RWLOCK KEYPATH CHAINS (from OptionalKeyPath) ==========

/// A composed async keypath chain from optional keypath through Arc<tokio::sync::RwLock<T>> - functional style
#[cfg(feature = "tokio")]
pub struct OptionalArcTokioRwLockKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    outer_keypath: OptionalKeyPath<Root, RwLockValue, F>,
    inner_keypath: KeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, RwLockValue, InnerValue, SubValue, F, G> OptionalArcTokioRwLockKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
{
    /// Apply the composed keypath chain to a container (read lock, async)
    pub async fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        if let Some(arc_rwlock_ref) = self.outer_keypath.get(container) { let arc_rwlock_ref = arc_rwlock_ref.borrow();
            let guard = arc_rwlock_ref.read().await;
            let value = self.inner_keypath.get(&*guard);
            callback(value);
            Some(())
        } else {
            None
        }
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioRwLockKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r NextValue + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = KeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcTokioRwLockKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            let sub = first.get(inner);
            second.get(sub)
        });
        
        OptionalArcTokioRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed optional async keypath chain from optional keypath through Arc<tokio::sync::RwLock<T>>
#[cfg(feature = "tokio")]
pub struct OptionalArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, RwLockValue, F>,
    inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, RwLockValue, InnerValue, SubValue, F, G> OptionalArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
{
    /// Apply the composed keypath chain to a container (read lock, async)
    pub async fn get<Callback>(self, container: &Root, callback: Callback) -> Option<()>
    where
        Callback: FnOnce(&SubValue) -> (),
    {
        if let Some(arc_rwlock_ref) = self.outer_keypath.get(container) { let arc_rwlock_ref = arc_rwlock_ref.borrow();
            let guard = arc_rwlock_ref.read().await;
            self.inner_keypath.get(&*guard).map(|value| callback(value))
        } else {
            None
        }
    }

    /// Chain with another readable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: KeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> &'r NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).map(|sub| second.get(sub))
        });
        
        OptionalArcTokioRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional readable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: OptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioRwLockOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r InnerValue) -> Option<&'r NextValue> + 'static>
    where
        H: for<'r> Fn(&'r SubValue) -> Option<&'r NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = OptionalKeyPath::new(move |inner: &InnerValue| {
            first.get(inner).and_then(|sub| second.get(sub))
        });
        
        OptionalArcTokioRwLockOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable async keypath chain from optional keypath through Arc<tokio::sync::RwLock<T>>
#[cfg(feature = "tokio")]
pub struct OptionalArcTokioRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    outer_keypath: OptionalKeyPath<Root, RwLockValue, F>,
    inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, RwLockValue, InnerValue, SubValue, F, G> OptionalArcTokioRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock, async)
    pub async fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        if let Some(arc_rwlock_ref) = self.outer_keypath.get(container) { let arc_rwlock_ref = arc_rwlock_ref.borrow();
            let mut guard = arc_rwlock_ref.write().await;
            let value_ref = self.inner_keypath.get_mut(&mut *guard);
            Some(callback(value_ref))
        } else {
            None
        }
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioRwLockWritableKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> &'r mut NextValue + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcTokioRwLockWritableKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            let sub = first.get_mut(inner);
            second.get_mut(sub)
        });
        
        OptionalArcTokioRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

/// A composed writable optional async keypath chain from optional keypath through Arc<tokio::sync::RwLock<T>>
#[cfg(feature = "tokio")]
pub struct OptionalArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    outer_keypath: OptionalKeyPath<Root, RwLockValue, F>,
    inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
}

#[cfg(feature = "tokio")]
impl<Root, RwLockValue, InnerValue, SubValue, F, G> OptionalArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, SubValue, F, G>
where
    RwLockValue: std::borrow::Borrow<Arc<TokioRwLock<InnerValue>>>,
    F: for<'r> Fn(&'r Root) -> Option<&'r RwLockValue>,
    G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
{
    /// Apply the composed keypath chain to a container with mutable access (write lock, if value exists, async)
    pub async fn get_mut<Callback, R>(self, container: &Root, callback: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut SubValue) -> R,
    {
        if let Some(arc_rwlock_ref) = self.outer_keypath.get(container) { let arc_rwlock_ref = arc_rwlock_ref.borrow();
            let mut guard = arc_rwlock_ref.write().await;
            self.inner_keypath.get_mut(&mut *guard).map(|value_ref| callback(value_ref))
        } else {
            None
        }
    }

    /// Chain with another writable keypath through another level
    pub fn then<NextValue, H>(
        self,
        next: WritableKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> &'r mut NextValue,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).map(|sub| second.get_mut(sub))
        });
        
        OptionalArcTokioRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }

    /// Chain with an optional writable keypath through another level
    pub fn chain_optional<NextValue, H>(
        self,
        next: WritableOptionalKeyPath<SubValue, NextValue, H>,
    ) -> OptionalArcTokioRwLockWritableOptionalKeyPathChain<Root, RwLockValue, InnerValue, NextValue, F, impl for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut NextValue> + 'static>
    where
        H: for<'r> Fn(&'r mut SubValue) -> Option<&'r mut NextValue>,
        G: 'static,
        H: 'static,
        InnerValue: 'static,
        SubValue: 'static,
        NextValue: 'static,
    {
        let first = self.inner_keypath;
        let second = next;
        
        let composed = WritableOptionalKeyPath::new(move |inner: &mut InnerValue| {
            first.get_mut(inner).and_then(|sub| second.get_mut(sub))
        });
        
        OptionalArcTokioRwLockWritableOptionalKeyPathChain {
            outer_keypath: self.outer_keypath,
            inner_keypath: composed,
        }
    }
}

#[cfg(feature = "tagged")]
use tagged_core::Tagged;


// ========== HELPER MACROS FOR KEYPATH CREATION ==========

/// Macro to create a `KeyPath` (readable, non-optional)
/// 
/// # Examples
/// 
/// ```rust
/// use rust_keypaths::keypath;
/// 
/// struct User { name: String, address: Address }
/// struct Address { street: String }
/// 
/// // Using a closure with type annotation
/// let kp = keypath!(|u: &User| &u.name);
/// 
/// // Nested field access
/// let kp = keypath!(|u: &User| &u.address.street);
/// 
/// // Or with automatic type inference
/// let kp = keypath!(|u| &u.name);
/// ```
#[macro_export]
macro_rules! keypath {
    // Accept a closure directly
    ($closure:expr) => {
        $crate::KeyPath::new($closure)
    };
}

/// Macro to create an `OptionalKeyPath` (readable, optional)
/// 
/// # Examples
/// 
/// ```rust
/// use rust_keypaths::opt_keypath;
/// 
/// struct User { metadata: Option<String>, address: Option<Address> }
/// struct Address { street: String }
/// 
/// // Using a closure with type annotation
/// let kp = opt_keypath!(|u: &User| u.metadata.as_ref());
/// 
/// // Nested field access through Option
/// let kp = opt_keypath!(|u: &User| u.address.as_ref().map(|a| &a.street));
/// 
/// // Or with automatic type inference
/// let kp = opt_keypath!(|u| u.metadata.as_ref());
/// ```
#[macro_export]
macro_rules! opt_keypath {
    // Accept a closure directly
    ($closure:expr) => {
        $crate::OptionalKeyPath::new($closure)
    };
}

/// Macro to create a `WritableKeyPath` (writable, non-optional)
/// 
/// # Examples
/// 
/// ```rust
/// use rust_keypaths::writable_keypath;
/// 
/// struct User { name: String, address: Address }
/// struct Address { street: String }
/// 
/// // Using a closure with type annotation
/// let kp = writable_keypath!(|u: &mut User| &mut u.name);
/// 
/// // Nested field access
/// let kp = writable_keypath!(|u: &mut User| &mut u.address.street);
/// 
/// // Or with automatic type inference
/// let kp = writable_keypath!(|u| &mut u.name);
/// ```
#[macro_export]
macro_rules! writable_keypath {
    // Accept a closure directly
    ($closure:expr) => {
        $crate::WritableKeyPath::new($closure)
    };
}

/// Macro to create a `WritableOptionalKeyPath` (writable, optional)
/// 
/// # Examples
/// 
/// ```rust
/// use rust_keypaths::writable_opt_keypath;
/// 
/// struct User { metadata: Option<String>, address: Option<Address> }
/// struct Address { street: String }
/// 
/// // Using a closure with type annotation
/// let kp = writable_opt_keypath!(|u: &mut User| u.metadata.as_mut());
/// 
/// // Nested field access through Option
/// let kp = writable_opt_keypath!(|u: &mut User| u.address.as_mut().map(|a| &mut a.street));
/// 
/// // Or with automatic type inference
/// let kp = writable_opt_keypath!(|u| u.metadata.as_mut());
/// ```
#[macro_export]
macro_rules! writable_opt_keypath {
    // Accept a closure directly
    ($closure:expr) => {
        $crate::WritableOptionalKeyPath::new($closure)
    };
}
// ========== BASE KEYPATH TYPES ==========
// Base KeyPath
#[derive(Clone)]
pub struct KeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Value,
{
    getter: F,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value, F> fmt::Display for KeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Value,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root_name = std::any::type_name::<Root>();
        let value_name = std::any::type_name::<Value>();
        // Simplify type names by removing module paths for cleaner output
        let root_short = root_name.split("::").last().unwrap_or(root_name);
        let value_short = value_name.split("::").last().unwrap_or(value_name);
        write!(f, "KeyPath<{} -> {}>", root_short, value_short)
    }
}

impl<Root, Value, F> fmt::Debug for KeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Value,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
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
    
    // Using fn pointer - works for identity
    // pub fn identity() -> KeyPath<Root, Root, fn(&Root) -> &Root> {
    //     KeyPath {
    //         getter: (|x: &Root| x) as fn(&Root) -> &Root,
    //         _phantom: PhantomData,
    //     }
    // }

    // pub fn leaf() -> KeyPath<Value, Value, fn(&Value) -> &Value> {
    //     KeyPath {
    //         getter: (|x: &Value| x) as fn(&Value) -> &Value,
    //         _phantom: PhantomData,
    //     }
    // }


    /// Chain this keypath with an inner keypath through Arc<Mutex<T>> - functional style
    /// Compose first, then apply container at get() time
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { mutex_data: Arc::new(Mutex::new(SomeStruct { data: "test".to_string() })) };
    /// 
    /// // Functional style: compose first, apply container at get()
    /// ContainerTest::mutex_data_r()
    ///     .chain_arc_mutex_at_kp(SomeStruct::data_r())
    ///     .get(&container, |value| println!("Data: {}", value));
    /// ```
    pub fn chain_arc_mutex_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> ArcMutexKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        ArcMutexKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with an optional inner keypath through Arc<Mutex<T>> - functional style
    /// Compose first, then apply container at get() time
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { mutex_data: Arc::new(Mutex::new(SomeStruct { optional_field: Some("test".to_string()) })) };
    /// 
    /// // Functional style: compose first, apply container at get()
    /// ContainerTest::mutex_data_r()
    ///     .chain_arc_mutex_optional_at_kp(SomeStruct::optional_field_fr())
    ///     .get(&container, |value| println!("Value: {}", value));
    /// ```
    pub fn chain_arc_mutex_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcMutexOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        ArcMutexOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with an inner keypath through Arc<RwLock<T>> - functional style
    /// Compose first, then apply container at get() time (uses read lock)
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { rwlock_data: Arc::new(RwLock::new(SomeStruct { data: "test".to_string() })) };
    /// 
    /// // Functional style: compose first, apply container at get()
    /// ContainerTest::rwlock_data_r()
    ///     .chain_arc_rwlock_at_kp(SomeStruct::data_r())
    ///     .get(&container, |value| println!("Data: {}", value));
    /// ```
    pub fn chain_arc_rwlock_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> ArcRwLockKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        ArcRwLockKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with an optional inner keypath through Arc<RwLock<T>> - functional style
    /// Compose first, then apply container at get() time (uses read lock)
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { rwlock_data: Arc::new(RwLock::new(SomeStruct { optional_field: Some("test".to_string()) })) };
    /// 
    /// // Functional style: compose first, apply container at get()
    /// ContainerTest::rwlock_data_r()
    ///     .chain_arc_rwlock_optional_at_kp(SomeStruct::optional_field_fr())
    ///     .get(&container, |value| println!("Value: {}", value));
    /// ```
    pub fn chain_arc_rwlock_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcRwLockOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        ArcRwLockOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with a writable inner keypath through Arc<Mutex<T>> - functional style
    /// Compose first, then apply container at get_mut() time
    /// 
    /// # Example
    /// ```rust
    /// ContainerTest::mutex_data_r()
    ///     .chain_arc_mutex_writable_at_kp(SomeStruct::data_w())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn chain_arc_mutex_writable_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> ArcMutexWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        ArcMutexWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with a writable optional inner keypath through Arc<Mutex<T>> - functional style
    /// Compose first, then apply container at get_mut() time
    /// 
    /// # Example
    /// ```rust
    /// ContainerTest::mutex_data_r()
    ///     .chain_arc_mutex_writable_optional_at_kp(SomeStruct::optional_field_fw())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn chain_arc_mutex_writable_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcMutexWritableOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        ArcMutexWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with a writable inner keypath through Arc<RwLock<T>> - functional style
    /// Compose first, then apply container at get_mut() time (uses write lock)
    /// 
    /// # Example
    /// ```rust
    /// ContainerTest::rwlock_data_r()
    ///     .chain_arc_rwlock_writable_at_kp(SomeStruct::data_w())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn chain_arc_rwlock_writable_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> ArcRwLockWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        ArcRwLockWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with a writable optional inner keypath through Arc<RwLock<T>> - functional style
    /// Compose first, then apply container at get_mut() time (uses write lock)
    /// 
    /// # Example
    /// ```rust
    /// ContainerTest::rwlock_data_r()
    ///     .chain_arc_rwlock_writable_optional_at_kp(SomeStruct::optional_field_fw())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn chain_arc_rwlock_writable_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcRwLockWritableOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        ArcRwLockWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this keypath with an inner keypath through Arc<tokio::sync::Mutex<T>> - functional style (async)
    /// Compose first, then apply container at get() time
    pub fn chain_arc_tokio_mutex_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> ArcTokioMutexKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        ArcTokioMutexKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this keypath with an optional inner keypath through Arc<tokio::sync::Mutex<T>> - functional style (async)
    pub fn chain_arc_tokio_mutex_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcTokioMutexOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::Mutex<InnerValue>>>,
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        ArcTokioMutexOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this keypath with a writable inner keypath through Arc<tokio::sync::Mutex<T>> - functional style (async)
    pub fn chain_arc_tokio_mutex_writable_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> ArcTokioMutexWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::Mutex<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        ArcTokioMutexWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this keypath with a writable optional inner keypath through Arc<tokio::sync::Mutex<T>> - functional style (async)
    pub fn chain_arc_tokio_mutex_writable_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcTokioMutexWritableOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::Mutex<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        ArcTokioMutexWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this keypath with an inner keypath through Arc<tokio::sync::RwLock<T>> - functional style (async, read lock)
    pub fn chain_arc_tokio_rwlock_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> ArcTokioRwLockKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        ArcTokioRwLockKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this keypath with an optional inner keypath through Arc<tokio::sync::RwLock<T>> - functional style (async, read lock)
    pub fn chain_arc_tokio_rwlock_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcTokioRwLockOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        ArcTokioRwLockOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this keypath with a writable inner keypath through Arc<tokio::sync::RwLock<T>> - functional style (async, write lock)
    pub fn chain_arc_tokio_rwlock_writable_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> ArcTokioRwLockWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        ArcTokioRwLockWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this keypath with a writable optional inner keypath through Arc<tokio::sync::RwLock<T>> - functional style (async, write lock)
    pub fn chain_arc_tokio_rwlock_writable_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcTokioRwLockWritableOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        ArcTokioRwLockWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    /// Monadic helper: shorthand for chain_arc_rwlock_writable_at_kp when Value is Arc<RwLock<T>>
    /// This allows chaining with .then().then().then() pattern for Arc<RwLock<T>> structures
    /// 
    /// # Example
    /// ```rust,ignore
    /// ContainerTest::rwlock_data_r()
    ///     .then_rwlock(SomeStruct::data_w())
    ///     .then(OtherStruct::field_w())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn then_rwlock<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> ArcRwLockWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        self.chain_arc_rwlock_writable_at_kp(inner_keypath)
    }

    // ========== LOCK KEYPATH CONVERSION METHODS ==========
    // These methods convert keypaths pointing to lock types into chain-ready keypaths

    /// Convert this keypath to an Arc<RwLock> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    /// 
    /// # Example
    /// ```rust,ignore
    /// Container::rwlock_data_r()
    ///     .to_arc_rwlock_kp()
    ///     .chain_arc_rwlock_at_kp(InnerStruct::field_r());
    /// ```
    pub fn to_arc_rwlock_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
    {
        self
    }

    /// Convert this keypath to an Arc<Mutex> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_mutex_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
    {
        self
    }

    #[cfg(feature = "parking_lot")]
    /// Convert this keypath to an Arc<parking_lot::RwLock> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    /// 
    /// # Example
    /// ```rust,ignore
    /// Container::rwlock_data_r()
    ///     .to_arc_parking_rwlock_kp()
    ///     .chain_arc_parking_rwlock_at_kp(InnerStruct::field_r());
    /// ```
    pub fn to_arc_parking_rwlock_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<parking_lot::RwLock<InnerValue>>>,
    {
        self
    }

    #[cfg(feature = "parking_lot")]
    /// Convert this keypath to an Arc<parking_lot::Mutex> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_parking_mutex_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<parking_lot::Mutex<InnerValue>>>,
    {
        self
    }

    /// Convert this keypath to an Arc<RwLock> chain keypath
    /// Creates a chain with an identity inner keypath, ready for further chaining
    /// 
    /// # Example
    /// ```rust,ignore
    /// Container::rwlock_data_r()
    ///     .to_arc_rwlock_chain()
    ///     .then(InnerStruct::field_r());
    /// ```
    /// Convert this keypath to an Arc<RwLock> chain keypath
    /// Creates a chain with an identity inner keypath, ready for further chaining
    /// Type inference automatically determines InnerValue from Value
    /// 
    /// # Example
    /// ```rust,ignore
    /// Container::rwlock_data_r()
    ///     .to_arc_rwlock_chain()
    ///     .then(InnerStruct::field_r());
    /// ```
    pub fn to_arc_rwlock_chain<InnerValue>(self) -> ArcRwLockKeyPathChain<Root, Value, InnerValue, InnerValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r InnerValue + 'static>
    where
        Value: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
        F: 'static,
        InnerValue: 'static,
    {
        let identity = KeyPath::new(|inner: &InnerValue| inner);
        ArcRwLockKeyPathChain {
            outer_keypath: self,
            inner_keypath: identity,
        }
    }

    /// Convert this keypath to an Arc<Mutex> chain keypath
    /// Creates a chain with an identity inner keypath, ready for further chaining
    /// Type inference automatically determines InnerValue from Value
    pub fn to_arc_mutex_chain<InnerValue>(self) -> ArcMutexKeyPathChain<Root, Value, InnerValue, InnerValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r InnerValue + 'static>
    where
        Value: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
        F: 'static,
        InnerValue: 'static,
    {
        let identity = KeyPath::new(|inner: &InnerValue| inner);
        ArcMutexKeyPathChain {
            outer_keypath: self,
            inner_keypath: identity,
        }
    }

    // #[cfg(feature = "parking_lot")]
    // /// Convert this keypath to an Arc<parking_lot::RwLock> chain keypath
    // /// Creates a chain with an identity inner keypath, ready for further chaining
    // /// Type inference automatically determines InnerValue from Value
    // /// 
    // /// # Example
    // /// ```rust,ignore
    // /// Container::rwlock_data_r()
    // ///     .to_arc_parking_rwlock_chain()
    // ///     .then(InnerStruct::field_r());
    // /// ```
    // pub fn to_arc_parking_rwlock_chain<InnerValue>(self) -> ArcParkingRwLockKeyPathChain<Root, InnerValue, InnerValue, impl for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>> + 'static, impl for<'r> Fn(&'r InnerValue) -> &'r InnerValue + 'static>
    // where
    //     Value: std::borrow::Borrow<Arc<parking_lot::RwLock<InnerValue>>>,
    //     F: 'static + Clone,
    //     InnerValue: 'static,
    // {
    //     let identity = KeyPath::new(|inner: &InnerValue| inner);
    //     let getter = self.getter.clone();
    //     // Create a new keypath with the exact type needed, following the proc macro pattern
    //     // KeyPath::new(|s: &Root| &s.field).chain_arc_parking_rwlock_at_kp(inner_kp)
    //     let lock_kp: KeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, _> = KeyPath::new(move |root: &Root| {
    //         // Safe: Value is Arc<parking_lot::RwLock<InnerValue>> at call site, enforced by Borrow bound
    //         unsafe {
    //             std::mem::transmute::<&Value, &Arc<parking_lot::RwLock<InnerValue>>>(getter(root))
    //         }
    //     });
    //     lock_kp.chain_arc_parking_rwlock_at_kp(identity)
    // }

    // #[cfg(feature = "parking_lot")]
    // /// Convert this keypath to an Arc<parking_lot::Mutex> chain keypath
    // /// Creates a chain with an identity inner keypath, ready for further chaining
    // /// Type inference automatically determines InnerValue from Value
    // pub fn to_arc_parking_mutex_chain<InnerValue>(self) -> ArcParkingMutexKeyPathChain<Root, InnerValue, InnerValue, impl for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>> + 'static, impl for<'r> Fn(&'r InnerValue) -> &'r InnerValue + 'static>
    // where
    //     Value: std::borrow::Borrow<Arc<parking_lot::Mutex<InnerValue>>>,
    //     F: 'static + Clone,
    //     InnerValue: 'static,
    // {
    //     let identity = KeyPath::new(|inner: &InnerValue| inner);
    //     let getter = self.getter.clone();
    //     let lock_kp: KeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, _> = KeyPath::new(move |root: &Root| {
    //         unsafe {
    //             std::mem::transmute::<&Value, &Arc<parking_lot::Mutex<InnerValue>>>(getter(root))
    //         }
    //     });
    //     lock_kp.chain_arc_parking_mutex_at_kp(identity)
    // }

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
    
    // Overload: Adapt root type to Arc<Root> when Value is Sized (not a container)
    pub fn for_arc_root(self) -> OptionalKeyPath<Arc<Root>, Value, impl for<'r> Fn(&'r Arc<Root>) -> Option<&'r Value> + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |arc: &Arc<Root>| {
                Some(getter(arc.as_ref()))
            },
            _phantom: PhantomData,
        }
    }
    
    // Overload: Adapt root type to Box<Root> when Value is Sized (not a container)
    pub fn for_box_root(self) -> OptionalKeyPath<Box<Root>, Value, impl for<'r> Fn(&'r Box<Root>) -> Option<&'r Value> + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |boxed: &Box<Root>| {
                Some(getter(boxed.as_ref()))
            },
            _phantom: PhantomData,
        }
    }
    
    // Overload: Adapt root type to Rc<Root> when Value is Sized (not a container)
    pub fn for_rc_root(self) -> OptionalKeyPath<Rc<Root>, Value, impl for<'r> Fn(&'r Rc<Root>) -> Option<&'r Value> + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |rc: &Rc<Root>| {
                Some(getter(rc.as_ref()))
            },
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Result<Root, E> instead of Root
    /// This unwraps the Result and applies the keypath to the Ok value
    pub fn for_result<E>(self) -> OptionalKeyPath<Result<Root, E>, Value, impl for<'r> Fn(&'r Result<Root, E>) -> Option<&'r Value> + 'static>
    where
        F: 'static,
        Root: 'static,
        Value: 'static,
        E: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |result: &Result<Root, E>| {
                result.as_ref().ok().map(|root| getter(root))
            },
            _phantom: PhantomData,
        }
    }
    
    /// Convert a KeyPath to OptionalKeyPath for chaining
    /// This allows non-optional keypaths to be chained with then()
    pub fn to_optional(self) -> OptionalKeyPath<Root, Value, impl for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static>
    where
        F: 'static,
    {
        let getter = self.getter;
        OptionalKeyPath::new(move |root: &Root| Some(getter(root)))
    }
    
    /// Execute a closure with a reference to the value inside an Option
    pub fn with_option<Callback, R>(&self, option: &Option<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        option.as_ref().map(|root| {
            let value = self.get(root);
            f(value)
        })
    }
    
    /// Execute a closure with a reference to the value inside a Result
    pub fn with_result<Callback, R, E>(&self, result: &Result<Root, E>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        result.as_ref().ok().map(|root| {
            let value = self.get(root);
            f(value)
        })
    }
    
    /// Execute a closure with a reference to the value inside a Box
    pub fn with_box<Callback, R>(&self, boxed: &Box<Root>, f: Callback) -> R
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        let value = self.get(boxed);
        f(value)
    }
    
    /// Execute a closure with a reference to the value inside an Arc
    pub fn with_arc<Callback, R>(&self, arc: &Arc<Root>, f: Callback) -> R
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        let value = self.get(arc);
        f(value)
    }
    
    /// Execute a closure with a reference to the value inside an Rc
    pub fn with_rc<Callback, R>(&self, rc: &Rc<Root>, f: Callback) -> R
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        let value = self.get(rc);
        f(value)
    }
    
    /// Execute a closure with a reference to the value inside a RefCell
    pub fn with_refcell<Callback, R>(&self, refcell: &RefCell<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        refcell.try_borrow().ok().map(|borrow| {
            let value = self.get(&*borrow);
            f(value)
        })
    }
    
    /// Execute a closure with a reference to the value inside a Mutex
    pub fn with_mutex<Callback, R>(&self, mutex: &Mutex<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        mutex.lock().ok().map(|guard| {
            let value = self.get(&*guard);
            f(value)
        })
    }
    
    /// Execute a closure with a reference to the value inside an RwLock
    pub fn with_rwlock<Callback, R>(&self, rwlock: &RwLock<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        rwlock.read().ok().map(|guard| {
            let value = self.get(&*guard);
            f(value)
        })
    }
    
    /// Execute a closure with a reference to the value inside an Arc<RwLock<Root>>
    pub fn with_arc_rwlock<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        arc_rwlock.read().ok().map(|guard| {
            let value = self.get(&*guard);
            f(value)
        })
    }
    
    /// Execute a closure with a reference to the value inside an Arc<Mutex<Root>>
    pub fn with_arc_mutex<Callback, R>(&self, arc_mutex: &Arc<Mutex<Root>>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        arc_mutex.lock().ok().map(|guard| {
            let value = self.get(&*guard);
            f(value)
        })
    }
    
    #[cfg(feature = "tagged")]
    /// Adapt this keypath to work with Tagged<Root, Tag> instead of Root
    /// This unwraps the Tagged wrapper and applies the keypath to the inner value
    pub fn for_tagged<Tag>(self) -> KeyPath<Tagged<Root, Tag>, Value, impl for<'r> Fn(&'r Tagged<Root, Tag>) -> &'r Value + 'static>
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        F: 'static,
        Root: 'static,
        Value: 'static,
        Tag: 'static,
    {
        use std::ops::Deref;
        let getter = self.getter;
        
        KeyPath {
            getter: move |tagged: &Tagged<Root, Tag>| {
                getter(tagged.deref())
            },
            _phantom: PhantomData,
        }
    }
    
    #[cfg(feature = "tagged")]
    /// Execute a closure with a reference to the value inside a Tagged
    /// This avoids cloning by working with references directly
    pub fn with_tagged<Tag, Callback, R>(&self, tagged: &Tagged<Root, Tag>, f: Callback) -> R
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        Callback: FnOnce(&Value) -> R,
    {
        use std::ops::Deref;
        let value = self.get(tagged.deref());
        f(value)
    }
    
    /// Adapt this keypath to work with Option<Root> instead of Root
    /// This converts the KeyPath to an OptionalKeyPath and unwraps the Option
    pub fn for_option(self) -> OptionalKeyPath<Option<Root>, Value, impl for<'r> Fn(&'r Option<Root>) -> Option<&'r Value> + 'static>
    where
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |opt: &Option<Root>| {
                opt.as_ref().map(|root| getter(root))
            },
            _phantom: PhantomData,
        }
    }
    
    /// Get an iterator over a Vec when Value is Vec<T>
    /// Returns Some(iterator) if the value is a Vec, None otherwise
    pub fn iter<'r, T>(&self, root: &'r Root) -> Option<std::slice::Iter<'r, T>>
    where
        Value: AsRef<[T]> + 'r,
    {
        let value_ref: &'r Value = self.get(root);
        Some(value_ref.as_ref().iter())
    }
    
    /// Extract values from a slice of owned values
    /// Returns a Vec of references to the extracted values
    pub fn extract_from_slice<'r>(&self, slice: &'r [Root]) -> Vec<&'r Value> {
        slice.iter().map(|item| self.get(item)).collect()
    }
    
    /// Extract values from a slice of references
    /// Returns a Vec of references to the extracted values
    pub fn extract_from_ref_slice<'r>(&self, slice: &'r [&Root]) -> Vec<&'r Value> {
        slice.iter().map(|item| self.get(item)).collect()
    }
    
    /// Chain this keypath with another keypath
    /// Returns a KeyPath that chains both keypaths
    pub fn then<SubValue, G>(
        self,
        next: KeyPath<Value, SubValue, G>,
    ) -> KeyPath<Root, SubValue, impl for<'r> Fn(&'r Root) -> &'r SubValue>
    where
        G: for<'r> Fn(&'r Value) -> &'r SubValue,
        F: 'static,
        G: 'static,
        Value: 'static,
    {
        let first = self.getter;
        let second = next.getter;
        
        KeyPath::new(move |root: &Root| {
            let value = first(root);
            second(value)
        })
    }
    
    /// Chain this keypath with an optional keypath
    /// Returns an OptionalKeyPath that chains both keypaths
    pub fn chain_optional<SubValue, G>(
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
            let value = first(root);
            second(value)
        })
    }
    
}

// Extension methods for KeyPath to support Arc<RwLock> and Arc<Mutex> directly
impl<Root, Value, F> KeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Value,
{
    /// Execute a closure with a reference to the value inside an Arc<RwLock<Root>>
    /// This is a convenience method that works directly with Arc<RwLock<T>>
    pub fn with_arc_rwlock_direct<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        arc_rwlock.read().ok().map(|guard| {
            let value = self.get(&*guard);
            f(value)
        })
    }
    
    /// Execute a closure with a reference to the value inside an Arc<Mutex<Root>>
    /// This is a convenience method that works directly with Arc<Mutex<T>>
    pub fn with_arc_mutex_direct<Callback, R>(&self, arc_mutex: &Arc<Mutex<Root>>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        arc_mutex.lock().ok().map(|guard| {
            let value = self.get(&*guard);
            f(value)
        })
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

// ========== PARKING_LOT CHAIN METHODS FOR KEYPATH ==========

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, F> KeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::Mutex<InnerValue>>,
{
    /// Chain this keypath with an inner keypath through Arc<parking_lot::Mutex<T>> - functional style
    pub fn chain_arc_parking_mutex_at_kp<SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> ArcParkingMutexKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        ArcParkingMutexKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with an optional inner keypath through Arc<parking_lot::Mutex<T>>
    pub fn then_arc_parking_mutex_optional_at_kp<SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcParkingMutexOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        ArcParkingMutexOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with a writable inner keypath through Arc<parking_lot::Mutex<T>>
    pub fn chain_arc_parking_mutex_writable_at_kp<SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> ArcParkingMutexWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        ArcParkingMutexWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with a writable optional inner keypath through Arc<parking_lot::Mutex<T>>
    pub fn then_arc_parking_mutex_writable_optional_at_kp<SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        ArcParkingMutexWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, F> KeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Arc<parking_lot::RwLock<InnerValue>>,
{
    /// Chain this keypath with an inner keypath through Arc<parking_lot::RwLock<T>> - functional style
    pub fn chain_arc_parking_rwlock_at_kp<SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> ArcParkingRwLockKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        ArcParkingRwLockKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with an optional inner keypath through Arc<parking_lot::RwLock<T>>
    pub fn then_arc_parking_rwlock_optional_at_kp<SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        ArcParkingRwLockOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with a writable inner keypath through Arc<parking_lot::RwLock<T>>
    pub fn chain_arc_parking_rwlock_writable_at_kp<SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> ArcParkingRwLockWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        ArcParkingRwLockWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this keypath with a writable optional inner keypath through Arc<parking_lot::RwLock<T>>
    pub fn then_arc_parking_rwlock_writable_optional_at_kp<SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> ArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        ArcParkingRwLockWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
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

impl<Root, Value, F> fmt::Display for OptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Value>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root_name = std::any::type_name::<Root>();
        let value_name = std::any::type_name::<Value>();
        // Simplify type names by removing module paths for cleaner output
        let root_short = root_name.split("::").last().unwrap_or(root_name);
        let value_short = value_name.split("::").last().unwrap_or(value_name);
        write!(f, "OptionalKeyPath<{} -> Option<{}>>", root_short, value_short)
    }
}

impl<Root, Value, F> fmt::Debug for OptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Value>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
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
    
    /// Chain this optional keypath with an inner keypath through Arc<Mutex<T>> - functional style
    /// Compose first, then apply container at get() time
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { mutex_data: Some(Arc::new(Mutex::new(SomeStruct { data: "test".to_string() }))) };
    /// 
    /// // Functional style: compose first, apply container at get()
    /// ContainerTest::mutex_data_fr()
    ///     .chain_arc_mutex_at_kp(SomeStruct::data_r())
    ///     .get(&container, |value| println!("Data: {}", value));
    /// ```
    pub fn chain_arc_mutex_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcMutexKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        OptionalArcMutexKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with an optional inner keypath through Arc<Mutex<T>> - functional style
    /// Compose first, then apply container at get() time
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { mutex_data: Some(Arc::new(Mutex::new(SomeStruct { optional_field: Some("test".to_string()) }))) };
    /// 
    /// // Functional style: compose first, apply container at get()
    /// ContainerTest::mutex_data_fr()
    ///     .chain_arc_mutex_optional_at_kp(SomeStruct::optional_field_fr())
    ///     .get(&container, |value| println!("Value: {}", value));
    /// ```
    pub fn chain_arc_mutex_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcMutexOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        OptionalArcMutexOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with an inner keypath through Arc<RwLock<T>> - functional style
    /// Compose first, then apply container at get() time (uses read lock)
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { rwlock_data: Some(Arc::new(RwLock::new(SomeStruct { data: "test".to_string() }))) };
    /// 
    /// // Functional style: compose first, apply container at get()
    /// ContainerTest::rwlock_data_fr()
    ///     .chain_arc_rwlock_at_kp(SomeStruct::data_r())
    ///     .get(&container, |value| println!("Data: {}", value));
    /// ```
    pub fn chain_arc_rwlock_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcRwLockKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        OptionalArcRwLockKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with an optional inner keypath through Arc<RwLock<T>> - functional style
    /// Compose first, then apply container at get() time (uses read lock)
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { rwlock_data: Some(Arc::new(RwLock::new(SomeStruct { optional_field: Some("test".to_string()) }))) };
    /// 
    /// // Functional style: compose first, apply container at get()
    /// ContainerTest::rwlock_data_fr()
    ///     .chain_arc_rwlock_optional_at_kp(SomeStruct::optional_field_fr())
    ///     .get(&container, |value| println!("Value: {}", value));
    /// ```
    pub fn chain_arc_rwlock_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcRwLockOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        OptionalArcRwLockOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with a writable inner keypath through Arc<Mutex<T>> - functional style
    /// Compose first, then apply container at get_mut() time
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { mutex_data: Some(Arc::new(Mutex::new(SomeStruct { data: "test".to_string() }))) };
    /// 
    /// // Functional style: compose first, apply container at get_mut()
    /// ContainerTest::mutex_data_fr()
    ///     .chain_arc_mutex_writable_at_kp(SomeStruct::data_w())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn chain_arc_mutex_writable_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcMutexWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        OptionalArcMutexWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with a writable optional inner keypath through Arc<Mutex<T>> - functional style
    /// Compose first, then apply container at get_mut() time
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { mutex_data: Some(Arc::new(Mutex::new(SomeStruct { optional_field: Some("test".to_string()) }))) };
    /// 
    /// // Functional style: compose first, apply container at get_mut()
    /// ContainerTest::mutex_data_fr()
    ///     .chain_arc_mutex_writable_optional_at_kp(SomeStruct::optional_field_fw())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn chain_arc_mutex_writable_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcMutexWritableOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        OptionalArcMutexWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with a writable inner keypath through Arc<RwLock<T>> - functional style
    /// Compose first, then apply container at get_mut() time (uses write lock)
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { rwlock_data: Some(Arc::new(RwLock::new(SomeStruct { data: "test".to_string() }))) };
    /// 
    /// // Functional style: compose first, apply container at get_mut()
    /// ContainerTest::rwlock_data_fr()
    ///     .chain_arc_rwlock_writable_at_kp(SomeStruct::data_w())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn chain_arc_rwlock_writable_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcRwLockWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        OptionalArcRwLockWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with a writable optional inner keypath through Arc<RwLock<T>> - functional style
    /// Compose first, then apply container at get_mut() time (uses write lock)
    /// 
    /// # Example
    /// ```rust
    /// let container = ContainerTest { rwlock_data: Some(Arc::new(RwLock::new(SomeStruct { optional_field: Some("test".to_string()) }))) };
    /// 
    /// // Functional style: compose first, apply container at get_mut()
    /// ContainerTest::rwlock_data_fr()
    ///     .chain_arc_rwlock_writable_optional_at_kp(SomeStruct::optional_field_fw())
    ///     .get_mut(&container, |value| *value = "new".to_string());
    /// ```
    pub fn chain_arc_rwlock_writable_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcRwLockWritableOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        OptionalArcRwLockWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this optional keypath with an inner keypath through Arc<tokio::sync::Mutex<T>> - functional style (async)
    pub fn chain_arc_tokio_mutex_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcTokioMutexKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::Mutex<InnerValue>>>,
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        OptionalArcTokioMutexKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this optional keypath with an optional inner keypath through Arc<tokio::sync::Mutex<T>> - functional style (async)
    pub fn chain_arc_tokio_mutex_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcTokioMutexOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::Mutex<InnerValue>>>,
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        OptionalArcTokioMutexOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this optional keypath with a writable inner keypath through Arc<tokio::sync::Mutex<T>> - functional style (async)
    pub fn chain_arc_tokio_mutex_writable_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcTokioMutexWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::Mutex<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        OptionalArcTokioMutexWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this optional keypath with a writable optional inner keypath through Arc<tokio::sync::Mutex<T>> - functional style (async)
    pub fn chain_arc_tokio_mutex_writable_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcTokioMutexWritableOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::Mutex<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        OptionalArcTokioMutexWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this optional keypath with an inner keypath through Arc<tokio::sync::RwLock<T>> - functional style (async, read lock)
    pub fn chain_arc_tokio_rwlock_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcTokioRwLockKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        OptionalArcTokioRwLockKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this optional keypath with an optional inner keypath through Arc<tokio::sync::RwLock<T>> - functional style (async, read lock)
    pub fn chain_arc_tokio_rwlock_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcTokioRwLockOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        OptionalArcTokioRwLockOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this optional keypath with a writable inner keypath through Arc<tokio::sync::RwLock<T>> - functional style (async, write lock)
    pub fn chain_arc_tokio_rwlock_writable_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcTokioRwLockWritableKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        OptionalArcTokioRwLockWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }

    #[cfg(feature = "tokio")]
    /// Chain this optional keypath with a writable optional inner keypath through Arc<tokio::sync::RwLock<T>> - functional style (async, write lock)
    pub fn chain_arc_tokio_rwlock_writable_optional_at_kp<InnerValue, SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcTokioRwLockWritableOptionalKeyPathChain<Root, Value, InnerValue, SubValue, F, G>
    where
        Value: std::borrow::Borrow<Arc<tokio::sync::RwLock<InnerValue>>>,
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        OptionalArcTokioRwLockWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
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
    
    #[cfg(feature = "tagged")]
    /// Adapt this keypath to work with Tagged<Root, Tag> instead of Root
    /// This unwraps the Tagged wrapper and applies the keypath to the inner value
    pub fn for_tagged<Tag>(self) -> OptionalKeyPath<Tagged<Root, Tag>, Value, impl for<'r> Fn(&'r Tagged<Root, Tag>) -> Option<&'r Value> + 'static>
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        F: 'static,
        Root: 'static,
        Value: 'static,
        Tag: 'static,
    {
        use std::ops::Deref;
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |tagged: &Tagged<Root, Tag>| {
                getter(tagged.deref())
            },
            _phantom: PhantomData,
        }
    }
    
    #[cfg(feature = "tagged")]
    /// Execute a closure with a reference to the value inside a Tagged
    /// This avoids cloning by working with references directly
    pub fn with_tagged<Tag, Callback, R>(&self, tagged: &Tagged<Root, Tag>, f: Callback) -> Option<R>
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        use std::ops::Deref;
        self.get(tagged.deref()).map(|value| f(value))
    }
    
    /// Adapt this keypath to work with Option<Root> instead of Root
    /// This unwraps the Option and applies the keypath to the inner value
    pub fn for_option(self) -> OptionalKeyPath<Option<Root>, Value, impl for<'r> Fn(&'r Option<Root>) -> Option<&'r Value> + 'static>
    where
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |opt: &Option<Root>| {
                opt.as_ref().and_then(|root| getter(root))
            },
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Result<Root, E> instead of Root
    /// This unwraps the Result and applies the keypath to the Ok value
    pub fn for_result<E>(self) -> OptionalKeyPath<Result<Root, E>, Value, impl for<'r> Fn(&'r Result<Root, E>) -> Option<&'r Value> + 'static>
    where
        F: 'static,
        Root: 'static,
        Value: 'static,
        E: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |result: &Result<Root, E>| {
                result.as_ref().ok().and_then(|root| getter(root))
            },
            _phantom: PhantomData,
        }
    }

    // ========== LOCK KEYPATH CONVERSION METHODS ==========
    // These methods convert keypaths pointing to lock types into chain-ready keypaths

    /// Convert this optional keypath to an Arc<RwLock> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_rwlock_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
    {
        self
    }

    /// Convert this optional keypath to an Arc<Mutex> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_mutex_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
    {
        self
    }

    #[cfg(feature = "parking_lot")]
    /// Convert this optional keypath to an Arc<parking_lot::RwLock> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_parking_rwlock_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<parking_lot::RwLock<InnerValue>>>,
    {
        self
    }

    #[cfg(feature = "parking_lot")]
    /// Convert this optional keypath to an Arc<parking_lot::Mutex> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_parking_mutex_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<parking_lot::Mutex<InnerValue>>>,
    {
        self
    }

    /// Convert this optional keypath to an Arc<RwLock> chain keypath
    /// Creates a chain with an identity inner keypath, ready for further chaining
    /// Type inference automatically determines InnerValue from Value
    pub fn to_arc_rwlock_chain<InnerValue>(self) -> OptionalArcRwLockKeyPathChain<Root, Value, InnerValue, InnerValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r InnerValue + 'static>
    where
        Value: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
        F: 'static,
        InnerValue: 'static,
    {
        let identity = KeyPath::new(|inner: &InnerValue| inner);
        OptionalArcRwLockKeyPathChain {
            outer_keypath: self,
            inner_keypath: identity,
        }
    }

    /// Convert this optional keypath to an Arc<Mutex> chain keypath
    /// Creates a chain with an identity inner keypath, ready for further chaining
    /// Type inference automatically determines InnerValue from Value
    pub fn to_arc_mutex_chain<InnerValue>(self) -> OptionalArcMutexKeyPathChain<Root, Value, InnerValue, InnerValue, F, impl for<'r> Fn(&'r InnerValue) -> &'r InnerValue + 'static>
    where
        Value: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
        F: 'static,
        InnerValue: 'static,
    {
        let identity = KeyPath::new(|inner: &InnerValue| inner);
        OptionalArcMutexKeyPathChain {
            outer_keypath: self,
            inner_keypath: identity,
        }
    }

    // #[cfg(feature = "parking_lot")]
    // /// Convert this optional keypath to an Arc<parking_lot::RwLock> chain keypath
    // /// Creates a chain with an identity inner keypath, ready for further chaining
    // /// Type inference automatically determines InnerValue from Value
    // pub fn to_arc_parking_rwlock_chain<InnerValue>(self) -> OptionalArcParkingRwLockKeyPathChain<Root, InnerValue, InnerValue, impl for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>> + 'static, impl for<'r> Fn(&'r InnerValue) -> &'r InnerValue + 'static>
    // where
    //     Value: std::borrow::Borrow<Arc<parking_lot::RwLock<InnerValue>>>,
    //     F: 'static + Clone,
    //     InnerValue: 'static,
    // {
    //     let identity = KeyPath::new(|inner: &InnerValue| inner);
    //     let getter = self.getter.clone();
    //     let lock_kp: OptionalKeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, _> = OptionalKeyPath::new(move |root: &Root| {
    //         getter(root).map(|v| {
    //             unsafe {
    //                 std::mem::transmute::<&Value, &Arc<parking_lot::RwLock<InnerValue>>>(v)
    //             }
    //         })
    //     });
    //     lock_kp.chain_arc_parking_rwlock_at_kp(identity)
    // }

    // #[cfg(feature = "parking_lot")]
    // /// Convert this optional keypath to an Arc<parking_lot::Mutex> chain keypath
    // /// Creates a chain with an identity inner keypath, ready for further chaining
    // /// Type inference automatically determines InnerValue from Value
    // pub fn to_arc_parking_mutex_chain<InnerValue>(self) -> OptionalArcParkingMutexKeyPathChain<Root, InnerValue, InnerValue, impl for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>> + 'static, impl for<'r> Fn(&'r InnerValue) -> &'r InnerValue + 'static>
    // where
    //     Value: std::borrow::Borrow<Arc<parking_lot::Mutex<InnerValue>>>,
    //     F: 'static + Clone,
    //     InnerValue: 'static,
    // {
    //     let identity = KeyPath::new(|inner: &InnerValue| inner);
    //     let getter = self.getter.clone();
    //     let lock_kp: OptionalKeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, _> = OptionalKeyPath::new(move |root: &Root| {
    //         getter(root).map(|v| {
    //             unsafe {
    //                 std::mem::transmute::<&Value, &Arc<parking_lot::Mutex<InnerValue>>>(v)
    //             }
    //         })
    //     });
    //     lock_kp.chain_arc_parking_mutex_at_kp(identity)
    // }
    
    // Overload: Adapt root type to Arc<Root> when Value is Sized (not a container)
    pub fn for_arc_root(self) -> OptionalKeyPath<Arc<Root>, Value, impl for<'r> Fn(&'r Arc<Root>) -> Option<&'r Value> + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |arc: &Arc<Root>| {
                getter(arc.as_ref())
            },
            _phantom: PhantomData,
        }
    }
    
    // Overload: Adapt root type to Rc<Root> when Value is Sized (not a container)
    pub fn for_rc_root(self) -> OptionalKeyPath<Rc<Root>, Value, impl for<'r> Fn(&'r Rc<Root>) -> Option<&'r Value> + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        OptionalKeyPath {
            getter: move |rc: &Rc<Root>| {
                getter(rc.as_ref())
            },
            _phantom: PhantomData,
        }
    }
    
    /// Execute a closure with a reference to the value inside an Option
    pub fn with_option<Callback, R>(&self, option: &Option<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        option.as_ref().and_then(|root| {
            self.get(root).map(|value| f(value))
        })
    }
    
    /// Execute a closure with a reference to the value inside a Mutex
    pub fn with_mutex<Callback, R>(&self, mutex: &Mutex<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        mutex.lock().ok().and_then(|guard| {
            self.get(&*guard).map(|value| f(value))
        })
    }
    
    /// Execute a closure with a reference to the value inside an RwLock
    pub fn with_rwlock<Callback, R>(&self, rwlock: &RwLock<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        rwlock.read().ok().and_then(|guard| {
            self.get(&*guard).map(|value| f(value))
        })
    }
    
    /// Execute a closure with a reference to the value inside an Arc<RwLock<Root>>
    pub fn with_arc_rwlock<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        arc_rwlock.read().ok().and_then(|guard| {
            self.get(&*guard).map(|value| f(value))
        })
    }
    
    /// Execute a closure with a reference to the value inside an Arc<RwLock<Root>>
    /// This is a convenience method that works directly with Arc<RwLock<T>>
    /// Unlike with_arc_rwlock, this doesn't require F: Clone
    pub fn with_arc_rwlock_direct<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        arc_rwlock.read().ok().and_then(|guard| {
            self.get(&*guard).map(|value| f(value))
        })
    }
    
    /// Execute a closure with a reference to the value inside an Arc<Mutex<Root>>
    pub fn with_arc_mutex<Callback, R>(&self, arc_mutex: &Arc<Mutex<Root>>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&Value) -> R,
    {
        arc_mutex.lock().ok().and_then(|guard| {
            self.get(&*guard).map(|value| f(value))
        })
    }
}

// ========== PARKING_LOT CHAIN METHODS FOR OPTIONAL KEYPATH ==========

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, F> OptionalKeyPath<Root, Arc<parking_lot::Mutex<InnerValue>>, F>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::Mutex<InnerValue>>>,
{
    /// Chain this optional keypath with an inner keypath through Arc<parking_lot::Mutex<T>>
    pub fn chain_arc_parking_mutex_at_kp<SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcParkingMutexKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        OptionalArcParkingMutexKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with an optional inner keypath through Arc<parking_lot::Mutex<T>>
    pub fn then_arc_parking_mutex_optional_at_kp<SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcParkingMutexOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        OptionalArcParkingMutexOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with a writable inner keypath through Arc<parking_lot::Mutex<T>>
    pub fn chain_arc_parking_mutex_writable_at_kp<SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcParkingMutexWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        OptionalArcParkingMutexWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with a writable optional inner keypath through Arc<parking_lot::Mutex<T>>
    pub fn then_arc_parking_mutex_writable_optional_at_kp<SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcParkingMutexWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        OptionalArcParkingMutexWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
}

#[cfg(feature = "parking_lot")]
impl<Root, InnerValue, F> OptionalKeyPath<Root, Arc<parking_lot::RwLock<InnerValue>>, F>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Arc<parking_lot::RwLock<InnerValue>>>,
{
    /// Chain this optional keypath with an inner keypath through Arc<parking_lot::RwLock<T>>
    pub fn chain_arc_parking_rwlock_at_kp<SubValue, G>(
        self,
        inner_keypath: KeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcParkingRwLockKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> &'r SubValue,
    {
        OptionalArcParkingRwLockKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with an optional inner keypath through Arc<parking_lot::RwLock<T>>
    pub fn then_arc_parking_rwlock_optional_at_kp<SubValue, G>(
        self,
        inner_keypath: OptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcParkingRwLockOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r InnerValue) -> Option<&'r SubValue>,
    {
        OptionalArcParkingRwLockOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with a writable inner keypath through Arc<parking_lot::RwLock<T>>
    pub fn chain_arc_parking_rwlock_writable_at_kp<SubValue, G>(
        self,
        inner_keypath: WritableKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcParkingRwLockWritableKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> &'r mut SubValue,
    {
        OptionalArcParkingRwLockWritableKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
    }
    
    /// Chain this optional keypath with a writable optional inner keypath through Arc<parking_lot::RwLock<T>>
    pub fn then_arc_parking_rwlock_writable_optional_at_kp<SubValue, G>(
        self,
        inner_keypath: WritableOptionalKeyPath<InnerValue, SubValue, G>,
    ) -> OptionalArcParkingRwLockWritableOptionalKeyPathChain<Root, InnerValue, SubValue, F, G>
    where
        G: for<'r> Fn(&'r mut InnerValue) -> Option<&'r mut SubValue>,
    {
        OptionalArcParkingRwLockWritableOptionalKeyPathChain {
            outer_keypath: self,
            inner_keypath,
        }
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

impl<Root, Value, F> fmt::Display for WritableKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> &'r mut Value,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root_name = std::any::type_name::<Root>();
        let value_name = std::any::type_name::<Value>();
        // Simplify type names by removing module paths for cleaner output
        let root_short = root_name.split("::").last().unwrap_or(root_name);
        let value_short = value_name.split("::").last().unwrap_or(value_name);
        write!(f, "WritableKeyPath<{} -> {}>", root_short, value_short)
    }
}

impl<Root, Value, F> fmt::Debug for WritableKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> &'r mut Value,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
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
    
    /// Adapt this keypath to work with Result<Root, E> instead of Root
    /// This unwraps the Result and applies the keypath to the Ok value
    pub fn for_result<E>(self) -> WritableOptionalKeyPath<Result<Root, E>, Value, impl for<'r> Fn(&'r mut Result<Root, E>) -> Option<&'r mut Value> + 'static>
    where
        F: 'static,
        Root: 'static,
        Value: 'static,
        E: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |result: &mut Result<Root, E>| {
                result.as_mut().ok().map(|root| getter(root))
            },
            _phantom: PhantomData,
        }
    }
    
    // Overload: Adapt root type to Box<Root> when Value is Sized (not a container)
    pub fn for_box_root(self) -> WritableKeyPath<Box<Root>, Value, impl for<'r> Fn(&'r mut Box<Root>) -> &'r mut Value + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableKeyPath {
            getter: move |boxed: &mut Box<Root>| {
                getter(boxed.as_mut())
            },
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Option<Root> instead of Root
    /// This unwraps the Option and applies the keypath to the Some value
    pub fn for_option(self) -> WritableOptionalKeyPath<Option<Root>, Value, impl for<'r> Fn(&'r mut Option<Root>) -> Option<&'r mut Value> + 'static>
    where
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |option: &mut Option<Root>| {
                option.as_mut().map(|root| getter(root))
            },
            _phantom: PhantomData,
        }
    }
    
    /// Convert a WritableKeyPath to WritableOptionalKeyPath for chaining
    /// This allows non-optional writable keypaths to be chained with then()
    pub fn to_optional(self) -> WritableOptionalKeyPath<Root, Value, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static>
    where
        F: 'static,
    {
        let getter = self.getter;
        WritableOptionalKeyPath::new(move |root: &mut Root| Some(getter(root)))
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
    
    /// Execute a closure with a mutable reference to the value inside a Box
    pub fn with_box_mut<Callback, R>(&self, boxed: &mut Box<Root>, f: Callback) -> R
    where
        F: Clone,
        Callback: FnOnce(&mut Value) -> R,
    {
        let value = self.get_mut(boxed);
        f(value)
    }
    
    /// Execute a closure with a mutable reference to the value inside a Result
    pub fn with_result_mut<Callback, R, E>(&self, result: &mut Result<Root, E>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&mut Value) -> R,
    {
        result.as_mut().ok().map(|root| {
            let value = self.get_mut(root);
            f(value)
        })
    }
    
    /// Execute a closure with a mutable reference to the value inside an Option
    pub fn with_option_mut<Callback, R>(&self, option: &mut Option<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&mut Value) -> R,
    {
        option.as_mut().map(|root| {
            let value = self.get_mut(root);
            f(value)
        })
    }
    
    /// Execute a closure with a mutable reference to the value inside a RefCell
    pub fn with_refcell_mut<Callback, R>(&self, refcell: &RefCell<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&mut Value) -> R,
    {
        refcell.try_borrow_mut().ok().map(|mut borrow| {
            let value = self.get_mut(&mut *borrow);
            f(value)
        })
    }
    
    /// Execute a closure with a mutable reference to the value inside a Mutex
    pub fn with_mutex_mut<Callback, R>(&self, mutex: &mut Mutex<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&mut Value) -> R,
    {
        mutex.get_mut().ok().map(|root| {
            let value = self.get_mut(root);
            f(value)
        })
    }
    
    /// Execute a closure with a mutable reference to the value inside an RwLock
    pub fn with_rwlock_mut<Callback, R>(&self, rwlock: &mut RwLock<Root>, f: Callback) -> Option<R>
    where
        F: Clone,
        Callback: FnOnce(&mut Value) -> R,
    {
        rwlock.write().ok().map(|mut guard| {
            let value = self.get_mut(&mut *guard);
            f(value)
        })
    }
    
    /// Get a mutable iterator over a Vec when Value is Vec<T>
    /// Returns Some(iterator) if the value is a Vec, None otherwise
    pub fn iter_mut<'r, T>(&self, root: &'r mut Root) -> Option<std::slice::IterMut<'r, T>>
    where
        Value: AsMut<[T]> + 'r,
    {
        let value_ref: &'r mut Value = self.get_mut(root);
        Some(value_ref.as_mut().iter_mut())
    }
    
    /// Extract mutable values from a slice of owned mutable values
    /// Returns a Vec of mutable references to the extracted values
    pub fn extract_mut_from_slice<'r>(&self, slice: &'r mut [Root]) -> Vec<&'r mut Value> {
        slice.iter_mut().map(|item| self.get_mut(item)).collect()
    }
    
    /// Extract mutable values from a slice of mutable references
    /// Returns a Vec of mutable references to the extracted values
    pub fn extract_mut_from_ref_slice<'r>(&self, slice: &'r mut [&'r mut Root]) -> Vec<&'r mut Value> {
        slice.iter_mut().map(|item| self.get_mut(*item)).collect()
    }
    
    /// Chain this keypath with another writable keypath
    /// Returns a WritableKeyPath that chains both keypaths
    pub fn then<SubValue, G>(
        self,
        next: WritableKeyPath<Value, SubValue, G>,
    ) -> WritableKeyPath<Root, SubValue, impl for<'r> Fn(&'r mut Root) -> &'r mut SubValue>
    where
        G: for<'r> Fn(&'r mut Value) -> &'r mut SubValue,
        F: 'static,
        G: 'static,
        Value: 'static,
    {
        let first = self.getter;
        let second = next.getter;
        
        WritableKeyPath::new(move |root: &mut Root| {
            let value = first(root);
            second(value)
        })
    }
    
    /// Chain this keypath with a writable optional keypath
    /// Returns a WritableOptionalKeyPath that chains both keypaths
    pub fn chain_optional<SubValue, G>(
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
            let value = first(root);
            second(value)
        })
    }

    // ========== LOCK KEYPATH CONVERSION METHODS ==========
    // These methods convert keypaths pointing to lock types into chain-ready keypaths

    /// Convert this writable keypath to an Arc<RwLock> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_rwlock_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
    {
        self
    }

    /// Convert this writable keypath to an Arc<Mutex> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_mutex_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
    {
        self
    }

    #[cfg(feature = "parking_lot")]
    /// Convert this writable keypath to an Arc<parking_lot::RwLock> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_parking_rwlock_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<parking_lot::RwLock<InnerValue>>>,
    {
        self
    }

    #[cfg(feature = "parking_lot")]
    /// Convert this writable keypath to an Arc<parking_lot::Mutex> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_parking_mutex_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<parking_lot::Mutex<InnerValue>>>,
    {
        self
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

impl<Root, Value, F> fmt::Display for WritableOptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root_name = std::any::type_name::<Root>();
        let value_name = std::any::type_name::<Value>();
        // Simplify type names by removing module paths for cleaner output
        let root_short = root_name.split("::").last().unwrap_or(root_name);
        let value_short = value_name.split("::").last().unwrap_or(value_name);
        write!(f, "WritableOptionalKeyPath<{} -> Option<{}>>", root_short, value_short)
    }
}

impl<Root, Value, F> fmt::Debug for WritableOptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show type information and indicate this is a chain that may fail
        let root_name = std::any::type_name::<Root>();
        let value_name = std::any::type_name::<Value>();
        let root_short = root_name.split("::").last().unwrap_or(root_name);
        let value_short = value_name.split("::").last().unwrap_or(value_name);
        
        // Use alternate format for more detailed debugging
        if f.alternate() {
            writeln!(f, "WritableOptionalKeyPath<{} -> Option<{}>>", root_short, value_short)?;
            writeln!(f, "  ⚠ Chain may break if any intermediate step returns None")?;
            writeln!(f, "  💡 Use trace_chain() to find where the chain breaks")
        } else {
            write!(f, "WritableOptionalKeyPath<{} -> Option<{}>>", root_short, value_short)
        }
    }
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
    
    /// Trace the chain to find where it breaks
    /// Returns Ok(()) if the chain succeeds, or Err with diagnostic information
    /// 
    /// # Example
    /// ```rust
    /// let path = SomeComplexStruct::scsf_fw()
    ///     .then(SomeOtherStruct::sosf_fw())
    ///     .then(SomeEnum::b_fw());
    /// 
    /// match path.trace_chain(&mut instance) {
    ///     Ok(()) => println!("Chain succeeded"),
    ///     Err(msg) => println!("Chain broken: {}", msg),
    /// }
    /// ```
    pub fn trace_chain(&self, root: &mut Root) -> Result<(), String> {
        match self.get_mut(root) {
            Some(_) => Ok(()),
            None => {
                let root_name = std::any::type_name::<Root>();
                let value_name = std::any::type_name::<Value>();
                let root_short = root_name.split("::").last().unwrap_or(root_name);
                let value_short = value_name.split("::").last().unwrap_or(value_name);
                Err(format!("{} -> Option<{}> returned None (chain broken at this step)", root_short, value_short))
            }
        }
    }
    
    /// Adapt this keypath to work with Option<Root> instead of Root
    /// This unwraps the Option and applies the keypath to the Some value
    pub fn for_option(self) -> WritableOptionalKeyPath<Option<Root>, Value, impl for<'r> Fn(&'r mut Option<Root>) -> Option<&'r mut Value> + 'static>
    where
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |option: &mut Option<Root>| {
                option.as_mut().and_then(|root| getter(root))
            },
            _phantom: PhantomData,
        }
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
    
    /// Adapt this keypath to work with Result<Root, E> instead of Root
    /// This unwraps the Result and applies the keypath to the Ok value
    pub fn for_result<E>(self) -> WritableOptionalKeyPath<Result<Root, E>, Value, impl for<'r> Fn(&'r mut Result<Root, E>) -> Option<&'r mut Value> + 'static>
    where
        F: 'static,
        Root: 'static,
        Value: 'static,
        E: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |result: &mut Result<Root, E>| {
                result.as_mut().ok().and_then(|root| getter(root))
            },
            _phantom: PhantomData,
        }
    }
    
    // Overload: Adapt root type to Box<Root> when Value is Sized (not a container)
    pub fn for_box_root(self) -> WritableOptionalKeyPath<Box<Root>, Value, impl for<'r> Fn(&'r mut Box<Root>) -> Option<&'r mut Value> + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |boxed: &mut Box<Root>| {
                getter(boxed.as_mut())
            },
            _phantom: PhantomData,
        }
    }
    
    // Overload: Adapt root type to Arc<Root> when Value is Sized (not a container)
    pub fn for_arc_root(self) -> WritableOptionalKeyPath<Arc<Root>, Value, impl for<'r> Fn(&'r mut Arc<Root>) -> Option<&'r mut Value> + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |arc: &mut Arc<Root>| {
                // Arc doesn't support mutable access without interior mutability
                // This will always return None, but we provide it for API consistency
                None
            },
            _phantom: PhantomData,
        }
    }
    
    // Overload: Adapt root type to Rc<Root> when Value is Sized (not a container)
    pub fn for_rc_root(self) -> WritableOptionalKeyPath<Rc<Root>, Value, impl for<'r> Fn(&'r mut Rc<Root>) -> Option<&'r mut Value> + 'static>
    where
        Value: Sized,
        F: 'static,
        Root: 'static,
        Value: 'static,
    {
        let getter = self.getter;
        
        WritableOptionalKeyPath {
            getter: move |rc: &mut Rc<Root>| {
                // Rc doesn't support mutable access without interior mutability
                // This will always return None, but we provide it for API consistency
                None
            },
            _phantom: PhantomData,
        }
    }

    // ========== LOCK KEYPATH CONVERSION METHODS ==========
    // These methods convert keypaths pointing to lock types into chain-ready keypaths

    /// Convert this writable optional keypath to an Arc<RwLock> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_rwlock_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<RwLock<InnerValue>>>,
    {
        self
    }

    /// Convert this writable optional keypath to an Arc<Mutex> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_mutex_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<Mutex<InnerValue>>>,
    {
        self
    }

    #[cfg(feature = "parking_lot")]
    /// Convert this writable optional keypath to an Arc<parking_lot::RwLock> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_parking_rwlock_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<parking_lot::RwLock<InnerValue>>>,
    {
        self
    }

    #[cfg(feature = "parking_lot")]
    /// Convert this writable optional keypath to an Arc<parking_lot::Mutex> chain-ready keypath
    /// Returns self, but serves as a marker for intent and enables chaining
    pub fn to_arc_parking_mutex_kp<InnerValue>(self) -> Self
    where
        Value: std::borrow::Borrow<Arc<parking_lot::Mutex<InnerValue>>>,
    {
        self
    }
}

// Static factory methods for WritableOptionalKeyPath
impl WritableOptionalKeyPath<(), (), fn(&mut ()) -> Option<&mut ()>> {
    // Static method for Option<T> -> Option<&mut T>
    // Note: This is a factory method. Use instance method `for_option()` to adapt existing keypaths.
    pub fn for_option_static<T>() -> WritableOptionalKeyPath<Option<T>, T, impl for<'r> Fn(&'r mut Option<T>) -> Option<&'r mut T>> {
        WritableOptionalKeyPath::new(|opt: &mut Option<T>| opt.as_mut())
    }
    
    /// Backword compatibility method for writable enum keypath
    // Create a writable enum keypath for enum variants
    /// This allows both reading and writing to enum variant fields
    /// 
    /// # Arguments
    /// * `embedder` - Function to embed a value into the enum variant (for API consistency, not used)
    /// * `read_extractor` - Function to extract a read reference from the enum (for API consistency, not used)
    /// * `write_extractor` - Function to extract a mutable reference from the enum
    /// 
    /// # Example
    /// ```rust
    /// enum Color { Other(RGBU8) }
    /// struct RGBU8(u8, u8, u8);
    /// 
    /// let case_path = WritableOptionalKeyPath::writable_enum(
    ///     |v| Color::Other(v),
    ///     |p: &Color| match p { Color::Other(rgb) => Some(rgb), _ => None },
    ///     |p: &mut Color| match p { Color::Other(rgb) => Some(rgb), _ => None },
    /// );
    /// ```
    pub fn writable_enum<Enum, Variant, EmbedFn, ReadExtractFn, WriteExtractFn>(
        _embedder: EmbedFn,
        _read_extractor: ReadExtractFn,
        write_extractor: WriteExtractFn,
    ) -> WritableOptionalKeyPath<Enum, Variant, impl for<'r> Fn(&'r mut Enum) -> Option<&'r mut Variant> + 'static>
    where
        EmbedFn: Fn(Variant) -> Enum + 'static,
        ReadExtractFn: for<'r> Fn(&'r Enum) -> Option<&'r Variant> + 'static,
        WriteExtractFn: for<'r> Fn(&'r mut Enum) -> Option<&'r mut Variant> + 'static,
    {
        WritableOptionalKeyPath::new(write_extractor)
    }
}

// Enum-specific keypaths
/// EnumKeyPath - A keypath for enum variants that supports both extraction and embedding
/// Uses generic type parameters instead of dynamic dispatch for zero-cost abstraction
/// 
/// This struct serves dual purpose:
/// 1. As a concrete keypath instance: `EnumKeyPath<Enum, Variant, ExtractFn, EmbedFn>`
/// 2. As a namespace for static factory methods: `EnumKeyPath::readable_enum(...)`
pub struct EnumKeyPath<Enum = (), Variant = (), ExtractFn = fn(&()) -> Option<&()>, EmbedFn = fn(()) -> ()> 
where
    ExtractFn: for<'r> Fn(&'r Enum) -> Option<&'r Variant> + 'static,
    EmbedFn: Fn(Variant) -> Enum + 'static,
{
    extractor: OptionalKeyPath<Enum, Variant, ExtractFn>,
    embedder: EmbedFn,
}

impl<Enum, Variant, ExtractFn, EmbedFn> EnumKeyPath<Enum, Variant, ExtractFn, EmbedFn>
where
    ExtractFn: for<'r> Fn(&'r Enum) -> Option<&'r Variant> + 'static,
    EmbedFn: Fn(Variant) -> Enum + 'static,
{
    /// Create a new EnumKeyPath with extractor and embedder functions
    pub fn new(
        extractor: ExtractFn,
        embedder: EmbedFn,
    ) -> Self {
        Self {
            extractor: OptionalKeyPath::new(extractor),
            embedder,
        }
    }
    
    /// Extract the value from an enum variant
    pub fn get<'r>(&self, enum_value: &'r Enum) -> Option<&'r Variant> {
        self.extractor.get(enum_value)
    }
    
    /// Embed a value into the enum variant
    pub fn embed(&self, value: Variant) -> Enum {
        (self.embedder)(value)
    }
    
    /// Get the underlying OptionalKeyPath for composition
    pub fn as_optional(&self) -> &OptionalKeyPath<Enum, Variant, ExtractFn> {
        &self.extractor
    }
    
    /// Convert to OptionalKeyPath (loses embedding capability but gains composition)
    pub fn to_optional(self) -> OptionalKeyPath<Enum, Variant, ExtractFn> {
        self.extractor
    }
}

// Static factory methods for EnumKeyPath
impl EnumKeyPath {
    /// Create a readable enum keypath with both extraction and embedding
    /// Returns an EnumKeyPath that supports both get() and embed() operations
    pub fn readable_enum<Enum, Variant, ExtractFn, EmbedFn>(
        embedder: EmbedFn,
        extractor: ExtractFn,
    ) -> EnumKeyPath<Enum, Variant, ExtractFn, EmbedFn>
    where
        ExtractFn: for<'r> Fn(&'r Enum) -> Option<&'r Variant> + 'static,
        EmbedFn: Fn(Variant) -> Enum + 'static,
    {
        EnumKeyPath::new(extractor, embedder)
    }
    
    /// Extract from a specific enum variant
    pub fn for_variant<Enum, Variant, ExtractFn>(
        extractor: ExtractFn
    ) -> OptionalKeyPath<Enum, Variant, impl for<'r> Fn(&'r Enum) -> Option<&'r Variant>>
    where
        ExtractFn: Fn(&Enum) -> Option<&Variant>,
    {
        OptionalKeyPath::new(extractor)
    }
    
    /// Match against multiple variants (returns a tagged union)
    pub fn for_match<Enum, Output, MatchFn>(
        matcher: MatchFn
    ) -> KeyPath<Enum, Output, impl for<'r> Fn(&'r Enum) -> &'r Output>
    where
        MatchFn: Fn(&Enum) -> &Output,
    {
        KeyPath::new(matcher)
    }
    
    /// Extract from Result<T, E> - Ok variant
    pub fn for_ok<T, E>() -> OptionalKeyPath<Result<T, E>, T, impl for<'r> Fn(&'r Result<T, E>) -> Option<&'r T>> {
        OptionalKeyPath::new(|result: &Result<T, E>| result.as_ref().ok())
    }
    
    /// Extract from Result<T, E> - Err variant
    pub fn for_err<T, E>() -> OptionalKeyPath<Result<T, E>, E, impl for<'r> Fn(&'r Result<T, E>) -> Option<&'r E>> {
        OptionalKeyPath::new(|result: &Result<T, E>| result.as_ref().err())
    }
    
    /// Extract from Option<T> - Some variant
    pub fn for_some<T>() -> OptionalKeyPath<Option<T>, T, impl for<'r> Fn(&'r Option<T>) -> Option<&'r T>> {
        OptionalKeyPath::new(|opt: &Option<T>| opt.as_ref())
    }
    
    /// Extract from Option<T> - Some variant (alias for for_some for consistency)
    pub fn for_option<T>() -> OptionalKeyPath<Option<T>, T, impl for<'r> Fn(&'r Option<T>) -> Option<&'r T>> {
        OptionalKeyPath::new(|opt: &Option<T>| opt.as_ref())
    }
    
    /// Unwrap Box<T> -> T
    pub fn for_box<T>() -> KeyPath<Box<T>, T, impl for<'r> Fn(&'r Box<T>) -> &'r T> {
        KeyPath::new(|b: &Box<T>| b.as_ref())
    }
    
    /// Unwrap Arc<T> -> T
    pub fn for_arc<T>() -> KeyPath<Arc<T>, T, impl for<'r> Fn(&'r Arc<T>) -> &'r T> {
        KeyPath::new(|arc: &Arc<T>| arc.as_ref())
    }
    
    /// Unwrap Rc<T> -> T
    pub fn for_rc<T>() -> KeyPath<std::rc::Rc<T>, T, impl for<'r> Fn(&'r std::rc::Rc<T>) -> &'r T> {
        KeyPath::new(|rc: &std::rc::Rc<T>| rc.as_ref())
    }

    /// Unwrap Box<T> -> T (mutable)
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
/// 
/// # Why PhantomData<Root>?
/// 
/// `PhantomData<Root>` is needed because:
/// 1. The `Root` type parameter is not actually stored in the struct (only used in the closure)
/// 2. Rust needs to know the generic type parameter for:
///    - Type checking at compile time
///    - Ensuring correct usage (e.g., `PartialKeyPath<User>` can only be used with `&User`)
///    - Preventing mixing different Root types
/// 3. Without `PhantomData`, Rust would complain that `Root` is unused
/// 4. `PhantomData` is zero-sized - it adds no runtime overhead
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
    
    /// Create a PartialKeyPath from a concrete KeyPath
    /// Alias for `new()` for consistency with `from()` pattern
    pub fn from<Value>(keypath: KeyPath<Root, Value, impl for<'r> Fn(&'r Root) -> &'r Value + 'static>) -> Self
    where
        Value: Any + 'static,
        Root: 'static,
    {
        Self::new(keypath)
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
    
    /// Get a human-readable name for the value type
    /// Returns a string representation of the TypeId
    pub fn kind_name(&self) -> String {
        format!("{:?}", self.value_type_id)
    }
    
    /// Adapt this keypath to work with Arc<Root> instead of Root
    pub fn for_arc(&self) -> PartialOptionalKeyPath<Arc<Root>>
    where
        Root: 'static,
    {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;
        
        PartialOptionalKeyPath {
            getter: Rc::new(move |arc: &Arc<Root>| {
                Some(getter(arc.as_ref()))
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Box<Root> instead of Root
    pub fn for_box(&self) -> PartialOptionalKeyPath<Box<Root>>
    where
        Root: 'static,
    {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;
        
        PartialOptionalKeyPath {
            getter: Rc::new(move |boxed: &Box<Root>| {
                Some(getter(boxed.as_ref()))
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Rc<Root> instead of Root
    pub fn for_rc(&self) -> PartialOptionalKeyPath<Rc<Root>>
    where
        Root: 'static,
    {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;
        
        PartialOptionalKeyPath {
            getter: Rc::new(move |rc: &Rc<Root>| {
                Some(getter(rc.as_ref()))
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Option<Root> instead of Root
    pub fn for_option(&self) -> PartialOptionalKeyPath<Option<Root>>
    where
        Root: 'static,
    {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;
        
        PartialOptionalKeyPath {
            getter: Rc::new(move |opt: &Option<Root>| {
                opt.as_ref().map(|root| getter(root))
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Result<Root, E> instead of Root
    pub fn for_result<E>(&self) -> PartialOptionalKeyPath<Result<Root, E>>
    where
        Root: 'static,
        E: 'static,
    {
        let getter = self.getter.clone();
        let value_type_id = self.value_type_id;
        
        PartialOptionalKeyPath {
            getter: Rc::new(move |result: &Result<Root, E>| {
                result.as_ref().ok().map(|root| getter(root))
            }),
            value_type_id,
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Arc<RwLock<Root>> instead of Root
    /// Note: This requires the Root to be cloned first, then use the keypath on the cloned value
    /// Example: `keypath.get_as::<Value>(&arc_rwlock.read().unwrap().clone())`
    pub fn for_arc_rwlock(&self) -> PartialOptionalKeyPath<Arc<RwLock<Root>>>
    where
        Root: Clone + 'static,
    {
        // We can't return a reference from a guard, so we return None
        // Users should clone the root first: arc_rwlock.read().unwrap().clone()
        PartialOptionalKeyPath {
            getter: Rc::new(move |_arc_rwlock: &Arc<RwLock<Root>>| {
                // Cannot return reference from temporary guard
                // User should clone the root first and use the keypath on the cloned value
                None
            }),
            value_type_id: self.value_type_id,
            _phantom: PhantomData,
        }
    }
    
    /// Adapt this keypath to work with Arc<Mutex<Root>> instead of Root
    /// Note: This requires the Root to be cloned first, then use the keypath on the cloned value
    /// Example: `keypath.get_as::<Value>(&arc_mutex.lock().unwrap().clone())`
    pub fn for_arc_mutex(&self) -> PartialOptionalKeyPath<Arc<Mutex<Root>>>
    where
        Root: Clone + 'static,
    {
        // We can't return a reference from a guard, so we return None
        // Users should clone the root first: arc_mutex.lock().unwrap().clone()
        PartialOptionalKeyPath {
            getter: Rc::new(move |_arc_mutex: &Arc<Mutex<Root>>| {
                // Cannot return reference from temporary guard
                // User should clone the root first and use the keypath on the cloned value
                None
            }),
            value_type_id: self.value_type_id,
            _phantom: PhantomData,
        }
    }
}

/// PartialOptionalKeyPath - Hides the Value type but keeps Root visible
/// Useful for storing optional keypaths in collections without knowing the exact Value type
/// 
/// # Why PhantomData<Root>?
/// 
/// See `PartialKeyPath` documentation for explanation of why `PhantomData` is needed.
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
/// 
/// # Why PhantomData<Root>?
/// 
/// See `PartialKeyPath` documentation for explanation of why `PhantomData` is needed.
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
    
    /// Create a PartialWritableKeyPath from a concrete WritableKeyPath
    /// Alias for `new()` for consistency with `from()` pattern
    pub fn from<Value>(keypath: WritableKeyPath<Root, Value, impl for<'r> Fn(&'r mut Root) -> &'r mut Value + 'static>) -> Self
    where
        Value: Any + 'static,
        Root: 'static,
    {
        Self::new(keypath)
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
/// 
/// # Why PhantomData<Root>?
/// 
/// See `PartialKeyPath` documentation for explanation of why `PhantomData` is needed.
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
/// 
/// # Why No PhantomData?
/// 
/// Unlike `PartialKeyPath`, `AnyKeyPath` doesn't need `PhantomData` because:
/// - Both `Root` and `Value` types are completely erased
/// - We store `TypeId` instead for runtime type checking
/// - The type information is encoded in the closure's behavior, not the struct
/// - There's no generic type parameter to track at compile time
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
    
    /// Create an AnyKeyPath from a concrete OptionalKeyPath
    /// Alias for `new()` for consistency with `from()` pattern
    pub fn from<Root, Value>(keypath: OptionalKeyPath<Root, Value, impl for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static>) -> Self
    where
        Root: Any + 'static,
        Value: Any + 'static,
    {
        Self::new(keypath)
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
    
    /// Get a human-readable name for the value type
    /// Returns a string representation of the TypeId
    pub fn kind_name(&self) -> String {
        format!("{:?}", self.value_type_id)
    }
    
    /// Adapt this keypath to work with Arc<Root> instead of Root
    pub fn for_arc<Root>(&self) -> AnyKeyPath
    where
        Root: Any + 'static,
    {
        let root_type_id = self.root_type_id;
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();
        
        AnyKeyPath {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(arc) = any.downcast_ref::<Arc<Root>>() {
                    getter(arc.as_ref() as &dyn Any)
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Arc<Root>>(),
            value_type_id,
        }
    }
    
    /// Adapt this keypath to work with Box<Root> instead of Root
    pub fn for_box<Root>(&self) -> AnyKeyPath
    where
        Root: Any + 'static,
    {
        let root_type_id = self.root_type_id;
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();
        
        AnyKeyPath {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(boxed) = any.downcast_ref::<Box<Root>>() {
                    getter(boxed.as_ref() as &dyn Any)
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Box<Root>>(),
            value_type_id,
        }
    }
    
    /// Adapt this keypath to work with Rc<Root> instead of Root
    pub fn for_rc<Root>(&self) -> AnyKeyPath
    where
        Root: Any + 'static,
    {
        let root_type_id = self.root_type_id;
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();
        
        AnyKeyPath {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(rc) = any.downcast_ref::<Rc<Root>>() {
                    getter(rc.as_ref() as &dyn Any)
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Rc<Root>>(),
            value_type_id,
        }
    }
    
    /// Adapt this keypath to work with Option<Root> instead of Root
    pub fn for_option<Root>(&self) -> AnyKeyPath
    where
        Root: Any + 'static,
    {
        let root_type_id = self.root_type_id;
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();
        
        AnyKeyPath {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(opt) = any.downcast_ref::<Option<Root>>() {
                    opt.as_ref().and_then(|root| getter(root as &dyn Any))
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Option<Root>>(),
            value_type_id,
        }
    }
    
    /// Adapt this keypath to work with Result<Root, E> instead of Root
    pub fn for_result<Root, E>(&self) -> AnyKeyPath
    where
        Root: Any + 'static,
        E: Any + 'static,
    {
        let root_type_id = self.root_type_id;
        let value_type_id = self.value_type_id;
        let getter = self.getter.clone();
        
        AnyKeyPath {
            getter: Rc::new(move |any: &dyn Any| {
                if let Some(result) = any.downcast_ref::<Result<Root, E>>() {
                    result.as_ref().ok().and_then(|root| getter(root as &dyn Any))
                } else {
                    None
                }
            }),
            root_type_id: TypeId::of::<Result<Root, E>>(),
            value_type_id,
        }
    }
    
    /// Adapt this keypath to work with Arc<RwLock<Root>> instead of Root
    /// Note: This requires the Root to be cloned first, then use the keypath on the cloned value
    pub fn for_arc_rwlock<Root>(&self) -> AnyKeyPath
    where
        Root: Any + Clone + 'static,
    {
        // We can't return a reference from a guard, so we return None
        // Users should clone the root first
        AnyKeyPath {
            getter: Rc::new(move |_any: &dyn Any| {
                // Cannot return reference from temporary guard
                // User should clone the root first and use the keypath on the cloned value
                None
            }),
            root_type_id: TypeId::of::<Arc<RwLock<Root>>>(),
            value_type_id: self.value_type_id,
        }
    }
    
    /// Adapt this keypath to work with Arc<Mutex<Root>> instead of Root
    /// Note: This requires the Root to be cloned first, then use the keypath on the cloned value
    pub fn for_arc_mutex<Root>(&self) -> AnyKeyPath
    where
        Root: Any + Clone + 'static,
    {
        // We can't return a reference from a guard, so we return None
        // Users should clone the root first
        AnyKeyPath {
            getter: Rc::new(move |_any: &dyn Any| {
                // Cannot return reference from temporary guard
                // User should clone the root first and use the keypath on the cloned value
                None
            }),
            root_type_id: TypeId::of::<Arc<Mutex<Root>>>(),
            value_type_id: self.value_type_id,
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

/// FailableCombinedKeyPath - A keypath that supports readable, writable, and owned access patterns
/// 
/// This keypath type combines the functionality of OptionalKeyPath, WritableOptionalKeyPath,
/// and adds owned access. It's useful when you need all three access patterns for the same field.
#[derive(Clone)]
pub struct FailableCombinedKeyPath<Root, Value, ReadFn, WriteFn, OwnedFn>
where
    ReadFn: for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static,
    WriteFn: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static,
    OwnedFn: Fn(Root) -> Option<Value> + 'static,
{
    readable: ReadFn,
    writable: WriteFn,
    owned: OwnedFn,
    _phantom: PhantomData<(Root, Value)>,
}

impl<Root, Value, ReadFn, WriteFn, OwnedFn> FailableCombinedKeyPath<Root, Value, ReadFn, WriteFn, OwnedFn>
where
    ReadFn: for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static,
    WriteFn: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static,
    OwnedFn: Fn(Root) -> Option<Value> + 'static,
{
    /// Create a new FailableCombinedKeyPath with all three access patterns
    pub fn new(readable: ReadFn, writable: WriteFn, owned: OwnedFn) -> Self {
        Self {
            readable,
            writable,
            owned,
            _phantom: PhantomData,
        }
    }
    
    /// Get an immutable reference to the value (readable access)
    pub fn get<'r>(&self, root: &'r Root) -> Option<&'r Value> {
        (self.readable)(root)
    }
    
    /// Get a mutable reference to the value (writable access)
    pub fn get_mut<'r>(&self, root: &'r mut Root) -> Option<&'r mut Value> {
        (self.writable)(root)
    }
    
    /// Get an owned value (owned access) - consumes the root
    pub fn get_failable_owned(&self, root: Root) -> Option<Value> {
        (self.owned)(root)
    }
    
    /// Convert to OptionalKeyPath (loses writable and owned capabilities)
    pub fn to_optional(self) -> OptionalKeyPath<Root, Value, ReadFn> {
        OptionalKeyPath::new(self.readable)
    }
    
    /// Convert to WritableOptionalKeyPath (loses owned capability)
    pub fn to_writable_optional(self) -> WritableOptionalKeyPath<Root, Value, WriteFn> {
        WritableOptionalKeyPath::new(self.writable)
    }
    
    /// Compose this keypath with another FailableCombinedKeyPath
    /// Returns a new FailableCombinedKeyPath that chains both keypaths
    pub fn then<SubValue, SubReadFn, SubWriteFn, SubOwnedFn>(
        self,
        next: FailableCombinedKeyPath<Value, SubValue, SubReadFn, SubWriteFn, SubOwnedFn>,
    ) -> FailableCombinedKeyPath<Root, SubValue, impl for<'r> Fn(&'r Root) -> Option<&'r SubValue> + 'static, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut SubValue> + 'static, impl Fn(Root) -> Option<SubValue> + 'static>
    where
        SubReadFn: for<'r> Fn(&'r Value) -> Option<&'r SubValue> + 'static,
        SubWriteFn: for<'r> Fn(&'r mut Value) -> Option<&'r mut SubValue> + 'static,
        SubOwnedFn: Fn(Value) -> Option<SubValue> + 'static,
        ReadFn: 'static,
        WriteFn: 'static,
        OwnedFn: 'static,
        Value: 'static,
        Root: 'static,
        SubValue: 'static,
    {
        let first_read = self.readable;
        let first_write = self.writable;
        let first_owned = self.owned;
        let second_read = next.readable;
        let second_write = next.writable;
        let second_owned = next.owned;
        
        FailableCombinedKeyPath::new(
            move |root: &Root| {
                first_read(root).and_then(|value| second_read(value))
            },
            move |root: &mut Root| {
                first_write(root).and_then(|value| second_write(value))
            },
            move |root: Root| {
                first_owned(root).and_then(|value| second_owned(value))
            },
        )
    }
    
    /// Compose with OptionalKeyPath (readable only)
    /// Returns a FailableCombinedKeyPath that uses the readable from OptionalKeyPath
    /// and creates dummy writable/owned closures that return None
    pub fn chain_optional<SubValue, SubReadFn>(
        self,
        next: OptionalKeyPath<Value, SubValue, SubReadFn>,
    ) -> FailableCombinedKeyPath<Root, SubValue, impl for<'r> Fn(&'r Root) -> Option<&'r SubValue> + 'static, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut SubValue> + 'static, impl Fn(Root) -> Option<SubValue> + 'static>
    where
        SubReadFn: for<'r> Fn(&'r Value) -> Option<&'r SubValue> + 'static,
        ReadFn: 'static,
        WriteFn: 'static,
        OwnedFn: 'static,
        Value: 'static,
        Root: 'static,
        SubValue: 'static,
    {
        let first_read = self.readable;
        let first_write = self.writable;
        let first_owned = self.owned;
        let second_read = next.getter;
        
        FailableCombinedKeyPath::new(
            move |root: &Root| {
                first_read(root).and_then(|value| second_read(value))
            },
            move |_root: &mut Root| {
                None // Writable not supported when composing with OptionalKeyPath
            },
            move |root: Root| {
                first_owned(root).and_then(|value| {
                    // Try to get owned value, but OptionalKeyPath doesn't support owned
                    None
                })
            },
        )
    }
}

// Factory function for FailableCombinedKeyPath
impl FailableCombinedKeyPath<(), (), fn(&()) -> Option<&()>, fn(&mut ()) -> Option<&mut ()>, fn(()) -> Option<()>> {
    /// Create a FailableCombinedKeyPath with all three access patterns
    pub fn failable_combined<Root, Value, ReadFn, WriteFn, OwnedFn>(
        readable: ReadFn,
        writable: WriteFn,
        owned: OwnedFn,
    ) -> FailableCombinedKeyPath<Root, Value, ReadFn, WriteFn, OwnedFn>
    where
        ReadFn: for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static,
        WriteFn: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static,
        OwnedFn: Fn(Root) -> Option<Value> + 'static,
    {
        FailableCombinedKeyPath::new(readable, writable, owned)
    }
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
    
    /// Alias for `to_partial()` - converts to PartialKeyPath
    pub fn to(self) -> PartialKeyPath<Root> {
        self.to_partial()
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
    
    /// Convert to PartialOptionalKeyPath (alias for `to_partial()`)
    pub fn to(self) -> PartialOptionalKeyPath<Root> {
        self.to_partial()
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
    
    /// Alias for `to_partial()` - converts to PartialWritableKeyPath
    pub fn to(self) -> PartialWritableKeyPath<Root> {
        self.to_partial()
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
    
    /// Convert to PartialWritableOptionalKeyPath (alias for `to_partial()`)
    pub fn to(self) -> PartialWritableOptionalKeyPath<Root> {
        self.to_partial()
    }
}

// pub trait KPTrait<R, V>: {
//     fn getter(r: &R) -> Option<&V>;
//     fn setter(r: &mut R) -> Option< &mut V>;
// }
// pub trait KPGTrait<R, V> {
//     fn getter(r: &R) -> Option<&V>;
// }

// pub trait KPSTrait<R, V> {
//         fn setter(r: &mut R) -> Option< &mut V>;
// }

pub fn test<R, V, F>(x: R) 
where  F: Fn(&R) -> Option<&V> 
{}

type Getter<R, V> = for<'r> fn(&'r R) -> Option<&'r V>;
type Setter<R, V> = for<'r> fn(&'r mut R) -> Option<&'r mut V>;
// type LockGetter<R, V> = for<'r> fn(&'r R) -> Option<Arc<&'r V>>;
// type LockSetter<R, V> = for<'r> fn(&'r mut R) -> Option<Arc<&'r mut V>>;

pub type Kp<R, V> = KpType<
R,
V, 
Getter<R, V>, 
Setter<R, V>,
//  LockGetter<R, V>, 
//  LockSetter<R, V>
 >;

 #[derive(Debug)]
pub struct KpType<R, V, G, S, 
// LG, SG
>
where 
G:for<'r> Fn(&'r R) -> Option<&'r V>,
S:for<'r>  Fn(&'r mut R) -> Option< &'r mut V>,
// LG:for<'r> Fn(&'r R) -> Option<Arc<&'r V>>, 
// SG:for<'r> Fn(&'r mut R) -> Option<Arc<&'r mut V>>,  
{
    g: G,
    s: S,
    // lg: LG,
    // sg: SG,
    _p: PhantomData<(R, V)>
}

impl<R, V, G, S, 
// LG, SG
> KpType<R, V, G, S, 
// LG, SG
>
where 
G:for<'r> Fn(&'r R) -> Option<&'r V>,
S:for<'r>  Fn(&'r mut R) -> Option< &'r mut V>, {
    pub fn new(get: G, set: S) -> Self {
        Self { g: get, s: set, _p: PhantomData }
    }

    pub fn get<'a>(&self, r: &'a R) -> Option<&'a V> {
        (self.g)(r)
    }
    
    pub fn get_mut<'a>(&self, r: &'a mut R) -> Option< &'a mut V> {
        (self.s)(r)
    }
    // pub fn then<SubValue>(
    //     self,
    //     next: Kp<V, SubValue>,
    // ) -> Kp<R, SubValue>
    // {
    //     let first_get = self.g;
    //     let second_get = next.g;
    //     let first_set = self.s;
    //     let second_set = next.s;
        
    //     KpType::new(
    //         move |root: &R| {
    //             first_get(root).and_then(|value| second_get(value))
    //         },
    //         move |root: &mut R| {
    //             first_set(root).and_then(|value| second_set(value))
    //         }
    //     )
    // }



    pub fn then<SubValue, G2, S2>(
        self,
        next: KpType<V, SubValue, G2, S2>,
    ) -> KpType<
        R,
        SubValue,
        impl for<'r> Fn(&'r R) -> Option<&'r SubValue>,
        impl for<'r> Fn(&'r mut R) -> Option<&'r mut SubValue>,
    >
    where
        G2: for<'r> Fn(&'r V) -> Option<&'r SubValue>,
        S2: for<'r> Fn(&'r mut V) -> Option<&'r mut SubValue>,
        V: 'static
    {
        // let first_get = self.g;
        // let second_get = next.g;
        // let first_set = self.s;
        // let second_set = next.s;
        
        KpType::new(
            move |root: &R| {
                (self.g)(root).and_then(|value| (next.g)(value))
            },
            move |root: &mut R| {
                (self.s)(root).and_then(|value| (next.s)(value))
            }
        )
    }
}

// Add identity as an associated function
impl<R> KpType<R, R, Getter<R, R>, Setter<R, R>> {
    /// Creates an identity keypath that returns the value itself
    pub fn identity() -> Self {
        Kp {
            g: |r: &R| { Some(r) },
            s: |r: &mut R| { Some(r) },
            _p: PhantomData
        }
    }
}

struct TestKP {
    a: String,
    b: String,
    c: Arc<String>,
    d: Mutex<String>,
    e: Arc<Mutex<TestKP2>>,
    f: Option<TestKP2>
}

impl TestKP {
    fn new() -> Self {
        Self {         
            a: String::from("a"),
            b: String::from("b"),
            c: Arc::new(String::from("c")),
            d: Mutex::new(String::from("d")),
            e: Arc::new(Mutex::new(TestKP2 { 
                a: String::from("a2"), 
                b: Arc::new(Mutex::new(String::from("b2"))) 
            })),
            f: Some(TestKP2 { 
                a: String::from("a3"), 
                b: Arc::new(Mutex::new(String::from("b3"))) 
            }),
        }
    }

    // Helper to create an identity keypath for TestKP2
    fn identity() -> Kp<TestKP2, TestKP2> {
        Kp {
            g: |r: &TestKP2| { Some(r) },
            s: |r: &mut TestKP2| { Some(r) },
            _p: PhantomData
        }
    }

}

struct TestKP2 {
    a: String,
    b: Arc<Mutex<String>>
}

impl TestKP2 {
    fn a() -> Kp<TestKP2, String> {
        Kp {
            g: |r: &TestKP2| { Some(&r.a) },
            s: |r: &mut TestKP2| { Some(&mut r.a) },
            _p: PhantomData
        }
    }

    fn b() -> Kp<TestKP2, Arc<Mutex<String>>> {
        Kp {
            g: |r: &TestKP2| { Some(&r.b) },
            s: |r: &mut TestKP2| { Some(&mut r.b) },
            _p: PhantomData
        }
    }

    // fn identity() -> Kp<Self, Self> {
    //     Kp::identity()
    // }

}

impl TestKP {
    fn a() -> Kp<TestKP, String> {
        Kp {
            g: |r: &TestKP| { Some(&r.a) },
            s: |r: &mut TestKP| { Some(&mut r.a) },
            _p: PhantomData
        }
    }

    fn b() -> Kp<TestKP, String> {
        Kp {
            g: |r: &TestKP| { Some(&r.b) },
            s: |r: &mut TestKP| { Some(&mut r.b) },
            _p: PhantomData
        }
    }

    fn c() -> Kp<TestKP, String> {
        Kp {
            g: |r: &TestKP| { Some(r.c.as_ref()) },
            s: |r: &mut TestKP| { None },
            _p: PhantomData
        }
    }

    fn d() -> Kp<TestKP, Mutex<String>> {
        Kp {
            g: |r: &TestKP| { Some(&r.d) },
            s: |r: &mut TestKP| { Some(&mut r.d) },
            _p: PhantomData
        }
    }

    fn e() -> Kp<TestKP, Arc<Mutex<TestKP2>>> {
        Kp {
            g: |r: &TestKP| { Some(&r.e) },
            s: |r: &mut TestKP| { Some(&mut r.e) },
            _p: PhantomData
        }
    }

    fn f() -> Kp<TestKP, TestKP2> {
        Kp {
            g: |r: &TestKP| { r.f.as_ref() },
            s: |r: &mut TestKP| { r.f.as_mut() },
            _p: PhantomData
        }
    }
}

#[cfg(test)]
mod testsas {
    use super::*;

    #[test]
    fn test_kp_for_struct() {
        let mut i = TestKP::new();
        let kp = TestKP::f().then(TestKP2::a());
        println!("initial value = {:?}", kp.get(&i));
        if let Some(x) = kp.get_mut(&mut i) {
            *x = "this is also working".to_string();
        }
        println!("updated value = {:?}", kp.get(&i));
        assert_eq!(kp.get(&i), Some(&"this is also working".to_string()));
    }

    #[test]
    fn test_single_mutex_access() {
        let mut root = TestKP::new();
        
        // Create LKp for TestKP.e -> TestKP2.a
        let lkp = LKp::new(TestKP::e(), TestKP2::a());
        
        // Get value through mutex
        let value = lkp.get_cloned(&root);
        println!("Single mutex - initial value: {:?}", value);
        assert_eq!(value, Some("a2".to_string()));
        
        // Mutate value through mutex
        lkp.get_mut(&mut root, |val| {
            *val = "modified a2".to_string();
        });
        
        let new_value = lkp.get_cloned(&root);
        println!("Single mutex - updated value: {:?}", new_value);
        assert_eq!(new_value, Some("modified a2".to_string()));
    }

    #[test]
    fn test_chained_mutex_access() {
        let mut root = TestKP::new();
        
        // Create LKp for TestKP.e -> TestKP2
        let outer_lkp = LKp::new(TestKP::e(), Kp {
            g: |r: &TestKP2| { Some(r) },
            s: |r: &mut TestKP2| { Some(r) },
            _p: PhantomData
        });
        
        // Chain to TestKP2.b (which is Arc<Mutex<String>>)
        let chained_lkp = outer_lkp.then(TestKP2::b());
        
        // Now we have: TestKP.e (Arc<Mutex<TestKP2>>) -> TestKP2 -> TestKP2.b (Arc<Mutex<String>>)
        // This gives us access to the Arc<Mutex<String>>
        let arc_mutex = chained_lkp.get_cloned(&root);
        println!("Chained mutex - Arc<Mutex>: {:?}", arc_mutex);
        
        // To access the inner String, we need to lock it
        if let Some(am) = arc_mutex {
            if let Ok(guard) = am.lock() {
                println!("Chained mutex - inner value: {:?}", *guard);
                assert_eq!(*guard, "b2".to_string());
            }
        }
        
        // Mutate the Arc<Mutex<String>> reference itself (swap it out)
        chained_lkp.get_mut(&mut root, |arc_mutex_ref| {
            *arc_mutex_ref = Arc::new(Mutex::new("replaced b2".to_string()));
        });
        
        // Verify the change
        let new_arc_mutex = chained_lkp.get_cloned(&root);
        if let Some(am) = new_arc_mutex {
            if let Ok(guard) = am.lock() {
                println!("Chained mutex - replaced value: {:?}", *guard);
                assert_eq!(*guard, "replaced b2".to_string());
            }
        }
    }

    #[test]
    fn test_double_nested_mutex() {
        let mut root = TestKP::new();
        
        // First level: TestKP.e -> TestKP2
        let first_lkp: LKp<TestKP, Arc<Mutex<TestKP2>>, TestKP2, TestKP2, _, _> = LKp::new(
            TestKP::e(),
            Kp {
                g: |r: &TestKP2| { Some(r) },
                s: |r: &mut TestKP2| { Some(r) },
                _p: PhantomData
            }
        );
        
        // Second level: chain to TestKP2.b (Arc<Mutex<String>>)
        // This creates: TestKP -> Arc<Mutex<TestKP2>> -> TestKP2.b
        let second_lkp = first_lkp.then(TestKP2::b());
        
        // Now create another LKp to go through the second mutex
        // TestKP -> Arc<Mutex<TestKP2>> -> Arc<Mutex<String>> -> String
        let final_lkp = LKp::new(
            TestKP::e(),
            {
                let inner_kp = Kp {
                    g: |r: &TestKP2| { Some(&r.b) },
                    s: |r: &mut TestKP2| { Some(&mut r.b) },
                    _p: PhantomData
                };
                inner_kp
            }
        );
        
        // Access the deeply nested String value
        let value = final_lkp.get_cloned(&root);
        println!("Double nested mutex - value: {:?}", value);
        
        // The value should be cloned from the inner Arc<Mutex<String>>
        if let Some(arc_string) = value {
            if let Ok(guard) = arc_string.lock() {
                assert_eq!(*guard, "b2".to_string());
            }
        }
    }


    #[test]
    fn test_lkp_then_chaining() {
        let mut root = TestKP::new();
        
        // Start with TestKP.e -> TestKP2 (identity)
        let first_lkp = LKp::new(TestKP::e(), Kp::identity());
        
        // Chain to TestKP2.a to get: TestKP.e -> TestKP2 -> TestKP2.a
        let chained = first_lkp.then(TestKP2::a());
        
        // Access the deeply nested String value
        let value = chained.get_cloned(&root);
        println!("LKp chained - value: {:?}", value);
        assert_eq!(value, Some("a2".to_string()));
        
        // Mutate it
        chained.get_mut(&mut root, |val| {
            *val = "chained modified".to_string();
        });
        
        let new_value = chained.get_cloned(&root);
        assert_eq!(new_value, Some("chained modified".to_string()));
    }

    #[test]
    fn test_simple_keypath_composition() {
        let mut root = TestKP::new();
        
        // Option 1: Direct access without LKp (when you have a direct path)
        let direct_kp = TestKP::f().then(TestKP2::a());
        assert_eq!(direct_kp.get(&root), Some(&"a3".to_string()));
        
        // Option 2: Through mutex using LKp
        let mutex_lkp = LKp::new(TestKP::e(), TestKP2::a());
        assert_eq!(mutex_lkp.get_cloned(&root), Some("a2".to_string()));
        
        // Option 3: Chain LKp for deeper access
        let deep_lkp = LKp::new(
            TestKP::e(),
            Kp::identity()
        ).then(TestKP2::a());
        
        assert_eq!(deep_lkp.get_cloned(&root), Some("a2".to_string()));
    }
}

// ========== SHR OPERATOR IMPLEMENTATIONS (>> operator) ==========
// 
// The `>>` operator provides the same functionality as `then()` methods.
// It requires nightly Rust with the `nightly` feature enabled.
//
// Usage example (requires nightly):
// ```rust
// #![feature(impl_trait_in_assoc_type)]  // Must be in YOUR code
// use rust_keypaths::{keypath, KeyPath};
// 
// struct User { address: Address }
// struct Address { street: String }
// 
// let kp1 = keypath!(|u: &User| &u.address);
// let kp2 = keypath!(|a: &Address| &a.street);
// let chained = kp1 >> kp2; // Works with nightly feature
// ```
//
// On stable Rust, use `keypath1.then(keypath2)` instead.
//
// Supported combinations (same as `then()` methods):
// - `KeyPath >> KeyPath` → `KeyPath`
// - `KeyPath >> OptionalKeyPath` → `OptionalKeyPath`
// - `OptionalKeyPath >> OptionalKeyPath` → `OptionalKeyPath`
// - `WritableKeyPath >> WritableKeyPath` → `WritableKeyPath`
// - `WritableKeyPath >> WritableOptionalKeyPath` → `WritableOptionalKeyPath`
// - `WritableOptionalKeyPath >> WritableOptionalKeyPath` → `WritableOptionalKeyPath`

// #[cfg(feature = "nightly")]
// mod shr_impls {
//     use super::*;
//
//     // Implement Shr for KeyPath >> KeyPath: returns KeyPath
//     impl<Root, Value, F, SubValue, G> Shr<KeyPath<Value, SubValue, G>> for KeyPath<Root, Value, F>
//     where
//         F: for<'r> Fn(&'r Root) -> &'r Value + 'static,
//         G: for<'r> Fn(&'r Value) -> &'r SubValue + 'static,
//         Value: 'static,
//     {
//         type Output = KeyPath<Root, SubValue, impl for<'r> Fn(&'r Root) -> &'r SubValue>;
//
//         fn shr(self, rhs: KeyPath<Value, SubValue, G>) -> Self::Output {
//             self.then(rhs)
//         }
//     }
//
//     // Implement Shr for KeyPath >> OptionalKeyPath: returns OptionalKeyPath
//     impl<Root, Value, F, SubValue, G> Shr<OptionalKeyPath<Value, SubValue, G>> for KeyPath<Root, Value, F>
//     where
//         F: for<'r> Fn(&'r Root) -> &'r Value + 'static,
//         G: for<'r> Fn(&'r Value) -> Option<&'r SubValue> + 'static,
//         Value: 'static,
//     {
//         type Output = OptionalKeyPath<Root, SubValue, impl for<'r> Fn(&'r Root) -> Option<&'r SubValue>>;
//
//         fn shr(self, rhs: OptionalKeyPath<Value, SubValue, G>) -> Self::Output {
//             self.chain_optional(rhs)
//         }
//     }
//
//     // Implement Shr for OptionalKeyPath >> OptionalKeyPath: returns OptionalKeyPath
//     impl<Root, Value, F, SubValue, G> Shr<OptionalKeyPath<Value, SubValue, G>> for OptionalKeyPath<Root, Value, F>
//     where
//         F: for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static,
//         G: for<'r> Fn(&'r Value) -> Option<&'r SubValue> + 'static,
//         Value: 'static,
//     {
//         type Output = OptionalKeyPath<Root, SubValue, impl for<'r> Fn(&'r Root) -> Option<&'r SubValue>>;
//
//         fn shr(self, rhs: OptionalKeyPath<Value, SubValue, G>) -> Self::Output {
//             self.then(rhs)
//         }
//     }
//
//     // Implement Shr for WritableKeyPath >> WritableKeyPath: returns WritableKeyPath
//     impl<Root, Value, F, SubValue, G> Shr<WritableKeyPath<Value, SubValue, G>> for WritableKeyPath<Root, Value, F>
//     where
//         F: for<'r> Fn(&'r mut Root) -> &'r mut Value + 'static,
//         G: for<'r> Fn(&'r mut Value) -> &'r mut SubValue + 'static,
//         Value: 'static,
//     {
//         type Output = WritableKeyPath<Root, SubValue, impl for<'r> Fn(&'r mut Root) -> &'r mut SubValue>;
//
//         fn shr(self, rhs: WritableKeyPath<Value, SubValue, G>) -> Self::Output {
//             self.then(rhs)
//         }
//     }
//
//     // Implement Shr for WritableKeyPath >> WritableOptionalKeyPath: returns WritableOptionalKeyPath
//     impl<Root, Value, F, SubValue, G> Shr<WritableOptionalKeyPath<Value, SubValue, G>> for WritableKeyPath<Root, Value, F>
//     where
//         F: for<'r> Fn(&'r mut Root) -> &'r mut Value + 'static,
//         G: for<'r> Fn(&'r mut Value) -> Option<&'r mut SubValue> + 'static,
//         Value: 'static,
//     {
//         type Output = WritableOptionalKeyPath<Root, SubValue, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut SubValue>>;
//
//         fn shr(self, rhs: WritableOptionalKeyPath<Value, SubValue, G>) -> Self::Output {
//             self.chain_optional(rhs)
//         }
//     }
//
//     // Implement Shr for WritableOptionalKeyPath >> WritableOptionalKeyPath: returns WritableOptionalKeyPath
//     impl<Root, Value, F, SubValue, G> Shr<WritableOptionalKeyPath<Value, SubValue, G>> for WritableOptionalKeyPath<Root, Value, F>
//     where
//         F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static,
//         G: for<'r> Fn(&'r mut Value) -> Option<&'r mut SubValue> + 'static,
//         Value: 'static,
//     {
//         type Output = WritableOptionalKeyPath<Root, SubValue, impl for<'r> Fn(&'r mut Root) -> Option<&'r mut SubValue>>;
//
//         fn shr(self, rhs: WritableOptionalKeyPath<Value, SubValue, G>) -> Self::Output {
//             self.then(rhs)
//         }
//     }
// }

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
            eprintln!("[DEBUG] NoCloneType should not be cloned! ID: {}", self.id);
            unreachable!("NoCloneType should not be cloned! ID: {}", self.id);
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
        name: "Akash".to_string(),
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
    
    // ========== MACRO USAGE EXAMPLES ==========
    
    #[derive(Debug, PartialEq)]
    struct TestUser {
        name: String,
        age: u32,
        metadata: Option<String>,
        address: Option<TestAddress>,
    }
    
    #[derive(Debug, PartialEq)]
    struct TestAddress {
        street: String,
        city: String,
        country: Option<TestCountry>,
    }
    
    #[derive(Debug, PartialEq)]
    struct TestCountry {
        name: String,
    }
    
    #[test]
    fn test_keypath_macro() {
        let user = TestUser {
            name: "Akash".to_string(),
            age: 30,
            metadata: None,
            address: None,
        };
        
        // Simple field access using closure
        let name_kp = keypath!(|u: &TestUser| &u.name);
        assert_eq!(name_kp.get(&user), "Akash");
        
        // Nested field access
        let user_with_address = TestUser {
            name: "Bob".to_string(),
            age: 25,
            metadata: None,
            address: Some(TestAddress {
                street: "123 Main St".to_string(),
                city: "New York".to_string(),
                country: None,
            }),
        };
        
        let street_kp = keypath!(|u: &TestUser| &u.address.as_ref().unwrap().street);
        assert_eq!(street_kp.get(&user_with_address), "123 Main St");
        
        // Deeper nesting
        let user_with_country = TestUser {
            name: "Charlie".to_string(),
            age: 35,
            metadata: None,
            address: Some(TestAddress {
                street: "456 Oak Ave".to_string(),
                city: "London".to_string(),
                country: Some(TestCountry {
                    name: "UK".to_string(),
                }),
            }),
        };
        
        let country_name_kp = keypath!(|u: &TestUser| &u.address.as_ref().unwrap().country.as_ref().unwrap().name);
        assert_eq!(country_name_kp.get(&user_with_country), "UK");
        
        // Fallback: using closure
        let age_kp = keypath!(|u: &TestUser| &u.age);
        assert_eq!(age_kp.get(&user), &30);
    }
    
    #[test]
    fn test_opt_keypath_macro() {
        let user = TestUser {
            name: "Akash".to_string(),
            age: 30,
            metadata: Some("admin".to_string()),
            address: None,
        };
        
        // Simple Option field access using closure
        let metadata_kp = opt_keypath!(|u: &TestUser| u.metadata.as_ref());
        assert_eq!(metadata_kp.get(&user), Some(&"admin".to_string()));
        
        // None case
        let user_no_metadata = TestUser {
            name: "Bob".to_string(),
            age: 25,
            metadata: None,
            address: None,
        };
        assert_eq!(metadata_kp.get(&user_no_metadata), None);
        
        // Nested Option access
        let user_with_address = TestUser {
            name: "Charlie".to_string(),
            age: 35,
            metadata: None,
            address: Some(TestAddress {
                street: "789 Pine Rd".to_string(),
                city: "Paris".to_string(),
                country: None,
            }),
        };
        
        let street_kp = opt_keypath!(|u: &TestUser| u.address.as_ref().map(|a| &a.street));
        assert_eq!(street_kp.get(&user_with_address), Some(&"789 Pine Rd".to_string()));
        
        // Deeper nesting through Options
        let user_with_country = TestUser {
            name: "David".to_string(),
            age: 40,
            metadata: None,
            address: Some(TestAddress {
                street: "321 Elm St".to_string(),
                city: "Tokyo".to_string(),
                country: Some(TestCountry {
                    name: "Japan".to_string(),
                }),
            }),
        };
        
        let country_name_kp = opt_keypath!(|u: &TestUser| u.address.as_ref().and_then(|a| a.country.as_ref().map(|c| &c.name)));
        assert_eq!(country_name_kp.get(&user_with_country), Some(&"Japan".to_string()));
        
        // Fallback: using closure
        let metadata_kp2 = opt_keypath!(|u: &TestUser| u.metadata.as_ref());
        assert_eq!(metadata_kp2.get(&user), Some(&"admin".to_string()));
    }
    
    #[test]
    fn test_writable_keypath_macro() {
        let mut user = TestUser {
            name: "Akash".to_string(),
            age: 30,
            metadata: None,
            address: None,
        };
        
        // Simple field mutation using closure
        let name_kp = writable_keypath!(|u: &mut TestUser| &mut u.name);
        *name_kp.get_mut(&mut user) = "Bob".to_string();
        assert_eq!(user.name, "Bob");
        
        // Nested field mutation
        let mut user_with_address = TestUser {
            name: "Charlie".to_string(),
            age: 25,
            metadata: None,
            address: Some(TestAddress {
                street: "123 Main St".to_string(),
                city: "New York".to_string(),
                country: None,
            }),
        };
        
        let street_kp = writable_keypath!(|u: &mut TestUser| &mut u.address.as_mut().unwrap().street);
        *street_kp.get_mut(&mut user_with_address) = "456 Oak Ave".to_string();
        assert_eq!(user_with_address.address.as_ref().unwrap().street, "456 Oak Ave");
        
        // Deeper nesting
        let mut user_with_country = TestUser {
            name: "David".to_string(),
            age: 35,
            metadata: None,
            address: Some(TestAddress {
                street: "789 Pine Rd".to_string(),
                city: "London".to_string(),
                country: Some(TestCountry {
                    name: "UK".to_string(),
                }),
            }),
        };
        
        let country_name_kp = writable_keypath!(|u: &mut TestUser| &mut u.address.as_mut().unwrap().country.as_mut().unwrap().name);
        *country_name_kp.get_mut(&mut user_with_country) = "United Kingdom".to_string();
        assert_eq!(user_with_country.address.as_ref().unwrap().country.as_ref().unwrap().name, "United Kingdom");
        
        // Fallback: using closure
        let age_kp = writable_keypath!(|u: &mut TestUser| &mut u.age);
        *age_kp.get_mut(&mut user) = 31;
        assert_eq!(user.age, 31);
    }
    
    #[test]
    fn test_writable_opt_keypath_macro() {
        let mut user = TestUser {
            name: "Akash".to_string(),
            age: 30,
            metadata: Some("user".to_string()),
            address: None,
        };
        
        // Simple Option field mutation using closure
        let metadata_kp = writable_opt_keypath!(|u: &mut TestUser| u.metadata.as_mut());
        if let Some(metadata) = metadata_kp.get_mut(&mut user) {
            *metadata = "admin".to_string();
        }
        assert_eq!(user.metadata, Some("admin".to_string()));
        
        // None case - should return None
        let mut user_no_metadata = TestUser {
            name: "Bob".to_string(),
            age: 25,
            metadata: None,
            address: None,
        };
        assert_eq!(metadata_kp.get_mut(&mut user_no_metadata), None);
        
        // Nested Option access
        let mut user_with_address = TestUser {
            name: "Charlie".to_string(),
            age: 35,
            metadata: None,
            address: Some(TestAddress {
                street: "123 Main St".to_string(),
                city: "New York".to_string(),
                country: None,
            }),
        };
        
        let street_kp = writable_opt_keypath!(|u: &mut TestUser| u.address.as_mut().map(|a| &mut a.street));
        if let Some(street) = street_kp.get_mut(&mut user_with_address) {
            *street = "456 Oak Ave".to_string();
        }
        assert_eq!(user_with_address.address.as_ref().unwrap().street, "456 Oak Ave");
        
        // Deeper nesting through Options
        let mut user_with_country = TestUser {
            name: "David".to_string(),
            age: 40,
            metadata: None,
            address: Some(TestAddress {
                street: "789 Pine Rd".to_string(),
                city: "Tokyo".to_string(),
                country: Some(TestCountry {
                    name: "Japan".to_string(),
                }),
            }),
        };
        
        let country_name_kp = writable_opt_keypath!(|u: &mut TestUser| u.address.as_mut().and_then(|a| a.country.as_mut().map(|c| &mut c.name)));
        if let Some(country_name) = country_name_kp.get_mut(&mut user_with_country) {
            *country_name = "Nippon".to_string();
        }
        assert_eq!(user_with_country.address.as_ref().unwrap().country.as_ref().unwrap().name, "Nippon");
        
        // Fallback: using closure
        let metadata_kp2 = writable_opt_keypath!(|u: &mut TestUser| u.metadata.as_mut());
        if let Some(metadata) = metadata_kp2.get_mut(&mut user) {
            *metadata = "super_admin".to_string();
        }
        assert_eq!(user.metadata, Some("super_admin".to_string()));
    }
}

// ========== WithContainer Trait ==========

/// Trait for no-clone callback-based access to container types
/// Provides methods to execute closures with references to values inside containers
/// without requiring cloning of the values
pub trait WithContainer<Root, Value> {
    /// Execute a closure with a reference to the value inside an Arc
    fn with_arc<F, R>(&self, arc: &Arc<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a reference to the value inside a Box
    fn with_box<F, R>(&self, boxed: &Box<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside a Box
    fn with_box_mut<F, R>(&self, boxed: &mut Box<Root>, f: F) -> R
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside an Rc
    fn with_rc<F, R>(&self, rc: &Rc<Root>, f: F) -> R
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a reference to the value inside a Result
    fn with_result<F, R, E>(&self, result: &Result<Root, E>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside a Result
    fn with_result_mut<F, R, E>(&self, result: &mut Result<Root, E>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside an Option
    fn with_option<F, R>(&self, option: &Option<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside an Option
    fn with_option_mut<F, R>(&self, option: &mut Option<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside a RefCell
    fn with_refcell<F, R>(&self, refcell: &RefCell<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside a RefCell
    fn with_refcell_mut<F, R>(&self, refcell: &RefCell<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    #[cfg(feature = "tagged")]
    /// Execute a closure with a reference to the value inside a Tagged
    fn with_tagged<F, R, Tag>(&self, tagged: &Tagged<Root, Tag>, f: F) -> R
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a reference to the value inside a Mutex
    fn with_mutex<F, R>(&self, mutex: &Mutex<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside a Mutex
    fn with_mutex_mut<F, R>(&self, mutex: &mut Mutex<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside an RwLock
    fn with_rwlock<F, R>(&self, rwlock: &RwLock<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside an RwLock
    fn with_rwlock_mut<F, R>(&self, rwlock: &mut RwLock<Root>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;

    /// Execute a closure with a reference to the value inside an Arc<RwLock<Root>>
    fn with_arc_rwlock<F, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: F) -> Option<R>
    where
        F: FnOnce(&Value) -> R;

    /// Execute a closure with a mutable reference to the value inside an Arc<RwLock<Root>>
    fn with_arc_rwlock_mut<F, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: F) -> Option<R>
    where
        F: FnOnce(&mut Value) -> R;
}

// Implement WithContainer for KeyPath
impl<Root, Value, F> WithContainer<Root, Value> for KeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> &'r Value + Clone,
{
    fn with_arc<Callback, R>(&self, arc: &Arc<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_arc(arc, f)
    }

    fn with_box<Callback, R>(&self, boxed: &Box<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_box(boxed, f)
    }

    fn with_box_mut<Callback, R>(&self, _boxed: &mut Box<Root>, _f: Callback) -> R
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        eprintln!("[DEBUG] KeyPath does not support mutable access - use WritableKeyPath instead");
        unreachable!("KeyPath does not support mutable access - use WritableKeyPath instead")
    }

    fn with_rc<Callback, R>(&self, rc: &Rc<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_rc(rc, f)
    }

    fn with_result<Callback, R, E>(&self, result: &Result<Root, E>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_result(result, f)
    }

    fn with_result_mut<Callback, R, E>(&self, _result: &mut Result<Root, E>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None
    }

    fn with_option<Callback, R>(&self, option: &Option<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_option(option, f)
    }

    fn with_option_mut<Callback, R>(&self, _option: &mut Option<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None
    }

    fn with_refcell<Callback, R>(&self, refcell: &RefCell<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_refcell(refcell, f)
    }

    fn with_refcell_mut<Callback, R>(&self, _refcell: &RefCell<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None
    }

    #[cfg(feature = "tagged")]
    fn with_tagged<Callback, R, Tag>(&self, tagged: &Tagged<Root, Tag>, f: Callback) -> R
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        Callback: FnOnce(&Value) -> R,
    {
        self.with_tagged(tagged, f)
    }

    fn with_mutex<Callback, R>(&self, mutex: &Mutex<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_mutex(mutex, f)
    }

    fn with_mutex_mut<Callback, R>(&self, _mutex: &mut Mutex<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None
    }

    fn with_rwlock<Callback, R>(&self, rwlock: &RwLock<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_rwlock(rwlock, f)
    }

    fn with_rwlock_mut<Callback, R>(&self, _rwlock: &mut RwLock<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None
    }

    fn with_arc_rwlock<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_arc_rwlock(arc_rwlock, f)
    }

    fn with_arc_rwlock_mut<Callback, R>(&self, _arc_rwlock: &Arc<RwLock<Root>>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None
    }
}

// Implement WithContainer for OptionalKeyPath - read-only operations only
impl<Root, Value, F> WithContainer<Root, Value> for OptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Value> + Clone,
{
    fn with_arc<Callback, R>(&self, arc: &Arc<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_arc(arc, f)
    }

    fn with_box<Callback, R>(&self, boxed: &Box<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_box(boxed, f)
    }

    fn with_box_mut<Callback, R>(&self, _boxed: &mut Box<Root>, _f: Callback) -> R
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        eprintln!("[DEBUG] OptionalKeyPath does not support mutable access - use WritableOptionalKeyPath instead");
        unreachable!("OptionalKeyPath does not support mutable access - use WritableOptionalKeyPath instead")
    }

    fn with_rc<Callback, R>(&self, rc: &Rc<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_rc(rc, f)
    }

    fn with_result<Callback, R, E>(&self, result: &Result<Root, E>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_result(result, f)
    }

    fn with_result_mut<Callback, R, E>(&self, _result: &mut Result<Root, E>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None // OptionalKeyPath doesn't support mutable access
    }

    fn with_option<Callback, R>(&self, option: &Option<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_option(option, f)
    }

    fn with_option_mut<Callback, R>(&self, _option: &mut Option<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None // OptionalKeyPath doesn't support mutable access
    }

    fn with_refcell<Callback, R>(&self, refcell: &RefCell<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_refcell(refcell, f)
    }

    fn with_refcell_mut<Callback, R>(&self, _refcell: &RefCell<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None // OptionalKeyPath doesn't support mutable access
    }

    #[cfg(feature = "tagged")]
    fn with_tagged<Callback, R, Tag>(&self, tagged: &Tagged<Root, Tag>, f: Callback) -> R
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        Callback: FnOnce(&Value) -> R,
    {
        use std::ops::Deref;
        self.get(tagged.deref())
            .map(|value| f(value))
            .expect("OptionalKeyPath::with_tagged: Tagged should always contain a value that matches the keypath")
    }

    fn with_mutex<Callback, R>(&self, mutex: &Mutex<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_mutex(mutex, f)
    }

    fn with_mutex_mut<Callback, R>(&self, _mutex: &mut Mutex<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None // OptionalKeyPath doesn't support mutable access
    }

    fn with_rwlock<Callback, R>(&self, rwlock: &RwLock<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_rwlock(rwlock, f)
    }

    fn with_rwlock_mut<Callback, R>(&self, _rwlock: &mut RwLock<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None // OptionalKeyPath doesn't support mutable access
    }

    fn with_arc_rwlock<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        self.with_arc_rwlock(arc_rwlock, f)
    }

    fn with_arc_rwlock_mut<Callback, R>(&self, _arc_rwlock: &Arc<RwLock<Root>>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        None // OptionalKeyPath doesn't support mutable access - use WritableOptionalKeyPath instead
    }
}

// Implement WithContainer for WritableKeyPath - supports all mutable operations
impl<Root, Value, F> WithContainer<Root, Value> for WritableKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> &'r mut Value,
{
    fn with_arc<Callback, R>(&self, _arc: &Arc<Root>, _f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        // Arc doesn't support mutable access without interior mutability
        // This method requires &mut Arc<Root> which we don't have
        eprintln!("[DEBUG] WritableKeyPath::with_arc requires &mut Arc<Root> or interior mutability");
        unreachable!("WritableKeyPath::with_arc requires &mut Arc<Root> or interior mutability")
    }

    fn with_box<Callback, R>(&self, boxed: &Box<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        // Box doesn't support getting mutable reference from immutable reference
        // This is a limitation - we'd need &mut Box<Root> for mutable access
        eprintln!("[DEBUG] WritableKeyPath::with_box requires &mut Box<Root> - use with_box_mut instead");
        unreachable!("WritableKeyPath::with_box requires &mut Box<Root> - use with_box_mut instead")
    }

    fn with_box_mut<Callback, R>(&self, boxed: &mut Box<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        let value = self.get_mut(boxed.as_mut());
        f(value)
    }

    fn with_rc<Callback, R>(&self, _rc: &Rc<Root>, _f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        // Rc doesn't support mutable access without interior mutability
        // This method requires &mut Rc<Root> which we don't have
        eprintln!("[DEBUG] WritableKeyPath::with_rc requires &mut Rc<Root> or interior mutability");
        unreachable!("WritableKeyPath::with_rc requires &mut Rc<Root> or interior mutability")
    }

    fn with_result<Callback, R, E>(&self, _result: &Result<Root, E>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // WritableKeyPath requires &mut Root, but we only have &Result<Root, E>
        // This is a limitation - use with_result_mut for mutable access
        None
    }

    fn with_result_mut<Callback, R, E>(&self, result: &mut Result<Root, E>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        result.as_mut().ok().map(|root| {
            let value = self.get_mut(root);
            f(value)
        })
    }

    fn with_option<Callback, R>(&self, _option: &Option<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // WritableKeyPath requires &mut Root, but we only have &Option<Root>
        // This is a limitation - use with_option_mut for mutable access
        None
    }

    fn with_option_mut<Callback, R>(&self, option: &mut Option<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        option.as_mut().map(|root| {
            let value = self.get_mut(root);
            f(value)
        })
    }

    fn with_refcell<Callback, R>(&self, refcell: &RefCell<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // RefCell doesn't allow getting mutable reference from immutable borrow
        // This is a limitation - we'd need try_borrow_mut for mutable access
        None
    }

    fn with_refcell_mut<Callback, R>(&self, refcell: &RefCell<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        refcell.try_borrow_mut().ok().map(|mut borrow| {
            let value = self.get_mut(&mut *borrow);
            f(value)
        })
    }

    #[cfg(feature = "tagged")]
    fn with_tagged<Callback, R, Tag>(&self, _tagged: &Tagged<Root, Tag>, _f: Callback) -> R
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        Callback: FnOnce(&Value) -> R,
    {
        // WritableKeyPath requires &mut Root, but we only have &Tagged<Root, Tag>
        // This is a limitation - Tagged doesn't support mutable access without interior mutability
        eprintln!("[DEBUG] WritableKeyPath::with_tagged requires &mut Tagged<Root, Tag> or interior mutability");
        unreachable!("WritableKeyPath::with_tagged requires &mut Tagged<Root, Tag> or interior mutability")
    }

    fn with_mutex<Callback, R>(&self, mutex: &Mutex<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        mutex.lock().ok().map(|mut guard| {
            let value = self.get_mut(&mut *guard);
            f(value)
        })
    }

    fn with_mutex_mut<Callback, R>(&self, mutex: &mut Mutex<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        // Mutex::get_mut returns Result<&mut Root, PoisonError>
        mutex.get_mut().ok().map(|root| {
            let value = self.get_mut(root);
            f(value)
        })
    }

    fn with_rwlock<Callback, R>(&self, rwlock: &RwLock<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // RwLock read guard doesn't allow mutable access
        // This is a limitation - we'd need write() for mutable access
        None
    }

    fn with_rwlock_mut<Callback, R>(&self, rwlock: &mut RwLock<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        // RwLock::get_mut returns Result<&mut Root, PoisonError>
        rwlock.get_mut().ok().map(|root| {
            let value = self.get_mut(root);
            f(value)
        })
    }

    fn with_arc_rwlock<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // Arc<RwLock> read guard doesn't allow mutable access
        // This is a limitation - we'd need write() for mutable access
        None
    }

    fn with_arc_rwlock_mut<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        arc_rwlock.write().ok().map(|mut guard| {
            let value = self.get_mut(&mut *guard);
            f(value)
        })
    }
}

// Implement WithContainer for WritableOptionalKeyPath - supports all mutable operations
impl<Root, Value, F> WithContainer<Root, Value> for WritableOptionalKeyPath<Root, Value, F>
where
    F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value>,
{
    fn with_arc<Callback, R>(&self, _arc: &Arc<Root>, _f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        // Arc doesn't support mutable access without interior mutability
        // This method requires &mut Arc<Root> which we don't have
        eprintln!("[DEBUG] WritableOptionalKeyPath::with_arc requires &mut Arc<Root> or interior mutability");
        unreachable!("WritableOptionalKeyPath::with_arc requires &mut Arc<Root> or interior mutability")
    }

    fn with_box<Callback, R>(&self, _boxed: &Box<Root>, _f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        // WritableOptionalKeyPath requires &mut Root, but we only have &Box<Root>
        // This is a limitation - use with_box_mut for mutable access
        eprintln!("[DEBUG] WritableOptionalKeyPath::with_box requires &mut Box<Root> - use with_box_mut instead");
        unreachable!("WritableOptionalKeyPath::with_box requires &mut Box<Root> - use with_box_mut instead")
    }

    fn with_box_mut<Callback, R>(&self, boxed: &mut Box<Root>, f: Callback) -> R
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        if let Some(value) = self.get_mut(boxed.as_mut()) {
            f(value)
        } else {
            eprintln!("[DEBUG] WritableOptionalKeyPath failed to get value from Box");
            unreachable!("WritableOptionalKeyPath failed to get value from Box")
        }
    }

    fn with_rc<Callback, R>(&self, _rc: &Rc<Root>, _f: Callback) -> R
    where
        Callback: FnOnce(&Value) -> R,
    {
        // Rc doesn't support mutable access without interior mutability
        // This method requires &mut Rc<Root> which we don't have
        eprintln!("[DEBUG] WritableOptionalKeyPath::with_rc requires &mut Rc<Root> or interior mutability");
        unreachable!("WritableOptionalKeyPath::with_rc requires &mut Rc<Root> or interior mutability")
    }

    fn with_result<Callback, R, E>(&self, _result: &Result<Root, E>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // WritableOptionalKeyPath requires &mut Root, but we only have &Result<Root, E>
        // This is a limitation - use with_result_mut for mutable access
        None
    }

    fn with_result_mut<Callback, R, E>(&self, result: &mut Result<Root, E>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        result.as_mut().ok().and_then(|root| {
            self.get_mut(root).map(|value| f(value))
        })
    }

    fn with_option<Callback, R>(&self, _option: &Option<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // WritableOptionalKeyPath requires &mut Root, but we only have &Option<Root>
        // This is a limitation - use with_option_mut for mutable access
        None
    }

    fn with_option_mut<Callback, R>(&self, option: &mut Option<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        option.as_mut().and_then(|root| {
            self.get_mut(root).map(|value| f(value))
        })
    }

    fn with_refcell<Callback, R>(&self, _refcell: &RefCell<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // RefCell doesn't allow getting mutable reference from immutable borrow
        // This is a limitation - we'd need try_borrow_mut for mutable access
        None
    }

    fn with_refcell_mut<Callback, R>(&self, refcell: &RefCell<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        refcell.try_borrow_mut().ok().and_then(|mut borrow| {
            self.get_mut(&mut *borrow).map(|value| f(value))
        })
    }

    #[cfg(feature = "tagged")]
    fn with_tagged<Callback, R, Tag>(&self, _tagged: &Tagged<Root, Tag>, _f: Callback) -> R
    where
        Tagged<Root, Tag>: std::ops::Deref<Target = Root>,
        Callback: FnOnce(&Value) -> R,
    {
        // WritableOptionalKeyPath requires &mut Root, but we only have &Tagged<Root, Tag>
        // This is a limitation - Tagged doesn't support mutable access without interior mutability
        eprintln!("[DEBUG] WritableOptionalKeyPath::with_tagged requires &mut Tagged<Root, Tag> or interior mutability");
        unreachable!("WritableOptionalKeyPath::with_tagged requires &mut Tagged<Root, Tag> or interior mutability")
    }

    fn with_mutex<Callback, R>(&self, mutex: &Mutex<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        mutex.lock().ok().and_then(|mut guard| {
            self.get_mut(&mut *guard).map(|value| f(value))
        })
    }

    fn with_mutex_mut<Callback, R>(&self, mutex: &mut Mutex<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        // Mutex::get_mut returns Result<&mut Root, PoisonError>
        mutex.get_mut().ok().and_then(|root| {
            self.get_mut(root).map(|value| f(value))
        })
    }

    fn with_rwlock<Callback, R>(&self, _rwlock: &RwLock<Root>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // RwLock read guard doesn't allow mutable access
        // This is a limitation - we'd need write() for mutable access
        None
    }

    fn with_rwlock_mut<Callback, R>(&self, rwlock: &mut RwLock<Root>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        // RwLock::get_mut returns Result<&mut Root, PoisonError>
        rwlock.get_mut().ok().and_then(|root| {
            self.get_mut(root).map(|value| f(value))
        })
    }

    fn with_arc_rwlock<Callback, R>(&self, _arc_rwlock: &Arc<RwLock<Root>>, _f: Callback) -> Option<R>
    where
        Callback: FnOnce(&Value) -> R,
    {
        // Arc<RwLock> read guard doesn't allow mutable access
        // This is a limitation - we'd need write() for mutable access
        None
    }

    fn with_arc_rwlock_mut<Callback, R>(&self, arc_rwlock: &Arc<RwLock<Root>>, f: Callback) -> Option<R>
    where
        Callback: FnOnce(&mut Value) -> R,
    {
        arc_rwlock.write().ok().and_then(|mut guard| {
            self.get_mut(&mut *guard).map(|value| f(value))
        })
    }
}

