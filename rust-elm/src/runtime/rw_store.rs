//! [`RwLock`](parking_lot::RwLock)-backed store for read-heavy workloads.
//!
//! Use one [`RwStore`] handle from [`RwRuntime::rw_store`](crate::RwRuntime::rw_store).
//! Call [`RwStore::read_store`] when you need concurrent readers — you do not need a
//! separate runtime accessor.

use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::Receiver;
use key_paths_core::{FieldDiff, hash_value};
use parking_lot::RwLock;

use crate::effect::EffectId;
use crate::optics::Casepath;
use crate::runtime::binding::{ReadStateBinding, RwStateBinding};
use crate::runtime::state_access::StateRead;
use crate::runtime::store::{ChangeSet, StoreHub, StoreTask, StoreWork};
use key_paths_core::RefKpTrait;

pub(crate) struct RwStoreBackend<S, M> {
    pub state: Arc<RwLock<S>>,
    pub hub: Arc<StoreHub<S, M>>,
}

impl<S, M> Clone for RwStoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            hub: self.hub.clone(),
        }
    }
}

impl<S, M> StoreWork<S, M> for RwStoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn hub(&self) -> &Arc<StoreHub<S, M>> {
        &self.hub
    }
}

impl<S, M> RwStoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    pub fn new(state: Arc<RwLock<S>>, hub: Arc<StoreHub<S, M>>) -> Self {
        Self { state, hub }
    }

    pub fn rw_store(&self) -> RwStore<S, M> {
        RwStore {
            backend: self.clone(),
        }
    }

    pub fn read_store(&self) -> ReadStore<S, M> {
        ReadStore {
            backend: self.clone(),
        }
    }
}

/// Writable store backed by [`RwLock`]. Readers use [`Self::read_store`].
pub struct RwStore<S, M> {
    pub(crate) backend: RwStoreBackend<S, M>,
}

impl<S, M> Clone for RwStore<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
        }
    }
}

impl<S, M> RwStore<S, M>
where
    S: Send + Sync + Clone + 'static,
    M: Send + 'static,
{
    pub fn send(&self, action: M) -> StoreTask {
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.backend.hub.effect_done.lock().push_back(tx);
        self.backend.begin_store_work();
        let _ = self.backend.hub.sender.send_blocking(action);
        StoreTask::from_rx(rx)
    }

    pub fn dispatch(&self, action: M) {
        let _ = self.send(action);
    }

    pub fn state(&self) -> S {
        self.backend.state.with_read(|s| s.clone())
    }

    pub fn binding(&self) -> RwStateBinding<S> {
        RwStateBinding::new(Arc::clone(&self.backend.state))
    }

    /// Read-only view for concurrent `read()` — obtain per reader thread from one [`RwStore`].
    pub fn read_store(&self) -> ReadStore<S, M> {
        self.backend.read_store()
    }

    pub fn cancel(&self, id: EffectId) {
        if let Some(handle) = self
            .backend
            .hub
            .interpreter
            .cancel_tokens
            .lock()
            .remove(&id)
        {
            handle.abort();
        }
    }

    pub fn subscribe_state(&self) -> RwStateSubscriber<S>
    where
        S: Clone + PartialEq,
    {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.backend.hub.state_listeners.lock().push(tx);
        RwStateSubscriber {
            rx,
            state: self.backend.state.clone(),
            last: Some(Arc::new(self.backend.state.with_read(|s| s.clone()))),
        }
    }

    pub fn subscribe_changes(&self) -> RwChangeSubscriber<S>
    where
        S: FieldDiff + Hash,
    {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.backend.hub.state_listeners.lock().push(tx);
        let (last_hash, last_fields) = self.backend.state.with_read(|guard| {
            let root = hash_value(guard);
            let mut fields = Vec::new();
            guard.field_hashes(&mut fields);
            (Some(root), fields)
        });
        RwChangeSubscriber {
            rx,
            state: self.backend.state.clone(),
            last_hash,
            last_fields,
            _marker: PhantomData,
        }
    }

    pub fn scope<CS, CM, AK, SK>(
        &self,
        state_kp: SK,
        action_kp: AK,
    ) -> ScopedRwStore<S, M, CS, CM, AK, SK>
    where
        CS: Clone + PartialEq + Send + Sync + 'static,
        CM: Clone + Send + 'static,
        S: 'static,
        M: 'static,
        AK: Casepath<M, CM> + Clone + Send + Sync + 'static,
        SK: RefKpTrait<S, CS> + Clone,
    {
        ScopedRwStore {
            store: self.clone(),
            state_kp,
            action_kp,
            _marker: PhantomData,
        }
    }
}

/// Read-only store handle — many threads can [`read`](parking_lot::RwLock::read) concurrently.
pub struct ReadStore<S, M> {
    pub(crate) backend: RwStoreBackend<S, M>,
}

impl<S, M> Clone for ReadStore<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
        }
    }
}

