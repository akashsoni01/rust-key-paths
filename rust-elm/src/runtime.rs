use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::RecvTimeoutError;
use parking_lot::Mutex;
use tokio::runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime};
use tokio::task::JoinHandle as TokioJoinHandle;
use tokio::time::timeout;

use crate::bus::{Bus, BusSender};
use crate::cmd::Cmd;
use crate::effect::{
    run_leaf, run_registered_env_task, run_registered_run, run_registered_task, Effect, EffectId,
};
use crate::env::Environment;
use crate::error::EffectError;
use crate::interp::flatten_effects;
use crate::program::{Program, ReducerProgram};
use crate::reducer::Reducer;
use crate::reduce_panic::catch_reduce_panic;
use crate::store::{StoreBackend, StoreWorkUnwindGuard};
use crate::sub::Sub;

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

/// Configuration for [`Runtime::from_program`] / [`Runtime::from_reducer_program`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Bounded capacity of the action bus (`send_blocking` back-pressures when full).
    pub bus_capacity: usize,
    /// Tokio runtime worker threads (async effects / subscriptions).
    pub worker_threads: usize,
    /// Name of the dedicated reducer thread.
    pub thread_name: &'static str,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            bus_capacity: 4_096,
            worker_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            thread_name: "rust-elm",
        }
    }
}

impl RuntimeConfig {
    /// `bus_capacity` only; other fields from [`Default`].
    pub fn new(bus_capacity: usize) -> Self {
        Self {
            bus_capacity,
            ..Self::default()
        }
    }
}

struct AppRunner {
    shutdown: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
    subscriptions: Arc<crate::subscription::SubscriptionHandles>,
}

impl AppRunner {
    fn new(
        shutdown: Arc<AtomicBool>,
        thread: JoinHandle<()>,
        subscriptions: Arc<crate::subscription::SubscriptionHandles>,
    ) -> Self {
        Self {
            shutdown,
            thread: Mutex::new(Some(thread)),
            subscriptions,
        }
    }

    fn shutdown(&self) {
        if self.shutdown.swap(true, Ordering::Relaxed) {
            return;
        }
        self.subscriptions.abort_all();
        if let Some(handle) = self.thread.lock().take() {
            let _ = handle.join();
        }
    }
}

fn build_tokio(config: &RuntimeConfig) -> Arc<TokioRuntime> {
    Arc::new(
        TokioRuntimeBuilder::new_multi_thread()
            .worker_threads(config.worker_threads.max(1))
            .thread_name(config.thread_name)
            .enable_all()
            .build()
            .expect("failed to create Tokio runtime for rust-elm"),
    )
}

/// Shared Tokio pool and environment for many isolated [`Runtime`] instances.
///
/// Each [`RuntimeHost::spawn_program`] still gets its own action bus, state, and reducer
/// thread, but async effects and subscriptions share one Tokio runtime.
///
/// ```
/// use rust_elm::{Environment, Program, RuntimeConfig, RuntimeHost};
///
/// fn init() -> (i32, rust_elm::Cmd<i32>) { (0, rust_elm::Cmd::none()) }
/// fn update(n: &mut i32, m: i32) -> rust_elm::Cmd<i32> { *n += m; rust_elm::Cmd::none() }
/// fn subs(_: &i32) -> rust_elm::Sub<i32> { rust_elm::Sub::none() }
///
/// let host = RuntimeHost::new(Environment::new(), RuntimeConfig::new(1024));
/// let programs = [
///     Program::new(init, update, subs),
///     Program::new(init, update, subs),
/// ];
/// let runtimes: Vec<_> = programs.iter().map(|p| host.spawn_program(*p)).collect();
/// // ... use runtimes ...
/// host.shutdown();
/// ```
pub struct RuntimeHost {
    tokio: Arc<TokioRuntime>,
    env: Environment,
    config: RuntimeConfig,
    apps: Mutex<Vec<Arc<AppRunner>>>,
}

impl RuntimeHost {
    pub fn new(env: Environment, config: RuntimeConfig) -> Self {
        let tokio = build_tokio(&config);
        Self {
            tokio,
            env,
            config,
            apps: Mutex::new(Vec::new()),
        }
    }

