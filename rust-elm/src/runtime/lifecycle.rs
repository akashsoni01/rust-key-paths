//! Shared runtime teardown — reducer thread join + subscription abort.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use super::subscription::SubscriptionHandles;

/// Signal shutdown, abort subscriptions, and join the reducer thread.
///
/// Idempotent: safe to call from [`Drop`] after an explicit [`Runtime::shutdown`].
pub(crate) fn join_reducer_thread(
    shutdown: &AtomicBool,
    subscriptions: &SubscriptionHandles,
    thread: &mut Option<JoinHandle<()>>,
) {
    shutdown.store(true, Ordering::Relaxed);
    subscriptions.abort_all();
    if let Some(handle) = thread.take() {
        let _ = handle.join();
    }
}
