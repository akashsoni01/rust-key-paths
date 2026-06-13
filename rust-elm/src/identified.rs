use std::collections::HashMap;
use std::hash::Hash;

/// Elements stored in [`IdentifiedVec`] must expose a stable identifier.
///
/// The id must not change while an element is stored in the collection. Mutating
/// the id through [`IdentifiedVec::get_mut`] without updating the index will
/// break lookups — prefer [`IdentifiedVec::update`] for in-place field changes.
pub trait Identifiable {
    type Id: Copy + Eq + Hash;
    fn id(&self) -> Self::Id;
}

/// Ordered collection with O(1) id lookup (TCA `IdentifiedArray` parity).
#[derive(Debug, Clone)]
pub struct IdentifiedVec<Id, T> {
    items: Vec<T>,
    index: HashMap<Id, usize>,
}

impl<Id, T> PartialEq for IdentifiedVec<Id, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}

impl<Id, T> Eq for IdentifiedVec<Id, T> where T: Eq {}

impl<Id, T> Default for IdentifiedVec<Id, T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            index: HashMap::new(),
        }
    }
}

impl<Id, T> IdentifiedVec<Id, T>
where
    Id: Copy + Eq + Hash,
    T: Identifiable<Id = Id>,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            index: HashMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.items.iter().map(Identifiable::id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        self.index.get(&id).map(|&idx| &self.items[idx])
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        self.index.get(&id).map(|&idx| &mut self.items[idx])
    }

    pub fn index_of(&self, id: Id) -> Option<usize> {
        self.index.get(&id).copied()
    }

    pub fn contains(&self, id: Id) -> bool {
        self.index.contains_key(&id)
    }

    /// Apply a closure to the element with `id`. Returns `false` when the id is missing.
    ///
    /// The closure must not change the element's id; use [`Self::insert`] to move an
    /// element under a new id.
    pub fn update(&mut self, id: Id, f: impl FnOnce(&mut T)) -> bool {
        if let Some(item) = self.get_mut(id) {
            f(item);
            debug_assert!(self.is_consistent());
            true
        } else {
            false
        }
    }

    /// Returns `true` when the internal index matches element order and ids.
    pub fn is_consistent(&self) -> bool {
        if self.index.len() != self.items.len() {
            return false;
        }
        self.items.iter().enumerate().all(|(i, item)| {
            self.index.get(&item.id()) == Some(&i)
        })
    }

    /// Inserts at the end, or replaces an existing element with the same id (order preserved).
    pub fn insert(&mut self, item: T) -> Option<T> {
        let id = item.id();
        if let Some(&idx) = self.index.get(&id) {
            let old = std::mem::replace(&mut self.items[idx], item);
            debug_assert!(self.is_consistent());
            return Some(old);
        }
        let idx = self.items.len();
        self.index.insert(id, idx);
        self.items.push(item);
        debug_assert!(self.is_consistent());
        None
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let idx = self.index.remove(&id)?;
        let removed = self.items.remove(idx);
        for i in idx..self.items.len() {
            self.index.insert(self.items[i].id(), i);
        }
        debug_assert!(self.is_consistent());
        Some(removed)
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from >= self.items.len() || to >= self.items.len() || from == to {
            return false;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        self.rebuild_index();
        debug_assert!(self.is_consistent());
        true
    }

    fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, item) in self.items.iter().enumerate() {
            self.index.insert(item.id(), i);
        }
    }
}

