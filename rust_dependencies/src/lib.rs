//! Typed dependency injection for Rust — UDF [`DependencyValues`] / [`DependencyKey`] parity.
//!
//! Built-in keys cover monotonic clock, wall time, UUID generation, and RNG with
//! deterministic test doubles. See the `book/dependencies.md` guide in the repository.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;

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

/// Fast hasher for [`TypeId`] keys — avoids SipHash overhead on already-distinct ids.
#[derive(Default)]
struct TypeIdHasher(u64);

impl Hasher for TypeIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut arr = [0u8; 8];
            arr[..chunk.len()].copy_from_slice(chunk);
            self.0 = self.0.wrapping_add(u64::from_le_bytes(arr).wrapping_mul(0x517c_c1b7_2722_0a95));
        }
    }
}

type DependencyMap = HashMap<TypeId, Arc<dyn Any + Send + Sync>, BuildHasherDefault<TypeIdHasher>>;

/// Typed dependency bag keyed by `TypeId` (UDF `DependencyValues`).
///
/// Reads use a shared [`RwLock`] read guard; writes take an exclusive guard.
/// Clone is cheap (shared map).
#[derive(Clone, Default)]
pub struct DependencyValues {
    entries: Arc<RwLock<DependencyMap>>,
}

impl DependencyValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<D: Send + Sync + 'static>(&self, value: D) {
        self.entries
            .write()
            .insert(TypeId::of::<D>(), Arc::new(value));
    }

    pub fn get<D: Send + Sync + 'static>(&self) -> Option<Arc<D>> {
        self.entries
            .read()
            .get(&TypeId::of::<D>())
            .and_then(|value| value.clone().downcast::<D>().ok())
    }

    pub fn contains<D: Send + Sync + 'static>(&self) -> bool {
        self.entries.read().contains_key(&TypeId::of::<D>())
    }

    pub fn remove<D: Send + Sync + 'static>(&self) -> Option<Arc<D>> {
        self.entries
            .write()
            .remove(&TypeId::of::<D>())
            .and_then(|value| value.downcast::<D>().ok())
    }

    pub fn get_or_insert_with<D, F>(&self, f: F) -> Arc<D>
    where
        D: Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        if let Some(value) = self.get::<D>() {
            return value;
        }
        let mut guard = self.entries.write();
        if let Some(value) = guard.get(&TypeId::of::<D>()) {
            return value.clone().downcast::<D>().expect("type id match");
        }
        let value = Arc::new(f());
        guard.insert(TypeId::of::<D>(), value.clone());
        value
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
            let overlay_guard = overlay.entries.read();
            overlay_guard
                .iter()
                .map(|(id, value)| (*id, value.clone()))
                .collect()
        };
        let mut guard = self.entries.write();
        for (id, value) in snapshot {
            guard.insert(id, value);
        }
    }

    /// Live dependency set for production runtimes.
    pub fn live() -> Self {
        let values = Self::new();
        {
            let mut guard = values.entries.write();
            guard.insert(TypeId::of::<ClockDep>(), Arc::new(ClockKey::live()));
            guard.insert(TypeId::of::<UuidDep>(), Arc::new(UuidKey::live()));
            guard.insert(TypeId::of::<NowDep>(), Arc::new(NowKey::live()));
            guard.insert(TypeId::of::<RngDep>(), Arc::new(RngKey::live()));
        }
        values
    }

    /// Test dependency set with deterministic built-ins.
    pub fn test() -> Self {
        let values = Self::new();
        let clock = shared_test_clock();
        let epoch_system = test_epoch_system();
        {
            let mut guard = values.entries.write();
            guard.insert(
                TypeId::of::<ClockDep>(),
                Arc::new(ClockDep(Arc::new(clock.clone()))),
            );
            guard.insert(
                TypeId::of::<NowDep>(),
                Arc::new(NowDep(Arc::new(TestNow::new(clock, epoch_system)))),
            );
            guard.insert(TypeId::of::<UuidDep>(), Arc::new(UuidKey::test()));
            guard.insert(TypeId::of::<RngDep>(), Arc::new(RngKey::test()));
        }
        values
    }
}

