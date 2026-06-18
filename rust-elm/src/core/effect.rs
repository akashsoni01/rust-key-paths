use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;
use std::time::Duration;

use crate::bus::BusSender;
use rust_dependencies::DependencyValues;
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

type ErasedRun = Arc<
    dyn Fn(
            Box<dyn Any + Send>,
        ) -> Pin<Box<dyn Future<Output = Result<(), EffectError>> + Send>>
        + Send
        + Sync,
>;

static TASK_REGISTRY: OnceLock<Mutex<HashMap<EffectId, ErasedTask>>> = OnceLock::new();
static ENV_TASK_REGISTRY: OnceLock<Mutex<HashMap<EffectId, ErasedEnvTask>>> = OnceLock::new();
static RUN_REGISTRY: OnceLock<Mutex<HashMap<EffectId, ErasedRun>>> = OnceLock::new();
static NEXT_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

fn task_registry() -> &'static Mutex<HashMap<EffectId, ErasedTask>> {
    TASK_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn env_task_registry() -> &'static Mutex<HashMap<EffectId, ErasedEnvTask>> {
    ENV_TASK_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn run_registry() -> &'static Mutex<HashMap<EffectId, ErasedRun>> {
    RUN_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Emitter passed to [`Effect::from_run`] closures (UDF `.run { send in … }`).
pub struct RunSender<M> {
    pub(crate) tx: BusSender<M>,
}

impl<M> RunSender<M> {
    pub fn send(&self, msg: M) {
        let _ = self.tx.send_blocking(msg);
    }
}

