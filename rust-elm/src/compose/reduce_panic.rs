//! Panic guards for reducers — runtime-agnostic (no Tokio).
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::cmd::Cmd;

/// A reducer panic caught by [`catch_reduce`] or [`CatchReducer`](crate::reducer::CatchReducer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducePanic {
    message: Option<String>,
}

impl ReducePanic {
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
/// panics are caught but `state` is not reverted. Use [`catch_reduce`] for rollback.
pub fn catch_reduce_panic<S, M, F>(state: &mut S, reduce: F, action: M) -> Result<Cmd<M>, ReducePanic>
where
    F: FnOnce(&mut S, M) -> Cmd<M>,
{
    match catch_unwind(AssertUnwindSafe(|| reduce(state, action))) {
        Ok(cmd) => Ok(cmd),
        Err(payload) => Err(ReducePanic::from_payload(payload)),
    }
}

/// Run `reduce` inside [`catch_unwind`], rolling back via `checkpoint` on panic.
pub fn catch_reduce<S, M, F>(
    state: &mut S,
    checkpoint: &mut S,
    reduce: F,
    action: M,
) -> Result<Cmd<M>, ReducePanic>
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
            Err(ReducePanic::from_payload(payload))
        }
    }
}