impl<S, M> ReadStore<S, M>
where
    S: Send + Sync + Clone + 'static,
    M: Send + 'static,
{
    pub fn with_read<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        self.backend.state.with_read(f)
    }

    pub fn state(&self) -> S {
        self.with_read(|s| s.clone())
    }

    pub fn read_binding(&self) -> ReadStateBinding<S> {
        ReadStateBinding::new(Arc::clone(&self.backend.state))
    }

    pub fn subscribe_state(&self) -> RwStateSubscriber<S>
    where
        S: Clone + PartialEq,
    {
        self.backend.rw_store().subscribe_state()
    }

    pub fn subscribe_changes(&self) -> RwChangeSubscriber<S>
    where
        S: FieldDiff + Hash,
    {
        self.backend.rw_store().subscribe_changes()
    }
}

pub struct RwStateSubscriber<S> {
    rx: Receiver<()>,
    state: Arc<RwLock<S>>,
    last: Option<Arc<S>>,
}

impl<S: PartialEq + Clone> RwStateSubscriber<S> {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Arc<S>> {
        loop {
            match self.rx.try_recv() {
                Ok(()) => {
                    while self.rx.try_recv().is_ok() {}
                    let snapshot = Arc::new(self.state.with_read(|s| s.clone()));
                    if self
                        .last
                        .as_ref()
                        .is_some_and(|prev| prev.as_ref() == snapshot.as_ref())
                    {
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

    pub fn wait_next(&mut self, timeout: Duration) -> Option<Arc<S>> {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            if let Some(s) = self.next() {
                return Some(s);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        None
    }

    pub fn latest(&self) -> Option<Arc<S>> {
        self.last.clone()
    }

    pub fn read_binding(&self) -> ReadStateBinding<S> {
        ReadStateBinding::new(Arc::clone(&self.state))
    }
}

pub struct RwChangeSubscriber<S>
where
    S: FieldDiff + Hash,
{
    rx: Receiver<()>,
    state: Arc<RwLock<S>>,
    last_hash: Option<u64>,
    last_fields: Vec<(S::Path, u64)>,
    _marker: PhantomData<S>,
}

impl<S> RwChangeSubscriber<S>
where
    S: FieldDiff + Hash,
{
    fn snapshot_fields(&self) -> (u64, Vec<(S::Path, u64)>) {
        self.state.with_read(|guard| {
            let root = hash_value(guard);
            let mut fields = Vec::new();
            guard.field_hashes(&mut fields);
            (root, fields)
        })
    }

    fn diff_fields(&self, fields: &[(S::Path, u64)]) -> Vec<S::Path> {
        fields
            .iter()
            .filter(|(path, hash)| {
                self.last_fields
                    .iter()
                    .find(|(prev_path, _)| prev_path == path)
                    .is_none_or(|(_, prev_hash)| prev_hash != hash)
            })
            .map(|(path, _)| *path)
            .collect()
    }

    pub fn read_binding(&self) -> ReadStateBinding<S> {
        ReadStateBinding::new(Arc::clone(&self.state))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<ChangeSet<S::Path>> {
        loop {
            match self.rx.try_recv() {
                Ok(()) => {
                    while self.rx.try_recv().is_ok() {}
                    let (root_hash, fields) = self.snapshot_fields();
                    if Some(root_hash) == self.last_hash {
                        continue;
                    }
                    let paths = self.diff_fields(&fields);
                    self.last_hash = Some(root_hash);
                    self.last_fields = fields;
                    if paths.is_empty() {
                        continue;
                    }
                    return Some(ChangeSet { paths });
                }
                Err(crossbeam_channel::TryRecvError::Empty) => return None,
                Err(crossbeam_channel::TryRecvError::Disconnected) => return None,
            }
        }
    }

    pub fn wait_next(&mut self, timeout: Duration) -> Option<ChangeSet<S::Path>> {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            if let Some(change) = self.next() {
                return Some(change);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        None
    }
}

/// Child [`RwStore`] routing actions through a parent action casepath.
pub struct ScopedRwStore<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK>
where
    SK: RefKpTrait<S, CS> + Clone,
{
    store: RwStore<S, M>,
    state_kp: SK,
    action_kp: AK,
    _marker: PhantomData<(M, CM, S, CS)>,
}

impl<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK> Clone
    for ScopedRwStore<S, M, CS, CM, AK, SK>
where
    S: Send,
    M: Send,
    AK: Clone,
    SK: RefKpTrait<S, CS> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            state_kp: self.state_kp.clone(),
            action_kp: self.action_kp.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK>
    ScopedRwStore<S, M, CS, CM, AK, SK>
where
    S: Send + Sync + Clone,
    M: Send,
    CS: Clone + PartialEq + Send + Sync,
    CM: Clone + Send,
    AK: Casepath<M, CM> + Clone + Send + Sync + 'static,
    SK: RefKpTrait<S, CS> + Clone,
{
    pub fn send(&self, action: CM) -> StoreTask {
        self.store.send(self.action_kp.wrap(action))
    }

    pub fn dispatch(&self, action: CM) {
        self.store.dispatch(self.action_kp.wrap(action));
    }

    pub fn child_state(&self) -> Option<CS> {
        self.state_kp.focus(&self.store.state()).cloned()
    }

    pub fn read_binding(&self) -> crate::runtime::binding::ReadProjectedBinding<S, CS, SK> {
        self.store
            .read_store()
            .read_binding()
            .project(self.state_kp.clone())
    }

    pub fn rw_binding(&self) -> crate::runtime::binding::RwProjectedBinding<S, CS, SK> {
        self.store.binding().project(self.state_kp.clone())
    }
}
