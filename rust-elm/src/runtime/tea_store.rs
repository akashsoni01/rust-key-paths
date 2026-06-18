//! Channel-delivered model snapshots — true TEA design.
//!
//! State [`S`] lives **only** on the reducer thread. After each update the runtime
//! **pushes** `Arc<S>` to subscribers over crossbeam channels — readers never lock or
//! share mutable state.
//!
//! Hold one [`TeaStore`] from [`TeaRuntime::tea_store`](crate::TeaRuntime::tea_store).
//! Reader threads clone [`TeaViewStore`] and [`TeaStateSubscriber::wait_next`].

use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use key_paths_core::{FieldDiff, hash_value, RefKpTrait};

use crate::effect::EffectId;
use crate::optics::Casepath;
use crate::runtime::store::{ChangeSet, StoreHub, StoreTask, StoreWork};

/// Errors reading or subscribing to channel-delivered model snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeaStoreError {
    RuntimeShutdown,
    SubscribeTimeout,
    SnapshotTimeout,
    SnapshotUnavailable,
}

impl fmt::Display for TeaStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeShutdown => write!(f, "tea runtime control channel disconnected"),
            Self::SubscribeTimeout => write!(f, "timed out waiting for snapshot subscription"),
            Self::SnapshotTimeout => write!(f, "timed out waiting for model snapshot"),
            Self::SnapshotUnavailable => write!(f, "model snapshot unavailable"),
        }
    }
}

impl std::error::Error for TeaStoreError {}

const DEFAULT_SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(5);

/// Control messages handled on the reducer thread (subscription + snapshot RPC).
pub(crate) enum TeaControl<S> {
    Subscribe {
        snapshot_tx: Sender<Arc<S>>,
        ready_tx: Sender<()>,
    },
    RequestSnapshot {
        reply: Sender<Arc<S>>,
    },
}

pub(crate) struct TeaStoreBackend<S, M> {
    pub hub: Arc<StoreHub<S, M>>,
    pub control_tx: Sender<TeaControl<S>>,
}

impl<S, M> Clone for TeaStoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            hub: self.hub.clone(),
            control_tx: self.control_tx.clone(),
        }
    }
}

impl<S, M> StoreWork<S, M> for TeaStoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn hub(&self) -> &Arc<StoreHub<S, M>> {
        &self.hub
    }

    /// Effect completion only — snapshots are pushed from the reducer thread after `reduce`.
    fn end_store_work(&self) {
        let prev = self.hub.in_flight.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        if prev == 1 {
            self.hub.signal_effect_done();
        }
    }
}

impl<S, M> TeaStoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    pub fn new(hub: Arc<StoreHub<S, M>>, control_tx: Sender<TeaControl<S>>) -> Self {
        Self { hub, control_tx }
    }

    pub fn tea_store(&self) -> TeaStore<S, M> {
        TeaStore {
            backend: self.clone(),
        }
    }

    pub fn view_store(&self) -> TeaViewStore<S, M> {
        TeaViewStore {
            backend: self.clone(),
        }
    }

    fn request_snapshot(&self, timeout: Duration) -> Result<Arc<S>, TeaStoreError> {
        let (reply_tx, reply_rx) = crossbeam_channel::bounded(1);
        self.control_tx
            .send(TeaControl::RequestSnapshot { reply: reply_tx })
            .map_err(|_| TeaStoreError::RuntimeShutdown)?;
        reply_rx
            .recv_timeout(timeout)
            .map_err(|_| TeaStoreError::SnapshotTimeout)
    }
}

/// Writable store — dispatch actions; model snapshots arrive on subscription channels.
pub struct TeaStore<S, M> {
    pub(crate) backend: TeaStoreBackend<S, M>,
}

impl<S, M> Clone for TeaStore<S, M>
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

