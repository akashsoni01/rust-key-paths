use crate::bus::BusSender;
use super::store::StoreBackend;

pub(crate) fn dispatch_from_effect<S, M>(
    backend: &StoreBackend<S, M>,
    tx: &BusSender<M>,
    msg: M,
) where
    S: Send + 'static,
    M: Send + 'static,
{
    backend.begin_store_work();
    let _ = tx.send_blocking(msg);
}

pub(crate) fn dispatch_from_subscription<S, M>(
    backend: &StoreBackend<S, M>,
    tx: &BusSender<M>,
    msg: M,
) where
    S: Send + 'static,
    M: Send + 'static,
{
    dispatch_from_effect(backend, tx, msg);
}
