use std::collections::HashMap;
use std::hash::Hash;

/// Elements stored in [`IdentifiedVec`] must expose a stable identifier.
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

    /// Inserts at the end, or replaces an existing element with the same id (order preserved).
    pub fn insert(&mut self, item: T) -> Option<T> {
        let id = item.id();
        if let Some(&idx) = self.index.get(&id) {
            return Some(std::mem::replace(&mut self.items[idx], item));
        }
        let idx = self.items.len();
        self.index.insert(id, idx);
        self.items.push(item);
        None
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let idx = self.index.remove(&id)?;
        let removed = self.items.remove(idx);
        for i in idx..self.items.len() {
            self.index.insert(self.items[i].id(), i);
        }
        Some(removed)
    }

    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        if from >= self.items.len() || to >= self.items.len() || from == to {
            return false;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        self.rebuild_index();
        true
    }

    fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, item) in self.items.iter().enumerate() {
            self.index.insert(item.id(), i);
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize)]
    struct IdentifiedVecData<'a, T> {
        items: &'a Vec<T>,
    }

    impl<Id, T> Serialize for IdentifiedVec<Id, T>
    where
        Id: Copy + Eq + Hash,
        T: Identifiable<Id = Id> + Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            IdentifiedVecData {
                items: &self.items,
            }
            .serialize(serializer)
        }
    }

    #[derive(Deserialize)]
    struct IdentifiedVecOwned<T> {
        items: Vec<T>,
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
            let data = IdentifiedVecOwned::<T>::deserialize(deserializer)?;
            let mut vec = IdentifiedVec::new();
            for item in data.items {
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
