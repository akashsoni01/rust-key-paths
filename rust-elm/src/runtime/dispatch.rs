use crate::bus::BusSender;
use super::store::StoreWork;

pub(crate) fn dispatch_from_effect<S, M, B>(backend: &B, tx: &BusSender<M>, msg: M)
where
    S: Send + 'static,
    M: Send + 'static,
    B: StoreWork<S, M>,
{
    backend.begin_store_work();
    let _ = tx.send_blocking(msg);
}

pub(crate) fn dispatch_from_subscription<S, M, B>(backend: &B, tx: &BusSender<M>, msg: M)
where
    S: Send + 'static,
    M: Send + 'static,
    B: StoreWork<S, M>,
{
    dispatch_from_effect(backend, tx, msg);
}
