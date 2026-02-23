//! Query builder and collection iteration over [rust_key_paths::KpType] when the value type is `Vec<Item>`.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use rust_key_paths::KpType;

/// Query builder for collection keypaths (KpType where value is `Vec<Item>`).
pub struct CollectionQuery<'a, Root, Item> {
    keypath: &'a KpType<'a, Root, Vec<Item>>,
    filters: Vec<Box<dyn Fn(&Item) -> bool + 'a>>,
    limit: Option<usize>,
    offset: usize,
}

impl<'a, Root, Item> CollectionQuery<'a, Root, Item> {
    pub fn new(keypath: &'a KpType<'a, Root, Vec<Item>>) -> Self {
        Self {
            keypath,
            filters: Vec::new(),
            limit: None,
            offset: 0,
        }
    }

    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&Item) -> bool + 'a,
    {
        self.filters.push(Box::new(predicate));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset = n;
        self
    }

    pub fn execute(&self, root: &'a Root) -> Vec<&'a Item> {
        if let Some(vec) = self.keypath.get(root) {
            let mut result: Vec<&'a Item> = vec
                .iter()
                .skip(self.offset)
                .filter(|item| self.filters.iter().all(|f| f(item)))
                .collect();

            if let Some(limit) = self.limit {
                result.truncate(limit);
            }

            result
        } else {
            Vec::new()
        }
    }

    pub fn count(&self, root: &'a Root) -> usize {
        if let Some(vec) = self.keypath.get(root) {
            vec.iter()
                .skip(self.offset)
                .filter(|item| self.filters.iter().all(|f| f(item)))
                .take(self.limit.unwrap_or(usize::MAX))
                .count()
        } else {
            0
        }
    }

    pub fn exists(&self, root: &'a Root) -> bool {
        self.count(root) > 0
    }

    pub fn first(&self, root: &'a Root) -> Option<&'a Item> {
        self.execute(root).into_iter().next()
    }
}

/// Implemented for keypath types that target `Vec<Item>`, enabling `.query()`.
/// The keypath and the reference passed to `query()` share the same lifetime.
pub trait QueryableCollection<'a, Root, Item> {
    fn query(&'a self) -> CollectionQuery<'a, Root, Item>;
}

impl<'a, Root, Item> QueryableCollection<'a, Root, Item> for KpType<'a, Root, Vec<Item>> {
    fn query(&'a self) -> CollectionQuery<'a, Root, Item> {
        CollectionQuery::new(self)
    }
}

// --- Support for KpType<'static, Root, Vec<Item>> (e.g. from #[derive(Kp)]) ---

/// Query builder for collection keypaths with `'static` lifetime (e.g. from #[derive(Kp)]).
/// Pass the root when calling `execute`, `count`, `exists`, or `first`.
pub struct CollectionQueryStatic<'q, Root, Item>
where
    Root: 'static,
    Item: 'static,
{
    keypath: &'q KpType<'static, Root, Vec<Item>>,
    filters: Vec<Box<dyn Fn(&Item) -> bool + 'q>>,
    limit: Option<usize>,
    offset: usize,
}

impl<'q, Root: 'static, Item: 'static> CollectionQueryStatic<'q, Root, Item> {
    pub fn new(keypath: &'q KpType<'static, Root, Vec<Item>>) -> Self {
        Self {
            keypath,
            filters: Vec::new(),
            limit: None,
            offset: 0,
        }
    }

    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&Item) -> bool + 'q,
    {
        self.filters.push(Box::new(predicate));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset = n;
        self
    }

    pub fn execute<'a>(&self, root: &'a Root) -> Vec<&'a Item> {
        if let Some(vec) = get_vec_static(self.keypath, root) {
            let mut result: Vec<&'a Item> = vec
                .iter()
                .skip(self.offset)
                .filter(|item| self.filters.iter().all(|f| f(item)))
                .collect();
            if let Some(limit) = self.limit {
                result.truncate(limit);
            }
            result
        } else {
            Vec::new()
        }
    }

    pub fn count<'a>(&self, root: &'a Root) -> usize {
        if let Some(vec) = get_vec_static(self.keypath, root) {
            vec.iter()
                .skip(self.offset)
                .filter(|item| self.filters.iter().all(|f| f(item)))
                .take(self.limit.unwrap_or(usize::MAX))
                .count()
        } else {
            0
        }
    }

    pub fn exists<'a>(&self, root: &'a Root) -> bool {
        self.count(root) > 0
    }

    pub fn first<'a>(&self, root: &'a Root) -> Option<&'a Item> {
        self.execute(root).into_iter().next()
    }
}