/// Register a dependency value under its key type (UDF `DependencyKey`).
///
/// `Value` must be `Send + Sync + 'static` so it can live in the shared dependency
/// bag and be accessed from async effects on any runtime thread, e.g.:
///
/// ```
/// use std::sync::Arc;
/// use rust_dependencies::{DependencyKey, DependencyValues};
///
/// struct ConfigKey;
///
/// #[derive(Clone)]
/// struct Config {
///     api_base: String,
/// }
///
/// impl DependencyKey for ConfigKey {
///     type Value = Config;
///     fn live() -> Self::Value {
///         Config { api_base: "https://api.example.com".into() }
///     }
/// }
///
/// let bag = DependencyValues::live().with(ConfigKey::live());
/// ```
pub trait DependencyKey: Send + Sync + 'static {
    type Value: Send + Sync + 'static;

    fn live() -> Self::Value;

    /// Fall back when no test double is registered for this key.
    fn try_test() -> Result<Self::Value, DependencyError> {
        Err(DependencyError::missing(std::any::type_name::<Self>()))
    }

    fn test() -> Self::Value {
        Self::try_test().unwrap_or_else(|_| {
            panic!(
                "missing test value for dependency `{}` — override `DependencyKey::try_test`",
                std::any::type_name::<Self>()
            )
        })
    }

    fn preview() -> Self::Value {
        Self::live()
    }

    fn register(values: &DependencyValues, value: Self::Value) {
        values.insert(value);
    }
}

fn test_epoch_system() -> SystemTime {
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000)
}

/// Shared [`TestClock`] for standalone `ClockKey::test()` / `NowKey::test()` and
/// [`DependencyValues::test()`] so clock and wall-time deps stay in sync.
fn shared_test_clock() -> TestClock {
    static CLOCK: OnceLock<TestClock> = OnceLock::new();
    CLOCK
        .get_or_init(|| TestClock::new(Instant::now()))
        .clone()
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn mix_entropy() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos ^ ((std::process::id() as u64) << 32)
}

fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
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

/// Deterministic monotonic clock for tests.
#[derive(Debug, Clone)]
pub struct TestClock {
    inner: Arc<parking_lot::Mutex<Instant>>,
}

impl TestClock {
    pub fn new(start: Instant) -> Self {
        Self {
            inner: Arc::new(parking_lot::Mutex::new(start)),
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

    fn try_test() -> Result<Self::Value, DependencyError> {
        Ok(ClockDep(Arc::new(shared_test_clock())))
    }
}

// --- Uuid ---

pub struct UuidKey;

pub trait UuidGen: Send + Sync {
    fn next(&self) -> u128;
}

/// Production UUID stand-in mixing wall time, pid, and a monotonic counter.
#[derive(Debug, Default)]
pub struct LiveUuidGen {
    seq: AtomicU64,
}

impl LiveUuidGen {
    pub fn new() -> Self {
        Self {
            seq: AtomicU64::new(0),
        }
    }
}

impl UuidGen for LiveUuidGen {
    fn next(&self) -> u128 {
        let seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u128)
            .unwrap_or(0);
        let pid = std::process::id() as u128;
        (nanos << 64) ^ (pid << 32) ^ (seq as u128)
    }
}

/// Deterministic UUID stand-in for tests — **not cryptographically random**.
///
/// Values come from a monotonic counter passed through splitmix64 so successive
/// outputs are uncorrelated within a test run.
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
        let hi = splitmix64(n);
        let lo = splitmix64(n.wrapping_add(1));
        ((hi as u128) << 64) | (lo as u128)
    }
}

#[derive(Clone)]
pub struct UuidDep(pub Arc<dyn UuidGen>);

impl DependencyKey for UuidKey {
    type Value = UuidDep;

    fn live() -> Self::Value {
        UuidDep(Arc::new(LiveUuidGen::new()))
    }

