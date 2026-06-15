use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use std::sync::atomic::{AtomicU64, Ordering};

/// Lock-free bounded MPMC message bus with explicit backpressure.
#[derive(Debug)]
pub struct Bus<M> {
    sender: Sender<M>,
    receiver: Receiver<M>,
    capacity: usize,
    dropped: AtomicU64,
}

impl<M: Send + 'static> Bus<M> {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity.max(1));
        Self {
            sender,
            receiver,
            capacity: capacity.max(1),
            dropped: AtomicU64::new(0),
        }
    }

    pub fn sender(&self) -> BusSender<M> {
        BusSender {
            inner: self.sender.clone(),
            dropped: &self.dropped,
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
    dropped: *const AtomicU64,
}

impl<M> Clone for BusSender<M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            dropped: self.dropped,
        }
    }
}

impl<M> BusSender<M> {
    pub fn send(&self, msg: M) -> Result<(), M> {
        match self.inner.try_send(msg) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(msg)) => {
                // Backpressure: drop and count when full (caller may retry).
                unsafe {
                    (*self.dropped).fetch_add(1, Ordering::Relaxed);
                }
                Err(msg)
            }
            Err(TrySendError::Disconnected(msg)) => Err(msg),
        }
    }

    pub fn send_blocking(&self, msg: M) -> Result<(), M> {
        self.inner.send(msg).map_err(|e| e.into_inner())
    }
}

unsafe impl<M> Send for BusSender<M> {}
unsafe impl<M> Sync for BusSender<M> {}

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
