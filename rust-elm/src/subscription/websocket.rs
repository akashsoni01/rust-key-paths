//! WebSocket subscription interpreter — real socket or simulated fallback.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::task::AbortHandle;

use crate::bus::BusSender;
use crate::store::StoreBackend;

#[cfg(feature = "websocket")]
pub(crate) fn spawn_websocket<S, M>(
    url: &'static str,
    reconnect_every: Duration,
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
            websocket_loop_impl(url, reconnect_every, produce, &backend, &tx, &shutdown).await;
        })
        .abort_handle()
}

#[cfg(feature = "websocket")]
pub(crate) fn spawn_websocket_mapped<S, M>(
    url: &'static str,
    reconnect_every: Duration,
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
            websocket_loop_impl(
                url,
                reconnect_every,
                move || map(()),
                &backend,
                &tx,
                &shutdown,
            )
            .await;
        })
        .abort_handle()
}

#[cfg(feature = "websocket")]
async fn websocket_simulated_loop<S, M, F>(
    every: Duration,
    produce: F,
    backend: &StoreBackend<S, M>,
    tx: &BusSender<M>,
    shutdown: &Arc<AtomicBool>,
) where
    S: Send + 'static,
    M: Send + 'static,
    F: Fn() -> M,
{
    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        tokio::time::sleep(every / 2).await;
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        crate::runtime::dispatch_from_subscription(backend, tx, produce());
        tokio::time::sleep(every).await;
    }
}

#[cfg(feature = "websocket")]
async fn websocket_loop_impl<S, M, F>(
    url: &str,
    reconnect_every: Duration,
    produce: F,
    backend: &StoreBackend<S, M>,
    tx: &BusSender<M>,
    shutdown: &Arc<AtomicBool>,
) where
    S: Send + 'static,
    M: Send + 'static,
    F: Fn() -> M,
{
    if url.starts_with("mock://") {
        websocket_simulated_loop(reconnect_every, produce, backend, tx, shutdown).await;
        return;
    }

    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    let mut backoff = reconnect_every;
    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        match connect_async(url).await {
            Ok((mut ws, _)) => {
                backoff = reconnect_every;
                let _ = ws.send(Message::Ping(Vec::new().into())).await;
                loop {
                    if shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    match ws.next().await {
                        Some(Ok(Message::Text(_)))
                        | Some(Ok(Message::Binary(_)))
                        | Some(Ok(Message::Ping(_)))
                        | Some(Ok(Message::Pong(_))) => {
                            crate::runtime::dispatch_from_subscription(backend, tx, produce());
                        }
                        Some(Ok(Message::Frame(_))) | Some(Ok(Message::Close(_))) => {}
                        Some(Err(_)) | None => break,
                    }
                }
            }
            Err(_) => {
                tokio::time::sleep(backoff).await;
                backoff = backoff.saturating_mul(2).min(Duration::from_secs(30));
            }
        }
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        tokio::time::sleep(reconnect_every).await;
    }
}

#[cfg(not(feature = "websocket"))]
pub(crate) fn spawn_websocket<S, M>(
    _url: &'static str,
    every: Duration,
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
    spawn_websocket_simulated(every, produce, handle, tx, backend, shutdown)
}

#[cfg(not(feature = "websocket"))]
pub(crate) fn spawn_websocket_mapped<S, M>(
    _url: &'static str,
    every: Duration,
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
    spawn_websocket_simulated_mapped(every, map, handle, tx, backend, shutdown)
}

#[cfg(not(feature = "websocket"))]
async fn websocket_simulated_loop<S, M, F>(
    every: Duration,
    produce: F,
    backend: &StoreBackend<S, M>,
    tx: &BusSender<M>,
    shutdown: &Arc<AtomicBool>,
) where
    S: Send + 'static,
    M: Send + 'static,
    F: Fn() -> M,
{
    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        tokio::time::sleep(every / 2).await;
        if shutdown.load(Ordering::Relaxed) {
            break;
        }
        crate::runtime::dispatch_from_subscription(backend, tx, produce());
        tokio::time::sleep(every).await;
    }
}

#[cfg(not(feature = "websocket"))]
fn spawn_websocket_simulated<S, M>(
    every: Duration,
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
            websocket_simulated_loop(every, produce, &backend, &tx, &shutdown).await;
        })
        .abort_handle()
}

#[cfg(not(feature = "websocket"))]
fn spawn_websocket_simulated_mapped<S, M>(
    every: Duration,
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
            websocket_simulated_loop(every, move || map(()), &backend, &tx, &shutdown).await;
        })
        .abort_handle()
}