impl<S, M> TeaStore<S, M>
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

    /// Request a point-in-time model snapshot from the reducer thread (blocks until reply).
    pub fn try_state(&self) -> Result<S, TeaStoreError> {
        self.backend
            .request_snapshot(DEFAULT_SNAPSHOT_TIMEOUT)
            .map(|s| (*s).clone())
    }

    /// Same as [`Self::try_state`] — provided for call sites that expect infallible reads.
    pub fn state(&self) -> Result<S, TeaStoreError> {
        self.try_state()
    }

    /// Read-only view handle — clone onto reader threads.
    pub fn view_store(&self) -> TeaViewStore<S, M> {
        self.backend.view_store()
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

    pub fn try_subscribe_state(&self) -> Result<TeaStateSubscriber<S>, TeaStoreError>
    where
        S: PartialEq,
    {
        self.register_snapshot_subscriber()
    }

    /// Subscribe to model snapshots pushed after each reduce (no shared-state lock).
    pub fn subscribe_state(&self) -> Result<TeaStateSubscriber<S>, TeaStoreError>
    where
        S: PartialEq,
    {
        self.try_subscribe_state()
    }

    pub fn try_subscribe_changes(&self) -> Result<TeaChangeSubscriber<S>, TeaStoreError>
    where
        S: FieldDiff + Hash + PartialEq,
    {
        let mut snapshot_sub = self.register_snapshot_subscriber()?;
        let initial = snapshot_sub
            .latest()
            .or_else(|| snapshot_sub.wait_next(DEFAULT_SNAPSHOT_TIMEOUT));
        let Some(initial) = initial else {
            return Err(TeaStoreError::SnapshotUnavailable);
        };
        let (last_hash, last_fields) = snapshot_hashes(&initial);
        Ok(TeaChangeSubscriber {
            snapshot_sub,
            last_hash: Some(last_hash),
            last_fields,
            _marker: PhantomData,
        })
    }

    /// Field-level changes derived from pushed snapshots (no lock on live state).
    pub fn subscribe_changes(&self) -> Result<TeaChangeSubscriber<S>, TeaStoreError>
    where
        S: FieldDiff + Hash + PartialEq,
    {
        self.try_subscribe_changes()
    }

    pub fn scope<CS, CM, AK, SK>(
        &self,
        state_kp: SK,
        action_kp: AK,
    ) -> ScopedTeaStore<S, M, CS, CM, AK, SK>
    where
        CS: Clone + PartialEq + Send + Sync + 'static,
        CM: Clone + Send + 'static,
        S: 'static,
        M: 'static,
        AK: Casepath<M, CM> + Clone + Send + Sync + 'static,
        SK: RefKpTrait<S, CS> + Clone,
    {
        ScopedTeaStore {
            store: self.clone(),
            state_kp,
            action_kp,
            _marker: PhantomData,
        }
    }

    fn register_snapshot_subscriber(&self) -> Result<TeaStateSubscriber<S>, TeaStoreError> {
        let (snapshot_tx, snapshot_rx) = crossbeam_channel::unbounded();
        let (ready_tx, ready_rx) = crossbeam_channel::bounded(1);
        self.backend
            .control_tx
            .send(TeaControl::Subscribe {
                snapshot_tx,
                ready_tx,
            })
            .map_err(|_| TeaStoreError::RuntimeShutdown)?;
        ready_rx
            .recv_timeout(DEFAULT_SNAPSHOT_TIMEOUT)
            .map_err(|_| TeaStoreError::SubscribeTimeout)?;
        let initial = snapshot_rx
            .recv_timeout(DEFAULT_SNAPSHOT_TIMEOUT)
            .map_err(|_| TeaStoreError::SnapshotUnavailable)?;
        Ok(TeaStateSubscriber {
            rx: snapshot_rx,
            last: Some(initial),
        })
    }
}

/// Read-only view — snapshots only via channel subscription or RPC.
pub struct TeaViewStore<S, M> {
    pub(crate) backend: TeaStoreBackend<S, M>,
}

impl<S, M> Clone for TeaViewStore<S, M>
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

impl<S, M> TeaViewStore<S, M>
where
    S: Send + Sync + Clone + 'static,
    M: Send + 'static,
{
    pub fn try_with_snapshot<R>(&self, f: impl FnOnce(&S) -> R) -> Result<R, TeaStoreError> {
        let snap = self
            .backend
            .request_snapshot(DEFAULT_SNAPSHOT_TIMEOUT)?;
        Ok(f(snap.as_ref()))
    }

    pub fn with_snapshot<R>(&self, f: impl FnOnce(&S) -> R) -> Result<R, TeaStoreError> {
        self.try_with_snapshot(f)
    }

    pub fn try_load(&self) -> Result<Arc<S>, TeaStoreError> {
        self.backend.request_snapshot(DEFAULT_SNAPSHOT_TIMEOUT)
    }

    pub fn load(&self) -> Result<Arc<S>, TeaStoreError> {
        self.try_load()
    }

    pub fn try_state(&self) -> Result<S, TeaStoreError> {
        self.try_with_snapshot(|s| s.clone())
    }

    pub fn state(&self) -> Result<S, TeaStoreError> {
        self.try_state()
    }

    pub fn subscribe_state(&self) -> Result<TeaStateSubscriber<S>, TeaStoreError>
    where
        S: PartialEq,
    {
        self.backend.tea_store().subscribe_state()
    }

    pub fn subscribe_changes(&self) -> Result<TeaChangeSubscriber<S>, TeaStoreError>
    where
        S: FieldDiff + Hash + PartialEq,
    {
        self.backend.tea_store().subscribe_changes()
    }
}

