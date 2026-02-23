//! Parallel collection operations for keypaths using [Rayon].
//!
//! Enable the `rayon` feature and use [ParallelCollectionKeyPath] on `KpType<'static, Root, Vec<Item>>`
//! (e.g. from `#[derive(Kp)]`) to run map, filter, reduce, sort, etc. in parallel.
//!
//! [Rayon]: https://docs.rs/rayon

use crate::get_vec_static;
use rayon::prelude::*;
use rust_key_paths::KpType;
use std::collections::HashMap;
use std::hash::Hash;

/// Parallel collection operations for keypaths to `Vec<Item>`.
/// Implemented for `KpType<'static, Root, Vec<Item>>` (e.g. from `#[derive(Kp)]`).
pub trait ParallelCollectionKeyPath<Root, Item> {
    // ── Mapping & transformation ───────────────────────────────────────────
    fn par_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> U + Sync + Send,
        U: Send,
        Item: Sync;

    fn par_filter<'a, F>(&self, root: &'a Root, predicate: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync;

    fn par_filter_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> Option<U> + Sync + Send,
        U: Send,
        Item: Sync;

    fn par_flat_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> Vec<U> + Sync + Send,
        U: Send,
        Item: Sync;

    fn par_map_with_index<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(usize, &Item) -> U + Sync + Send,
        U: Send,
        Item: Sync;

    // ── Reduction & aggregation ────────────────────────────────────────────
    /// Parallel fold: `fold_op` combines accumulator with each item; `reduce_op` combines per-thread results.
    fn par_fold<F, R, ID, Acc>(&self, root: &Root, identity: &ID, fold_op: F, reduce_op: R) -> Acc
    where
        F: Fn(Acc, &Item) -> Acc + Sync + Send,
        R: Fn(Acc, Acc) -> Acc + Sync + Send,
        ID: Fn() -> Acc + Sync + Send,
        Acc: Send + Clone,
        Item: Sync;

    fn par_reduce<F>(&self, root: &Root, op: F) -> Option<Item>
    where
        F: Fn(Item, Item) -> Item + Sync + Send,
        Item: Clone + Sync + Send;

    fn par_count(&self, root: &Root) -> usize
    where
        Item: Sync;

    fn par_count_by<F>(&self, root: &Root, predicate: F) -> usize
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync;

    // ── Search & find ──────────────────────────────────────────────────────
    fn par_find<'a, F>(&self, root: &'a Root, predicate: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync;

    fn par_find_any<'a, F>(&self, root: &'a Root, predicate: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync;

    fn par_any<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync;

    fn par_all<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync;

    // ── Min/Max ────────────────────────────────────────────────────────────
    fn par_min<'a>(&self, root: &'a Root) -> Option<&'a Item>
    where
        Item: Ord + Sync;

    fn par_max<'a>(&self, root: &'a Root) -> Option<&'a Item>
    where
        Item: Ord + Sync;

    fn par_min_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> K + Sync + Send,
        K: Ord + Send,
        Item: Sync;

    fn par_max_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> K + Sync + Send,
        K: Ord + Send,
        Item: Sync;

    // ── Partitioning & grouping ─────────────────────────────────────────────
    fn par_partition<'a, F>(&self, root: &'a Root, predicate: F) -> (Vec<&'a Item>, Vec<&'a Item>)
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync;

    fn par_group_by<'a, F, K>(&self, root: &'a Root, key_fn: F) -> HashMap<K, Vec<&'a Item>>
    where
        F: Fn(&Item) -> K + Sync + Send,
        K: Eq + Hash + Send,
        Item: Sync;

    // ── Sorting ────────────────────────────────────────────────────────────
    fn par_sort(&self, root: &Root) -> Vec<Item>
    where
        Item: Ord + Clone + Sync + Send;

    fn par_sort_by_key<F, K>(&self, root: &Root, f: F) -> Vec<Item>
    where
        F: Fn(&Item) -> K + Sync + Send,
        K: Ord + Send,
        Item: Clone + Sync + Send;

    // ── Side effects ────────────────────────────────────────────────────────
    fn par_for_each<F>(&self, root: &Root, f: F)
    where
        F: Fn(&Item) + Sync + Send,
        Item: Sync;

    fn par_contains(&self, root: &Root, item: &Item) -> bool
    where
        Item: PartialEq + Sync;
}

