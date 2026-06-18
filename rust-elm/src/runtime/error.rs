use std::fmt;
use std::io;

/// Errors starting or configuring a live runtime.
#[derive(Debug)]
pub enum RuntimeError {
    TokioBuild(io::Error),
    ThreadSpawn(io::Error),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TokioBuild(err) => write!(f, "failed to create Tokio runtime: {err}"),
            Self::ThreadSpawn(err) => write!(f, "failed to spawn reducer thread: {err}"),
        }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TokioBuild(err) | Self::ThreadSpawn(err) => Some(err),
        }
    }
}

pub(crate) fn build_tokio(
    worker_threads: usize,
    thread_name: &str,
) -> Result<std::sync::Arc<tokio::runtime::Runtime>, RuntimeError> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads.max(1))
        .thread_name(thread_name)
        .enable_all()
        .build()
        .map(std::sync::Arc::new)
        .map_err(RuntimeError::TokioBuild)
}