/// Get `&'a Vec<Item>` from a `'static` keypath and `&'a Root`.
#[inline]
fn get_vec_static<'a, Root: 'static, Item: 'static>(
    keypath: &KpType<'static, Root, Vec<Item>>,
    root: &'a Root,
) -> Option<&'a Vec<Item>> {
    let root_static: &'static Root = unsafe { std::mem::transmute(root) };
    let opt = keypath.get(root_static);
    unsafe { std::mem::transmute(opt) }
}

/// Get `&'a mut Vec<Item>` from a `'static` keypath and `&'a mut Root`.
#[inline]
fn get_mut_vec_static<'a, Root: 'static, Item: 'static>(
    keypath: &KpType<'static, Root, Vec<Item>>,
    root: &'a mut Root,
) -> Option<&'a mut Vec<Item>> {
    let root_static: &'static mut Root = unsafe { std::mem::transmute(root) };
    let opt = keypath.get_mut(root_static);
    unsafe { std::mem::transmute(opt) }
}

/// Implemented for `KpType<'static, Root, Vec<Item>>` (e.g. from #[derive(Kp)]), enabling `.query()`.
pub trait QueryableCollectionStatic<Root, Item>
where
    Root: 'static,
    Item: 'static,
{
    fn query(&self) -> CollectionQueryStatic<'_, Root, Item>;
}

impl<Root: 'static, Item: 'static> QueryableCollectionStatic<Root, Item>
    for KpType<'static, Root, Vec<Item>>
{
    fn query(&self) -> CollectionQueryStatic<'_, Root, Item> {
        CollectionQueryStatic::new(self)
    }
}

// ============================================================================
// Core Collection HOF Trait (prior art: iter, predicates, search, transform, etc.)
// ============================================================================

