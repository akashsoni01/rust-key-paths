//! Keypath extensions for [`crate::Kp`]: chaining, coercion, and higher-order helpers.
//!
//! Core read/write traits live in [`key_paths_core`]. This module adds `Kp`-specific APIs.

pub use key_paths_core::{
    AccessorTrait, KeyPath, KeyPathAccess, KeyPathGet, KeyPathValueTarget, KpTrait, NavigateVia,
    Readable, Writable,
};

use crate::Kp;

/// Non-reference link roots for [`NavigateVia`] on [`Kp`] (e.g. `Arc<R>`, `Box<R>`).
pub trait LinkRoot {}

impl<T> LinkRoot for std::sync::Arc<T> {}
impl<T> LinkRoot for Box<T> {}

pub trait ChainExt<R, V, Root, Value, MutRoot, MutValue> {
    /// Chain with a sync [`crate::sync_kp::SyncKp`]. Use `.get(root)` / `.get_mut(root)` on the returned keypath.
    fn then_sync<
        Lock,
        Mid,
        V2,
        LockValue,
        MidValue,
        Value2,
        MutLock,
        MutMid,
        MutValue2,
        G1,
        S1,
        L,
        G2,
        S2,
    >(
        self,
        lock_kp: crate::sync_kp::SyncKp<
            V,
            Lock,
            Mid,
            V2,
            Value,
            LockValue,
            MidValue,
            Value2,
            MutValue,
            MutLock,
            MutMid,
            MutValue2,
            G1,
            S1,
            L,
            G2,
            S2,
        >,
    ) -> crate::sync_kp::KpThenSyncKp<
        R,
        V,
        V2,
        Root,
        Value,
        Value2,
        MutRoot,
        MutValue,
        MutValue2,
        Self,
        crate::sync_kp::SyncKp<
            V,
            Lock,
            Mid,
            V2,
            Value,
            LockValue,
            MidValue,
            Value2,
            MutValue,
            MutLock,
            MutMid,
            MutValue2,
            G1,
            S1,
            L,
            G2,
            S2,
        >,
    >
    where
        V: 'static,
        V2: 'static,
        Value: std::borrow::Borrow<V>,
        Value2: std::borrow::Borrow<V2>,
        MutValue: std::borrow::BorrowMut<V>,
        MutValue2: std::borrow::BorrowMut<V2>,
        LockValue: std::borrow::Borrow<Lock>,
        MidValue: std::borrow::Borrow<Mid>,
        MutLock: std::borrow::BorrowMut<Lock>,
        MutMid: std::borrow::BorrowMut<Mid>,
        G1: Fn(Value) -> Option<LockValue>,
        S1: Fn(MutValue) -> Option<MutLock>,
        L: crate::sync_kp::LockAccess<Lock, MidValue> + crate::sync_kp::LockAccess<Lock, MutMid>,
        G2: Fn(MidValue) -> Option<Value2>,
        S2: Fn(MutMid) -> Option<MutValue2>,
        Self: Sized;

    /// Chain with a `#[pin]` Future field await (pin_project pattern). Use `.get_mut(&mut root).await` on the returned keypath.
    #[cfg(feature = "pin_project")]
    fn then_pin_future<Struct, Output, L>(
        self,
        pin_fut: L,
    ) -> crate::pin::KpThenPinFuture<R, Struct, Output, Root, MutRoot, Value, MutValue, Self, L>
    where
        Struct: std::marker::Unpin + 'static,
        Output: 'static,
        Value: std::borrow::Borrow<Struct>,
        MutValue: std::borrow::BorrowMut<Struct>,
        L: crate::pin::PinFutureAwaitLike<Struct, Output> + Sync,
        Self: Sized;

