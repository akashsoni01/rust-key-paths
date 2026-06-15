use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::{Mutex, RwLock};

use rust_dependencies::{Clock, ClockDep, DependencyError, DependencyValues};

pub use rust_dependencies::{
    ClockKey, DepRng, DependencyKey, LiveRng, LiveUuidGen, Now, NowDep, NowKey, RealClock,
    RngDep, RngKey, SeededRng, SeededUuidGen, TestNow, UuidGen, UuidKey,
};

/// Test double clock for deterministic effect tests.
#[derive(Debug, Clone)]
pub struct FakeClock {
    inner: Arc<Mutex<Instant>>,
}

impl FakeClock {
    pub fn new(start: Instant) -> Self {
        Self {
            inner: Arc::new(Mutex::new(start)),
        }
    }

    pub fn now(&self) -> Instant {
        *self.inner.lock()
    }

    pub fn advance(&self, duration: Duration) {
        *self.inner.lock() += duration;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        FakeClock::now(self)
    }
}

/// Minimal HTTP mock for env-backed effects.
#[derive(Debug, Clone, Default)]
pub struct MockHttp {
    responses: Arc<Mutex<Vec<(String, String)>>>,
}

impl MockHttp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stub(&self, url: impl Into<String>, body: impl Into<String>) {
        self.responses.lock().push((url.into(), body.into()));
    }

    pub fn get(&self, url: &str) -> Option<String> {
        self.responses
            .lock()
            .iter()
            .find(|(u, _)| u == url)
            .map(|(_, b)| b.clone())
    }
}

/// Layered environment with typed dependencies (UDF `DependencyValues` + scoped overrides).
#[derive(Clone)]
pub struct Environment {
    layers: Arc<Mutex<Vec<DependencyValues>>>,
    merged_cache: Arc<RwLock<Option<DependencyValues>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::live()
    }
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Environment")
            .field("layers", &self.layers.lock().len())
            .finish()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self::live()
    }

    pub fn live() -> Self {
        Self::from_values(DependencyValues::live())
    }

    pub fn test() -> Self {
        Self::from_values(DependencyValues::test())
    }

    pub fn from_values(base: DependencyValues) -> Self {
        Self {
            layers: Arc::new(Mutex::new(vec![base])),
            merged_cache: Arc::new(RwLock::new(None)),
        }
    }

    fn invalidate_merged_cache(&self) {
        *self.merged_cache.write() = None;
    }

    pub fn values(&self) -> DependencyValues {
        if let Some(cached) = self.merged_cache.read().clone() {
            return cached;
        }
        let layers = self.layers.lock();
        let merged = DependencyValues::new();
        for layer in layers.iter() {
            merged.merge_from(layer);
        }
        *self.merged_cache.write() = Some(merged.clone());
        merged
    }

    pub fn get<D: Send + Sync + 'static>(&self) -> Option<Arc<D>> {
        let layers = self.layers.lock();
        for layer in layers.iter().rev() {
            if let Some(value) = layer.get::<D>() {
                return Some(value);
            }
        }
        None
    }

    pub fn require<D: Send + Sync + 'static>(&self) -> Result<Arc<D>, DependencyError> {
        self.get::<D>()
            .ok_or_else(|| DependencyError::missing(std::any::type_name::<D>()))
    }

    pub fn with<D: Send + Sync + 'static>(self, value: D) -> Self {
        let overlay = DependencyValues::new();
        overlay.insert(value);
        self.push_values(overlay);
        self
    }

    pub fn push_values(&self, overlay: DependencyValues) {
        self.layers.lock().push(overlay);
        self.invalidate_merged_cache();
    }

    /// Fork the environment and append overlay dependency layers (for `Effect::provide`).
    pub fn scoped_with(&self, overlay: Environment) -> Self {
        let mut layers = self.layers.lock().clone();
        layers.extend(overlay.layers.lock().iter().cloned());
        Self {
            layers: Arc::new(Mutex::new(layers)),
            merged_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_clock(self, clock: FakeClock) -> Self {
        self.with(clock.clone()).with(ClockDep(Arc::new(clock)))
    }

    pub fn with_http(self, http: MockHttp) -> Self {
        self.with(http)
    }

    pub fn clock(&self) -> Option<FakeClock> {
        self.get::<FakeClock>().map(|c| c.as_ref().clone())
    }

    pub fn http(&self) -> Option<MockHttp> {
        self.get::<MockHttp>().map(|arc| (*arc).clone())
    }

    /// Back-compat alias for [`Self::scoped_with`].
    pub fn push_layer(&self, overlay: Environment) {
        self.layers
            .lock()
            .extend(overlay.layers.lock().iter().cloned());
        self.invalidate_merged_cache();
    }
}

thread_local! {
    static BATCH_QUEUE: RefCell<Vec<Box<dyn FnOnce() + 'static>>> = RefCell::new(Vec::new());
}

/// Fiber-local batching for coalescing synchronous dispatches within one update tick.
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    BATCH_QUEUE.with(|q| q.borrow_mut().clear());
    let result = f();
    BATCH_QUEUE.with(|q| {
        let mut queue = q.borrow_mut();
        while let Some(job) = queue.pop() {
            job();
        }
    });
    result
}

pub fn defer_batch(job: impl FnOnce() + 'static) {
    BATCH_QUEUE.with(|q| q.borrow_mut().push(Box::new(job)));
}

pub type EnvFuture<M> = Pin<Box<dyn Future<Output = Result<M, crate::error::EffectError>> + Send>>;

#[cfg(test)]
mod tests {
    use super::*;
    use rust_dependencies::{RngDep, SeededUuidGen, UuidDep};

    #[test]
    fn fake_clock_advances() {
        let start = Instant::now();
        let clock = FakeClock::new(start);
        clock.advance(Duration::from_millis(100));
        assert_eq!(clock.now(), start + Duration::from_millis(100));
    }

    #[test]
    fn mock_http_stubs_responses() {
        let http = MockHttp::new();
        http.stub("https://example.com", "ok");
        assert_eq!(http.get("https://example.com"), Some("ok".into()));
    }

    #[test]
    fn test_env_is_deterministic() {
        let env = Environment::test();
        let uuid1 = env.require::<UuidDep>().unwrap().0.next();
        let rng1 = env.require::<RngDep>().unwrap().0.next_u64();
        let env2 = Environment::test();
        let uuid2 = env2.require::<UuidDep>().unwrap().0.next();
        let rng2 = env2.require::<RngDep>().unwrap().0.next_u64();
        assert_eq!(uuid1, uuid2);
        assert_eq!(rng1, rng2);
    }

    #[test]
    fn scoped_override_replaces_dependency() {
        let env = Environment::test();
        let base = env.require::<UuidDep>().unwrap().0.next();
        let scoped = env.scoped_with(Environment::new().with(UuidDep(Arc::new(SeededUuidGen::new(
            99,
        )))));
        let overridden = scoped.require::<UuidDep>().unwrap().0.next();
        assert_ne!(base, overridden);
        assert_eq!(overridden, SeededUuidGen::new(99).next());
    }
}