/// Collection operations over a keypath to `Vec<Item>`. Use [`.iter()`](CollectionKeyPath::iter) for
/// iteration, or the various predicates, search, and aggregation methods.
pub trait CollectionKeyPath<Root, Item> {
    // ── Iterators ──────────────────────────────────────────────────────────
    fn iter<'a>(&self, root: &'a Root) -> Box<dyn Iterator<Item = &'a Item> + 'a>;
    fn iter_mut<'a>(&self, root: &'a mut Root) -> Box<dyn Iterator<Item = &'a mut Item> + 'a>;

    // ── Predicates ─────────────────────────────────────────────────────────
    fn all<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool;

    fn any<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool;

    fn none<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool;

    // ── Search & Find ──────────────────────────────────────────────────────
    fn find<'a, F>(&self, root: &'a Root, predicate: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> bool;

    fn find_mut<'a, F>(&self, root: &'a mut Root, predicate: F) -> Option<&'a mut Item>
    where
        F: Fn(&Item) -> bool;

    fn position<F>(&self, root: &Root, predicate: F) -> Option<usize>
    where
        F: Fn(&Item) -> bool;

    fn contains(&self, root: &Root, item: &Item) -> bool
    where
        Item: PartialEq;

    fn contains_by<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool;

    // ── Transformations ────────────────────────────────────────────────────
    fn map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> U;

    fn filter<'a, F>(&self, root: &'a Root, predicate: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> bool;

    fn filter_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> Option<U>;

    fn flat_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> Vec<U>;

    // ── Reductions ─────────────────────────────────────────────────────────
    fn fold<F, Acc>(&self, root: &Root, init: Acc, f: F) -> Acc
    where
        F: Fn(Acc, &Item) -> Acc,
        Acc: Clone;

    fn reduce<F>(&self, root: &Root, f: F) -> Option<Item>
    where
        F: Fn(&Item, &Item) -> Item,
        Item: Clone;

    fn scan<F, St, B>(&self, root: &Root, initial_state: St, f: F) -> Vec<B>
    where
        F: FnMut(&mut St, &Item) -> Option<B>;

    // ── Aggregations ───────────────────────────────────────────────────────
    fn count(&self, root: &Root) -> usize;

    fn count_by<F>(&self, root: &Root, predicate: F) -> usize
    where
        F: Fn(&Item) -> bool;

    // ── Min/Max ────────────────────────────────────────────────────────────
    fn min<'a>(&self, root: &'a Root) -> Option<&'a Item>
    where
        Item: Ord;

    fn max<'a>(&self, root: &'a Root) -> Option<&'a Item>
    where
        Item: Ord;

    fn min_by<'a, F>(&self, root: &'a Root, compare: F) -> Option<&'a Item>
    where
        F: Fn(&Item, &Item) -> Ordering;

    fn max_by<'a, F>(&self, root: &'a Root, compare: F) -> Option<&'a Item>
    where
        F: Fn(&Item, &Item) -> Ordering;

    fn min_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> K,
        K: Ord;

    fn max_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> K,
        K: Ord;

    // ── Partitioning ────────────────────────────────────────────────────────
    fn partition<'a, F>(&self, root: &'a Root, predicate: F) -> (Vec<&'a Item>, Vec<&'a Item>)
    where
        F: Fn(&Item) -> bool;

    fn group_by<'a, F, K>(&self, root: &'a Root, key_fn: F) -> HashMap<K, Vec<&'a Item>>
    where
        F: Fn(&Item) -> K,
        K: Eq + Hash;

    // ── Ordering & Sorting ──────────────────────────────────────────────────
    fn sorted<'a>(&self, root: &'a Root) -> Vec<&'a Item>
    where
        Item: Ord;

    fn sorted_by<'a, F>(&self, root: &'a Root, compare: F) -> Vec<&'a Item>
    where
        F: Fn(&Item, &Item) -> Ordering;

    fn sorted_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> K,
        K: Ord;

    fn reversed<'a>(&self, root: &'a Root) -> Vec<&'a Item>;

    // ── Take/Skip/Slice ────────────────────────────────────────────────────
    fn take<'a>(&self, root: &'a Root, n: usize) -> Vec<&'a Item>;
    fn take_while<'a, F>(&self, root: &'a Root, predicate: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> bool;
    fn skip<'a>(&self, root: &'a Root, n: usize) -> Vec<&'a Item>;
    fn skip_while<'a, F>(&self, root: &'a Root, predicate: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> bool;
    fn slice<'a>(&self, root: &'a Root, start: usize, end: usize) -> Vec<&'a Item>;

    // ── Chunking ────────────────────────────────────────────────────────────
    fn chunks<'a>(&self, root: &'a Root, size: usize) -> Vec<Vec<&'a Item>>;
    fn windows<'a>(&self, root: &'a Root, size: usize) -> Vec<Vec<&'a Item>>;

    // ── Enumeration ────────────────────────────────────────────────────────
    fn enumerate<'a>(&self, root: &'a Root) -> Vec<(usize, &'a Item)>;

    // ── Index Access ───────────────────────────────────────────────────────
    fn at<'a>(&self, root: &'a Root, index: usize) -> Option<&'a Item>;
    fn at_mut<'a>(&self, root: &'a mut Root, index: usize) -> Option<&'a mut Item>;
    fn first<'a>(&self, root: &'a Root) -> Option<&'a Item>;
    fn last<'a>(&self, root: &'a Root) -> Option<&'a Item>;
    fn nth<'a>(&self, root: &'a Root, n: usize) -> Option<&'a Item>;

    // ── Testing ─────────────────────────────────────────────────────────────
    fn is_empty(&self, root: &Root) -> bool;
    fn is_sorted(&self, root: &Root) -> bool
    where
        Item: Ord;
    fn is_sorted_by<F>(&self, root: &Root, compare: F) -> bool
    where
        F: Fn(&Item, &Item) -> Ordering;

    // ── Mutations ──────────────────────────────────────────────────────────
    fn for_each<F>(&self, root: &Root, f: F)
    where
        F: Fn(&Item);
    fn for_each_mut<F>(&self, root: &mut Root, f: F)
    where
        F: Fn(&mut Item);
    fn retain<F>(&self, root: &mut Root, predicate: F)
    where
        F: Fn(&Item) -> bool;

    // ── Advanced ───────────────────────────────────────────────────────────
    fn inspect<'a, F>(&self, root: &'a Root, f: F) -> Vec<&'a Item>
    where
        F: Fn(&Item);
    fn cycle<'a>(&self, root: &'a Root, times: usize) -> Vec<&'a Item>;
    fn step_by<'a>(&self, root: &'a Root, step: usize) -> Vec<&'a Item>;
}