    /// Chain with an async keypath (e.g. [`crate::async_lock::AsyncLockKp`]). Use `.get(&root).await` on the returned keypath.
    fn then_async<AsyncKp>(
        self,
        async_kp: AsyncKp,
    ) -> crate::async_lock::KpThenAsyncKeyPath<
        R,
        V,
        <AsyncKp::Value as KeyPathValueTarget>::Target,
        Root,
        Value,
        AsyncKp::Value,
        MutRoot,
        MutValue,
        AsyncKp::MutValue,
        Self,
        AsyncKp,
    >
    where
        Value: std::borrow::Borrow<V>,
        MutValue: std::borrow::BorrowMut<V>,
        AsyncKp: crate::async_lock::AsyncKeyPathLike<Value, MutValue>,
        AsyncKp::Value: KeyPathValueTarget
            + std::borrow::Borrow<<AsyncKp::Value as KeyPathValueTarget>::Target>,
        AsyncKp::MutValue: std::borrow::BorrowMut<<AsyncKp::Value as KeyPathValueTarget>::Target>,
        <AsyncKp::Value as KeyPathValueTarget>::Target: 'static,
        Self: Sized;
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S> ChainExt<R, V, Root, Value, MutRoot, MutValue>
    for Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    fn then_sync<
        Lock,
        Mid,
        V2,
        LockValue,
        MidValue,
        Value2,
        MutLock,
        MutMid,
        MutValue2,
        G1,
        S1,
        L,
        G2,
        S2,
    >(
        self,
        lock_kp: crate::sync_kp::SyncKp<
            V,
            Lock,
            Mid,
            V2,
            Value,
            LockValue,
            MidValue,
            Value2,
            MutValue,
            MutLock,
            MutMid,
            MutValue2,
            G1,
            S1,
            L,
            G2,
            S2,
        >,
    ) -> crate::sync_kp::KpThenSyncKp<
        R,
        V,
        V2,
        Root,
        Value,
        Value2,
        MutRoot,
        MutValue,
        MutValue2,
        Self,
        crate::sync_kp::SyncKp<
            V,
            Lock,
            Mid,
            V2,
            Value,
            LockValue,
            MidValue,
            Value2,
            MutValue,
            MutLock,
            MutMid,
            MutValue2,
            G1,
            S1,
            L,
            G2,
            S2,
        >,
    >
    where
        V: 'static,
        V2: 'static,
        Value: std::borrow::Borrow<V>,
        Value2: std::borrow::Borrow<V2>,
        MutValue: std::borrow::BorrowMut<V>,
        MutValue2: std::borrow::BorrowMut<V2>,
        LockValue: std::borrow::Borrow<Lock>,
        MidValue: std::borrow::Borrow<Mid>,
        MutLock: std::borrow::BorrowMut<Lock>,
        MutMid: std::borrow::BorrowMut<Mid>,
        G1: Fn(Value) -> Option<LockValue>,
        S1: Fn(MutValue) -> Option<MutLock>,
        L: crate::sync_kp::LockAccess<Lock, MidValue> + crate::sync_kp::LockAccess<Lock, MutMid>,
        G2: Fn(MidValue) -> Option<Value2>,
        S2: Fn(MutMid) -> Option<MutValue2>,
    {
        let first = self;
        let second = lock_kp;

        crate::sync_kp::KpThenSyncKp::new(first, second)
    }

    #[cfg(feature = "pin_project")]
    fn then_pin_future<Struct, Output, L>(
        self,
        pin_fut: L,
    ) -> crate::pin::KpThenPinFuture<R, Struct, Output, Root, MutRoot, Value, MutValue, Self, L>
    where
        Struct: std::marker::Unpin + 'static,
        Output: 'static,
        Value: std::borrow::Borrow<Struct>,
        MutValue: std::borrow::BorrowMut<Struct>,
        L: crate::pin::PinFutureAwaitLike<Struct, Output> + Sync,
    {
        let first = self;
        let second = pin_fut;

        crate::pin::KpThenPinFuture::new(first, second)
    }

    fn then_async<AsyncKp>(
        self,
        async_kp: AsyncKp,
    ) -> crate::async_lock::KpThenAsyncKeyPath<
        R,
        V,
        <AsyncKp::Value as KeyPathValueTarget>::Target,
        Root,
        Value,
        AsyncKp::Value,
        MutRoot,
        MutValue,
        AsyncKp::MutValue,
        Self,
        AsyncKp,
    >
    where
        Value: std::borrow::Borrow<V>,
        MutValue: std::borrow::BorrowMut<V>,
        AsyncKp: crate::async_lock::AsyncKeyPathLike<Value, MutValue>,
        AsyncKp::Value: KeyPathValueTarget
            + std::borrow::Borrow<<AsyncKp::Value as KeyPathValueTarget>::Target>,
        AsyncKp::MutValue: std::borrow::BorrowMut<<AsyncKp::Value as KeyPathValueTarget>::Target>,
        <AsyncKp::Value as KeyPathValueTarget>::Target: 'static,
    {
        let first = self;
        let second = async_kp;

        crate::async_lock::KpThenAsyncKeyPath::new(first, second)
    }
}

