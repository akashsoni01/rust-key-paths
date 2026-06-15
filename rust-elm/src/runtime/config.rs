/// Configuration for [`super::engine::Runtime::from_program`] / [`super::engine::Runtime::from_reducer_program`].
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
