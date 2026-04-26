// use crate::Kp;
// use async_trait::async_trait;
// use std::pin::Pin;

// /// Used so that `then_async` can infer `V2` from `AsyncKp::Value` without ambiguity
// /// (e.g. `&i32` has both `Borrow<i32>` and `Borrow<&i32>`; this picks the referent).
// /// Implemented only for reference types so there is no overlap with the blanket impl.
// pub trait KeyPathValueTarget {
//     type Target: Sized;
// }
// impl<T> KeyPathValueTarget for &T {
//     type Target = T;
// }
// impl<T> KeyPathValueTarget for &mut T {
//     type Target = T;
// }

// pub trait KpTrait<R, V>: KpReadable<R, V> + KPWritable<R, V> {
//     fn type_id_of_root() -> std::any::TypeId
//     where
//         R: 'static,
//     {
//         std::any::TypeId::of::<R>()
//     }
//     fn type_id_of_value() -> std::any::TypeId
//     where
//         V: 'static,
//     {
//         std::any::TypeId::of::<V>()
//     }

//     fn then<SV, G2, S2>(
//         self,
//         next: Kp<V, SV, G2, S2>,
//     ) -> Kp<
//         R,
//         SV,
//         impl for<'r> Fn(&'r R) -> Option<&'r SV>,
//         impl for<'r> Fn(&'r mut R) -> Option<&'r mut SV>,
//     >
//     where
//         G2: for<'r> Fn(&'r V) -> Option<&'r SV>,
//         S2: for<'r> Fn(&'r mut V) -> Option<&'r mut SV>,
//         for<'r> V: 'r;
// }

// pub trait KpReadable<R, V> {
//     fn get<'a>(&self, root: &'a R) -> Option<&'a V>;
// }
// pub trait KPWritable<R, V> {
//     fn set<'a>(&self, root: &'a mut R) -> Option<&'a mut V>;
// }

// pub trait AccessorTrait<R, V>: KpTrait<R, V> {
//     /// Like [get](KpReadable::get), but takes an optional root.
//     #[inline]
//     fn get_optional<'a>(&self, root: Option<&'a R>) -> Option<&'a V> {
//         root.and_then(|r| self.get(r))
//     }

//     /// Like [set](KPWritable::set), but takes an optional mutable root.
//     #[inline]
//     fn get_mut_optional<'a>(&self, root: Option<&'a mut R>) -> Option<&'a mut V> {
//         root.and_then(|r| self.set(r))
//     }

//     /// Returns the value if the keypath succeeds, otherwise returns fallback from `f`.
//     #[inline]
//     fn get_or_else<'a, F>(&self, root: &'a R, f: F) -> &'a V
//     where
//         F: FnOnce() -> &'a V,
//     {
//         self.get(root).unwrap_or_else(f)
//     }

//     /// Returns the mutable value if the keypath succeeds, otherwise returns fallback from `f`.
//     #[inline]
//     fn get_mut_or_else<'a, F>(&self, root: &'a mut R, f: F) -> &'a mut V
//     where
//         F: FnOnce() -> &'a mut V,
//     {
//         self.set(root).unwrap_or_else(f)
//     }
// }

// pub trait CoercionTrait<R, V>: KpTrait<R, V> {
//     fn for_arc(
//         &self,
//     ) -> Kp<
//         std::sync::Arc<R>,
//         V,
//         impl for<'r> Fn(&'r std::sync::Arc<R>) -> Option<&'r V> + '_,
//         impl for<'r> Fn(&'r mut std::sync::Arc<R>) -> Option<&'r mut V> + '_,
//     > {
//         Kp::new(
//             move |arc_root: &std::sync::Arc<R>| self.get(arc_root.as_ref()),
//             move |arc_root: &mut std::sync::Arc<R>| {
//                 std::sync::Arc::get_mut(arc_root).and_then(|r_mut| self.set(r_mut))
//             },
//         )
//     }

//     fn for_box(
//         &self,
//     ) -> Kp<
//         Box<R>,
//         V,
//         impl for<'r> Fn(&'r Box<R>) -> Option<&'r V> + '_,
//         impl for<'r> Fn(&'r mut Box<R>) -> Option<&'r mut V> + '_,
//     > {
//         Kp::new(
//             move |boxed_root: &Box<R>| self.get(boxed_root.as_ref()),
//             move |boxed_root: &mut Box<R>| self.set(boxed_root.as_mut()),
//         )
//     }

