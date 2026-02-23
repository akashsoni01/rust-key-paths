//! Query builder for collection keypaths over [rust_key_paths::KpType] when the value type is `Vec<Item>`.

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

#[cfg(test)]
mod tests {
    use super::*;
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

        // Query: active users over 26, limit 2
        let results = users_kp
            .query()
            .filter(|u| u.active)
            .filter(|u| u.age > 26)
            .limit(2)
            .execute(&db);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Charlie");

        // Check if any active user exists
        assert!(users_kp.query().filter(|u| u.active).exists(&db));

        // Count active users
        let count = users_kp.query().filter(|u| u.active).count(&db);
        assert_eq!(count, 3);
    }
}
