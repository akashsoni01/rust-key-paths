use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;

/// Errors from [`Storage`] backends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    NotFound,
    Io(String),
    #[cfg(feature = "serde")]
    Serde(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "storage key not found"),
            Self::Io(msg) => write!(f, "io error: {msg}"),
            #[cfg(feature = "serde")]
            Self::Serde(msg) => write!(f, "serde error: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}

/// Key-value persistence for [`Shared`] values (TCA `PersistenceKey` engine subset).
pub trait Storage<T> {
    fn save(&self, key: &str, value: &T) -> Result<(), StorageError>;
    fn load(&self, key: &str) -> Result<Option<T>, StorageError>;
    fn remove(&self, key: &str) -> Result<(), StorageError> {
        let _ = key;
        Ok(())
    }
}

/// In-process storage — useful in tests and previews.
#[derive(Debug, Clone)]
pub struct InMemoryStorage<T> {
    data: Arc<Mutex<HashMap<String, T>>>,
}

impl<T> Default for InMemoryStorage<T> {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<T: Clone> InMemoryStorage<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Clone> Storage<T> for InMemoryStorage<T> {
    fn save(&self, key: &str, value: &T) -> Result<(), StorageError> {
        self.data.lock().insert(key.to_owned(), value.clone());
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Option<T>, StorageError> {
        Ok(self.data.lock().get(key).cloned())
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        self.data.lock().remove(key);
        Ok(())
    }
}

/// JSON file storage under a directory root (requires `serde` feature).
#[cfg(feature = "serde")]
use std::path::PathBuf;

#[cfg(feature = "serde")]
#[derive(Debug, Clone)]
pub struct FileStorage {
    root: PathBuf,
}

#[cfg(feature = "serde")]
impl FileStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.json"))
    }
}

#[cfg(feature = "serde")]
impl FileStorage {
    pub fn ensure_root(&self) -> Result<(), StorageError> {
        std::fs::create_dir_all(&self.root)
            .map_err(|err| StorageError::Io(err.to_string()))
    }
}

#[cfg(feature = "serde")]
impl<T> Storage<T> for FileStorage
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn save(&self, key: &str, value: &T) -> Result<(), StorageError> {
        self.ensure_root()?;
        let json = serde_json::to_string_pretty(value)
            .map_err(|err| StorageError::Serde(err.to_string()))?;
        std::fs::write(self.path_for(key), json).map_err(|err| StorageError::Io(err.to_string()))
    }

    fn load(&self, key: &str) -> Result<Option<T>, StorageError> {
        let path = self.path_for(key);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path).map_err(|err| StorageError::Io(err.to_string()))?;
        let value = serde_json::from_slice(&bytes)
            .map_err(|err| StorageError::Serde(err.to_string()))?;
        Ok(Some(value))
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        let path = self.path_for(key);
        if path.exists() {
            std::fs::remove_file(path).map_err(|err| StorageError::Io(err.to_string()))?;
        }
        Ok(())
    }
}

struct SharedInner<T> {
    value: Mutex<T>,
    listeners: Mutex<Vec<Sender<()>>>,
}

/// Ref-counted shared value with change notification (TCA `Shared` engine subset).
#[derive(Clone)]
pub struct Shared<T> {
    inner: Arc<SharedInner<T>>,
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(SharedInner {
                value: Mutex::new(value),
                listeners: Mutex::new(Vec::new()),
            }),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&*self.inner.value.lock())
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R
    where
        T: Clone + PartialEq,
    {
        let mut guard = self.inner.value.lock();
        let before = guard.clone();
        let result = f(&mut guard);
        let changed = before != *guard;
        drop(guard);
        if changed {
            self.notify();
        }
        result
    }

    pub fn set(&self, value: T)
    where
        T: PartialEq,
    {
        let mut guard = self.inner.value.lock();
        if *guard != value {
            *guard = value;
            drop(guard);
            self.notify();
        }
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.value.lock().clone()
    }

    pub fn load<S>(storage: &S, key: &str, default: T) -> Result<Self, StorageError>
    where
        S: Storage<T>,
    {
        Ok(Self::new(storage.load(key)?.unwrap_or(default)))
    }

    pub fn persist<S>(&self, storage: &S, key: &str) -> Result<(), StorageError>
    where
        S: Storage<T>,
        T: Clone,
    {
        storage.save(key, &self.get())
    }

    pub fn subscribe(&self) -> SharedSubscriber<T>
    where
        T: Clone + PartialEq,
    {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.inner.listeners.lock().push(tx);
        SharedSubscriber {
            shared: self.clone(),
            rx,
            last: Some(self.get()),
        }
    }

    fn notify(&self) {
        self.inner
            .listeners
            .lock()
            .retain(|tx| tx.send(()).is_ok());
    }
}

/// Subscription to a [`Shared`] value — deduplicates consecutive equal snapshots.
pub struct SharedSubscriber<T> {
    shared: Shared<T>,
    rx: Receiver<()>,
    last: Option<T>,
}

impl<T: Clone + PartialEq> SharedSubscriber<T> {
    pub fn next(&mut self) -> Option<T> {
        loop {
            match self.rx.try_recv() {
                Ok(()) => {
                    while self.rx.try_recv().is_ok() {}
                    let snapshot = self.shared.get();
                    if self.last.as_ref() == Some(&snapshot) {
                        continue;
                    }
                    self.last = Some(snapshot.clone());
                    return Some(snapshot);
                }
                Err(crossbeam_channel::TryRecvError::Empty) => return None,
                Err(crossbeam_channel::TryRecvError::Disconnected) => return None,
            }
        }
    }

    pub fn latest(&self) -> Option<T> {
        self.last.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panic_on_state_clone;
    use crate::test_support::{allow_state_clones, shared_get};

    panic_on_state_clone! {
        #[derive(Debug, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        struct Counter {
            n: i32,
        }
    }

    #[test]
    fn clones_share_underlying_value() {
        let shared = Shared::new(Counter { n: 0 });
        let scoped = shared.clone();
        shared.set(Counter { n: 3 });
        assert_eq!(shared_get(&scoped).n, 3);
    }

    #[test]
    fn subscribe_notifies_on_change() {
        let shared = Shared::new(Counter { n: 0 });
        let mut sub = allow_state_clones(1, || shared.subscribe());
        shared.set(Counter { n: 1 });
        let next = allow_state_clones(2, || sub.next());
        assert_eq!(next.map(|c| c.n), Some(1));
        assert!(sub.next().is_none());
    }

    #[test]
    fn in_memory_storage_round_trip() {
        let storage = InMemoryStorage::new();
        let shared = Shared::new(Counter { n: 7 });
        allow_state_clones(2, || shared.persist(&storage, "counter")).unwrap();
        let loaded =
            allow_state_clones(1, || Shared::load(&storage, "counter", Counter { n: 0 })).unwrap();
        assert_eq!(shared_get(&loaded).n, 7);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn file_storage_round_trip() {
        let dir = std::env::temp_dir().join(format!("rust_elm_shared_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let storage = FileStorage::new(&dir);
        let shared = Shared::new(Counter { n: 42 });
        allow_state_clones(2, || shared.persist(&storage, "counter")).unwrap();
        let loaded =
            allow_state_clones(1, || Shared::load(&storage, "counter", Counter { n: 0 })).unwrap();
        assert_eq!(shared_get(&loaded).n, 42);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