//     /// Convert a keypath-like object into a getter closure.
//     fn into_get(self) -> impl for<'r> Fn(&'r R) -> Option<&'r V>
//     where
//         Self: Sized,
//     {
//         move |root: &R| self.get(root)
//     }

//     /// Convert a keypath-like object into a setter closure.
//     fn into_set(self) -> impl for<'r> Fn(&'r mut R) -> Option<&'r mut V>
//     where
//         Self: Sized,
//     {
//         move |root: &mut R| self.set(root)
//     }
// }

// pub trait HofTrait<R, V, G, S>: KpTrait<R, V>
// where
//     G: for<'r> Fn(&'r R) -> Option<&'r V>,
//     S: for<'r> Fn(&'r mut R) -> Option<&'r mut V>,
// {
//     /// Maps the keypath value into an owned transformed value.
//     fn map<MappedValue, F>(&self, mapper: F) -> impl for<'r> Fn(&'r R) -> Option<MappedValue> + '_
//     where
//         F: Fn(&V) -> MappedValue + 'static,
//     {
//         move |root: &R| self.get(root).map(&mapper)
//     }

//     /// Filters values using a predicate and returns a new keypath.
//     fn filter<F>(
//         &self,
//         predicate: F,
//     ) -> Kp<
//         R,
//         V,
//         impl for<'r> Fn(&'r R) -> Option<&'r V> + '_,
//         impl for<'r> Fn(&'r mut R) -> Option<&'r mut V> + '_,
//     >
//     where
//         F: Fn(&V) -> bool + Clone + 'static,
//     {
//         let predicate_for_get = predicate.clone();
//         Kp::new(
//             move |root: &R| self.get(root).filter(|value| predicate_for_get(value)),
//             move |root: &mut R| self.set(root).filter(|value| predicate(value)),
//         )
//     }

//     /// Maps and flattens the keypath value when mapper returns `Option`.
//     fn filter_map<MappedValue, F>(
//         &self,
//         mapper: F,
//     ) -> impl for<'r> Fn(&'r R) -> Option<MappedValue> + '_
//     where
//         F: Fn(&V) -> Option<MappedValue> + 'static,
//     {
//         move |root: &R| self.get(root).and_then(&mapper)
//     }

//     /// Runs `inspector` for side effects and returns a keypath for the same value.
//     fn inspect<F>(
//         &self,
//         inspector: F,
//     ) -> Kp<
//         R,
//         V,
//         impl for<'r> Fn(&'r R) -> Option<&'r V> + '_,
//         impl for<'r> Fn(&'r mut R) -> Option<&'r mut V> + '_,
//     >
//     where
//         F: Fn(&V) + Clone + 'static,
//     {
//         let inspector_for_get = inspector.clone();
//         Kp::new(
//             move |root: &R| {
//                 self.get(root).inspect(|value| {
//                     inspector_for_get(value);
//                 })
//             },
//             move |root: &mut R| {
//                 self.set(root).inspect(|value| {
//                     inspector(value);
//                 })
//             },
//         )
//     }

//     /// Flat map - maps to an iterator and flattens.
//     fn flat_map<I, Item, F>(&self, mapper: F) -> impl for<'r> Fn(&'r R) -> Vec<Item> + '_
//     where
//         F: Fn(&V) -> I + 'static,
//         I: IntoIterator<Item = Item>,
//     {
//         move |root: &R| {
//             self.get(root)
//                 .map(|value| mapper(value).into_iter().collect())
//                 .unwrap_or_else(Vec::new)
//         }
//     }

//     /// Fold/reduce the value using an accumulator function.
//     fn fold_value<Acc, F>(&self, init: Acc, folder: F) -> impl for<'r> Fn(&'r R) -> Acc + '_
//     where
//         F: Fn(Acc, &V) -> Acc + 'static,
//         Acc: Copy + 'static,
//     {
//         move |root: &R| {
//             self.get(root)
//                 .map(|value| folder(init, value))
//                 .unwrap_or(init)
//         }
//     }

//     /// Check if the value satisfies a predicate.
//     fn any<F>(&self, predicate: F) -> impl for<'r> Fn(&'r R) -> bool + '_
//     where
//         F: Fn(&V) -> bool + 'static,
//     {
//         move |root: &R| self.get(root).map(&predicate).unwrap_or(false)
//     }

//     /// Check if the value satisfies a predicate; returns true for missing values.
//     fn all<F>(&self, predicate: F) -> impl for<'r> Fn(&'r R) -> bool + '_
//     where
//         F: Fn(&V) -> bool + 'static,
//     {
//         move |root: &R| self.get(root).map(&predicate).unwrap_or(true)
//     }

