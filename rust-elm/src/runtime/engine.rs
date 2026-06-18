use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::RecvTimeoutError;
use parking_lot::Mutex;
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
use super::store::{StoreBackend, StoreWork, StoreWorkUnwindGuard};
use super::subscription;

/// Live runtime — bus-driven update loop on a pinned thread with Tokio effect interpreter.
pub struct Runtime<S, M> {
    pub state: Arc<Mutex<S>>,
    pub bus: Bus<M>,
    pub env: Environment,
    backend: StoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
    _thread: Option<JoinHandle<()>>,
    _tokio: Arc<TokioRuntime>,
    _subscriptions: Arc<subscription::SubscriptionHandles>,
}

impl<S, M> Runtime<S, M>
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
        let state = Arc::new(Mutex::new(initial_state));
        let bus = Bus::new(bus_capacity);
        let shutdown = Arc::new(AtomicBool::new(false));
        let tokio = build_tokio(worker_threads, thread_name)?;
        let interpreter = InterpreterState::new();
        let sub_handles = Arc::new(subscription::SubscriptionHandles::new());
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

                sync_subs(&state_for_thread.lock());

                while !shutdown_for_thread.load(Ordering::Relaxed) {
                    match receiver.recv_timeout(Duration::from_millis(50)) {
                        Ok(msg) => {
                            let mut unwind = StoreWorkUnwindGuard::new(&backend_for_thread);
                            let cmd = {
                                let mut guard = state_for_thread.lock();
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

                            sync_subs(&state_for_thread.lock());
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
        if let Some(handle) = self.backend.hub.interpreter.cancel_tokens.lock().remove(&id) {
            handle.abort();
        }
    }

    pub fn shutdown(mut self) {
        super::lifecycle::join_reducer_thread(&self.shutdown, &self._subscriptions, &mut self._thread);
    }
}

impl<S, M> Drop for Runtime<S, M> {
    fn drop(&mut self) {
        super::lifecycle::join_reducer_thread(&self.shutdown, &self._subscriptions, &mut self._thread);
    }
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

    fn boot(program: Program<Counter, i32>) -> Runtime<Counter, i32> {
        match Runtime::from_program(program, Environment::new(), RuntimeConfig::new(16)) {
            Ok(runtime) => runtime,
            Err(err) => panic!("test runtime bootstrap failed: {err}"),
        }
    }

    #[test]
    fn runtime_tick_subscription_dispatches() {
        fn subs(_: &Counter) -> Sub<i32> {
            Sub::tick(7, Duration::from_millis(30), || 1)
        }

        let program = Program::new(init, update, subs);
        let runtime = boot(program);
        std::thread::sleep(Duration::from_millis(200));
        assert!(runtime.state.lock().n >= 1);
        runtime.shutdown();
    }

    #[test]
    fn runtime_applies_dispatched_messages() {
        let program = Program::new(init, update, subs);
        let runtime = boot(program);
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
        let runtime = boot(program);
        runtime.dispatch(0);
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(runtime.state.lock().n, 10);
        runtime.shutdown();
    }

    #[test]
    fn runtime_safe_reduce_update_unwinds_store_work() {
        fn panicking_update(s: &mut Counter, msg: i32) -> Cmd<i32> {
            s.n = 99;
            if msg < 0 {
                panic!("reduce panic");
            }
            s.n = msg;
            Cmd::none()
        }

        let program = Program::new(init, panicking_update, subs);
        let runtime = boot(program);
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