pub trait CoercionTrait<R, V, Root, Value, MutRoot, MutValue, G, S>:
    KpTrait<R, V, Root, Value, MutRoot, MutValue>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    fn for_arc<'b>(
        &self,
    ) -> Kp<
        std::sync::Arc<R>,
        V,
        std::sync::Arc<R>,
        Value,
        std::sync::Arc<R>,
        MutValue,
        impl Fn(std::sync::Arc<R>) -> Option<Value>,
        impl Fn(std::sync::Arc<R>) -> Option<MutValue>,
    >
    where
        R: 'b,
        V: 'b,
        Root: for<'a> From<&'a R>,
        MutRoot: for<'a> From<&'a mut R>;

    fn for_box<'a>(
        &self,
    ) -> Kp<
        Box<R>,
        V,
        Box<R>,
        Value,
        Box<R>,
        MutValue,
        impl Fn(Box<R>) -> Option<Value>,
        impl Fn(Box<R>) -> Option<MutValue>,
    >
    where
        R: 'a,
        V: 'a,
        Root: for<'b> From<&'b R>,
        MutRoot: for<'b> From<&'b mut R>;

    fn into_set(self) -> impl Fn(MutRoot) -> Option<MutValue>;

    fn into_get(self) -> impl Fn(Root) -> Option<Value>;
}