impl<'a, Id, T> IntoIterator for &'a IdentifiedVec<Id, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'a, Id, T> IntoIterator for &'a mut IdentifiedVec<Id, T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter_mut()
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl<Id, T> Serialize for IdentifiedVec<Id, T>
    where
        Id: Copy + Eq + Hash,
        T: Identifiable<Id = Id> + Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.items.serialize(serializer)
        }
    }

    impl<'de, Id, T> Deserialize<'de> for IdentifiedVec<Id, T>
    where
        Id: Copy + Eq + Hash,
        T: Identifiable<Id = Id> + Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let items = Vec::<T>::deserialize(deserializer)?;
            let mut vec = IdentifiedVec::new();
            for item in items {
                vec.insert(item);
            }
            Ok(vec)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    struct Item {
        id: u64,
        value: i32,
    }

    impl Identifiable for Item {
        type Id = u64;
        fn id(&self) -> u64 {
            self.id
        }
    }

    #[test]
    fn insert_and_get_preserves_order() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 10 });
        vec.insert(Item { id: 2, value: 20 });
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(1).unwrap().value, 10);
        assert_eq!(vec.ids().collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn insert_same_id_replaces_in_place() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 1 });
        vec.insert(Item { id: 2, value: 2 });
        vec.insert(Item { id: 1, value: 99 });
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(1).unwrap().value, 99);
        assert_eq!(vec.ids().collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn remove_by_id_updates_index() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 1 });
        vec.insert(Item { id: 2, value: 2 });
        vec.insert(Item { id: 3, value: 3 });
        assert_eq!(vec.remove(2).unwrap().value, 2);
        assert!(vec.get(2).is_none());
        assert_eq!(vec.get(3).unwrap().value, 3);
        assert_eq!(vec.ids().collect::<Vec<_>>(), vec![1, 3]);
    }

    #[test]
    fn reorder_moves_element() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 1 });
        vec.insert(Item { id: 2, value: 2 });
        vec.insert(Item { id: 3, value: 3 });
        assert!(vec.reorder(2, 0));
        assert_eq!(vec.ids().collect::<Vec<_>>(), vec![3, 1, 2]);
        assert_eq!(vec.index_of(3), Some(0));
    }

    #[test]
    fn empty_vec_operations() {
        let mut vec: IdentifiedVec<u64, Item> = IdentifiedVec::new();
        assert!(vec.is_empty());
        assert!(!vec.contains(1));
        assert!(vec.get(1).is_none());
        assert!(vec.remove(1).is_none());
        assert!(!vec.reorder(0, 0));
        assert!(vec.is_consistent());
    }

    #[test]
    fn insert_returns_replaced_element() {
        let mut vec = IdentifiedVec::new();
        assert!(vec.insert(Item { id: 1, value: 10 }).is_none());
        let old = vec.insert(Item { id: 1, value: 20 }).unwrap();
        assert_eq!(old.value, 10);
        assert_eq!(vec.get(1).unwrap().value, 20);
    }

    #[test]
    fn remove_first_and_last() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 1 });
        vec.insert(Item { id: 2, value: 2 });
        vec.insert(Item { id: 3, value: 3 });

        assert_eq!(vec.remove(1).unwrap().value, 1);
        assert_eq!(vec.ids().collect::<Vec<_>>(), vec![2, 3]);
        assert_eq!(vec.index_of(2), Some(0));

        assert_eq!(vec.remove(3).unwrap().value, 3);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(2).unwrap().value, 2);
        assert!(vec.is_consistent());
    }

    #[test]
    fn reorder_forward_and_backward() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 1 });
        vec.insert(Item { id: 2, value: 2 });
        vec.insert(Item { id: 3, value: 3 });

        assert!(vec.reorder(0, 2));
        assert_eq!(vec.ids().collect::<Vec<_>>(), vec![2, 3, 1]);

        assert!(vec.reorder(2, 0));
        assert_eq!(vec.ids().collect::<Vec<_>>(), vec![1, 2, 3]);
        assert!(vec.is_consistent());
    }

    #[test]
    fn reorder_out_of_bounds_is_noop() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 1 });
        assert!(!vec.reorder(0, 1));
        assert!(!vec.reorder(1, 0));
        assert!(!vec.reorder(0, 0));
    }

    #[test]
    fn update_mutates_without_changing_id() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 1 });
        assert!(vec.update(1, |item| item.value = 42));
        assert_eq!(vec.get(1).unwrap().value, 42);
        assert!(!vec.update(99, |_| {}));
        assert!(vec.is_consistent());
    }

    #[test]
    fn mutating_id_via_get_mut_breaks_consistency() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 1 });
        if let Some(item) = vec.get_mut(1) {
            item.id = 99;
        }
        assert!(!vec.is_consistent());
        // Index still maps old id `1` to slot 0 even though the stored id changed.
        assert!(vec.get(1).is_some());
        assert_eq!(vec.get(1).unwrap().id, 99);
        assert!(vec.get(99).is_none());
    }

    #[test]
    fn into_iterator_matches_ids_order() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 10 });
        vec.insert(Item { id: 2, value: 20 });
        let values: Vec<i32> = (&vec).into_iter().map(|i| i.value).collect();
        assert_eq!(values, vec![10, 20]);
    }

    #[test]
    fn partial_eq_ignores_index_only_compares_items() {
        let mut a = IdentifiedVec::new();
        a.insert(Item { id: 1, value: 1 });
        a.insert(Item { id: 2, value: 2 });

        let mut b = IdentifiedVec::new();
        b.insert(Item { id: 2, value: 2 });
        b.insert(Item { id: 1, value: 1 });
        assert_ne!(a, b);

        b.reorder(1, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn with_capacity_and_default() {
        let vec = IdentifiedVec::<u64, Item>::with_capacity(8);
        assert!(vec.is_empty());
        assert!(vec.is_consistent());
        assert_eq!(IdentifiedVec::<u64, Item>::default().len(), 0);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_empty_array() {
        let vec: IdentifiedVec<u64, Item> = IdentifiedVec::new();
        let json = serde_json::to_string(&vec).unwrap();
        let decoded: IdentifiedVec<u64, Item> = serde_json::from_str(&json).unwrap();
        assert!(decoded.is_empty());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_duplicate_ids_last_wins() {
        let json = r#"[{"id":1,"value":1},{"id":2,"value":2},{"id":1,"value":99}]"#;
        let decoded: IdentifiedVec<u64, Item> = serde_json::from_str(json).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.get(1).unwrap().value, 99);
        assert_eq!(decoded.ids().collect::<Vec<_>>(), vec![1, 2]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        let mut vec = IdentifiedVec::new();
        vec.insert(Item { id: 1, value: 7 });
        vec.insert(Item { id: 2, value: 8 });
        let json = serde_json::to_string(&vec).unwrap();
        let decoded: IdentifiedVec<u64, Item> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, vec);
    }
}
