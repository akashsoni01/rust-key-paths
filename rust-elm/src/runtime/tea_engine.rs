use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
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
use super::store::{StoreHub, StoreWork, StoreWorkUnwindGuard};
use super::subscription;
use super::tea_store::{TeaControl, TeaStore, TeaStoreBackend};

/// Live runtime with **channel-delivered model** — true Elm Architecture.
///
/// `S` is owned exclusively by the reducer thread. After each `update`, the runtime
/// pushes `Arc<S>` to subscribers. Readers never lock shared state.
pub struct TeaRuntime<S, M> {
    pub bus: Bus<M>,
    pub env: Environment,
    backend: TeaStoreBackend<S, M>,
    shutdown: Arc<AtomicBool>,
    _thread: Option<JoinHandle<()>>,
    _tokio: Arc<TokioRuntime>,
    _subscriptions: Arc<subscription::SubscriptionHandles>,
}

impl<S, M> TeaRuntime<S, M>
where
    S: Send + Sync + Clone + 'static,
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
        let (mut state, init_cmd) = init();
        let bus = Bus::new(bus_capacity);
        let shutdown = Arc::new(AtomicBool::new(false));
        let tokio = build_tokio(worker_threads, thread_name)?;
        let interpreter = InterpreterState::new();
        let sub_handles = Arc::new(subscription::SubscriptionHandles::new());
        let hub = StoreHub::new(bus.sender(), interpreter);
        let (control_tx, control_rx) = crossbeam_channel::unbounded();
        let backend = TeaStoreBackend::new(hub, control_tx);
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

        let action_rx = bus.receiver().clone();
        let subs_fn = subscriptions;
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
                let mut snapshot_subscribers: Vec<Sender<Arc<S>>> = Vec::new();

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

                sync_subs(&state);

                while !shutdown_for_thread.load(Ordering::Relaxed) {
                    if let Some(msg) = recv_action_or_control(
                        &action_rx,
                        &control_rx,
                        Duration::from_millis(50),
                    ) {
                        match msg {
                            LoopMsg::Action(msg) => {
                                let mut unwind =
                                    StoreWorkUnwindGuard::new(&backend_for_thread);
                                let cmd = match safe_reduce_update(
                                    &mut state,
                                    |s, a| update(s, a),
                                    msg,
                                ) {
                                    Ok(cmd) => cmd,
                                    Err(_) => {
                                        publish_snapshots(&state, &mut snapshot_subscribers);
                                        continue;
                                    }
                                };
                                publish_snapshots(&state, &mut snapshot_subscribers);
                                unwind.disarm();
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

                                sync_subs(&state);
                            }
                            LoopMsg::Control(TeaControl::Subscribe {
                                snapshot_tx,
                                ready_tx,
                            }) => {
                                let snap = Arc::new(state.clone());
                                let _ = snapshot_tx.send(snap);
                                snapshot_subscribers.push(snapshot_tx);
                                let _ = ready_tx.send(());
                            }
                            LoopMsg::Control(TeaControl::RequestSnapshot { reply }) => {
                                let _ = reply.send(Arc::new(state.clone()));
                            }
                        }
                    }
                }
            })
            .map_err(RuntimeError::ThreadSpawn)?;

        Ok(Self {
            bus,
            env,
            backend,
            shutdown,
            _thread: Some(thread),
            _tokio: tokio,
            _subscriptions: sub_handles,
        })
    }

    pub fn tea_store(&self) -> TeaStore<S, M> {
        self.backend.tea_store()
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

impl<S, M> Drop for TeaRuntime<S, M> {
    fn drop(&mut self) {
        super::lifecycle::join_reducer_thread(&self.shutdown, &self._subscriptions, &mut self._thread);
    }
}

enum LoopMsg<S, M> {
    Action(M),
    Control(TeaControl<S>),
}

fn recv_action_or_control<S, M>(
    action_rx: &Receiver<M>,
    control_rx: &Receiver<TeaControl<S>>,
    timeout: Duration,
) -> Option<LoopMsg<S, M>>
where
    S: Send + 'static,
    M: Send + 'static,
{
    crossbeam_channel::select! {
        recv(action_rx) -> msg => msg.ok().map(LoopMsg::Action),
        recv(control_rx) -> ctrl => ctrl.ok().map(LoopMsg::Control),
        default(timeout) => None,
    }
}

fn publish_snapshots<S>(state: &S, subscribers: &mut Vec<Sender<Arc<S>>>)
where
    S: Clone,
{
    let snap = Arc::new(state.clone());
    subscribers.retain(|tx| tx.send(Arc::clone(&snap)).is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Clone, PartialEq)]
    struct Counter {
        n: i32,
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

    fn boot(program: Program<Counter, i32>) -> TeaRuntime<Counter, i32> {
        match TeaRuntime::from_program(program, Environment::new(), RuntimeConfig::new(16)) {
            Ok(runtime) => runtime,
            Err(err) => panic!("test runtime bootstrap failed: {err}"),
        }
    }

    #[test]
    fn tea_runtime_dispatches_and_pushes_snapshots() {
        let runtime = boot(Program::new(init, update, subs));
        let store = runtime.tea_store();
        let Ok(mut sub) = store.subscribe_state() else {
            panic!("expected subscribe_state to succeed in test");
        };
        store.dispatch(3);
        store.dispatch(4);
        std::thread::sleep(Duration::from_millis(200));
        while sub.next().is_some() {}
        assert_eq!(sub.latest().map(|s| s.n), Some(7));
    }

    #[test]
    fn tea_view_store_requests_snapshot() {
        let runtime = boot(Program::new(init, update, subs));
        let store = runtime.tea_store();
        store.dispatch(5);
        std::thread::sleep(Duration::from_millis(100));
        let n = store
            .view_store()
            .try_with_snapshot(|s| s.n)
            .map_err(|e| format!("{e}"))
            .ok();
        assert_eq!(n, Some(5));
        runtime.shutdown();
    }
}