/// Model snapshots pushed from the reducer thread (`Arc<S>` per update).
pub struct TeaStateSubscriber<S> {
    rx: Receiver<Arc<S>>,
    last: Option<Arc<S>>,
}

impl<S: PartialEq> TeaStateSubscriber<S> {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Arc<S>> {
        loop {
            match self.rx.try_recv() {
                Ok(first) => {
                    let mut snapshot = first;
                    while let Ok(newer) = self.rx.try_recv() {
                        snapshot = newer;
                    }
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

    pub fn with_latest<R>(&self, f: impl FnOnce(&S) -> R) -> Option<R> {
        self.last.as_ref().map(|s| f(s.as_ref()))
    }
}

pub struct TeaChangeSubscriber<S>
where
    S: FieldDiff + Hash,
{
    snapshot_sub: TeaStateSubscriber<S>,
    last_hash: Option<u64>,
    last_fields: Vec<(S::Path, u64)>,
    _marker: PhantomData<S>,
}

fn snapshot_hashes<S: FieldDiff + Hash>(snap: &Arc<S>) -> (u64, Vec<(S::Path, u64)>) {
    let root = hash_value(snap.as_ref());
    let mut fields = Vec::new();
    snap.field_hashes(&mut fields);
    (root, fields)
}

impl<S> TeaChangeSubscriber<S>
where
    S: FieldDiff + Hash + PartialEq,
{
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

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<ChangeSet<S::Path>> {
        loop {
            let snap = self.snapshot_sub.next()?;
            let (root_hash, fields) = snapshot_hashes(&snap);
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

    pub fn latest_snapshot(&self) -> Option<Arc<S>> {
        self.snapshot_sub.latest()
    }
}

/// Child [`TeaStore`] routing actions through a parent action casepath.
pub struct ScopedTeaStore<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK>
where
    SK: RefKpTrait<S, CS> + Clone,
{
    store: TeaStore<S, M>,
    state_kp: SK,
    action_kp: AK,
    _marker: PhantomData<(M, CM, S, CS)>,
}

impl<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK> Clone
    for ScopedTeaStore<S, M, CS, CM, AK, SK>
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
    ScopedTeaStore<S, M, CS, CM, AK, SK>
where
    S: Send + Sync + Clone + PartialEq,
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

    pub fn child_state(&self) -> Result<Option<CS>, TeaStoreError> {
        let parent = self.store.state()?;
        Ok(self.state_kp.focus(&parent).cloned())
    }

    pub fn subscribe_state(&self) -> Result<ScopedTeaStateSubscriber<S, CS, SK>, TeaStoreError>
    where
        S: PartialEq,
        SK: Clone,
    {
        Ok(ScopedTeaStateSubscriber {
            inner: self.store.subscribe_state()?,
            state_kp: self.state_kp.clone(),
            _marker: PhantomData,
        })
    }
}

pub struct ScopedTeaStateSubscriber<S: 'static, CS: 'static, SK>
where
    SK: RefKpTrait<S, CS> + Clone,
{
    inner: TeaStateSubscriber<S>,
    state_kp: SK,
    _marker: PhantomData<(S, CS)>,
}

impl<S, CS, SK> ScopedTeaStateSubscriber<S, CS, SK>
where
    S: PartialEq,
    CS: Clone + PartialEq,
    SK: RefKpTrait<S, CS> + Clone,
{
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<CS> {
        loop {
            let parent = self.inner.next()?;
            let Some(child) = self.state_kp.focus(parent.as_ref()).cloned() else {
                continue;
            };
            return Some(child);
        }
    }

    pub fn wait_next(&mut self, timeout: Duration) -> Option<CS> {
        let deadline = std::time::Instant::now() + timeout;
        while std::time::Instant::now() < deadline {
            if let Some(s) = self.next() {
                return Some(s);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tea_change_subscriber_diffs_snapshots() {
        #[derive(Clone, Hash, PartialEq, Debug)]
        struct App {
            n: i32,
        }

        impl FieldDiff for App {
            type Path = &'static str;
            fn field_hashes(&self, out: &mut Vec<(Self::Path, u64)>) {
                out.push(("n", hash_value(&self.n)));
            }
        }

        let (tx, rx) = crossbeam_channel::unbounded();
        let mut sub = TeaChangeSubscriber {
            snapshot_sub: TeaStateSubscriber {
                rx,
                last: Some(Arc::new(App { n: 0 })),
            },
            last_hash: Some(hash_value(&App { n: 0 })),
            last_fields: vec![("n", hash_value(&0))],
            _marker: PhantomData,
        };
        tx.send(Arc::new(App { n: 1 })).ok();
        let change = sub.next();
        assert!(change.is_some());
        if let Some(change) = change {
            assert_eq!(change.paths, vec!["n"]);
        }
    }
}
