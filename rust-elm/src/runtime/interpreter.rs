use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::task::JoinHandle as TokioJoinHandle;

use crate::effect::{Effect, EffectId};
use crate::env::Environment;
use crate::error::EffectError;
use crate::interp::flatten_effects;
use crate::bus::BusSender;
use super::dispatch::dispatch_from_effect;
use super::store::StoreWork;

pub(crate) struct InterpreterState<M> {
    pub cancel_tokens: Mutex<HashMap<EffectId, tokio::task::AbortHandle>>,
    debounce_timers: Mutex<HashMap<EffectId, TokioJoinHandle<()>>>,
    throttle_gates: Mutex<HashMap<EffectId, ThrottleGate<M>>>,
}

impl<M> InterpreterState<M> {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            cancel_tokens: Mutex::new(HashMap::new()),
            debounce_timers: Mutex::new(HashMap::new()),
            throttle_gates: Mutex::new(HashMap::new()),
        })
    }
}

struct ThrottleGate<M> {
    latest: bool,
    pending: Option<Effect<M>>,
    timer: Option<TokioJoinHandle<()>>,
}

pub(super) async fn interpret_effects_async<S, M, B>(
    effects: Vec<Effect<M>>,
    tx: BusSender<M>,
    env: Environment,
    handle: tokio::runtime::Handle,
    backend: B,
) -> Vec<TokioJoinHandle<()>>
where
    S: Send + 'static,
    M: Send + 'static,
    B: StoreWork<S, M> + Clone + Send + Sync + 'static,
{
    let interpreter = backend.hub().interpreter.clone();
    let mut handles = Vec::new();
    for effect in effects {
        for leaf in flatten_effects(effect) {
            handles.extend(spawn_effect(
                leaf,
                tx.clone(),
                env.clone(),
                handle.clone(),
                interpreter.clone(),
                backend.clone(),
            ));
        }
    }
    handles
}

fn spawn_effect<S, M, B>(
    effect: Effect<M>,
    tx: BusSender<M>,
    env: Environment,
    handle: tokio::runtime::Handle,
    interpreter: Arc<InterpreterState<M>>,
    backend: B,
) -> Vec<TokioJoinHandle<()>>
where
    S: Send + 'static,
    M: Send + 'static,
    B: StoreWork<S, M> + Clone + Send + Sync + 'static,
{
    let mut batch = Vec::new();
    spawn_effect_inner(
        effect,
        tx,
        env,
        handle,
        interpreter,
        backend,
        &mut batch,
    );
    batch
}

fn track_spawn<M>(
    join: TokioJoinHandle<()>,
    cancel_id: Option<EffectId>,
    interpreter: &InterpreterState<M>,
    batch: &mut Vec<TokioJoinHandle<()>>,
) {
    if let Some(id) = cancel_id {
        interpreter
            .cancel_tokens
            .lock()
            .insert(id, join.abort_handle());
    }
    batch.push(join);
}

