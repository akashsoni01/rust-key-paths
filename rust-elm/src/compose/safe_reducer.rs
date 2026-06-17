//! Safe reducer updates — catch panics during `update` without crashing the runtime.
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::cmd::Cmd;

/// A reducer panic caught by [`safe_reduce_update`] or [`CatchReducer`](crate::reducer::CatchReducer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeReduceError {
    message: Option<String>,
}

impl SafeReduceError {
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn from_payload(payload: Box<dyn Any + Send>) -> Self {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned());
        Self { message }
    }
}

/// Run `reduce` inside [`catch_unwind`]. Default for [`CatchReducer`] and [`Runtime`](crate::Runtime) (with `runtime` feature):
/// panics are caught but `state` is not reverted. Use [`safe_reduce_rollback`] for rollback.
pub fn safe_reduce_update<S, M, F>(state: &mut S, reduce: F, action: M) -> Result<Cmd<M>, SafeReduceError>
where
    F: FnOnce(&mut S, M) -> Cmd<M>,
{
    match catch_unwind(AssertUnwindSafe(|| reduce(state, action))) {
        Ok(cmd) => Ok(cmd),
        Err(payload) => Err(SafeReduceError::from_payload(payload)),
    }
}

/// Run `reduce` inside [`catch_unwind`], rolling back via `checkpoint` on panic.
pub fn safe_reduce_rollback<S, M, F>(
    state: &mut S,
    checkpoint: &mut S,
    reduce: F,
    action: M,
) -> Result<Cmd<M>, SafeReduceError>
where
    S: Clone,
    F: FnOnce(&mut S, M) -> Cmd<M>,
{
    match catch_unwind(AssertUnwindSafe(|| reduce(state, action))) {
        Ok(cmd) => {
            checkpoint.clone_from(state);
            Ok(cmd)
        }
        Err(payload) => {
            std::mem::swap(state, checkpoint);
            Err(SafeReduceError::from_payload(payload))
        }
    }
}
