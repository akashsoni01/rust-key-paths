use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;

use crate::bus::BusSender;
use crate::effect::EffectId;
use crate::optics::Casepath;
use key_paths_core::RefKpTrait;
use super::interpreter::InterpreterState;

/// Calls [`StoreBackend::end_store_work`] on drop unless [`Self::disarm`]d.
///
/// Mirrors stack-unwind / `finally` semantics so [`StoreTask`] waiters are not left
/// hanging when a dispatch aborts (e.g. reducer panic).
pub(crate) struct StoreWorkUnwindGuard<'a, S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    backend: &'a StoreBackend<S, M>,
    active: bool,
}

impl<'a, S, M> StoreWorkUnwindGuard<'a, S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    pub(crate) fn new(backend: &'a StoreBackend<S, M>) -> Self {
        Self {
            backend,
            active: true,
        }
    }

    pub(crate) fn disarm(&mut self) {
        self.active = false;
    }
}

impl<S, M> Drop for StoreWorkUnwindGuard<'_, S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn drop(&mut self) {
        if self.active {
            self.backend.end_store_work();
        }
    }
}

/// Cloneable dispatch handle for a running [`Runtime`](crate::Runtime).
///
/// `update` runs on the runtime thread — actions are never applied synchronously inside
/// `send`, so reducers cannot re-enter themselves from the caller's stack (UDF parity).
pub struct Store<S, M> {
    pub(crate) backend: StoreBackend<S, M>,
}

impl<S, M> Clone for Store<S, M>
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

pub(crate) struct StoreBackend<S, M> {
    pub state: Arc<Mutex<S>>,
    pub sender: BusSender<M>,
    state_listeners: Arc<Mutex<Vec<Sender<()>>>>,
    effect_done: Arc<Mutex<VecDeque<Sender<()>>>>,
    in_flight: Arc<AtomicUsize>,
    pub interpreter: Arc<InterpreterState<M>>,
}

impl<S, M> Clone for StoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            sender: self.sender.clone(),
            state_listeners: self.state_listeners.clone(),
            effect_done: self.effect_done.clone(),
            in_flight: self.in_flight.clone(),
            interpreter: self.interpreter.clone(),
        }
    }
}

impl<S, M> StoreBackend<S, M>
where
    S: Send + 'static,
    M: Send + 'static,
{
    pub fn new(
        state: Arc<Mutex<S>>,
        sender: BusSender<M>,
        interpreter: Arc<InterpreterState<M>>,
    ) -> Self {
        Self {
            state,
            sender,
            state_listeners: Arc::new(Mutex::new(Vec::new())),
            effect_done: Arc::new(Mutex::new(VecDeque::new())),
            in_flight: Arc::new(AtomicUsize::new(0)),
            interpreter,
        }
    }

    pub fn begin_store_work(&self) {
        self.in_flight.fetch_add(1, Ordering::SeqCst);
    }

    pub fn end_store_work(&self) {
        let prev = self.in_flight.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 {
            self.notify_state();
            self.signal_effect_done();
        }
    }

    pub fn store(&self) -> Store<S, M> {
        Store {
            backend: self.clone(),
        }
    }

    pub fn notify_state(&self) {
        self.state_listeners
            .lock()
            .retain(|tx| tx.send(()).is_ok());
    }

    pub fn pop_effect_done(&self) -> Option<Sender<()>> {
        self.effect_done.lock().pop_front()
    }

    pub fn signal_effect_done(&self) {
        if let Some(tx) = self.pop_effect_done() {
            let _ = tx.send(());
        }
    }
}