//     /// Count elements in a collection-like value.
//     fn count_items<F>(&self, counter: F) -> impl for<'r> Fn(&'r R) -> Option<usize> + '_
//     where
//         F: Fn(&V) -> usize + 'static,
//     {
//         move |root: &R| self.get(root).map(&counter)
//     }

//     /// Find an item in a collection-like value.
//     fn find_in<Item, F>(&self, finder: F) -> impl for<'r> Fn(&'r R) -> Option<Item> + '_
//     where
//         F: Fn(&V) -> Option<Item> + 'static,
//     {
//         move |root: &R| self.get(root).and_then(&finder)
//     }

//     /// Take first N elements from a collection-like value.
//     fn take<Output, F>(&self, n: usize, taker: F) -> impl for<'r> Fn(&'r R) -> Option<Output> + '_
//     where
//         F: Fn(&V, usize) -> Output + 'static,
//     {
//         move |root: &R| self.get(root).map(|value| taker(value, n))
//     }

//     /// Skip first N elements from a collection-like value.
//     fn skip<Output, F>(&self, n: usize, skipper: F) -> impl for<'r> Fn(&'r R) -> Option<Output> + '_
//     where
//         F: Fn(&V, usize) -> Output + 'static,
//     {
//         move |root: &R| self.get(root).map(|value| skipper(value, n))
//     }

//     /// Partition a collection-like value into two groups.
//     fn partition_value<Output, F>(
//         &self,
//         partitioner: F,
//     ) -> impl for<'r> Fn(&'r R) -> Option<Output> + '_
//     where
//         F: Fn(&V) -> Output + 'static,
//     {
//         move |root: &R| self.get(root).map(&partitioner)
//     }

//     /// Get min value from a collection-like value.
//     fn min_value<Item, F>(&self, min_fn: F) -> impl for<'r> Fn(&'r R) -> Option<Item> + '_
//     where
//         F: Fn(&V) -> Option<Item> + 'static,
//     {
//         move |root: &R| self.get(root).and_then(&min_fn)
//     }

//     /// Get max value from a collection-like value.
//     fn max_value<Item, F>(&self, max_fn: F) -> impl for<'r> Fn(&'r R) -> Option<Item> + '_
//     where
//         F: Fn(&V) -> Option<Item> + 'static,
//     {
//         move |root: &R| self.get(root).and_then(&max_fn)
//     }

//     /// Sum values from a collection-like value.
//     fn sum_value<Sum, F>(&self, sum_fn: F) -> impl for<'r> Fn(&'r R) -> Option<Sum> + '_
//     where
//         F: Fn(&V) -> Sum + 'static,
//     {
//         move |root: &R| self.get(root).map(&sum_fn)
//     }
// }

// /// Lock adapter abstraction used by sync lock keypaths.
// pub trait LockAccess<Lock, Mid> {
//     fn with_read<Rv, F>(&self, lock: &Lock, f: F) -> Option<Rv>
//     where
//         F: FnOnce(&Mid) -> Option<Rv>;

//     fn with_write<Rv, F>(&self, lock: &Lock, f: F) -> Option<Rv>
//     where
//         F: FnOnce(&mut Mid) -> Option<Rv>;
// }

// /// Sync keypath abstraction used by composed async/pin keypaths.
// pub trait SyncKeyPathLike<R, V> {
//     fn sync_get<'a>(&self, root: &'a R) -> Option<&'a V>;
//     fn sync_get_mut<'a>(&self, root: &'a mut R) -> Option<&'a mut V>;
// }

// /// Await abstraction for `#[pin]` future keypaths.
// #[async_trait(?Send)]
// pub trait PinFutureAwaitLike<S, Output> {
//     async fn get_await(&self, this: Pin<&mut S>) -> Option<Output>;
// }

// /// Async lock adapter abstraction used by async lock keypaths.
// #[async_trait(?Send)]
// pub trait AsyncLockLike<Lock, Mid>: Send + Sync {
//     async fn with_read<Rv, F>(&self, lock: &Lock, f: F) -> Option<Rv>
//     where
//         F: FnOnce(&Mid) -> Option<Rv>;

//     async fn with_write<Rv, F>(&self, lock: &Lock, f: F) -> Option<Rv>
//     where
//         F: FnOnce(&mut Mid) -> Option<Rv>;
// }

// /// Async keypath abstraction for async composition.
// #[async_trait(?Send)]
// pub trait AsyncKeyPathLike<R> {
//     type Value;
//     async fn get(&self, root: &R) -> Option<Self::Value>;
// }
