use key_paths_derive::Kp;
use rust_key_paths::{KpType, LockKp};
use std::collections::VecDeque;
use std::sync::Arc;
use rust_key_paths::async_lock::SyncKeyPathLike;

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

    // items() returns container; items_at() returns first element
    let container_kp = Collections::items();
    assert_eq!(container_kp.get(&collections).map(|v| v.len()), Some(5));

    let first_kp = Collections::items_at();
    assert_eq!(first_kp.get(&collections), Some(&10));
}

#[test]
fn test_vec_mutable() {
    let mut collections = Collections {
        items: vec![1, 2, 3, 4, 5],
        queue: VecDeque::new(),
    };

    // Mutate first element through items_at()
    let items_at_kp = Collections::items_at();
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
    // queue() returns container; queue_at() returns front element
    let front_kp = Collections::queue_at();
    assert_eq!(front_kp.get(&collections), Some(&1.1));
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