fn spawn_effect_inner<S, M, B>(
    effect: Effect<M>,
    tx: BusSender<M>,
    env: Environment,
    handle: tokio::runtime::Handle,
    interpreter: Arc<InterpreterState<M>>,
    backend: B,
    batch: &mut Vec<TokioJoinHandle<()>>,
) where
    S: Send + 'static,
    M: Send + 'static,
    B: StoreWork<S, M> + Clone + Send + Sync + 'static,
{
    use crate::effect::{
        run_registered_env_task, run_registered_run, run_registered_task,
    };
    use tokio::time::timeout;

    match effect {
        Effect::None => {}
        Effect::Cancel { id } => {
            if let Some(old) = interpreter.cancel_tokens.lock().remove(&id) {
                old.abort();
            }
        }
        Effect::Task { id, run } => {
            let tx = tx.clone();
            let backend = backend.clone();
            track_spawn(
                handle.spawn(async move {
                    if let Ok(msg) = run().await {
                        dispatch_from_effect(&backend, &tx, msg);
                    }
                }),
                Some(id),
                &interpreter,
                batch,
            );
        }
        Effect::RegisteredTask { id } => {
            let tx = tx.clone();
            let backend = backend.clone();
            track_spawn(
                handle.spawn(async move {
                    if let Ok(msg) = run_registered_task::<M>(id).await {
                        dispatch_from_effect(&backend, &tx, msg);
                    }
                }),
                Some(id),
                &interpreter,
                batch,
            );
        }
        Effect::RegisteredRun { id } => {
            let tx = tx.clone();
            track_spawn(
                handle.spawn(async move {
                    let _ = run_registered_run::<M>(id, tx.clone()).await;
                }),
                Some(id),
                &interpreter,
                batch,
            );
        }
        Effect::EnvTask { id, run } => {
            let tx = tx.clone();
            let env = env.clone();
            let backend = backend.clone();
            track_spawn(
                handle.spawn(async move {
                    if let Ok(msg) = run(&env).await {
                        dispatch_from_effect(&backend, &tx, msg);
                    }
                }),
                Some(id),
                &interpreter,
                batch,
            );
        }
        Effect::RegisteredEnvTask { id } => {
            let tx = tx.clone();
            let env = env.clone();
            let backend = backend.clone();
            track_spawn(
                handle.spawn(async move {
                    if let Ok(msg) = run_registered_env_task::<M>(&env, id).await {
                        dispatch_from_effect(&backend, &tx, msg);
                    }
                }),
                Some(id),
                &interpreter,
                batch,
            );
        }
        Effect::Batch(items) | Effect::Race(items) => {
            for item in items {
                spawn_effect_inner(
                    item,
                    tx.clone(),
                    env.clone(),
                    handle.clone(),
                    interpreter.clone(),
                    backend.clone(),
                    batch,
                );
            }
        }
        Effect::Sequence(items) => {
            let tx = tx.clone();
            let env = env.clone();
            let backend = backend.clone();
            track_spawn(
                handle.spawn(async move {
                    for item in items {
                        if let Ok(msg) = run_effect_once(item, env.clone()).await {
                            dispatch_from_effect(&backend, &tx, msg);
                        }
                    }
                }),
                None,
                &interpreter,
                batch,
            );
        }
        Effect::Cancellable {
            id,
            cancel_in_flight,
            inner,
        } => {
            if cancel_in_flight
                && let Some(old) = interpreter.cancel_tokens.lock().remove(&id)
            {
                old.abort();
            }
            spawn_effect_inner(*inner, tx, env, handle, interpreter, backend, batch);
        }
        Effect::Debounce { id, duration, inner } => {
            if let Some(old) = interpreter.debounce_timers.lock().remove(&id) {
                old.abort();
            }
            let inner = *inner;
            let tx = tx.clone();
            let env = env.clone();
            let handle_worker = handle.clone();
            let interpreter_worker = interpreter.clone();
            let backend_worker = backend.clone();
            let join = handle.spawn(async move {
                tokio::time::sleep(duration).await;
                interpreter_worker.debounce_timers.lock().remove(&id);
                spawn_effect(
                    inner,
                    tx,
                    env,
                    handle_worker,
                    interpreter_worker,
                    backend_worker,
                );
            });
            interpreter.debounce_timers.lock().insert(id, join);
        }
        Effect::Throttle {
            id,
            duration,
            latest,
            inner,
        } => {
            let mut gates = interpreter.throttle_gates.lock();
            let gate = gates.entry(id).or_insert(ThrottleGate {
                latest,
                pending: None,
                timer: None,
            });
            gate.latest = latest;

            if gate.timer.is_none() {
                if latest {
                    gate.pending = Some(*inner);
                } else {
                    let tx_now = tx.clone();
                    let env_now = env.clone();
                    let handle_now = handle.clone();
                    let interpreter_now = interpreter.clone();
                    let backend_now = backend.clone();
                    batch.extend(spawn_effect(
                        *inner,
                        tx_now,
                        env_now,
                        handle_now,
                        interpreter_now,
                        backend_now,
                    ));
                }
                let tx_timer = tx.clone();
                let env_timer = env.clone();
                let handle_timer = handle.clone();
                let interpreter_timer = interpreter.clone();
                let backend_timer = backend.clone();
                let join = handle.spawn(async move {
                    tokio::time::sleep(duration).await;
                    let effect_to_run = {
                        let mut gates = interpreter_timer.throttle_gates.lock();
                        if let Some(g) = gates.get_mut(&id) {
                            g.timer = None;
                            if g.latest {
                                g.pending.take()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };
                    if let Some(effect) = effect_to_run {
                        spawn_effect(
                            effect,
                            tx_timer,
                            env_timer,
                            handle_timer,
                            interpreter_timer,
                            backend_timer,
                        );
                    } else {
                        interpreter_timer.throttle_gates.lock().remove(&id);
                    }
                });
                gate.timer = Some(join);
            } else if latest {
                gate.pending = Some(*inner);
            }
        }
        Effect::Provide { env: layer, inner } => {
            let scoped = env.scoped_with(layer);
            spawn_effect_inner(*inner, tx, scoped, handle, interpreter, backend, batch);
        }
        Effect::Retry { attempts, inner } => {
            let tx = tx.clone();
            let env = env.clone();
            let backend = backend.clone();
            let inner = *inner;
            track_spawn(
                handle.spawn(async move {
                    for _ in 0..attempts.max(1) {
                        match run_effect_once(inner.clone(), env.clone()).await {
                            Ok(msg) => {
                                dispatch_from_effect(&backend, &tx, msg);
                                break;
                            }
                            Err(_) => continue,
                        }
                    }
                }),
                None,
                &interpreter,
                batch,
            );
        }
        Effect::Timeout { duration, inner } => {
            let tx = tx.clone();
            let env = env.clone();
            let backend = backend.clone();
            track_spawn(
                handle.spawn(async move {
                    if let Ok(Ok(msg)) = timeout(duration, run_effect_once(*inner, env)).await {
                        dispatch_from_effect(&backend, &tx, msg);
                    }
                }),
                None,
                &interpreter,
                batch,
            );
        }
        Effect::Catch { inner, recover } => {
            let tx = tx.clone();
            let env = env.clone();
            let handle_worker = handle.clone();
            let backend_worker = backend.clone();
            track_spawn(
                handle.spawn(async move {
                    match run_effect_once(*inner, env.clone()).await {
                        Ok(msg) => {
                            dispatch_from_effect(&backend_worker, &tx, msg);
                        }
                        Err(err) => {
                            for join in interpret_effects_async(
                                flatten_effects(recover(err)),
                                tx,
                                env,
                                handle_worker.clone(),
                                backend_worker,
                            )
                            .await
                            {
                                let _ = join.await;
                            }
                        }
                    }
                }),
                None,
                &interpreter,
                batch,
            );
        }
    }
}

async fn run_effect_once<M>(effect: Effect<M>, env: Environment) -> Result<M, EffectError>
where
    M: Send + 'static,
{
    crate::effect::run_leaf(effect, &env).await
}
