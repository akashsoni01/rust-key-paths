//! Shared state lock helpers for [`Mutex`](parking_lot::Mutex) and [`RwLock`](parking_lot::RwLock) stores.

use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

pub(crate) trait StateRead<S> {
    fn with_read<R>(&self, f: impl FnOnce(&S) -> R) -> R;
}

pub(crate) trait StateWrite<S>: StateRead<S> {
    fn with_write<R>(&self, f: impl FnOnce(&mut S) -> R) -> R;
}

impl<S> StateRead<S> for Arc<Mutex<S>> {
    fn with_read<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        f(&*self.lock())
    }
}

impl<S> StateWrite<S> for Arc<Mutex<S>> {
    fn with_write<R>(&self, f: impl FnOnce(&mut S) -> R) -> R {
        f(&mut *self.lock())
    }
}

impl<S> StateRead<S> for Arc<RwLock<S>> {
    fn with_read<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        f(&*self.read())
    }
}

impl<S> StateWrite<S> for Arc<RwLock<S>> {
    fn with_write<R>(&self, f: impl FnOnce(&mut S) -> R) -> R {
        f(&mut *self.write())
    }
}