    pub fn env(&self) -> &Environment {
        &self.env
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub fn tokio(&self) -> &TokioRuntime {
        &self.tokio
    }

    pub fn app_count(&self) -> usize {
        self.apps.lock().len()
    }

    pub fn spawn_program<S, M>(&self, program: Program<S, M>) -> Runtime<S, M>
    where
        S: Send + Sync + 'static,
        M: Send + 'static,
    {
        self.spawn(program.init, program.update, program.subscriptions)
    }

    pub fn spawn_reducer_program<R>(&self, program: ReducerProgram<R>) -> Runtime<R::State, R::Action>
    where
        R: Reducer + Send + Sync + 'static,
        R::State: Send + Sync + 'static,
        R::Action: Send + 'static,
    {
        let reducer = Arc::new(program.reducer);
        let init = program.init;
        let subscriptions = program.subscriptions;
        self.spawn(
            init,
            move |state, msg| reducer.reduce(state, msg),
            subscriptions,
        )
    }

    fn spawn<S, M>(
        &self,
        init: fn() -> (S, Cmd<M>),
        update: impl Fn(&mut S, M) -> Cmd<M> + Send + Sync + 'static,
        subscriptions: fn(&S) -> Sub<M>,
    ) -> Runtime<S, M>
    where
        S: Send + Sync + 'static,
        M: Send + 'static,
    {
        let app_id = self.apps.lock().len();
        let reducer_thread_name = format!("{}-{}", self.config.thread_name, app_id);
        let runtime = Runtime::bootstrap(
            init,
            update,
            subscriptions,
            self.env.clone(),
            self.config.clone(),
            self.tokio.clone(),
            reducer_thread_name,
        );
        self.apps.lock().push(runtime.runner.clone());
        runtime
    }

    /// Shut down every program spawned on this host (idempotent).
    pub fn shutdown(self) {
        for app in self.apps.lock().drain(..) {
            app.shutdown();
        }
    }
}

/// Live runtime — bus-driven update loop on a pinned thread with Tokio effect interpreter.
pub struct Runtime<S, M> {
    pub state: Arc<Mutex<S>>,
    pub bus: Bus<M>,
    pub env: Environment,
    backend: StoreBackend<S, M>,
    runner: Arc<AppRunner>,
    _tokio: Arc<TokioRuntime>,
}

impl<S, M> Runtime<S, M>
where
    S: Send + Sync + 'static,
    M: Send + 'static,
{
    pub fn from_program(program: Program<S, M>, env: Environment, config: RuntimeConfig) -> Self {
        let tokio = build_tokio(&config);
        let reducer_thread_name = config.thread_name.to_string();
        Self::bootstrap(
            program.init,
            program.update,
            program.subscriptions,
            env,
            config,
            tokio,
            reducer_thread_name,
        )
    }

    pub fn from_reducer_program<R>(
        program: ReducerProgram<R>,
        env: Environment,
        config: RuntimeConfig,
    ) -> Self
    where
        R: Reducer<State = S, Action = M> + Send + Sync + 'static,
    {
        let tokio = build_tokio(&config);
        let reducer_thread_name = config.thread_name.to_string();
        let reducer = Arc::new(program.reducer);
        let init = program.init;
        let subscriptions = program.subscriptions;
        Self::bootstrap(
            init,
            move |state, msg| reducer.reduce(state, msg),
            subscriptions,
            env,
            config,
            tokio,
            reducer_thread_name,
        )
    }

    fn bootstrap(
        init: fn() -> (S, Cmd<M>),
        update: impl Fn(&mut S, M) -> Cmd<M> + Send + Sync + 'static,
        subscriptions: fn(&S) -> Sub<M>,
        env: Environment,
        config: RuntimeConfig,
        tokio: Arc<TokioRuntime>,
        reducer_thread_name: String,
    ) -> Self {
        let RuntimeConfig { bus_capacity, .. } = config;
        let (initial_state, init_cmd) = init();
        let state = Arc::new(Mutex::new(initial_state));
        let bus = Bus::new(bus_capacity);
        let shutdown = Arc::new(AtomicBool::new(false));
        let interpreter = InterpreterState::new();
        let sub_handles = Arc::new(crate::subscription::SubscriptionHandles::new());
        let backend = StoreBackend::new(state.clone(), bus.sender(), interpreter);
        let tx = bus.sender();
        let init_handles = tokio.block_on(interpret_effects_async(
            init_cmd.into_effects(),
            tx.clone(),
            env.clone(),
            tokio.handle().clone(),
            backend.clone(),
        ));
        tokio.handle().spawn(async move {
            for join in init_handles {
                let _ = join.await;
            }
        });

        let receiver = bus.receiver().clone();
        let subs_fn = subscriptions;
        let state_for_thread = state.clone();
        let env_for_thread = env.clone();
        let shutdown_for_thread = shutdown.clone();
        let tokio_for_thread = tokio.clone();
        let backend_for_thread = backend.clone();
        let tx_for_thread = tx;
        let subs_registry = sub_handles.clone();

        let thread = thread::Builder::new()
            .name(reducer_thread_name)
            .spawn(move || {
            let rt = tokio_for_thread;
            let sync_subs = |state: &S| {
                let sub = subs_fn(state);
                crate::subscription::sync_subscriptions(
                    &sub,
                    &subs_registry,
                    rt.handle().clone(),
                    tx_for_thread.clone(),
                    backend_for_thread.clone(),
                    shutdown_for_thread.clone(),
                );
            };

            sync_subs(&state_for_thread.lock());

            while !shutdown_for_thread.load(Ordering::Relaxed) {
                match receiver.recv_timeout(Duration::from_millis(50)) {
                    Ok(msg) => {
                        let mut unwind = StoreWorkUnwindGuard::new(&backend_for_thread);
                        let cmd = {
                            let mut guard = state_for_thread.lock();
                            match catch_reduce_panic(&mut *guard, |s, a| update(s, a), msg) {
                                Ok(cmd) => cmd,
                                Err(_) => {
                                    drop(guard);
                                    backend_for_thread.notify_state();
                                    continue;
                                }
                            }
                        };
                        unwind.disarm();
                        backend_for_thread.notify_state();
                        let handles = rt.block_on(interpret_effects_async(
                            cmd.into_effects(),
                            tx_for_thread.clone(),
                            env_for_thread.clone(),
                            rt.handle().clone(),
                            backend_for_thread.clone(),
                        ));
                        if handles.is_empty() {
                            backend_for_thread.end_store_work();
                        } else {
                            let backend_wait = backend_for_thread.clone();
                            rt.handle().spawn(async move {
                                for join in handles {
                                    let _ = join.await;
                                }
                                backend_wait.end_store_work();
                            });
                        }

                        sync_subs(&state_for_thread.lock());
                    }
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .expect("failed to spawn rust-elm reducer thread");

        let runner = Arc::new(AppRunner::new(shutdown, thread, sub_handles));

        Self {
            state,
            bus,
            env,
            backend,
            runner,
            _tokio: tokio,
        }
    }

    pub fn store(&self) -> crate::Store<S, M>
    where
        S: Clone + Send + Sync + 'static,
    {
        self.backend.store()
    }

    pub fn dispatch(&self, msg: M) {
        let _ = self.bus.sender().send_blocking(msg);
    }

    pub fn sender(&self) -> BusSender<M> {
        self.bus.sender()
    }

    pub fn cancel(&self, id: EffectId) {
        if let Some(handle) = self.backend.interpreter.cancel_tokens.lock().remove(&id) {
            handle.abort();
        }
    }

    pub fn shutdown(self) {
        self.runner.shutdown();
    }
}

fn dispatch_from_effect<S, M>(
    backend: &StoreBackend<S, M>,
    tx: &BusSender<M>,
    msg: M,
) where
    S: Send + 'static,
    M: Send + 'static,
{
    backend.begin_store_work();
    let _ = tx.send_blocking(msg);
}

pub(crate) fn dispatch_from_subscription<S, M>(
    backend: &StoreBackend<S, M>,
    tx: &BusSender<M>,
    msg: M,
) where
    S: Send + 'static,
    M: Send + 'static,
{
    dispatch_from_effect(backend, tx, msg);
}

async fn interpret_effects_async<S, M>(
    effects: Vec<Effect<M>>,
    tx: BusSender<M>,
    env: Environment,
    handle: tokio::runtime::Handle,
    backend: StoreBackend<S, M>,
) -> Vec<TokioJoinHandle<()>>
where
    S: Send + 'static,
    M: Send + 'static,
{
    let interpreter = backend.interpreter.clone();
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

fn spawn_effect<S, M>(
    effect: Effect<M>,
    tx: BusSender<M>,
    env: Environment,
    handle: tokio::runtime::Handle,
    interpreter: Arc<InterpreterState<M>>,
    backend: StoreBackend<S, M>,
) -> Vec<TokioJoinHandle<()>>
where
    S: Send + 'static,
    M: Send + 'static,
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

fn spawn_effect_inner<S, M>(
    effect: Effect<M>,
    tx: BusSender<M>,
    env: Environment,
    handle: tokio::runtime::Handle,
    interpreter: Arc<InterpreterState<M>>,
    backend: StoreBackend<S, M>,
    batch: &mut Vec<TokioJoinHandle<()>>,
) where
    S: Send + 'static,
    M: Send + 'static,
{
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
    run_leaf(effect, &env).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;
    use crate::panic_on_state_clone;

    panic_on_state_clone! {
        #[derive(Default)]
        struct Counter {
            n: i32,
        }
    }

    fn init() -> (Counter, Cmd<i32>) {
        (Counter::default(), Cmd::none())
    }

    fn update(s: &mut Counter, msg: i32) -> Cmd<i32> {
        s.n += msg;
        Cmd::none()
    }

    fn subs(_: &Counter) -> Sub<i32> {
        Sub::none()
    }

    #[test]
    fn runtime_tick_subscription_dispatches() {
        fn subs(_: &Counter) -> Sub<i32> {
            Sub::tick(7, Duration::from_millis(30), || 1)
        }

        let program = Program::new(init, update, subs);
        let runtime = Runtime::from_program(program, Environment::new(), RuntimeConfig::new(16));
        std::thread::sleep(Duration::from_millis(200));
        assert!(runtime.state.lock().n >= 1);
        runtime.shutdown();
    }

    #[test]
    fn runtime_applies_dispatched_messages() {
        let program = Program::new(init, update, subs);
        let runtime = Runtime::from_program(program, Environment::new(), RuntimeConfig::new(16));
        runtime.dispatch(3);
        runtime.dispatch(4);
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(runtime.state.lock().n, 7);
        runtime.shutdown();
    }

    #[test]
    fn runtime_runs_task_effects() {
        fn update_with_effect(s: &mut Counter, msg: i32) -> Cmd<i32> {
            if msg == 0 {
                Cmd::single(Effect::task(1, || Box::pin(async { Ok(10) })))
            } else {
                s.n += msg;
                Cmd::none()
            }
        }
        let program = Program::new(init, update_with_effect, subs);
        let runtime = Runtime::from_program(program, Environment::new(), RuntimeConfig::new(16));
        runtime.dispatch(0);
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(runtime.state.lock().n, 10);
        runtime.shutdown();
    }

    #[test]
    fn runtime_host_spawns_multiple_programs_on_shared_tokio() {
        let host = RuntimeHost::new(Environment::new(), RuntimeConfig::new(16));
        let a = host.spawn_program(Program::new(init, update, subs));
        let b = host.spawn_program(Program::new(init, update, subs));
        a.dispatch(2);
        b.dispatch(5);
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(a.state.lock().n, 2);
        assert_eq!(b.state.lock().n, 5);
        assert_eq!(host.app_count(), 2);
        host.shutdown();
    }

    #[test]
    fn runtime_catches_reduce_panic_and_unwinds_store_work() {
        fn panicking_update(s: &mut Counter, msg: i32) -> Cmd<i32> {
            s.n = 99;
            if msg < 0 {
                panic!("reduce panic");
            }
            s.n = msg;
            Cmd::none()
        }

        let program = Program::new(init, panicking_update, subs);
        let runtime = Runtime::from_program(program, Environment::new(), RuntimeConfig::new(16));
        let store = runtime.store();
        let task = store.send(-1);
        assert!(task.finish().is_ok());
        assert_eq!(runtime.state.lock().n, 99);

        runtime.dispatch(4);
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(runtime.state.lock().n, 4);
        runtime.shutdown();
    }
}
