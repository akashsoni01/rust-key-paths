use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;

/// Error when a required dependency is absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyError {
    pub name: &'static str,
}

impl DependencyError {
    pub const fn missing(name: &'static str) -> Self {
        Self { name }
    }
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "missing dependency: {}", self.name)
    }
}

impl std::error::Error for DependencyError {}

/// Typed dependency bag keyed by `TypeId` (TCA `DependencyValues`).
#[derive(Clone, Default)]
pub struct DependencyValues {
    entries: Arc<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl DependencyValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<D: Send + Sync + 'static>(&self, value: D) {
        self.entries
            .lock()
            .insert(TypeId::of::<D>(), Arc::new(value));
    }

    pub fn get<D: Send + Sync + 'static>(&self) -> Option<Arc<D>> {
        self.entries
            .lock()
            .get(&TypeId::of::<D>())
            .and_then(|value| value.clone().downcast::<D>().ok())
    }

    pub fn require<D: Send + Sync + 'static>(&self) -> Result<Arc<D>, DependencyError> {
        self.get::<D>()
            .ok_or_else(|| DependencyError::missing(std::any::type_name::<D>()))
    }

    pub fn with<D: Send + Sync + 'static>(self, value: D) -> Self {
        self.insert(value);
        self
    }

    pub fn merge_from(&self, overlay: &DependencyValues) {
        // Snapshot the overlay before locking `self` so we never hold both
        // locks simultaneously (avoids lock-order deadlocks between two bags).
        let snapshot: Vec<(TypeId, Arc<dyn Any + Send + Sync>)> = {
            let overlay_guard = overlay.entries.lock();
            overlay_guard
                .iter()
                .map(|(id, value)| (*id, value.clone()))
                .collect()
        };
        let mut guard = self.entries.lock();
        for (id, value) in snapshot {
            guard.insert(id, value);
        }
    }

    /// Live dependency set for production runtimes.
    pub fn live() -> Self {
        let values = Self::new();
        ClockKey::register(&values, ClockKey::live());
        UuidKey::register(&values, UuidKey::live());
        NowKey::register(&values, NowKey::live());
        RngKey::register(&values, RngKey::live());
        values
    }

    /// Test dependency set with deterministic built-ins.
    pub fn test() -> Self {
        let values = Self::new();
        let clock = TestClock::new(Instant::now());
        let epoch_system =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        values.insert(ClockDep(Arc::new(clock.clone())));
        values.insert(NowDep(Arc::new(TestNow::new(clock, epoch_system))));
        UuidKey::register(&values, UuidKey::test());
        RngKey::register(&values, RngKey::test());
        values
    }
}

/// Register a dependency value under its key type (TCA `DependencyKey`).
pub trait DependencyKey: Send + Sync + 'static {
    type Value: Send + Sync + 'static;

    fn live() -> Self::Value;
    fn test() -> Self::Value {
        panic!(
            "missing test value for dependency `{}` — override `DependencyKey::test`",
            std::any::type_name::<Self>()
        );
    }
    fn preview() -> Self::Value {
        Self::live()
    }

    fn register(values: &DependencyValues, value: Self::Value) {
        values.insert(value);
    }
}

// --- Clock ---

pub struct ClockKey;

pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Deterministic clock for tests (also available as [`crate::FakeClock`]).
#[derive(Debug, Clone)]
pub struct TestClock {
    inner: Arc<Mutex<Instant>>,
}

impl TestClock {
    pub fn new(start: Instant) -> Self {
        Self {
            inner: Arc::new(Mutex::new(start)),
        }
    }

    pub fn now(&self) -> Instant {
        *self.inner.lock()
    }

    pub fn advance(&self, duration: std::time::Duration) {
        *self.inner.lock() += duration;
    }
}

impl Clock for TestClock {
    fn now(&self) -> Instant {
        TestClock::now(self)
    }
}

#[derive(Clone)]
pub struct ClockDep(pub Arc<dyn Clock>);

impl DependencyKey for ClockKey {
    type Value = ClockDep;

    fn live() -> Self::Value {
        ClockDep(Arc::new(RealClock))
    }

    fn test() -> Self::Value {
        ClockDep(Arc::new(TestClock::new(Instant::now())))
    }
}

// --- Uuid ---

pub struct UuidKey;

pub trait UuidGen: Send + Sync {
    fn next(&self) -> u128;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LiveUuidGen;

impl UuidGen for LiveUuidGen {
    fn next(&self) -> u128 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u128)
            .unwrap_or(0);
        nanos ^ ((std::process::id() as u128) << 32)
    }
}

#[derive(Debug)]
pub struct SeededUuidGen {
    next: AtomicU64,
}

impl SeededUuidGen {
    pub fn new(seed: u64) -> Self {
        Self {
            next: AtomicU64::new(seed),
        }
    }
}