impl<S, M> Store<S, M>
where
    S: Send + Sync + Clone + 'static,
    M: Send + 'static,
{
    /// Dispatch an action and return a handle that completes when its effects finish.
    pub fn send(&self, action: M) -> StoreTask {
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.backend.effect_done.lock().push_back(tx);
        self.backend.begin_store_work();
        let _ = self.backend.sender.send_blocking(action);
        StoreTask { rx }
    }

    /// Fire-and-forget dispatch.
    pub fn dispatch(&self, action: M) {
        let _ = self.send(action);
    }

    pub fn state(&self) -> S {
        self.backend.state.lock().clone()
    }

    pub fn cancel(&self, id: EffectId) {
        if let Some(handle) = self.backend.interpreter.cancel_tokens.lock().remove(&id) {
            handle.abort();
        }
    }

    /// Subscribe to deduplicated `Arc` state snapshots (skips consecutive equal states).
    pub fn subscribe_state(&self) -> StateSubscriber<S>
    where
        S: Clone + PartialEq,
    {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.backend.state_listeners.lock().push(tx);
        StateSubscriber {
            rx,
            state: self.backend.state.clone(),
            last: Some(Arc::new(self.backend.state.lock().clone())),
        }
    }

    /// Focus a child store via state and action casepaths (see [`crate::optics`]).
    pub fn scope<CS, CM, AK, SK>(
        &self,
        state_kp: SK,
        action_kp: AK,
    ) -> ScopedStore<S, M, CS, CM, AK, SK>
    where
        CS: Clone + PartialEq + Send + Sync + 'static,
        CM: Clone + Send + 'static,
        S: 'static,
        M: 'static,
        AK: Casepath<M, CM> + Clone + Send + Sync + 'static,
        SK: RefKpTrait<S, CS> + Clone,
    {
        ScopedStore {
            store: self.clone(),
            state_kp,
            action_kp,
            _marker: PhantomData,
        }
    }
}

/// Awaitable handle for in-flight effects from a single [`Store::send`].
pub struct StoreTask {
    rx: Receiver<()>,
}

impl StoreTask {
    pub fn finish(self) -> Result<(), StoreTaskError> {
        self.finish_with_timeout(Duration::from_secs(5))
    }

    pub fn finish_with_timeout(self, timeout: Duration) -> Result<(), StoreTaskError> {
        self.rx
            .recv_timeout(timeout)
            .map_err(|_| StoreTaskError::Timeout)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreTaskError {
    Timeout,
}

impl std::fmt::Display for StoreTaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "timed out waiting for store effects to finish"),
        }
    }
}

impl std::error::Error for StoreTaskError {}

/// Iterator-like subscription to state snapshots.
pub struct StateSubscriber<S> {
    rx: Receiver<()>,
    state: Arc<Mutex<S>>,
    last: Option<Arc<S>>,
}

impl<S: PartialEq + Clone> StateSubscriber<S> {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Arc<S>> {
        loop {
            match self.rx.try_recv() {
                Ok(()) => {
                    while self.rx.try_recv().is_ok() {}
                    let snapshot = Arc::new(self.state.lock().clone());
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
}

/// Child store routing actions through a parent action casepath.
pub struct ScopedStore<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK>
where
    SK: RefKpTrait<S, CS> + Clone,
{
    store: Store<S, M>,
    state_kp: SK,
    action_kp: AK,
    _marker: PhantomData<(M, CM, S, CS)>,
}

impl<S: 'static, M: 'static, CS: 'static, CM: 'static, AK: 'static, SK> Clone
    for ScopedStore<S, M, CS, CM, AK, SK>
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
    ScopedStore<S, M, CS, CM, AK, SK>
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

    pub fn subscribe_state(&self) -> ScopedStateSubscriber<S, CS, SK>
    where
        S: Clone + PartialEq,
        SK: Clone,
    {
        ScopedStateSubscriber {
            inner: self.store.subscribe_state(),
            state_kp: self.state_kp.clone(),
            _marker: PhantomData,
        }
    }
}

pub struct ScopedStateSubscriber<S: 'static, CS: 'static, SK>
where
    SK: RefKpTrait<S, CS> + Clone,
{
    inner: StateSubscriber<S>,
    state_kp: SK,
    _marker: PhantomData<(S, CS)>,
}

impl<S, CS, SK> ScopedStateSubscriber<S, CS, SK>
where
    S: PartialEq + Clone,
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
}
