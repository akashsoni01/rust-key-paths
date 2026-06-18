use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Lock-free bounded MPMC message bus with explicit backpressure.
#[derive(Debug)]
pub struct Bus<M> {
    sender: Sender<M>,
    receiver: Receiver<M>,
    capacity: usize,
    dropped: Arc<AtomicU64>,
}

impl<M: Send + 'static> Bus<M> {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity.max(1));
        Self {
            sender,
            receiver,
            capacity: capacity.max(1),
            dropped: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn sender(&self) -> BusSender<M> {
        BusSender {
            inner: self.sender.clone(),
            dropped: Arc::clone(&self.dropped),
        }
    }

    pub fn receiver(&self) -> &Receiver<M> {
        &self.receiver
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct BusSender<M> {
    inner: Sender<M>,
    dropped: Arc<AtomicU64>,
}

impl<M> Clone for BusSender<M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            dropped: Arc::clone(&self.dropped),
        }
    }
}

impl<M> BusSender<M> {
    pub fn send(&self, msg: M) -> Result<(), M> {
        match self.inner.try_send(msg) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(msg)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                Err(msg)
            }
            Err(TrySendError::Disconnected(msg)) => Err(msg),
        }
    }

    pub fn send_blocking(&self, msg: M) -> Result<(), M> {
        self.inner.send(msg).map_err(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backpressure_counts_drops() {
        let bus = Bus::<u32>::new(1);
        let tx = bus.sender();
        assert!(tx.send(1).is_ok());
        let dropped = tx.send(2).unwrap_err();
        assert_eq!(dropped, 2);
        assert_eq!(bus.dropped_count(), 1);
    }
}
