//! Subscription interpreter — starts/stops long-lived Tokio sources declared by [`Sub`].
mod websocket;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::runtime::Handle;
use tokio::task::AbortHandle;
use tokio::time::{interval, MissedTickBehavior};

use crate::bus::BusSender;
use crate::store::StoreBackend;
use crate::sub::Sub;

pub(crate) struct SubscriptionHandles {
    pub handles: parking_lot::Mutex<HashMap<u64, AbortHandle>>,
}

impl SubscriptionHandles {
    pub fn new() -> Self {
        Self {
            handles: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    pub fn abort_all(&self) {
        self.handles
            .lock()
            .drain()
            .for_each(|(_, handle)| handle.abort());
    }
}

/// Reconcile running subscription tasks with the desired [`Sub`] tree.
pub(crate) fn sync_subscriptions<S, M>(
    sub: &Sub<M>,
    registry: &SubscriptionHandles,
    handle: Handle,
    tx: BusSender<M>,
    backend: StoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
) where
    S: Send + 'static,
    M: Send + 'static,
{
    let mut desired = HashSet::new();
    collect_sub_ids(sub, &mut desired);

    let mut active = registry.handles.lock();
    active.retain(|id, abort| {
        if desired.contains(id) {
            true
        } else {
            abort.abort();
            false
        }
    });

    spawn_subscriptions(
        sub,
        &mut active,
        handle,
        tx,
        backend,
        shutdown,
    );
}

fn spawn_subscriptions<S, M>(
    sub: &Sub<M>,
    active: &mut HashMap<u64, AbortHandle>,
    handle: Handle,
    tx: BusSender<M>,
    backend: StoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
) where
    S: Send + 'static,
    M: Send + 'static,
{
    match sub {
        Sub::None => {}
        Sub::Tick { id, every, produce } => {
            spawn_if_new(*id, active, || {
                spawn_tick(*every, *produce, handle, tx, backend, shutdown)
            });
        }
        Sub::Stream { id, name, every, produce } => {
            spawn_if_new(*id, active, || {
                spawn_stream(name, *every, *produce, handle, tx, backend, shutdown)
            });
        }
        Sub::WebSocket { id, url, every, produce } => {
            spawn_if_new(*id, active, || {
                websocket::spawn_websocket(url, *every, *produce, handle, tx, backend, shutdown)
            });
        }
        Sub::MapMsg { inner, map } => spawn_unit_subscriptions(
            inner.as_ref(),
            *map,
            active,
            handle,
            tx,
            backend,
            shutdown,
        ),
        Sub::Batch(items) => {
            for item in items {
                spawn_subscriptions(
                    item,
                    active,
                    handle.clone(),
                    tx.clone(),
                    backend.clone(),
                    shutdown.clone(),
                );
            }
        }
    }
}

fn spawn_unit_subscriptions<S, M>(
    sub: &Sub<()>,
    map: fn(()) -> M,
    active: &mut HashMap<u64, AbortHandle>,
    handle: Handle,
    tx: BusSender<M>,
    backend: StoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
) where
    S: Send + 'static,
    M: Send + 'static,
{
    match sub {
        Sub::None => {}
        Sub::Tick { id, every, .. } => {
            spawn_if_new(*id, active, || {
                spawn_tick_mapped(*every, map, handle, tx, backend, shutdown)
            });
        }
        Sub::Stream { id, name, every, .. } => {
            spawn_if_new(*id, active, || {
                spawn_stream_mapped(name, *every, map, handle, tx, backend, shutdown)
            });
        }
        Sub::WebSocket { id, url, every, .. } => {
            spawn_if_new(*id, active, || {
                websocket::spawn_websocket_mapped(url, *every, map, handle, tx, backend, shutdown)
            });
        }
        Sub::MapMsg { .. } => {}
        Sub::Batch(items) => {
            for item in items {
                spawn_unit_subscriptions(
                    item,
                    map,
                    active,
                    handle.clone(),
                    tx.clone(),
                    backend.clone(),
                    shutdown.clone(),
                );
            }
        }
    }
}

fn spawn_if_new(id: u64, active: &mut HashMap<u64, AbortHandle>, spawn: impl FnOnce() -> AbortHandle) {
    if active.contains_key(&id) {
        return;
    }
    active.insert(id, spawn());
}

fn spawn_tick<S, M>(
    every: std::time::Duration,
    produce: fn() -> M,
    handle: Handle,
    tx: BusSender<M>,
    backend: StoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
) -> AbortHandle
where
    S: Send + 'static,
    M: Send + 'static,
{
    handle
        .spawn(async move {
            let mut ticker = interval(every);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            ticker.tick().await;
            loop {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                ticker.tick().await;
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                crate::runtime::dispatch::dispatch_from_subscription(&backend, &tx, produce());
            }
        })
        .abort_handle()
}

fn spawn_tick_mapped<S, M>(
    every: std::time::Duration,
    map: fn(()) -> M,
    handle: Handle,
    tx: BusSender<M>,
    backend: StoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
) -> AbortHandle
where
    S: Send + 'static,
    M: Send + 'static,
{
    handle
        .spawn(async move {
            let mut ticker = interval(every);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            ticker.tick().await;
            loop {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                ticker.tick().await;
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                crate::runtime::dispatch::dispatch_from_subscription(&backend, &tx, map(()));
            }
        })
        .abort_handle()
}

fn spawn_stream<S, M>(
    _name: &'static str,
    every: std::time::Duration,
    produce: fn() -> M,
    handle: Handle,
    tx: BusSender<M>,
    backend: StoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
) -> AbortHandle
where
    S: Send + 'static,
    M: Send + 'static,
{
    handle
        .spawn(async move {
            let mut ticker = interval(every);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                ticker.tick().await;
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                crate::runtime::dispatch::dispatch_from_subscription(&backend, &tx, produce());
            }
        })
        .abort_handle()
}

fn spawn_stream_mapped<S, M>(
    _name: &'static str,
    every: std::time::Duration,
    map: fn(()) -> M,
    handle: Handle,
    tx: BusSender<M>,
    backend: StoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
) -> AbortHandle
where
    S: Send + 'static,
    M: Send + 'static,
{
    handle
        .spawn(async move {
            let mut ticker = interval(every);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                ticker.tick().await;
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                crate::runtime::dispatch::dispatch_from_subscription(&backend, &tx, map(()));
            }
        })
        .abort_handle()
}

