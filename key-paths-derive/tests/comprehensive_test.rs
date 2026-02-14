use key_paths_derive::Kp;
use rust_key_paths::{KpType, LockKp};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::Arc;

// Test collections
#[derive(Kp)]
struct Collections {
    items: Vec<i32>,
    queue: VecDeque<f64>,
}

// Test smart pointers
#[derive(Kp)]
struct SmartPointers {
    boxed: Box<String>,
    rc: std::rc::Rc<i32>,
    arc: std::sync::Arc<String>,
}

// Test locks
#[derive(Kp)]
struct WithLocks {
    std_mutex: std::sync::Mutex<i32>,
    std_rwlock: std::sync::RwLock<String>,
}

// Test tokio async locks (requires rust-key-paths tokio feature)
#[derive(Kp)]
struct WithTokioLocks {
    data: Arc<tokio::sync::RwLock<Vec<i32>>>,
}

// Test Option<Arc<tokio::sync::RwLock<T>>>
#[derive(Kp)]
struct WithOptionTokioLocks {
    data: Option<Arc<tokio::sync::RwLock<i32>>>,
}

// Test Cow and Option<Cow>
#[derive(Kp)]
struct WithCow {
    cow_owned: Cow<'static, String>,
    cow_borrowed: Cow<'static, String>,
    opt_cow: Option<Cow<'static, String>>,
}

// Test HashMap and BTreeMap _at(key)
#[derive(Kp)]
struct WithMaps {
    users: HashMap<String, i32>,
    cache: BTreeMap<u64, String>,
}

// Test atomic types
#[derive(Kp)]
struct WithAtomics {
    counter: AtomicI32,
    flags: AtomicU64,
}

// Test Option<Atomic>
#[derive(Kp)]
struct WithOptionAtomic {
    opt_counter: Option<AtomicI32>,
}

#[test]
fn test_identity_keypath() {
    let collections = Collections {
        items: vec![1, 2, 3],
        queue: VecDeque::new(),
    };

    // Test identity keypath returns the struct itself
    let identity_kp = Collections::identity();
    let result = identity_kp.get(&collections);
    assert!(result.is_some());
    assert_eq!(result.unwrap().items.len(), 3);
}

#[test]
fn test_identity_mutable() {
    let mut collections = Collections {
        items: vec![1, 2, 3],
        queue: VecDeque::new(),
    };

    // Test identity keypath can mutate the struct
    let identity_kp = Collections::identity();
    identity_kp.get_mut(&mut collections).map(|c| {
        c.items.push(4);
    });

    assert_eq!(collections.items.len(), 4);
}

#[test]
fn test_identity_typed() {
    let smart = SmartPointers {
        boxed: Box::new("test".to_string()),
        rc: std::rc::Rc::new(42),
        arc: std::sync::Arc::new("arc".to_string()),
    };

    // Test typed identity keypath
    let identity_kp = SmartPointers::identity_typed::<&SmartPointers, &mut SmartPointers>();
    let result = identity_kp.get(&smart);
    assert!(result.is_some());
}

#[test]
fn test_vec_access() {
    let collections = Collections {
        items: vec![10, 20, 30, 40, 50],
        queue: VecDeque::new(),
    };

    // items() returns container; items_at(index) returns element at index
    let container_kp = Collections::items();
    assert_eq!(container_kp.get(&collections).map(|v| v.len()), Some(5));

    let first_kp = Collections::items_at(0);
    assert_eq!(first_kp.get(&collections), Some(&10));
}

#[test]
fn test_vec_mutable() {
    let mut collections = Collections {
        items: vec![1, 2, 3, 4, 5],
        queue: VecDeque::new(),
    };

    // Mutate first element through items_at(index)
    let items_at_kp = Collections::items_at(0);
    items_at_kp.get_mut(&mut collections).map(|v| *v = 200);

    assert_eq!(collections.items[0], 200);
}

#[test]
fn test_vecdeque_access() {
    let mut queue = VecDeque::new();
    queue.push_back(1.1);
    queue.push_back(2.2);
    queue.push_back(3.3);

    let collections = Collections {
        items: vec![],
        queue,
    };
    // queue() returns container; queue_at(index) returns element at index
    let front_kp = Collections::queue_at(0);
    assert_eq!(front_kp.get(&collections), Some(&1.1));
}

#[test]
fn test_cow_access() {
    let data = WithCow {
        cow_owned: Cow::Owned("owned".to_string()),
        cow_borrowed: Cow::Owned("borrowed".to_string()),
        opt_cow: Some(Cow::Owned("optional".to_string())),
    };

    let cow_owned_kp = WithCow::cow_owned();
    assert_eq!(cow_owned_kp.get(&data).map(|s| s.as_str()), Some("owned"));

    let cow_borrowed_kp = WithCow::cow_borrowed();
    assert_eq!(cow_borrowed_kp.get(&data).map(|s| s.as_str()), Some("borrowed"));

    let opt_cow_kp = WithCow::opt_cow();
    assert_eq!(opt_cow_kp.get(&data).map(|s| s.as_str()), Some("optional"));
}