    fn try_test() -> Result<Self::Value, DependencyError> {
        Ok(UuidDep(Arc::new(SeededUuidGen::new(1))))
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

    fn try_test() -> Result<Self::Value, DependencyError> {
        Ok(NowDep(Arc::new(TestNow::new(
            shared_test_clock(),
            test_epoch_system(),
        ))))
    }
}

// --- Rng ---

pub struct RngKey;

pub trait DepRng: Send + Sync {
    fn next_u64(&self) -> u64;
}

/// Production RNG backed by a splitmix-seeded xorshift64 state.
#[derive(Debug)]
pub struct LiveRng {
    state: AtomicU64,
}

impl LiveRng {
    pub fn new() -> Self {
        Self {
            state: AtomicU64::new(splitmix64(mix_entropy()).max(1)),
        }
    }
}

impl Default for LiveRng {
    fn default() -> Self {
        Self::new()
    }
}

impl DepRng for LiveRng {
    fn next_u64(&self) -> u64 {
        // `fetch_update` returns the *previous* state; map through xorshift so callers
        // receive the newly computed draw, not the raw internal seed.
        self.state
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |state| {
                Some(xorshift64(state))
            })
            .map(xorshift64)
            .expect("xorshift always produces a value")
    }
}

#[derive(Debug)]
pub struct SeededRng {
    state: parking_lot::Mutex<u64>,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: parking_lot::Mutex::new(seed.max(1)),
        }
    }
}

impl DepRng for SeededRng {
    fn next_u64(&self) -> u64 {
        // Hold a single guard for the whole read-modify-write so concurrent
        // callers cannot observe the same state and emit duplicate values.
        let mut guard = self.state.lock();
        let mut state = *guard;
        state = xorshift64(state);
        *guard = state;
        state
    }
}

#[derive(Clone)]
pub struct RngDep(pub Arc<dyn DepRng>);

impl DependencyKey for RngKey {
    type Value = RngDep;

    fn live() -> Self::Value {
        RngDep(Arc::new(LiveRng::new()))
    }