impl UuidGen for SeededUuidGen {
    fn next(&self) -> u128 {
        let n = self.next.fetch_add(1, Ordering::SeqCst);
        ((n as u128) << 64) | (n as u128 ^ 0xDEAD_BEEF_CAFE)
    }
}

#[derive(Clone)]
pub struct UuidDep(pub Arc<dyn UuidGen>);

impl DependencyKey for UuidKey {
    type Value = UuidDep;

    fn live() -> Self::Value {
        UuidDep(Arc::new(LiveUuidGen))
    }

    fn test() -> Self::Value {
        UuidDep(Arc::new(SeededUuidGen::new(1)))
    }
}

// --- Now ---

pub struct NowKey;

pub trait Now: Send + Sync {
    fn system_time(&self) -> SystemTime;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LiveNow;

impl Now for LiveNow {
    fn system_time(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[derive(Debug, Clone)]
pub struct TestNow {
    clock: TestClock,
    epoch: Instant,
    epoch_system: SystemTime,
}

impl TestNow {
    pub fn new(clock: TestClock, epoch_system: SystemTime) -> Self {
        let epoch = clock.now();
        Self {
            clock,
            epoch,
            epoch_system,
        }
    }
}

impl Now for TestNow {
    fn system_time(&self) -> SystemTime {
        let elapsed = self.clock.now().duration_since(self.epoch);
        self.epoch_system + elapsed
    }
}

#[derive(Clone)]
pub struct NowDep(pub Arc<dyn Now>);

impl DependencyKey for NowKey {
    type Value = NowDep;

    fn live() -> Self::Value {
        NowDep(Arc::new(LiveNow))
    }

    fn test() -> Self::Value {
        let clock = TestClock::new(Instant::now());
        NowDep(Arc::new(TestNow::new(
            clock,
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000),
        )))
    }
}

// --- Rng ---

pub struct RngKey;

pub trait DepRng: Send + Sync {
    fn next_u64(&self) -> u64;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LiveRng;

impl DepRng for LiveRng {
    fn next_u64(&self) -> u64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        nanos ^ (std::process::id() as u64)
    }
}

#[derive(Debug)]
pub struct SeededRng {
    state: Mutex<u64>,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: Mutex::new(seed.max(1)),
        }
    }
}

impl DepRng for SeededRng {
    fn next_u64(&self) -> u64 {
        // Hold a single guard for the whole read-modify-write so concurrent
        // callers cannot observe the same state and emit duplicate values.
        let mut guard = self.state.lock();
        let mut state = *guard;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *guard = state;
        state
    }
}

#[derive(Clone)]
pub struct RngDep(pub Arc<dyn DepRng>);

impl DependencyKey for RngKey {
    type Value = RngDep;

    fn live() -> Self::Value {
        RngDep(Arc::new(LiveRng))
    }

    fn test() -> Self::Value {
        RngDep(Arc::new(SeededRng::new(0xC0FFEE)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_and_test_uuid_are_deterministic_in_test() {
        let test = DependencyValues::test();
        let a = test.require::<UuidDep>().unwrap().0.next();
        let b = test.require::<UuidDep>().unwrap().0.next();
        let test2 = DependencyValues::test();
        let a2 = test2.require::<UuidDep>().unwrap().0.next();
        assert_eq!(a, a2);
        assert_ne!(a, b);
    }

    #[test]
    fn seeded_rng_is_repeatable() {
        let rng = SeededRng::new(99);
        let first = rng.next_u64();
        let rng2 = SeededRng::new(99);
        assert_eq!(first, rng2.next_u64());
    }

    #[test]
    fn test_clock_advances() {
        let clock = TestClock::new(Instant::now());
        let start = clock.now();
        clock.advance(std::time::Duration::from_secs(10));
        assert_eq!(clock.now(), start + std::time::Duration::from_secs(10));
    }

    #[test]
    fn test_now_tracks_test_clock() {
        let clock = TestClock::new(Instant::now());
        let epoch_system =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        let now = TestNow::new(clock.clone(), epoch_system);
        let before = now.system_time();
        clock.advance(std::time::Duration::from_secs(5));
        assert!(now.system_time() > before);
    }

    #[test]
    #[should_panic(expected = "missing test value")]
    fn missing_test_override_panics() {
        struct UnimplementedKey;
        impl DependencyKey for UnimplementedKey {
            type Value = u32;
            fn live() -> Self::Value {
                1
            }
        }
        let _ = UnimplementedKey::test();
    }

    #[test]
    fn with_overrides_single_dependency() {
        let values = DependencyValues::test().with(UuidDep(Arc::new(SeededUuidGen::new(42))));
        let uuid = values.require::<UuidDep>().unwrap();
        assert_eq!(uuid.0.next(), SeededUuidGen::new(42).next());
    }
}