#[test]
fn test_cow_mutable() {
    let mut data = WithCow {
        cow_owned: Cow::Owned("original".to_string()),
        cow_borrowed: Cow::Owned("borrowed".to_string()),
        opt_cow: Some(Cow::Owned("opt_original".to_string())),
    };

    let cow_owned_kp = WithCow::cow_owned();
    cow_owned_kp.get_mut(&mut data).map(|s| s.make_ascii_uppercase());
    assert_eq!(data.cow_owned.as_str(), "ORIGINAL");

    let opt_cow_kp = WithCow::opt_cow();
    opt_cow_kp.get_mut(&mut data).map(|s| s.make_ascii_uppercase());
    assert_eq!(data.opt_cow.as_ref().map(|c| c.as_str()), Some("OPT_ORIGINAL"));
}

#[test]
fn test_atomic_types() {
    let mut data = WithAtomics {
        counter: AtomicI32::new(42),
        flags: AtomicU64::new(0xFF),
    };

    let counter_kp = WithAtomics::counter();
    let atomic = counter_kp.get(&data).unwrap();
    assert_eq!(atomic.load(Ordering::SeqCst), 42);
    counter_kp.get_mut(&mut data).unwrap().store(100, Ordering::SeqCst);
    assert_eq!(data.counter.load(Ordering::SeqCst), 100);

    let flags_kp = WithAtomics::flags();
    assert_eq!(flags_kp.get(&data).unwrap().load(Ordering::SeqCst), 0xFF);
}

#[test]
fn test_option_atomic() {
    let mut data = WithOptionAtomic {
        opt_counter: Some(AtomicI32::new(10)),
    };
    let kp = WithOptionAtomic::opt_counter();
    assert_eq!(kp.get(&data).unwrap().load(Ordering::SeqCst), 10);
    kp.get_mut(&mut data).unwrap().store(20, Ordering::SeqCst);
    assert_eq!(data.opt_counter.unwrap().load(Ordering::SeqCst), 20);

    let data_none = WithOptionAtomic { opt_counter: None };
    assert!(kp.get(&data_none).is_none());
}

#[test]
fn test_cow_option_none() {
    let data = WithCow {
        cow_owned: Cow::Owned("x".to_string()),
        cow_borrowed: Cow::Owned("y".to_string()),
        opt_cow: None,
    };

    let opt_cow_kp = WithCow::opt_cow();
    assert_eq!(opt_cow_kp.get(&data), None);
}

#[test]
fn test_hashmap_at() {
    let mut users = HashMap::new();
    users.insert("alice".to_string(), 100);
    users.insert("bob".to_string(), 200);
    let data = WithMaps {
        users: users.clone(),
        cache: BTreeMap::new(),
    };

    let kp = WithMaps::users_at("alice".to_string());
    assert_eq!(kp.get(&data), Some(&100));

    let mut data_mut = WithMaps { users, cache: BTreeMap::new() };
    let kp_mut = WithMaps::users_at("alice".to_string());
    *kp_mut.get_mut(&mut data_mut).unwrap() = 150;
    assert_eq!(data_mut.users.get("alice"), Some(&150));
}

#[test]
fn test_btreemap_at() {
    let mut cache = BTreeMap::new();
    cache.insert(1u64, "one".to_string());
    cache.insert(2u64, "two".to_string());
    let data = WithMaps {
        users: HashMap::new(),
        cache: cache.clone(),
    };

    let kp = WithMaps::cache_at(1);
    assert_eq!(kp.get(&data), Some(&"one".to_string()));

    let mut data_mut = WithMaps { users: HashMap::new(), cache };
    let kp_mut = WithMaps::cache_at(1);
    *kp_mut.get_mut(&mut data_mut).unwrap() = "1".to_string();
    assert_eq!(data_mut.cache.get(&1), Some(&"1".to_string()));
}

#[test]
fn test_box_access() {
    let smart = SmartPointers {
        boxed: Box::new("boxed_value".to_string()),
        rc: std::rc::Rc::new(42),
        arc: std::sync::Arc::new("arc_value".to_string()),
    };

    let boxed_kp = SmartPointers::boxed();
    assert_eq!(
        boxed_kp.get(&smart).map(|s| s.as_str()),
        Some("boxed_value")
    );
}