impl<Root: 'static, Item: 'static> ParallelCollectionKeyPath<Root, Item>
    for KpType<'static, Root, Vec<Item>>
{
    fn par_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> U + Sync + Send,
        U: Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().map(f).collect())
            .unwrap_or_default()
    }

    fn par_filter<'a, F>(&self, root: &'a Root, predicate: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().filter(|x| predicate(x)).collect())
            .unwrap_or_default()
    }

    fn par_filter_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> Option<U> + Sync + Send,
        U: Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().filter_map(f).collect())
            .unwrap_or_default()
    }

    fn par_flat_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> Vec<U> + Sync + Send,
        U: Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().flat_map(f).collect())
            .unwrap_or_default()
    }

    fn par_map_with_index<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(usize, &Item) -> U + Sync + Send,
        U: Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().enumerate().map(|(i, x)| f(i, x)).collect())
            .unwrap_or_default()
    }

    fn par_fold<F, R, ID, Acc>(&self, root: &Root, identity: &ID, fold_op: F, reduce_op: R) -> Acc
    where
        F: Fn(Acc, &Item) -> Acc + Sync + Send,
        R: Fn(Acc, Acc) -> Acc + Sync + Send,
        ID: Fn() -> Acc + Sync + Send,
        Acc: Send + Clone,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| {
                vec.par_iter()
                    .fold(|| identity(), |acc, x| fold_op(acc, x))
                    .reduce(|| identity(), reduce_op)
            })
            .unwrap_or_else(identity)
    }

    fn par_reduce<F>(&self, root: &Root, op: F) -> Option<Item>
    where
        F: Fn(Item, Item) -> Item + Sync + Send,
        Item: Clone + Sync + Send,
    {
        get_vec_static(self, root).and_then(|vec| vec.par_iter().cloned().reduce_with(op))
    }

    fn par_count(&self, root: &Root) -> usize
    where
        Item: Sync,
    {
        get_vec_static(self, root).map(|v| v.len()).unwrap_or(0)
    }

    fn par_count_by<F>(&self, root: &Root, predicate: F) -> usize
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().filter(|x| predicate(x)).count())
            .unwrap_or(0)
    }

    fn par_find<'a, F>(&self, root: &'a Root, predicate: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync,
    {
        get_vec_static(self, root).and_then(|vec| vec.par_iter().find_first(|x| predicate(x)).map(|r| r))
    }

    fn par_find_any<'a, F>(&self, root: &'a Root, predicate: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync,
    {
        get_vec_static(self, root).and_then(|vec| vec.par_iter().find_any(|x| predicate(x)).map(|r| r))
    }

    fn par_any<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().any(predicate))
            .unwrap_or(false)
    }

    fn par_all<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().all(predicate))
            .unwrap_or(true)
    }

    fn par_min<'a>(&self, root: &'a Root) -> Option<&'a Item>
    where
        Item: Ord + Sync,
    {
        get_vec_static(self, root).and_then(|vec| vec.par_iter().min())
    }

    fn par_max<'a>(&self, root: &'a Root) -> Option<&'a Item>
    where
        Item: Ord + Sync,
    {
        get_vec_static(self, root).and_then(|vec| vec.par_iter().max())
    }

    fn par_min_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> K + Sync + Send,
        K: Ord + Send,
        Item: Sync,
    {
        get_vec_static(self, root).and_then(|vec| vec.par_iter().min_by_key(|x| f(x)))
    }

    fn par_max_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> K + Sync + Send,
        K: Ord + Send,
        Item: Sync,
    {
        get_vec_static(self, root).and_then(|vec| vec.par_iter().max_by_key(|x| f(x)))
    }

    fn par_partition<'a, F>(&self, root: &'a Root, predicate: F) -> (Vec<&'a Item>, Vec<&'a Item>)
    where
        F: Fn(&Item) -> bool + Sync + Send,
        Item: Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().partition(|x| predicate(x)))
            .unwrap_or_default()
    }

    fn par_group_by<'a, F, K>(&self, root: &'a Root, key_fn: F) -> HashMap<K, Vec<&'a Item>>
    where
        F: Fn(&Item) -> K + Sync + Send,
        K: Eq + Hash + Send,
        Item: Sync,
    {
        use std::sync::Mutex;
        get_vec_static(self, root)
            .map(|vec| {
                let map = Mutex::new(HashMap::<K, Vec<&'a Item>>::new());
                vec.par_iter().for_each(|x| {
                    let k = key_fn(x);
                    map.lock().unwrap().entry(k).or_default().push(x);
                });
                map.into_inner().unwrap()
            })
            .unwrap_or_default()
    }

    fn par_sort(&self, root: &Root) -> Vec<Item>
    where
        Item: Ord + Clone + Sync + Send,
    {
        get_vec_static(self, root)
            .map(|vec| {
                let mut out = vec.to_vec();
                out.par_sort_unstable();
                out
            })
            .unwrap_or_default()
    }

    fn par_sort_by_key<F, K>(&self, root: &Root, f: F) -> Vec<Item>
    where
        F: Fn(&Item) -> K + Sync + Send,
        K: Ord + Send,
        Item: Clone + Sync + Send,
    {
        get_vec_static(self, root)
            .map(|vec| {
                let mut out = vec.to_vec();
                out.par_sort_unstable_by_key(|x| f(x));
                out
            })
            .unwrap_or_default()
    }

    fn par_for_each<F>(&self, root: &Root, f: F)
    where
        F: Fn(&Item) + Sync + Send,
        Item: Sync,
    {
        if let Some(vec) = get_vec_static(self, root) {
            vec.par_iter().for_each(f);
        }
    }

    fn par_contains(&self, root: &Root, item: &Item) -> bool
    where
        Item: PartialEq + Sync,
    {
        get_vec_static(self, root)
            .map(|vec| vec.par_iter().any(|x| x == item))
            .unwrap_or(false)
    }
}
