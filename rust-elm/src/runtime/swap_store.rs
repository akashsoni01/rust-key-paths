//! [`ArcSwap`](arc_swap::ArcSwap)-backed store — lock-free snapshot reads, copy-on-write updates.
//!
//! Hold one [`SwapStore`] from [`SwapRuntime::swap_store`](crate::SwapRuntime::swap_store).
//! Readers call [`SwapStore::snapshot_store`] for wait-free `load()`; the reducer thread
//! clones the current snapshot, reduces, and atomically publishes a new `Arc`.

use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use crossbeam_channel::Receiver;
use key_paths_core::{FieldDiff, hash_value};

use crate::effect::EffectId;
use crate::optics::Casepath;
use crate::runtime::binding::SnapshotStateBinding;
use crate::runtime::state_access::SnapshotRead;
use crate::runtime::store::{ChangeSet, StoreHub, StoreTask, StoreWork};
use key_paths_core::RefKpTrait;

pub(crate) struct SwapStoreBackend<S, M> {
    pub state: Arc<ArcSwap<S>>,
    pub hub: Arc<StoreHub<S, M>>,
}

impl<S, M> Clone for SwapStoreBackend<S, M>
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

impl<S, M> StoreWork<S, M> for SwapStoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn hub(&self) -> &Arc<StoreHub<S, M>> {
        &self.hub
    }
}

impl<S, M> SwapStoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    pub fn new(state: Arc<ArcSwap<S>>, hub: Arc<StoreHub<S, M>>) -> Self {
        Self { state, hub }
    }

    pub fn swap_store(&self) -> SwapStore<S, M> {
        SwapStore {
            backend: self.clone(),
        }
    }

    pub fn snapshot_store(&self) -> SnapshotStore<S, M> {
        SnapshotStore {
            backend: self.clone(),
        }
    }
}

/// Writable store — reducer publishes new `Arc` snapshots; readers never block.
pub struct SwapStore<S, M> {
    pub(crate) backend: SwapStoreBackend<S, M>,
}

impl<S, M> Clone for SwapStore<S, M>
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

impl<S, M> SwapStore<S, M>
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
        self.backend.state.with_snapshot(|s| s.clone())
    }

    pub fn snapshot_binding(&self) -> SnapshotStateBinding<S> {
        SnapshotStateBinding::new(Arc::clone(&self.backend.state))
    }

    /// Lock-free read handle — clone per reader thread.
    pub fn snapshot_store(&self) -> SnapshotStore<S, M> {
        self.backend.snapshot_store()
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

    pub fn subscribe_state(&self) -> SwapStateSubscriber<S>
    where
        S: Clone + PartialEq,
    {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.backend.hub.state_listeners.lock().push(tx);
        SwapStateSubscriber {
            rx,
            state: self.backend.state.clone(),
            last: Some(self.backend.state.load_full()),
        }
    }

    pub fn subscribe_changes(&self) -> SwapChangeSubscriber<S>
    where
        S: FieldDiff + Hash,
    {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.backend.hub.state_listeners.lock().push(tx);
        let (last_hash, last_fields) = self.backend.state.with_snapshot(|guard| {
            let root = hash_value(guard);
            let mut fields = Vec::new();
            guard.field_hashes(&mut fields);
            (Some(root), fields)
        });
        SwapChangeSubscriber {
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
    ) -> ScopedSwapStore<S, M, CS, CM, AK, SK>
    where
        CS: Clone + PartialEq + Send + Sync + 'static,
        CM: Clone + Send + 'static,
        S: 'static,
        M: 'static,
        AK: Casepath<M, CM> + Clone + Send + Sync + 'static,
        SK: RefKpTrait<S, CS> + Clone,
    {
        ScopedSwapStore {
            store: self.clone(),
            state_kp,
            action_kp,
            _marker: PhantomData,
        }
    }
}

/// Read-only lock-free snapshot handle.
pub struct SnapshotStore<S, M> {
    pub(crate) backend: SwapStoreBackend<S, M>,
}

impl<S, M> Clone for SnapshotStore<S, M>
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

impl<S, M> SnapshotStore<S, M>
where
    S: Send + Sync + Clone + 'static,
    M: Send + 'static,
{
    pub fn with_snapshot<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        self.backend.state.with_snapshot(f)
    }

    pub fn load(&self) -> Arc<S> {
        self.backend.state.load_full()
    }

    pub fn state(&self) -> S {
        self.with_snapshot(|s| s.clone())
    }

    pub fn snapshot_binding(&self) -> SnapshotStateBinding<S> {
        SnapshotStateBinding::new(Arc::clone(&self.backend.state))
    }

    pub fn subscribe_state(&self) -> SwapStateSubscriber<S>
    where
        S: Clone + PartialEq,
    {
        self.backend.swap_store().subscribe_state()
    }

    pub fn subscribe_changes(&self) -> SwapChangeSubscriber<S>
    where
        S: FieldDiff + Hash,
    {
        self.backend.swap_store().subscribe_changes()
    }
}

pub struct SwapStateSubscriber<S> {
    rx: Receiver<()>,
    state: Arc<ArcSwap<S>>,
    last: Option<Arc<S>>,
}

impl<S: PartialEq + Clone> SwapStateSubscriber<S> {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Arc<S>> {
        loop {
            match self.rx.try_recv() {
                Ok(()) => {
                    while self.rx.try_recv().is_ok() {}
                    let snapshot = self.state.load_full();
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

    pub fn snapshot_binding(&self) -> SnapshotStateBinding<S> {
        SnapshotStateBinding::new(Arc::clone(&self.state))
    }
}

pub struct SwapChangeSubscriber<S>
where
    S: FieldDiff + Hash,
{
    rx: Receiver<()>,
    state: Arc<ArcSwap<S>>,
    last_hash: Option<u64>,
    last_fields: Vec<(S::Path, u64)>,
    _marker: PhantomData<S>,
}

impl<S> SwapChangeSubscriber<S>
where
    S: FieldDiff + Hash,
{
    fn snapshot_fields(&self) -> (u64, Vec<(S::Path, u64)>) {
        self.state.with_snapshot(|guard| {
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

    pub fn snapshot_binding(&self) -> SnapshotStateBinding<S> {
        SnapshotStateBinding::new(Arc::clone(&self.state))
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

/// Child [`SwapStore`] routing actions through a parent action casepath.
pub struct ScopedSwapStore<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK>
where
    SK: RefKpTrait<S, CS> + Clone,
{
    store: SwapStore<S, M>,
    state_kp: SK,
    action_kp: AK,
    _marker: PhantomData<(M, CM, S, CS)>,
}

impl<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK> Clone
    for ScopedSwapStore<S, M, CS, CM, AK, SK>
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
    ScopedSwapStore<S, M, CS, CM, AK, SK>
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

    pub fn snapshot_binding(&self) -> crate::runtime::binding::SnapshotProjectedBinding<S, CS, SK> {
        self.store
            .snapshot_store()
            .snapshot_binding()
            .project(self.state_kp.clone())
    }
}
