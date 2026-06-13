use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;

use crate::bus::BusSender;
use crate::effect::EffectId;
use crate::optics::KpType;
use crate::runtime::InterpreterState;

/// Cloneable dispatch handle for a running [`Runtime`](crate::Runtime).
///
/// `update` runs on the runtime thread — actions are never applied synchronously inside
/// `send`, so reducers cannot re-enter themselves from the caller's stack (TCA parity).
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

    /// Focus a child store via a state keypath and parent action embed fn (see [`crate::optics`]).
    pub fn scope<CS, CM>(
        &self,
        state_kp: KpType<'static, S, CS>,
        embed: fn(CM) -> M,
    ) -> ScopedStore<S, M, CS, CM>
    where
        CS: Clone + PartialEq + Send + Sync + 'static,
        CM: Send + 'static,
        S: 'static,
    {
        ScopedStore {
            store: self.clone(),
            state_kp,
            embed,
        }
    }
}

/// Awaitable handle for in-flight effects from a single [`Store::send`].
pub struct StoreTask {
    rx: Receiver<()>,
}

impl StoreTask {
    pub fn finish(self) -> Result<(), StoreTaskError> {
        self.rx
            .recv_timeout(Duration::from_secs(5))
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

/// Child store routing actions through a parent action embed fn.
pub struct ScopedStore<S: 'static, M, CS: 'static, CM> {
    store: Store<S, M>,
    state_kp: KpType<'static, S, CS>,
    embed: fn(CM) -> M,
}

impl<S, M, CS, CM> Clone for ScopedStore<S, M, CS, CM>
where
    S: Send + 'static,
    M: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            state_kp: self.state_kp,
            embed: self.embed,
        }
    }
}

impl<S, M, CS, CM> ScopedStore<S, M, CS, CM>
where
    S: Send + Sync + Clone + 'static,
    M: Send + 'static,
    CS: Clone + PartialEq + Send + Sync + 'static,
    CM: Send + 'static,
{
    pub fn send(&self, action: CM) -> StoreTask {
        self.store.send((self.embed)(action))
    }

    pub fn dispatch(&self, action: CM) {
        self.store.dispatch((self.embed)(action));
    }

    pub fn child_state(&self) -> Option<CS> {
        self.state_kp.get_ref(&self.store.state()).cloned()
    }

    pub fn subscribe_state(&self) -> ScopedStateSubscriber<S, CS>
    where
        S: Clone + PartialEq,
    {
        ScopedStateSubscriber {
            inner: self.store.subscribe_state(),
            state_kp: self.state_kp,
        }
    }
}

pub struct ScopedStateSubscriber<S: 'static, CS: 'static> {
    inner: StateSubscriber<S>,
    state_kp: KpType<'static, S, CS>,
}

impl<S, CS> ScopedStateSubscriber<S, CS>
where
    S: PartialEq + Clone,
    CS: Clone + PartialEq,
{
    pub fn next(&mut self) -> Option<CS> {
        loop {
            let parent = self.inner.next()?;
            let Some(child) = self.state_kp.get_ref(parent.as_ref()).cloned() else {
                continue;
            };
            return Some(child);
        }
    }
}
