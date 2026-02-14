//! Test enum with complex containers like Arc<RwLock<T>> (reusing struct prior art)

use key_paths_derive::Kp;
use std::sync::Arc;

#[derive(Debug, Kp)]
enum Message {
    Text(String),
    Data(Arc<std::sync::RwLock<String>>),
    /// Arc<tokio::sync::RwLock<T>> - has tokio_data_async()
    TokioData(Arc<tokio::sync::RwLock<String>>),
    Empty,
}

#[test]
fn test_enum_arc_rwlock() {
    let msg = Message::Data(Arc::new(std::sync::RwLock::new("hello".to_string())));
    let data_kp = Message::data();
    let arc = data_kp.get(&msg);
    assert!(arc.is_some());

    let lock_kp = Message::data_lock();
    let value = lock_kp.get(&msg).unwrap();
    assert_eq!(value.as_str(), "hello");
}

#[test]
fn test_enum_text() {
    let msg = Message::Text("hi".to_string());
    let text_kp = Message::text();
    assert_eq!(text_kp.get(&msg), Some(&"hi".to_string()));
}

#[test]
fn test_enum_empty() {
    let msg = Message::Empty;
    let empty_kp = Message::empty();
    assert!(empty_kp.get(&msg).is_some());
}

#[tokio::test]
async fn test_enum_tokio_async() {
    let msg = Message::TokioData(Arc::new(tokio::sync::RwLock::new("async_hello".to_string())));
    let arc_kp = Message::tokio_data();
    let arc = arc_kp.get(&msg);
    assert!(arc.is_some());

    let kp = Message::tokio_data_async();
    let guard = kp.get(&msg).await.unwrap();
    assert_eq!(guard.as_str(), "async_hello");
}
