//! Test enum with complex containers like Arc<RwLock<T>> (reusing struct prior art)

use key_paths_derive::Kp;
use std::sync::Arc;

#[derive(Debug, Kp)]
enum Message {
    Text(String),
    Data(Arc<std::sync::RwLock<String>>),
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
