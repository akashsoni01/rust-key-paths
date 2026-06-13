use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

/// Test double clock for deterministic effect tests.
#[derive(Debug, Clone)]
pub struct FakeClock {
    inner: std::sync::Arc<Mutex<Instant>>,
}

impl FakeClock {
    pub fn new(start: Instant) -> Self {
        Self {
            inner: std::sync::Arc::new(Mutex::new(start)),
        }
    }

    pub fn now(&self) -> Instant {
        *self.inner.lock()
    }

    pub fn advance(&self, duration: Duration) {
        *self.inner.lock() += duration;
    }
}

/// Minimal HTTP mock for env-backed effects.
#[derive(Debug, Clone, Default)]
pub struct MockHttp {
    responses: std::sync::Arc<Mutex<Vec<(String, String)>>>,
}

impl MockHttp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stub(&self, url: impl Into<String>, body: impl Into<String>) {
        self.responses
            .lock()
            .push((url.into(), body.into()));
    }

    pub fn get(&self, url: &str) -> Option<String> {
        self.responses
            .lock()
            .iter()
            .find(|(u, _)| u == url)
            .map(|(_, b)| b.clone())
    }
}

/// Layered environment stack for effect interpretation.
#[derive(Debug, Clone, Default)]
pub struct Environment {
    clock: Option<FakeClock>,
    http: Option<MockHttp>,
    locals: std::sync::Arc<Mutex<Vec<EnvironmentLayer>>>,
}

#[derive(Debug, Clone)]
enum EnvironmentLayer {
    Clock(FakeClock),
    Http(MockHttp),
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_clock(mut self, clock: FakeClock) -> Self {
        self.clock = Some(clock.clone());
        self.locals.lock().push(EnvironmentLayer::Clock(clock));
        self
    }

    pub fn with_http(mut self, http: MockHttp) -> Self {
        self.http = Some(http.clone());
        self.locals.lock().push(EnvironmentLayer::Http(http));
        self
    }

    pub fn clock(&self) -> Option<FakeClock> {
        self.clock.clone()
    }

    pub fn http(&self) -> Option<MockHttp> {
        self.http.clone()
    }

    pub fn push_layer(&self, layer: Environment) {
        let mut guard = self.locals.lock();
        if let Some(c) = layer.clock {
            guard.push(EnvironmentLayer::Clock(c));
        }
        if let Some(h) = layer.http {
            guard.push(EnvironmentLayer::Http(h));
        }
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
}