pub(crate) fn collect_sub_ids<M>(sub: &Sub<M>, out: &mut HashSet<u64>) {
    match sub {
        Sub::None => {}
        Sub::Tick { id, .. }
        | Sub::Stream { id, .. }
        | Sub::WebSocket { id, .. } => {
            out.insert(*id);
        }
        Sub::MapMsg { inner, .. } => collect_sub_ids(inner, out),
        Sub::Batch(items) => {
            for item in items {
                collect_sub_ids(item, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn unit() {}

    fn mapped(_: ()) -> i32 {
        7
    }

    #[test]
    fn collect_ids_from_batch() {
        fn zero() -> i32 {
            0
        }
        let sub = Sub::batch([
            Sub::tick(1, Duration::from_secs(1), zero),
            Sub::stream(2, "events", Duration::from_secs(1), zero),
        ]);
        let mut ids = HashSet::new();
        collect_sub_ids(&sub, &mut ids);
        assert_eq!(ids, HashSet::from([1, 2]));
    }

    #[test]
    fn collect_ids_from_map_msg() {
        let sub = Sub::MapMsg {
            inner: Box::new(Sub::tick(9, Duration::from_millis(1), unit)),
            map: mapped,
        };
        let mut ids = HashSet::new();
        collect_sub_ids(&sub, &mut ids);
        assert_eq!(ids, HashSet::from([9]));
    }
}