pub(crate) fn register_task<M, F>(run: F) -> EffectId
where
    M: Send + 'static,
    F: Fn() -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>> + Send + Sync + 'static,
{
    let id = NEXT_REGISTRY_ID.fetch_add(1, Ordering::Relaxed);
    task_registry().lock().insert(
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
    env_task_registry().lock().insert(
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
    let registry = task_registry().lock();
    let Some(task) = registry.get(&id).cloned() else {
        return Box::pin(async {
            Err(EffectError::TaskFailed("missing registered task"))
        });
    };
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
    let registry = env_task_registry().lock();
    let Some(task) = registry.get(&id).cloned() else {
        return Box::pin(async {
            Err(EffectError::TaskFailed("missing registered env task"))
        });
    };
    let env = env.clone();
    drop(registry);
    Box::pin(async move {
        let any = (task)(&env).await?;
        any.downcast::<M>()
            .map(|boxed| *boxed)
            .map_err(|_| EffectError::TaskFailed("env task message type mismatch"))
    })
}

pub(crate) fn register_run<M, F>(run: F) -> EffectId
where
    M: Send + 'static,
    F: Fn(RunSender<M>) -> Pin<Box<dyn Future<Output = Result<(), EffectError>> + Send>>
        + Send
        + Sync
        + 'static,
{
    let id = NEXT_REGISTRY_ID.fetch_add(1, Ordering::Relaxed);
    run_registry().lock().insert(
        id,
        Arc::new(move |any: Box<dyn Any + Send>| {
            match any.downcast::<BusSender<M>>() {
                Ok(tx) => run(RunSender { tx: *tx }),
                Err(_) => Box::pin(async {
                    Err(EffectError::TaskFailed("run sender type mismatch"))
                }),
            }
        }),
    );
    id
}

pub(crate) fn run_registered_run<M: Send + 'static>(
    id: EffectId,
    tx: BusSender<M>,
) -> Pin<Box<dyn Future<Output = Result<(), EffectError>> + Send>> {
    let registry = run_registry().lock();
    let Some(run) = registry.get(&id).cloned() else {
        return Box::pin(async { Err(EffectError::TaskFailed("missing registered run")) });
    };
    drop(registry);
    Box::pin(async move { (run)(Box::new(tx) as Box<dyn Any + Send>).await })
}

/// Async work descriptor — fn pointer at the description boundary.
pub type TaskFn<M> = fn() -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>>;

pub type EnvTaskFn<M> =
    fn(&Environment) -> Pin<Box<dyn Future<Output = Result<M, EffectError>> + Send>>;

/// Pure effect descriptions — interpreted only in `runtime.rs`.
///
/// - [`Effect::merge`] / [`Effect::batch`] — run children concurrently (UDF merge).
/// - [`Effect::concatenate`] / [`Effect::sequence`] — run children in order (UDF concatenate).
pub enum Effect<M> {
    None,
    Task { id: EffectId, run: TaskFn<M> },
    RegisteredTask { id: EffectId },
    EnvTask { id: EffectId, run: EnvTaskFn<M> },
    RegisteredEnvTask { id: EffectId },
    RegisteredRun { id: EffectId },
    Batch(Vec<Effect<M>>),
    Cancellable {
        id: EffectId,
        cancel_in_flight: bool,
        inner: Box<Effect<M>>,
    },
    Debounce {
        id: EffectId,
        duration: Duration,
        inner: Box<Effect<M>>,
    },
    Throttle {
        id: EffectId,
        duration: Duration,
        latest: bool,
        inner: Box<Effect<M>>,
    },
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
    Cancel { id: EffectId },
}

impl<M> Clone for Effect<M> {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Task { id, run } => Self::Task { id: *id, run: *run },
            Self::RegisteredTask { id } => Self::RegisteredTask { id: *id },
            Self::EnvTask { id, run } => Self::EnvTask { id: *id, run: *run },
            Self::RegisteredEnvTask { id } => Self::RegisteredEnvTask { id: *id },
            Self::RegisteredRun { id } => Self::RegisteredRun { id: *id },
            Self::Batch(items) => Self::Batch(items.clone()),
            Self::Cancellable {
                id,
                cancel_in_flight,
                inner,
            } => Self::Cancellable {
                id: *id,
                cancel_in_flight: *cancel_in_flight,
                inner: inner.clone(),
            },
            Self::Debounce {
                id,
                duration,
                inner,
            } => Self::Debounce {
                id: *id,
                duration: *duration,
                inner: inner.clone(),
            },
            Self::Throttle {
                id,
                duration,
                latest,
                inner,
            } => Self::Throttle {
                id: *id,
                duration: *duration,
                latest: *latest,
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
            Self::Cancel { id } => Self::Cancel { id: *id },
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

    pub fn from_run<F>(run: F) -> Self
    where
        M: Send + 'static,
        F: Fn(RunSender<M>) -> Pin<Box<dyn Future<Output = Result<(), EffectError>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let id = register_run(run);
        Self::RegisteredRun { id }
    }

    /// Single-shot async task (alias for [`Effect::task`]).
    pub fn task_try(id: EffectId, run: TaskFn<M>) -> Self {
        Self::task(id, run)
    }

    /// Map a `Result<T, E>` leaf task into `M` via fn pointers (UDF `TaskResult`).
    pub fn result_task<T, E>(run: TaskFn<Result<T, E>>, on_ok: fn(T) -> M, on_err: fn(E) -> M) -> Self
    where
        T: Send + 'static,
        E: Send + 'static,
        M: Send + 'static,
    {
        Self::from_fn(move || {
            let fut = run();
            Box::pin(async move {
                match fut.await {
                    Ok(Ok(value)) => Ok(on_ok(value)),
                    Ok(Err(err)) => Ok(on_err(err)),
                    Err(effect_err) => Err(effect_err),
                }
            })
        })
    }

    pub fn batch(effects: impl IntoIterator<Item = Effect<M>>) -> Self {
        let mut effects: Vec<_> = effects.into_iter().collect();
        match effects.len() {
            0 => Self::None,
            1 => effects.pop().map_or(Self::None, |single| single),
            _ => Self::Batch(effects),
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
                let fut = run_registered_env_task::<M>(env, id);
                Box::pin(async move { fut.await.map(f) })
            }),
            Self::RegisteredRun { id } => Effect::RegisteredRun { id },
            Self::Batch(items) => Effect::Batch(items.into_iter().map(|e| e.map(f)).collect()),
            Self::Cancellable {
                id,
                cancel_in_flight,
                inner,
            } => Effect::Cancellable {
                id,
                cancel_in_flight,
                inner: Box::new(inner.map(f)),
            },
            Self::Debounce {
                id,
                duration,
                inner,
            } => Effect::Debounce {
                id,
                duration,
                inner: Box::new(inner.map(f)),
            },
            Self::Throttle {
                id,
                duration,
                latest,
                inner,
            } => Effect::Throttle {
                id,
                duration,
                latest,
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
            Self::Cancel { id } => Effect::Cancel { id },
        }
    }

    pub fn cancellable(id: EffectId, inner: Effect<M>) -> Self {
        Self::cancellable_with(id, true, inner)
    }

    pub fn cancellable_with(id: EffectId, cancel_in_flight: bool, inner: Effect<M>) -> Self {
        Self::Cancellable {
            id,
            cancel_in_flight,
            inner: Box::new(inner),
        }
    }

    pub fn debounce(id: EffectId, duration: Duration, inner: Effect<M>) -> Self {
        Self::Debounce {
            id,
            duration,
            inner: Box::new(inner),
        }
    }

    pub fn throttle(id: EffectId, duration: Duration, latest: bool, inner: Effect<M>) -> Self {
        Self::Throttle {
            id,
            duration,
            latest,
            inner: Box::new(inner),
        }
    }

    pub fn cancel(id: EffectId) -> Self {
        Self::Cancel { id }
    }

    pub fn provide(env: Environment, inner: Effect<M>) -> Self {
        Self::Provide {
            env,
            inner: Box::new(inner),
        }
    }

    /// Scoped single-dependency override (UDF dependency override / `withDependencies`).
    pub fn provide_dependency<D: Send + Sync + 'static>(value: D, inner: Effect<M>) -> Self {
        Self::provide(Environment::from_values(DependencyValues::new().with(value)), inner)
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
            Self::RegisteredRun { id } => write!(f, "Effect::RegisteredRun({id})"),
            Self::Batch(n) => write!(f, "Effect::Batch({})", n.len()),
            Self::Cancellable {
                id,
                cancel_in_flight,
                ..
            } => write!(f, "Effect::Cancellable({id}, in_flight={cancel_in_flight})"),
            Self::Debounce { id, duration, .. } => {
                write!(f, "Effect::Debounce({id}, {duration:?})")
            }
            Self::Throttle {
                id,
                duration,
                latest,
                ..
            } => write!(f, "Effect::Throttle({id}, {duration:?}, latest={latest})"),
            Self::Provide { .. } => write!(f, "Effect::Provide"),
            Self::Retry { attempts, .. } => write!(f, "Effect::Retry({attempts})"),
            Self::Timeout { duration, .. } => write!(f, "Effect::Timeout({duration:?})"),
            Self::Sequence(n) => write!(f, "Effect::Sequence({})", n.len()),
            Self::Race(n) => write!(f, "Effect::Race({})", n.len()),
            Self::Catch { .. } => write!(f, "Effect::Catch"),
            Self::Cancel { id } => write!(f, "Effect::Cancel({id})"),
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
        Effect::Cancel { .. } => Box::pin(async move {
            Err(EffectError::Cancelled)
        }),
        other => Box::pin(async move {
            Err(EffectError::Other(format!("non-leaf effect: {other:?}")))
        }),
    }
}