pub trait HofTrait<R, V, Root, Value, MutRoot, MutValue, G, S>:
    KpTrait<R, V, Root, Value, MutRoot, MutValue>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    fn map<MappedValue, F>(
        &self,
        mapper: F,
    ) -> Kp<
        R,
        MappedValue,
        Root,
        MappedValue,
        MutRoot,
        MappedValue,
        impl Fn(Root) -> Option<MappedValue> + '_,
        impl Fn(MutRoot) -> Option<MappedValue> + '_,
    >
    where
        F: Fn(&V) -> MappedValue + Copy + 'static,
        MappedValue: 'static,
    {
        Kp::new(
            move |root: Root| {
                Readable::get(self, root).map(|value| {
                    let v: &V = value.borrow();
                    mapper(v)
                })
            },
            move |root: MutRoot| {
                Writable::set(self, root).map(|value| {
                    let v: &V = value.borrow();
                    mapper(v)
                })
            },
        )
    }

    fn filter<F>(
        &self,
        predicate: F,
    ) -> Kp<
        R,
        V,
        Root,
        Value,
        MutRoot,
        MutValue,
        impl Fn(Root) -> Option<Value> + '_,
        impl Fn(MutRoot) -> Option<MutValue> + '_,
    >
    where
        F: Fn(&V) -> bool + Copy + 'static,
    {
        Kp::new(
            move |root: Root| {
                Readable::get(self, root).filter(|value| {
                    let v: &V = value.borrow();
                    predicate(v)
                })
            },
            move |root: MutRoot| {
                Writable::set(self, root).filter(|value| {
                    let v: &V = value.borrow();
                    predicate(v)
                })
            },
        )
    }

    fn filter_map<MappedValue, F>(
        &self,
        mapper: F,
    ) -> Kp<
        R,
        MappedValue,
        Root,
        MappedValue,
        MutRoot,
        MappedValue,
        impl Fn(Root) -> Option<MappedValue> + '_,
        impl Fn(MutRoot) -> Option<MappedValue> + '_,
    >
    where
        F: Fn(&V) -> Option<MappedValue> + Copy + 'static,
    {
        Kp::new(
            move |root: Root| {
                Readable::get(self, root).and_then(|value| {
                    let v: &V = value.borrow();
                    mapper(v)
                })
            },
            move |root: MutRoot| {
                Writable::set(self, root).and_then(|value| {
                    let v: &V = value.borrow();
                    mapper(v)
                })
            },
        )
    }

    fn inspect<F>(
        &self,
        inspector: F,
    ) -> Kp<
        R,
        V,
        Root,
        Value,
        MutRoot,
        MutValue,
        impl Fn(Root) -> Option<Value> + '_,
        impl Fn(MutRoot) -> Option<MutValue> + '_,
    >
    where
        F: Fn(&V) + Clone + 'static,
    {
        let inspector_for_get = inspector.clone();
        Kp::new(
            move |root: Root| {
                Readable::get(self, root).map(|value| {
                    let v: &V = value.borrow();
                    inspector_for_get(v);
                    value
                })
            },
            move |root: MutRoot| {
                Writable::set(self, root).map(|value| {
                    let v: &V = value.borrow();
                    inspector(v);
                    value
                })
            },
        )
    }

    fn flat_map<I, Item, F>(&self, mapper: F) -> impl Fn(Root) -> Vec<Item> + '_
    where
        F: Fn(&V) -> I + 'static,
        I: IntoIterator<Item = Item>,
    {
        move |root: Root| {
            Readable::get(self, root)
                .map(|value| {
                    let v: &V = value.borrow();
                    mapper(v).into_iter().collect()
                })
                .unwrap_or_else(Vec::new)
        }
    }

    fn fold_value<Acc, F>(&self, init: Acc, folder: F) -> impl Fn(Root) -> Acc + '_
    where
        F: Fn(Acc, &V) -> Acc + 'static,
        Acc: Copy + 'static,
    {
        move |root: Root| {
            Readable::get(self, root)
                .map(|value| {
                    let v: &V = value.borrow();
                    folder(init, v)
                })
                .unwrap_or(init)
        }
    }

    fn any<F>(&self, predicate: F) -> impl Fn(Root) -> bool + '_
    where
        F: Fn(&V) -> bool + 'static,
    {
        move |root: Root| {
            Readable::get(self, root)
                .map(|value| {
                    let v: &V = value.borrow();
                    predicate(v)
                })
                .unwrap_or(false)
        }
    }

    fn all<F>(&self, predicate: F) -> impl Fn(Root) -> bool + '_
    where
        F: Fn(&V) -> bool + 'static,
    {
        move |root: Root| {
            Readable::get(self, root)
                .map(|value| {
                    let v: &V = value.borrow();
                    predicate(v)
                })
                .unwrap_or(true)
        }
    }

    fn count_items<F>(&self, counter: F) -> impl Fn(Root) -> Option<usize> + '_
    where
        F: Fn(&V) -> usize + 'static,
    {
        move |root: Root| {
            Readable::get(self, root).map(|value| {
                let v: &V = value.borrow();
                counter(v)
            })
        }
    }

    fn find_in<Item, F>(&self, finder: F) -> impl Fn(Root) -> Option<Item> + '_
    where
        F: Fn(&V) -> Option<Item> + 'static,
    {
        move |root: Root| {
            Readable::get(self, root).and_then(|value| {
                let v: &V = value.borrow();
                finder(v)
            })
        }
    }

    fn take<Output, F>(&self, n: usize, taker: F) -> impl Fn(Root) -> Option<Output> + '_
    where
        F: Fn(&V, usize) -> Output + 'static,
    {
        move |root: Root| {
            Readable::get(self, root).map(|value| {
                let v: &V = value.borrow();
                taker(v, n)
            })
        }
    }

    fn skip<Output, F>(&self, n: usize, skipper: F) -> impl Fn(Root) -> Option<Output> + '_
    where
        F: Fn(&V, usize) -> Output + 'static,
    {
        move |root: Root| {
            Readable::get(self, root).map(|value| {
                let v: &V = value.borrow();
                skipper(v, n)
            })
        }
    }

    fn partition_value<Output, F>(&self, partitioner: F) -> impl Fn(Root) -> Option<Output> + '_
    where
        F: Fn(&V) -> Output + 'static,
    {
        move |root: Root| {
            Readable::get(self, root).map(|value| {
                let v: &V = value.borrow();
                partitioner(v)
            })
        }
    }

    fn min_value<Item, F>(&self, min_fn: F) -> impl Fn(Root) -> Option<Item> + '_
    where
        F: Fn(&V) -> Option<Item> + 'static,
    {
        move |root: Root| {
            Readable::get(self, root).and_then(|value| {
                let v: &V = value.borrow();
                min_fn(v)
            })
        }
    }

    fn max_value<Item, F>(&self, max_fn: F) -> impl Fn(Root) -> Option<Item> + '_
    where
        F: Fn(&V) -> Option<Item> + 'static,
    {
        move |root: Root| {
            Readable::get(self, root).and_then(|value| {
                let v: &V = value.borrow();
                max_fn(v)
            })
        }
    }

    fn sum_value<Sum, F>(&self, sum_fn: F) -> impl Fn(Root) -> Option<Sum> + '_
    where
        F: Fn(&V) -> Sum + 'static,
    {
        move |root: Root| {
            Readable::get(self, root).map(|value| {
                let v: &V = value.borrow();
                sum_fn(v)
            })
        }
    }
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S> Readable<Root, Value>
    for Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    #[inline]
    fn get(&self, root: Root) -> Option<Value> {
        (self.get)(root)
    }
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S> Writable<MutRoot, MutValue>
    for Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    #[inline]
    fn set(&self, root: MutRoot) -> Option<MutValue> {
        (self.set)(root)
    }
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S> NavigateVia<Kp<R, V, Root, Value, MutRoot, MutValue, G, S>>
    for Root
