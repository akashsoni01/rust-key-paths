use std::fmt;

/// Errors surfaced while interpreting effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectError {
    Cancelled,
    Timeout,
    RetryExhausted,
    EnvMissing(&'static str),
    TaskFailed(&'static str),
    Other(String),
}

impl fmt::Display for EffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => write!(f, "effect cancelled"),
            Self::Timeout => write!(f, "effect timed out"),
            Self::RetryExhausted => write!(f, "retry attempts exhausted"),
            Self::EnvMissing(name) => write!(f, "missing environment dependency: {name}"),
            Self::TaskFailed(msg) => write!(f, "task failed: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for EffectError {}