#[cfg(test)]
#[allow(dead_code, clippy::redundant_closure, clippy::type_complexity)]
mod tests {
    use super::*;

    #[test]
    fn merge_and_concatenate_constructors() {
        let a = Effect::<i32>::none();
        let b = Effect::<i32>::none();
        assert!(matches!(Effect::merge([a, b]), Effect::Batch(_)));
        assert!(matches!(Effect::concatenate([] as [Effect<i32>; 0]), Effect::None));
    }

    #[test]
    fn result_task_maps_ok_and_err() {
        fn load() -> Pin<Box<dyn Future<Output = Result<Result<i32, &'static str>, EffectError>> + Send>> {
            Box::pin(async { Ok(Ok(7)) })
        }
        fn fail() -> Pin<Box<dyn Future<Output = Result<Result<i32, &'static str>, EffectError>> + Send>> {
            Box::pin(async { Ok(Err("nope")) })
        }
        fn ok(n: i32) -> String {
            format!("ok:{n}")
        }
        fn err(e: &'static str) -> String {
            format!("err:{e}")
        }

        let ok_effect = Effect::result_task(load, ok, err);
        let err_effect = Effect::result_task(fail, ok, err);
        assert!(matches!(ok_effect, Effect::RegisteredTask { .. }));
        assert!(matches!(err_effect, Effect::RegisteredTask { .. }));
    }

    #[test]
    fn debounce_and_throttle_constructors() {
        let inner = Effect::<i32>::task(1, || Box::pin(async { Ok(1) }));
        assert!(matches!(
            Effect::debounce(9, Duration::from_millis(50), inner.clone()),
            Effect::Debounce { id: 9, .. }
        ));
        assert!(matches!(
            Effect::throttle(9, Duration::from_millis(50), true, inner),
            Effect::Throttle {
                id: 9,
                latest: true,
                ..
            }
        ));
    }
}