    fn try_test() -> Result<Self::Value, DependencyError> {
        Ok(RngDep(Arc::new(SeededRng::new(0xC0FFEE))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Barrier;
    use std::thread;

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
    fn seeded_rng_concurrent_unique_values() {
        let rng = Arc::new(SeededRng::new(12345));
        const THREADS: usize = 8;
        const PER_THREAD: usize = 1_000;
        let barrier = Arc::new(Barrier::new(THREADS));
        let results = Arc::new(parking_lot::Mutex::new(Vec::with_capacity(
            THREADS * PER_THREAD,
        )));

        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let rng = rng.clone();
                let barrier = barrier.clone();
                let results = results.clone();
                thread::spawn(move || {
                    barrier.wait();
                    let mut local = Vec::with_capacity(PER_THREAD);
                    for _ in 0..PER_THREAD {
                        local.push(rng.next_u64());
                    }
                    results.lock().extend(local);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let all = results.lock();
        let set: HashSet<_> = all.iter().collect();
        assert_eq!(set.len(), all.len(), "concurrent draws must be unique");
    }

    #[test]
    fn merge_from_concurrent_no_deadlock() {
        let a = DependencyValues::new();
        let b = DependencyValues::new();
        a.insert(42_u32);
        b.insert(99_u64);

        let barrier = Arc::new(Barrier::new(2));
        let a1 = a.clone();
        let b1 = b.clone();
        let a2 = a.clone();
        let b2 = b.clone();
        let c1 = barrier.clone();
        let c2 = barrier.clone();

        let t1 = thread::spawn(move || {
            c1.wait();
            a1.merge_from(&b1);
        });
        let t2 = thread::spawn(move || {
            c2.wait();
            b2.merge_from(&a2);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        assert!(a.contains::<u32>());
        assert!(a.contains::<u64>());
        assert!(b.contains::<u32>());
        assert!(b.contains::<u64>());
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
        let epoch_system = test_epoch_system();
        let now = TestNow::new(clock.clone(), epoch_system);
        let before = now.system_time();
        clock.advance(std::time::Duration::from_secs(5));
        assert!(now.system_time() > before);
    }

    #[test]
    fn test_bag_clock_and_now_stay_in_sync() {
        let values = DependencyValues::test();
        let now_dep = values.require::<NowDep>().unwrap();
        let clock = shared_test_clock();
        let before = now_dep.0.system_time();
        clock.advance(std::time::Duration::from_secs(5));
        assert!(now_dep.0.system_time() > before);
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
    fn try_test_returns_err_for_unimplemented_key() {
        struct UnimplementedKey;
        impl DependencyKey for UnimplementedKey {
            type Value = u32;
            fn live() -> Self::Value {
                1
            }
        }
        assert!(UnimplementedKey::try_test().is_err());
    }

    #[test]
    fn with_overrides_single_dependency() {
        let values = DependencyValues::test().with(UuidDep(Arc::new(SeededUuidGen::new(42))));
        let uuid = values.require::<UuidDep>().unwrap();
        assert_eq!(uuid.0.next(), SeededUuidGen::new(42).next());
    }

    #[test]
    fn get_or_insert_with_and_remove() {
        let values = DependencyValues::new();
        assert!(!values.contains::<String>());

        let inserted = values.get_or_insert_with(|| "hello".to_string());
        assert_eq!(&*inserted, "hello");
        assert!(values.contains::<String>());

        let again = values.get_or_insert_with(|| "world".to_string());
        assert_eq!(&*again, "hello");

        let removed = values.remove::<String>().unwrap();
        assert_eq!(&*removed, "hello");
        assert!(!values.contains::<String>());
    }

    #[test]
    fn require_missing_returns_typed_error() {
        let values = DependencyValues::new();
        let err = match values.require::<UuidDep>() {
            Err(e) => e,
            Ok(_) => panic!("expected missing dependency"),
        };
        assert!(err.to_string().contains("missing dependency"));
        assert!(err.name.contains("UuidDep"));
    }

    #[test]
    fn insert_overwrites_existing_entry() {
        let values = DependencyValues::new();
        values.insert(1_u32);
        values.insert(2_u32);
        assert_eq!(*values.get::<u32>().unwrap(), 2);
    }

    #[test]
    fn merge_from_overwrites_conflicting_keys() {
        let base = DependencyValues::new();
        base.insert("base".to_string());
        let overlay = DependencyValues::new();
        overlay.insert("overlay".to_string());
        base.merge_from(&overlay);
        assert_eq!(*base.get::<String>().unwrap(), "overlay");
    }

    #[test]
    fn merge_from_empty_overlay_is_noop() {
        let base = DependencyValues::new();
        base.insert(7_u32);
        base.merge_from(&DependencyValues::new());
        assert_eq!(*base.get::<u32>().unwrap(), 7);
    }

    #[test]
    fn merge_from_self_does_not_deadlock() {
        let values = DependencyValues::test();
        values.merge_from(&values);
        assert!(values.contains::<UuidDep>());
    }

    #[test]
    fn cloned_bags_share_underlying_map() {
        let a = DependencyValues::new();
        a.insert(1_u32);
        let b = a.clone();
        b.insert(99_u64);
        assert!(a.contains::<u64>());
        assert_eq!(*b.get::<u32>().unwrap(), 1);
    }

    #[test]
    fn live_and_test_bags_include_all_builtins() {
        for bag in [DependencyValues::live(), DependencyValues::test()] {
            assert!(bag.require::<ClockDep>().is_ok());
            assert!(bag.require::<UuidDep>().is_ok());
            assert!(bag.require::<NowDep>().is_ok());
            assert!(bag.require::<RngDep>().is_ok());
        }
    }

    #[test]
    fn seeded_rng_zero_seed_clamps_to_one() {
        let zero = SeededRng::new(0);
        let one = SeededRng::new(1);
        assert_eq!(zero.next_u64(), one.next_u64());
    }

    #[test]
    fn seeded_uuid_gen_produces_distinct_sequence() {
        let uuid_gen = SeededUuidGen::new(10);
        let a = uuid_gen.next();
        let b = uuid_gen.next();
        let c = uuid_gen.next();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn live_uuid_gen_instances_are_independent() {
        let a = LiveUuidGen::new();
        let b = LiveUuidGen::new();
        let seq_a: Vec<_> = (0..4).map(|_| a.next()).collect();
        let seq_b: Vec<_> = (0..4).map(|_| b.next()).collect();
        assert_ne!(seq_a, seq_b);
    }

    #[test]
    fn live_uuid_gen_concurrent_unique() {
        let uuid_gen = Arc::new(LiveUuidGen::new());
        const THREADS: usize = 4;
        const PER_THREAD: usize = 500;
        let barrier = Arc::new(Barrier::new(THREADS));
        let results = Arc::new(parking_lot::Mutex::new(Vec::new()));

        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let uuid_gen = uuid_gen.clone();
                let barrier = barrier.clone();
                let results = results.clone();
                thread::spawn(move || {
                    barrier.wait();
                    let mut local = Vec::with_capacity(PER_THREAD);
                    for _ in 0..PER_THREAD {
                        local.push(uuid_gen.next());
                    }
                    results.lock().extend(local);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let all = results.lock();
        let set: HashSet<_> = all.iter().collect();
        assert_eq!(set.len(), all.len());
    }

    #[test]
    fn live_rng_returns_xorshift_draws() {
        let rng = LiveRng::from_seed(99);
        assert_eq!(rng.next_u64(), xorshift64(99));
        assert_eq!(rng.next_u64(), xorshift64(xorshift64(99)));
    }

    #[test]
    fn live_rng_concurrent_unique_values() {
        let rng = Arc::new(LiveRng::from_seed(0xBEEF));
        const THREADS: usize = 4;
        const PER_THREAD: usize = 500;
        let barrier = Arc::new(Barrier::new(THREADS));
        let results = Arc::new(parking_lot::Mutex::new(Vec::new()));

        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                let rng = rng.clone();
                let barrier = barrier.clone();
                let results = results.clone();
                thread::spawn(move || {
                    barrier.wait();
                    let mut local = Vec::with_capacity(PER_THREAD);
                    for _ in 0..PER_THREAD {
                        local.push(rng.next_u64());
                    }
                    results.lock().extend(local);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let all = results.lock();
        let set: HashSet<_> = all.iter().collect();
        assert_eq!(set.len(), all.len());
    }

    #[test]
    fn test_now_elapsed_matches_clock_advance() {
        let clock = TestClock::new(Instant::now());
        let epoch = test_epoch_system();
        let now = TestNow::new(clock.clone(), epoch);
        let before = now.system_time();
        clock.advance(std::time::Duration::from_millis(250));
        assert_eq!(
            now.system_time(),
            before + std::time::Duration::from_millis(250)
        );
    }

    #[test]
    fn test_clock_clones_share_state() {
        let clock = TestClock::new(Instant::now());
        let clone = clock.clone();
        clock.advance(std::time::Duration::from_secs(3));
        assert_eq!(clone.now(), clock.now());
    }

    #[test]
    fn dependency_key_register_inserts_into_bag() {
        let bag = DependencyValues::new();
        UuidKey::register(&bag, UuidKey::test());
        assert!(bag.contains::<UuidDep>());
    }

    #[test]
    fn dependency_error_display_and_eq() {
        let a = DependencyError::missing("Foo");
        let b = DependencyError::missing("Foo");
        assert_eq!(a, b);
        assert_eq!(a.to_string(), "missing dependency: Foo");
    }

    #[test]
    fn remove_missing_returns_none() {
        let values = DependencyValues::new();
        assert!(values.remove::<i32>().is_none());
    }

    #[test]
    fn get_or_insert_with_runs_factory_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let values = DependencyValues::new();
        let factory_calls = Arc::new(AtomicUsize::new(0));
        let on_first = factory_calls.clone();
        values.get_or_insert_with(|| {
            on_first.fetch_add(1, Ordering::SeqCst);
            42_u32
        });
        let on_second = factory_calls.clone();
        values.get_or_insert_with(|| {
            on_second.fetch_add(1, Ordering::SeqCst);
            99_u32
        });
        assert_eq!(factory_calls.load(Ordering::SeqCst), 1);
        assert_eq!(*values.get::<u32>().unwrap(), 42);
    }
}

#[cfg(test)]
impl LiveRng {
    fn from_seed(seed: u64) -> Self {
        Self {
            state: AtomicU64::new(seed.max(1)),
        }
    }
}
