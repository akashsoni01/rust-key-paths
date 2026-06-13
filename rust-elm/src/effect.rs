use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use crate::env::Environment;
use crate::error::EffectError;

pub type EffectId = u64;

type ErasedTask = Arc<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<Box<dyn Any + Send>, EffectError>> + Send>>
        + Send
        + Sync,
>;

type ErasedEnvTask = Arc<
    dyn Fn(
            &Environment,
        ) -> Pin<Box<dyn Future<Output = Result<Box<dyn Any + Send>, EffectError>> + Send>>
        + Send
        + Sync,
>;

static TASK_REGISTRY: OnceLock<Mutex<HashMap<EffectId, ErasedTask>>> = OnceLock::new();
static ENV_TASK_REGISTRY: OnceLock<Mutex<HashMap<EffectId, ErasedEnvTask>>> = OnceLock::new();
static NEXT_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

fn task_registry() -> &'static Mutex<HashMap<EffectId, ErasedTask>> {
    TASK_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn env_task_registry() -> &'static Mutex<HashMap<EffectId, ErasedEnvTask>> {
    ENV_TASK_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn register_task<M, F>(run: F) -> EffectId
where
    M: Send + 'static,
    F: Fn() -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>> + Send + Sync + 'static,
{
    let id = NEXT_REGISTRY_ID.fetch_add(1, Ordering::Relaxed);
    task_registry().lock().unwrap().insert(
        id,
        Arc::new(move || {
            let fut = run();
            Box::pin(async move {
                fut.await.map(|value| Box::new(value) as Box<dyn Any + Send>)
            })
        }),
    );
    id
}

pub(crate) fn register_env_task<M, F>(run: F) -> EffectId
where
    M: Send + 'static,
    F: Fn(&Environment) -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>>
        + Send
        + Sync
        + 'static,
{
    let id = NEXT_REGISTRY_ID.fetch_add(1, Ordering::Relaxed);
    env_task_registry().lock().unwrap().insert(
        id,
        Arc::new(move |env| {
            let fut = run(env);
            Box::pin(async move {
                fut.await.map(|value| Box::new(value) as Box<dyn Any + Send>)
            })
        }),
    );
    id
}

pub(crate) fn run_registered_task<M: Send + 'static>(
    id: EffectId,
) -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>> {
    let registry = task_registry().lock().unwrap();
    let task = registry.get(&id).expect("missing registered task").clone();
    drop(registry);
    Box::pin(async move {
        let any = (task)().await?;
        any.downcast::<M>()
            .map(|boxed| *boxed)
            .map_err(|_| EffectError::TaskFailed("task message type mismatch"))
    })
}

pub(crate) fn run_registered_env_task<M: Send + 'static>(
    env: &Environment,
    id: EffectId,
) -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>> {
    let registry = env_task_registry().lock().unwrap();
    let task = registry.get(&id).expect("missing registered env task").clone();
    let env = env.clone();
    drop(registry);
    Box::pin(async move {
        let any = (task)(&env).await?;
        any.downcast::<M>()
            .map(|boxed| *boxed)
            .map_err(|_| EffectError::TaskFailed("env task message type mismatch"))
    })
}

/// Async work descriptor — fn pointer at the description boundary.
pub type TaskFn<M> = fn() -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>>;

pub type EnvTaskFn<M> =
    fn(&Environment) -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>>;

/// Pure effect descriptions — interpreted only in `runtime.rs`.
pub enum Effect<M> {
    None,
    Task { id: EffectId, run: TaskFn<M> },
    RegisteredTask { id: EffectId },
    EnvTask { id: EffectId, run: EnvTaskFn<M> },
    RegisteredEnvTask { id: EffectId },
    Batch(Vec<Effect<M>>),
    Cancellable { id: EffectId, inner: Box<Effect<M>> },
    Provide {
        env: Environment,
        inner: Box<Effect<M>>,
    },
    Retry {
        attempts: u32,
        inner: Box<Effect<M>>,
    },
    Timeout {
        duration: Duration,
        inner: Box<Effect<M>>,
    },
    Sequence(Vec<Effect<M>>),
    Race(Vec<Effect<M>>),
    Catch {
        inner: Box<Effect<M>>,
        recover: fn(EffectError) -> Effect<M>,
    },
}

