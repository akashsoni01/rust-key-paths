use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::RecvTimeoutError;
use parking_lot::RwLock;
use tokio::runtime::Runtime as TokioRuntime;

use crate::bus::{Bus, BusSender};
use crate::cmd::Cmd;
use crate::effect::EffectId;
use crate::env::Environment;
use crate::program::{Program, ReducerProgram};
use crate::reducer::Reducer;
use crate::safe_reducer::safe_reduce_update;
use crate::sub::Sub;

use super::config::RuntimeConfig;
use super::error::{build_tokio, RuntimeError};
use super::interpreter::{interpret_effects_async, InterpreterState};
use super::rw_store::{RwStore, RwStoreBackend};
use super::store::{StoreHub, StoreWork, StoreWorkUnwindGuard};
use super::subscription;

/// Live runtime with [`RwLock`] state — many concurrent readers, exclusive writer on reducer thread.
pub struct RwRuntime<S, M> {
    pub state: Arc<RwLock<S>>,
    pub bus: Bus<M>,
    pub env: Environment,
    backend: RwStoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
    _thread: Option<JoinHandle<()>>,
    _tokio: Arc<TokioRuntime>,
    _subscriptions: Arc<subscription::SubscriptionHandles>,
}

impl<S, M> RwRuntime<S, M>
where
    S: Send + Sync + 'static,
    M: Send + 'static,
{
    pub fn from_program(
        program: Program<S, M>,
        env: Environment,
        config: RuntimeConfig,
    ) -> Result<Self, RuntimeError> {
        Self::bootstrap(
            program.init,
            program.update,
            program.subscriptions,
            env,
            config,
        )
    }

    pub fn from_reducer_program<R>(
        program: ReducerProgram<R>,
        env: Environment,
        config: RuntimeConfig,
    ) -> Result<Self, RuntimeError>
    where
        R: Reducer<State = S, Action = M> + Send + Sync + 'static,
    {
        let reducer = Arc::new(program.reducer);
        let init = program.init;
        let subscriptions = program.subscriptions;
        Self::bootstrap(
            init,
            move |state, msg| reducer.reduce(state, msg),
            subscriptions,
            env,
            config,
        )
    }

    fn bootstrap(
        init: fn() -> (S, Cmd<M>),
        update: impl Fn(&mut S, M) -> Cmd<M> + Send + Sync + 'static,
        subscriptions: fn(&S) -> Sub<M>,
        env: Environment,
        config: RuntimeConfig,
    ) -> Result<Self, RuntimeError> {
        let RuntimeConfig {
            bus_capacity,
            worker_threads,
            thread_name,
        } = config;
        let (initial_state, init_cmd) = init();
        let state = Arc::new(RwLock::new(initial_state));
        let bus = Bus::new(bus_capacity);
        let shutdown = Arc::new(AtomicBool::new(false));
        let tokio = build_tokio(worker_threads, thread_name)?;
        let interpreter = InterpreterState::new();
        let sub_handles = Arc::new(subscription::SubscriptionHandles::new());
        let hub = StoreHub::new(bus.sender(), interpreter);
        let backend = RwStoreBackend::new(state.clone(), hub);
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
            .name(thread_name.to_string())
            .spawn(move || {
                let rt = tokio_for_thread;
                let sync_subs = |state: &S| {
                    let sub = subs_fn(state);
                    subscription::sync_subscriptions(
                        &sub,
                        &subs_registry,
                        rt.handle().clone(),
                        tx_for_thread.clone(),
                        backend_for_thread.clone(),
                        shutdown_for_thread.clone(),
                    );
                };

                sync_subs(&state_for_thread.read());

                while !shutdown_for_thread.load(Ordering::Relaxed) {
                    match receiver.recv_timeout(Duration::from_millis(50)) {
                        Ok(msg) => {
                            let mut unwind = StoreWorkUnwindGuard::new(&backend_for_thread);
                            let cmd = {
                                let mut guard = state_for_thread.write();
                                match safe_reduce_update(&mut *guard, |s, a| update(s, a), msg) {
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

                            sync_subs(&state_for_thread.read());
                        }
                        Err(RecvTimeoutError::Timeout) => continue,
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .map_err(RuntimeError::ThreadSpawn)?;

        Ok(Self {
            state,
            bus,
            env,
            backend,
            shutdown,
            _thread: Some(thread),
            _tokio: tokio,
            _subscriptions: sub_handles,
        })
    }

    pub fn rw_store(&self) -> RwStore<S, M>
    where
        S: Clone + Send + Sync + 'static,
    {
        self.backend.rw_store()
    }

    pub fn dispatch(&self, msg: M) {
        let _ = self.bus.sender().send_blocking(msg);
    }

    pub fn sender(&self) -> BusSender<M> {
        self.bus.sender()
    }

    pub fn cancel(&self, id: EffectId) {
        if let Some(handle) = self.backend.hub.interpreter.cancel_tokens.lock().remove(&id) {
            handle.abort();
        }
    }

    pub fn shutdown(mut self) {
        super::lifecycle::join_reducer_thread(&self.shutdown, &self._subscriptions, &mut self._thread);
    }
}

impl<S, M> Drop for RwRuntime<S, M> {
    fn drop(&mut self) {
        super::lifecycle::join_reducer_thread(&self.shutdown, &self._subscriptions, &mut self._thread);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn boot(program: Program<Counter, i32>) -> RwRuntime<Counter, i32> {
        match RwRuntime::from_program(program, Environment::new(), RuntimeConfig::new(16)) {
            Ok(runtime) => runtime,
            Err(err) => panic!("test runtime bootstrap failed: {err}"),
        }
    }

    #[test]
    fn rw_runtime_applies_dispatched_messages() {
        let runtime = boot(Program::new(init, update, subs));
        runtime.dispatch(3);
        runtime.dispatch(4);
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(runtime.state.read().n, 7);
        runtime.shutdown();
    }

    #[test]
    fn read_store_allows_concurrent_reads() {
        let runtime = boot(Program::new(init, update, subs));
        let store = runtime.rw_store();
        store.dispatch(5);
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(store.read_store().with_read(|s| s.n), 5);
        runtime.shutdown();
    }
}