where
    Root: LinkRoot,
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    type Output = Option<Value>;

    #[inline]
    fn navigate_via(self, kp: &Kp<R, V, Root, Value, MutRoot, MutValue, G, S>) -> Option<Value> {
        Readable::get(kp, self)
    }
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S> KpTrait<R, V, Root, Value, MutRoot, MutValue>
    for Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    fn then<SV, SubValue, MutSubValue, Next>(
        self,
        next: Next,
    ) -> impl KeyPath<Root, SubValue, MutRoot, MutSubValue>
    where
        SubValue: std::borrow::Borrow<SV>,
        MutSubValue: std::borrow::BorrowMut<SV>,
        Next: Readable<Value, SubValue> + Writable<MutValue, MutSubValue> + Clone,
    {
        let first_get = self.get;
        let first_set = self.set;
        let next_get = next.clone();
        let next_set = next;

        Kp::new(
            move |root: Root| {
                first_get(root).and_then(|value| Readable::get(&next_get, value))
            },
            move |root: MutRoot| {
                first_set(root).and_then(|value| Writable::set(&next_set, value))
            },
        )
    }
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S>
    CoercionTrait<R, V, Root, Value, MutRoot, MutValue, G, S>
    for Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
    fn for_arc<'b>(
        &self,
    ) -> Kp<
        std::sync::Arc<R>,
        V,
        std::sync::Arc<R>,
        Value,
        std::sync::Arc<R>,
        MutValue,
        impl Fn(std::sync::Arc<R>) -> Option<Value>,
        impl Fn(std::sync::Arc<R>) -> Option<MutValue>,
    >
    where
        R: 'b,
        V: 'b,
        Root: for<'a> From<&'a R>,
        MutRoot: for<'a> From<&'a mut R>,
    {
        Kp::new(
            move |arc_root: std::sync::Arc<R>| {
                let r_ref: &R = &*arc_root;
                (self.get)(Root::from(r_ref))
            },
            move |mut arc_root: std::sync::Arc<R>| {
                std::sync::Arc::get_mut(&mut arc_root)
                    .and_then(|r_mut| (self.set)(MutRoot::from(r_mut)))
            },
        )
    }

    fn for_box<'a>(
        &self,
    ) -> Kp<
        Box<R>,
        V,
        Box<R>,
        Value,
        Box<R>,
        MutValue,
        impl Fn(Box<R>) -> Option<Value>,
        impl Fn(Box<R>) -> Option<MutValue>,
    >
    where
        R: 'a,
        V: 'a,
        Root: for<'b> From<&'b R>,
        MutRoot: for<'b> From<&'b mut R>,
    {
        Kp::new(
            move |r: Box<R>| {
                let r_ref: &R = r.as_ref();
                (self.get)(Root::from(r_ref))
            },
            move |mut r: Box<R>| (self.set)(MutRoot::from(r.as_mut())),
        )
    }

    #[inline]
    fn into_set(self) -> impl Fn(MutRoot) -> Option<MutValue> {
        self.set
    }

    #[inline]
    fn into_get(self) -> impl Fn(Root) -> Option<Value> {
        self.get
    }
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S>
    HofTrait<R, V, Root, Value, MutRoot, MutValue, G, S>
    for Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
}

impl<R, V, Root, Value, MutRoot, MutValue, G, S> AccessorTrait<Root, Value, MutRoot, MutValue>
    for Kp<R, V, Root, Value, MutRoot, MutValue, G, S>
where
    Root: std::borrow::Borrow<R>,
    Value: std::borrow::Borrow<V>,
    MutRoot: std::borrow::BorrowMut<R>,
    MutValue: std::borrow::BorrowMut<V>,
    G: Fn(Root) -> Option<Value>,
    S: Fn(MutRoot) -> Option<MutValue>,
{
}