// Implement for KpType<'static, Root, Vec<Item>> (e.g. from #[derive(Kp)])
impl<Root: 'static, Item: 'static> CollectionKeyPath<Root, Item>
    for KpType<'static, Root, Vec<Item>>
{
    fn iter<'a>(&self, root: &'a Root) -> Box<dyn Iterator<Item = &'a Item> + 'a> {
        match get_vec_static(self, root) {
            Some(v) => Box::new(v.iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    fn iter_mut<'a>(&self, root: &'a mut Root) -> Box<dyn Iterator<Item = &'a mut Item> + 'a> {
        match get_mut_vec_static(self, root) {
            Some(v) => Box::new(v.iter_mut()),
            None => Box::new(std::iter::empty()),
        }
    }

    fn all<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or(true, |v| v.iter().all(predicate))
    }

    fn any<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or(false, |v| v.iter().any(|x| predicate(x)))
    }

    fn none<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or(true, |v| v.iter().all(|x| !predicate(x)))
    }

    fn find<'a, F>(&self, root: &'a Root, predicate: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).and_then(|v| v.iter().find(|x| predicate(x)).map(|r| r))
    }

    fn find_mut<'a, F>(&self, root: &'a mut Root, predicate: F) -> Option<&'a mut Item>
    where
        F: Fn(&Item) -> bool,
    {
        get_mut_vec_static(self, root).and_then(|v| {
            let i = v.iter().position(|x| predicate(x))?;
            v.get_mut(i)
        })
    }

    fn position<F>(&self, root: &Root, predicate: F) -> Option<usize>
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).and_then(|v| v.iter().position(predicate))
    }

    fn contains(&self, root: &Root, item: &Item) -> bool
    where
        Item: PartialEq,
    {
        get_vec_static(self, root).map_or(false, |v| v.contains(item))
    }

    fn contains_by<F>(&self, root: &Root, predicate: F) -> bool
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or(false, |v| v.iter().any(|x| predicate(x)))
    }

    fn map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> U,
    {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().map(f).collect())
    }

    fn filter<'a, F>(&self, root: &'a Root, predicate: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().filter(|x| predicate(x)).collect())
    }

    fn filter_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> Option<U>,
    {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().filter_map(f).collect())
    }

    fn flat_map<F, U>(&self, root: &Root, f: F) -> Vec<U>
    where
        F: Fn(&Item) -> Vec<U>,
    {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().flat_map(f).collect())
    }

    fn fold<F, Acc>(&self, root: &Root, init: Acc, f: F) -> Acc
    where
        F: Fn(Acc, &Item) -> Acc,
        Acc: Clone,
    {
        get_vec_static(self, root).map_or_else(|| init.clone(), |v| v.iter().fold(init, f))
    }

    fn reduce<F>(&self, root: &Root, f: F) -> Option<Item>
    where
        F: Fn(&Item, &Item) -> Item,
        Item: Clone,
    {
        get_vec_static(self, root).and_then(|v| {
            let mut it = v.iter();
            let first = it.next()?.clone();
            Some(it.fold(first, |a, b| f(&a, b)))
        })
    }

    fn scan<F, St, B>(&self, root: &Root, initial_state: St, mut f: F) -> Vec<B>
    where
        F: FnMut(&mut St, &Item) -> Option<B>,
    {
        get_vec_static(self, root).map_or_else(Vec::new, |v| {
            v.iter().scan(initial_state, |st, x| f(st, x)).collect()
        })
    }

    fn count(&self, root: &Root) -> usize {
        get_vec_static(self, root).map_or(0, |v| v.len())
    }

    fn count_by<F>(&self, root: &Root, predicate: F) -> usize
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or(0, |v| v.iter().filter(|x| predicate(x)).count())
    }

    fn min<'a>(&self, root: &'a Root) -> Option<&'a Item>
    where
        Item: Ord,
    {
        get_vec_static(self, root).and_then(|v| v.iter().min()).map(|r| r)
    }

    fn max<'a>(&self, root: &'a Root) -> Option<&'a Item>
    where
        Item: Ord,
    {
        get_vec_static(self, root).and_then(|v| v.iter().max()).map(|r| r)
    }

    fn min_by<'a, F>(&self, root: &'a Root, compare: F) -> Option<&'a Item>
    where
        F: Fn(&Item, &Item) -> Ordering,
    {
        get_vec_static(self, root).and_then(|v| v.iter().min_by(|a, b| compare(a, b))).map(|r| r)
    }

    fn max_by<'a, F>(&self, root: &'a Root, compare: F) -> Option<&'a Item>
    where
        F: Fn(&Item, &Item) -> Ordering,
    {
        get_vec_static(self, root).and_then(|v| v.iter().max_by(|a, b| compare(a, b))).map(|r| r)
    }

    fn min_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> K,
        K: Ord,
    {
        get_vec_static(self, root).and_then(|v| v.iter().min_by_key(|x| f(x))).map(|r| r)
    }

    fn max_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Option<&'a Item>
    where
        F: Fn(&Item) -> K,
        K: Ord,
    {
        get_vec_static(self, root).and_then(|v| v.iter().max_by_key(|x| f(x))).map(|r| r)
    }

    fn partition<'a, F>(&self, root: &'a Root, predicate: F) -> (Vec<&'a Item>, Vec<&'a Item>)
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or((vec![], vec![]), |v| {
            v.iter().partition(|x| predicate(x))
        })
    }

    fn group_by<'a, F, K>(&self, root: &'a Root, key_fn: F) -> HashMap<K, Vec<&'a Item>>
    where
        F: Fn(&Item) -> K,
        K: Eq + Hash,
    {
        let mut map: HashMap<K, Vec<&'a Item>> = HashMap::new();
        if let Some(v) = get_vec_static(self, root) {
            for x in v.iter() {
                map.entry(key_fn(x)).or_default().push(x);
            }
        }
        map
    }

    fn sorted<'a>(&self, root: &'a Root) -> Vec<&'a Item>
    where
        Item: Ord,
    {
        let mut out: Vec<&'a Item> = get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().collect());
        out.sort();
        out
    }

    fn sorted_by<'a, F>(&self, root: &'a Root, compare: F) -> Vec<&'a Item>
    where
        F: Fn(&Item, &Item) -> Ordering,
    {
        let mut out: Vec<&'a Item> = get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().collect());
        out.sort_by(|a, b| compare(a, b));
        out
    }

    fn sorted_by_key<'a, F, K>(&self, root: &'a Root, f: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> K,
        K: Ord,
    {
        let mut out: Vec<&'a Item> = get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().collect());
        out.sort_by_key(|x| f(x));
        out
    }

    fn reversed<'a>(&self, root: &'a Root) -> Vec<&'a Item> {
        let mut out: Vec<&'a Item> = get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().collect());
        out.reverse();
        out
    }

    fn take<'a>(&self, root: &'a Root, n: usize) -> Vec<&'a Item> {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().take(n).collect())
    }

    fn take_while<'a, F>(&self, root: &'a Root, predicate: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().take_while(|x| predicate(x)).collect())
    }

    fn skip<'a>(&self, root: &'a Root, n: usize) -> Vec<&'a Item> {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().skip(n).collect())
    }

    fn skip_while<'a, F>(&self, root: &'a Root, predicate: F) -> Vec<&'a Item>
    where
        F: Fn(&Item) -> bool,
    {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().skip_while(|x| predicate(x)).collect())
    }

    fn slice<'a>(&self, root: &'a Root, start: usize, end: usize) -> Vec<&'a Item> {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.get(start..end).map_or_else(Vec::new, |s| s.iter().collect()))
    }

    fn chunks<'a>(&self, root: &'a Root, size: usize) -> Vec<Vec<&'a Item>> {
        get_vec_static(self, root).map_or_else(Vec::new, |v| {
            v.chunks(size).map(|c| c.iter().collect()).collect()
        })
    }

    fn windows<'a>(&self, root: &'a Root, size: usize) -> Vec<Vec<&'a Item>> {
        get_vec_static(self, root).map_or_else(Vec::new, |v| {
            v.windows(size).map(|w| w.iter().collect()).collect()
        })
    }

    fn enumerate<'a>(&self, root: &'a Root) -> Vec<(usize, &'a Item)> {
        get_vec_static(self, root).map_or_else(Vec::new, |v| v.iter().enumerate().map(|(i, x)| (i, x)).collect())
    }

    fn at<'a>(&self, root: &'a Root, index: usize) -> Option<&'a Item> {
        get_vec_static(self, root).and_then(|v| v.get(index))
    }

    fn at_mut<'a>(&self, root: &'a mut Root, index: usize) -> Option<&'a mut Item> {
        get_mut_vec_static(self, root).and_then(|v| v.get_mut(index))
    }

    fn first<'a>(&self, root: &'a Root) -> Option<&'a Item> {
        get_vec_static(self, root).and_then(|v| v.first())
    }

    fn last<'a>(&self, root: &'a Root) -> Option<&'a Item> {
        get_vec_static(self, root).and_then(|v| v.last())
    }

    fn nth<'a>(&self, root: &'a Root, n: usize) -> Option<&'a Item> {
        get_vec_static(self, root).and_then(|v| v.get(n))
    }

    fn is_empty(&self, root: &Root) -> bool {
        get_vec_static(self, root).map_or(true, |v| v.is_empty())
    }

    fn is_sorted(&self, root: &Root) -> bool
    where
        Item: Ord,
    {
        get_vec_static(self, root).map_or(true, |v| v.windows(2).all(|w| w[0] <= w[1]))
    }

    fn is_sorted_by<F>(&self, root: &Root, compare: F) -> bool
    where
        F: Fn(&Item, &Item) -> Ordering,
    {
        get_vec_static(self, root).map_or(true, |v| v.windows(2).all(|w| compare(&w[0], &w[1]) != Ordering::Greater))
    }

    fn for_each<F>(&self, root: &Root, f: F)
    where
        F: Fn(&Item),
    {
        if let Some(v) = get_vec_static(self, root) {
            v.iter().for_each(f);
        }
    }

    fn for_each_mut<F>(&self, root: &mut Root, f: F)
    where
        F: Fn(&mut Item),
    {
        if let Some(v) = get_mut_vec_static(self, root) {
            v.iter_mut().for_each(f);
        }
    }

    fn retain<F>(&self, root: &mut Root, predicate: F)
    where
        F: Fn(&Item) -> bool,
    {
        if let Some(v) = get_mut_vec_static(self, root) {
            v.retain(predicate);
        }
    }

    fn inspect<'a, F>(&self, root: &'a Root, f: F) -> Vec<&'a Item>
    where
        F: Fn(&Item),
    {
        get_vec_static(self, root).map_or_else(Vec::new, |v| {
            v.iter().inspect(|x| f(x)).collect()
        })
    }

    fn cycle<'a>(&self, root: &'a Root, times: usize) -> Vec<&'a Item> {
        get_vec_static(self, root).map_or_else(Vec::new, |v| {
            v.iter().cycle().take(v.len() * times).collect()
        })
    }

    fn step_by<'a>(&self, root: &'a Root, step: usize) -> Vec<&'a Item> {
        get_vec_static(self, root).map_or_else(Vec::new, |v| {
            v.iter().step_by(step).collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{QueryableCollection, *};
    use rust_key_paths::Kp;

    #[test]
    fn test_query_dsl() {
        struct Database {
            users: Vec<User>,
        }

        struct User {
            id: u32,
            name: String,
            age: u32,
            active: bool,
        }

        // Type annotation so the keypath gets a concrete lifetime tied to this scope
        let users_kp: KpType<'_, Database, Vec<User>> = Kp::new(
            |db: &Database| Some(&db.users),
            |db: &mut Database| Some(&mut db.users),
        );

        let db = Database {
            users: vec![
                User {
                    id: 1,
                    name: "Alice".into(),
                    age: 25,
                    active: true,
                },
                User {
                    id: 2,
                    name: "Bob".into(),
                    age: 30,
                    active: false,
                },
                User {
                    id: 3,
                    name: "Charlie".into(),
                    age: 35,
                    active: true,
                },
                User {
                    id: 4,
                    name: "Diana".into(),
                    age: 28,
                    active: true,
                },
            ],
        };

        // Query: active users over 26, limit 2 (use trait to disambiguate from QueryableCollectionStatic)
        let results = QueryableCollection::query(&users_kp)
            .filter(|u| u.active)
            .filter(|u| u.age > 26)
            .limit(2)
            .execute(&db);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Charlie");

        // Check if any active user exists
        assert!(QueryableCollection::query(&users_kp).filter(|u| u.active).exists(&db));

        // Count active users
        let count = QueryableCollection::query(&users_kp).filter(|u| u.active).count(&db);
        assert_eq!(count, 3);
    }
}