impl<M> Clone for Effect<M> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Task { id, run } => Self::Task { id: *id, run: *run },
            Self::RegisteredTask { id } => Self::RegisteredTask { id: *id },
            Self::EnvTask { id, run } => Self::EnvTask { id: *id, run: *run },
            Self::RegisteredEnvTask { id } => Self::RegisteredEnvTask { id: *id },
            Self::Batch(items) => Self::Batch(items.clone()),
            Self::Cancellable { id, inner } => Self::Cancellable {
                id: *id,
                inner: inner.clone(),
            },
            Self::Provide { env, inner } => Self::Provide {
                env: env.clone(),
                inner: inner.clone(),
            },
            Self::Retry { attempts, inner } => Self::Retry {
                attempts: *attempts,
                inner: inner.clone(),
            },
            Self::Timeout { duration, inner } => Self::Timeout {
                duration: *duration,
                inner: inner.clone(),
            },
            Self::Sequence(items) => Self::Sequence(items.clone()),
            Self::Race(items) => Self::Race(items.clone()),
            Self::Catch { inner, recover } => Self::Catch {
                inner: inner.clone(),
                recover: *recover,
            },
        }
    }
}

impl<M> Effect<M> {
    pub fn none() -> Self {
        Self::None
    }

    pub fn task(id: EffectId, run: TaskFn<M>) -> Self {
        Self::Task { id, run }
    }

    pub fn from_fn<F>(run: F) -> Self
    where
        M: Send + 'static,
        F: Fn() -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let id = register_task(run);
        Self::RegisteredTask { id }
    }

    pub fn env_task(id: EffectId, run: EnvTaskFn<M>) -> Self {
        Self::EnvTask { id, run }
    }

    pub fn from_env_fn<F>(run: F) -> Self
    where
        M: Send + 'static,
        F: Fn(&Environment) -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let id = register_env_task(run);
        Self::RegisteredEnvTask { id }
    }

    pub fn batch(effects: impl IntoIterator<Item = Effect<M>>) -> Self {
        let effects: Vec<_> = effects.into_iter().collect();
        if effects.is_empty() {
            Self::None
        } else if effects.len() == 1 {
            effects.into_iter().next().unwrap()
        } else {
            Self::Batch(effects)
        }
    }

    pub fn merge(effects: impl IntoIterator<Item = Effect<M>>) -> Self {
        Self::batch(effects)
    }

    pub fn concatenate(effects: impl IntoIterator<Item = Effect<M>>) -> Self {
        let effects: Vec<_> = effects.into_iter().collect();
        if effects.is_empty() {
            Self::None
        } else {
            Self::Sequence(effects)
        }
    }

    pub fn map<N>(self, f: fn(M) -> N) -> Effect<N>
    where
        M: Send + 'static,
        N: Send + 'static,
    {
        match self {
            Self::None => Effect::None,
            Self::Task { run, .. } => Effect::from_fn(move || {
                let fut = run();
                Box::pin(async move { fut.await.map(f) })
            }),
            Self::RegisteredTask { id } => Effect::from_fn(move || {
                let fut = run_registered_task::<M>(id);
                Box::pin(async move { fut.await.map(f) })
            }),
            Self::EnvTask { run, .. } => Effect::from_env_fn(move |env| {
                let fut = run(env);
                Box::pin(async move { fut.await.map(f) })
            }),
            Self::RegisteredEnvTask { id } => Effect::from_env_fn(move |env| {
                let fut = run_registered_env_task::<M>(&env, id);
                Box::pin(async move { fut.await.map(f) })
            }),
            Self::Batch(items) => Effect::Batch(items.into_iter().map(|e| e.map(f)).collect()),
            Self::Cancellable { id, inner } => Effect::Cancellable {
                id,
                inner: Box::new(inner.map(f)),
            },
            Self::Provide { env, inner } => Effect::Provide {
                env,
                inner: Box::new(inner.map(f)),
            },
            Self::Retry { attempts, inner } => Effect::Retry {
                attempts,
                inner: Box::new(inner.map(f)),
            },
            Self::Timeout { duration, inner } => Effect::Timeout {
                duration,
                inner: Box::new(inner.map(f)),
            },
            Self::Sequence(items) => Effect::Sequence(items.into_iter().map(|e| e.map(f)).collect()),
            Self::Race(items) => Effect::Race(items.into_iter().map(|e| e.map(f)).collect()),
            Self::Catch { inner, recover: _ } => inner.map(f),
        }
    }

    pub fn cancellable(id: EffectId, inner: Effect<M>) -> Self {
        Self::Cancellable {
            id,
            inner: Box::new(inner),
        }
    }

    pub fn provide(env: Environment, inner: Effect<M>) -> Self {
        Self::Provide {
            env,
            inner: Box::new(inner),
        }
    }

    pub fn retry(attempts: u32, inner: Effect<M>) -> Self {
        Self::Retry {
            attempts,
            inner: Box::new(inner),
        }
    }

    pub fn timeout(duration: Duration, inner: Effect<M>) -> Self {
        Self::Timeout {
            duration,
            inner: Box::new(inner),
        }
    }

    pub fn sequence(effects: impl IntoIterator<Item = Effect<M>>) -> Self {
        Self::concatenate(effects)
    }

    pub fn race(effects: impl IntoIterator<Item = Effect<M>>) -> Self {
        let effects: Vec<_> = effects.into_iter().collect();
        if effects.is_empty() {
            Self::None
        } else {
            Self::Race(effects)
        }
    }

    pub fn catch(inner: Effect<M>, recover: fn(EffectError) -> Effect<M>) -> Self {
        Self::Catch {
            inner: Box::new(inner),
            recover,
        }
    }
}