#[test]
fn test_box_mutable() {
    let mut smart = SmartPointers {
        boxed: Box::new("original".to_string()),
        rc: std::rc::Rc::new(1),
        arc: std::sync::Arc::new("test".to_string()),
    };

    let boxed_kp = SmartPointers::boxed();
    boxed_kp
        .get_mut(&mut smart)
        .map(|s| *s = "modified".to_string());

    assert_eq!(smart.boxed.as_str(), "modified");
}

#[test]
fn test_rc_access() {
    let mut smart = SmartPointers {
        boxed: Box::new("test".to_string()),
        rc: std::rc::Rc::new(42),
        arc: std::sync::Arc::new("test".to_string()),
    };

    let rc_kp = SmartPointers::rc();
    assert_eq!(rc_kp.get(&smart), Some(&42));

    // Test mutable access when Rc has only one reference
    rc_kp.get_mut(&mut smart).map(|v| *v = 100);
    assert_eq!(*smart.rc, 100);
}

#[test]
fn test_arc_access() {
    let mut smart = SmartPointers {
        boxed: Box::new("test".to_string()),
        rc: std::rc::Rc::new(1),
        arc: std::sync::Arc::new("original".to_string()),
    };

    let arc_kp = SmartPointers::arc();
    assert_eq!(arc_kp.get(&smart).map(|s| s.as_str()), Some("original"));

    // Test mutable access when Arc has only one reference
    arc_kp
        .get_mut(&mut smart)
        .map(|v| *v = "modified".to_string());
    assert_eq!(smart.arc.as_str(), "modified");
}

#[test]
fn test_rc_no_mut_with_multiple_refs() {
    let rc = std::rc::Rc::new(42);
    let rc_clone = rc.clone(); // Now there are 2 references

    let mut smart = SmartPointers {
        boxed: Box::new("test".to_string()),
        rc,
        arc: std::sync::Arc::new("test".to_string()),
    };

    let rc_kp = SmartPointers::rc();

    // Should return None because there are multiple references
    assert_eq!(rc_kp.get_mut(&mut smart), None);

    // Cleanup
    drop(rc_clone);
}

#[test]
fn test_arc_no_mut_with_multiple_refs() {
    let arc = std::sync::Arc::new("test".to_string());
    let arc_clone = arc.clone(); // Now there are 2 references

    let mut smart = SmartPointers {
        boxed: Box::new("test".to_string()),
        rc: std::rc::Rc::new(1),
        arc,
    };

    let arc_kp = SmartPointers::arc();

    // Should return None because there are multiple references
    assert_eq!(arc_kp.get_mut(&mut smart), None);

    // Cleanup
    drop(arc_clone);
}

#[test]
fn test_std_mutex_with_lockkp() {
    use std::sync::Mutex;

    let locks = WithLocks {
        std_mutex: Mutex::new(99),
        std_rwlock: std::sync::RwLock::new("test".to_string()),
    };

    // Get keypath to mutex
    let mutex_kp = WithLocks::std_mutex();
    let rwlock_kp = WithLocks::std_rwlock();
    // rwlock_kp.get()
    // rwlock_kp.sync_get(&locks).unwrap();
    // rwlock_kp.sync_get_mut()
    
    // Create LockKp for accessing the inner value
    let next: KpType<i32, i32> = rust_key_paths::Kp::new(|i: &i32| Some(i), |i: &mut i32| Some(i));

    let lock_kp = LockKp::new(mutex_kp, rust_key_paths::StdMutexAccess::new(), next);

    // Access through lock
    let value = lock_kp.get(&locks);
    assert_eq!(value, Some(&99));
}

#[tokio::test]
async fn test_tokio_rwlock_async_kp() {
    let root = WithTokioLocks {
        data: Arc::new(tokio::sync::RwLock::new(vec![1, 2, 3, 4, 5])),
    };

    // data() returns KpType to container
    let container_kp = WithTokioLocks::data();
    let arc_ref = container_kp.get(&root);
    assert!(arc_ref.is_some());

    // data_async() returns AsyncLockKpRwLockFor - use .get(&root).await for async access
    let async_kp = WithTokioLocks::data_async();
    let value = async_kp.get(&root).await;
    assert!(value.is_some());
    assert_eq!(value.unwrap().len(), 5);
}

#[tokio::test]
async fn test_option_tokio_rwlock_async_kp() {
    let root_some = WithOptionTokioLocks {
        data: Some(Arc::new(tokio::sync::RwLock::new(42))),
    };
    let root_none = WithOptionTokioLocks { data: None };

    // data_async() - when Some, returns the value
    let async_kp = WithOptionTokioLocks::data_async();
    let value = async_kp.get(&root_some).await;
    assert!(value.is_some());
    assert_eq!(*value.unwrap(), 42);

    // When None, returns None
    let value_none = async_kp.get(&root_none).await;
    assert!(value_none.is_none());
}