impl<M> PartialEq for Effect<M> {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl<M> std::fmt::Debug for Effect<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Effect::None"),
            Self::Task { id, .. } => write!(f, "Effect::Task({id})"),
            Self::RegisteredTask { id } => write!(f, "Effect::RegisteredTask({id})"),
            Self::EnvTask { id, .. } => write!(f, "Effect::EnvTask({id})"),
            Self::RegisteredEnvTask { id } => write!(f, "Effect::RegisteredEnvTask({id})"),
            Self::Batch(n) => write!(f, "Effect::Batch({})", n.len()),
            Self::Cancellable { id, .. } => write!(f, "Effect::Cancellable({id})"),
            Self::Provide { .. } => write!(f, "Effect::Provide"),
            Self::Retry { attempts, .. } => write!(f, "Effect::Retry({attempts})"),
            Self::Timeout { duration, .. } => write!(f, "Effect::Timeout({duration:?})"),
            Self::Sequence(n) => write!(f, "Effect::Sequence({})", n.len()),
            Self::Race(n) => write!(f, "Effect::Race({})", n.len()),
            Self::Catch { .. } => write!(f, "Effect::Catch"),
        }
    }
}

pub(crate) fn run_leaf<M>(
    effect: Effect<M>,
    env: &Environment,
) -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>>
where
    M: Send + 'static,
{
    match effect {
        Effect::Task { run, .. } => run(),
        Effect::RegisteredTask { id } => run_registered_task(id),
        Effect::EnvTask { run, .. } => run(env),
        Effect::RegisteredEnvTask { id } => run_registered_env_task(env, id),
        other => Box::pin(async move {
            Err(EffectError::Other(format!("non-leaf effect: {other:?}")))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_and_concatenate_constructors() {
        let a = Effect::<i32>::none();
        let b = Effect::<i32>::none();
        assert!(matches!(Effect::merge([a, b]), Effect::Batch(_)));
        assert!(matches!(Effect::concatenate([] as [Effect<i32>; 0]), Effect::None));
    }
}
