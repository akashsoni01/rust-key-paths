//! Integration test: Kp derive with Option, Box, Rc, Arc, Vec, Option<Vec>.

use keypaths_proc::Kp;
use std::sync::Arc;

#[derive(Debug, Clone, Kp)]
struct ContainerFields {
    name: String,
    age: Option<u32>,
    boxed: Box<String>,
    rc: std::rc::Rc<i32>,
    arc: Arc<bool>,
    vec: Vec<String>,
    opt_vec: Option<Vec<String>>,
}

#[test]
fn test_container_keypaths() {
    let value = ContainerFields {
        name: "Alice".to_string(),
        age: Some(30),
        boxed: Box::new("boxed".to_string()),
        rc: std::rc::Rc::new(42),
        arc: Arc::new(true),
        vec: vec!["a".into(), "b".into(), "c".into()],
        opt_vec: Some(vec!["one".into(), "two".into()]),
    };

    assert_eq!(ContainerFields::name_r().get(&value), "Alice");
    assert_eq!(ContainerFields::age_fr().get(&value), Some(&30u32));
    assert_eq!(ContainerFields::boxed_r().get(&value).as_str(), "boxed");
    assert_eq!(*ContainerFields::rc_r().get(&value), 42);
    assert!(*ContainerFields::arc_r().get(&value));
    assert_eq!(ContainerFields::vec_r().get(&value).len(), 3);
    assert_eq!(ContainerFields::vec_r().get(&value)[0], "a");
    assert_eq!(ContainerFields::vec_fr().get(&value), Some(&"a".to_string()));
    assert_eq!(ContainerFields::vec_fr_at(1).get(&value), Some(&"b".to_string()));
    assert_eq!(ContainerFields::opt_vec_fr().get(&value), Some(&"one".to_string()));
    assert_eq!(ContainerFields::opt_vec_fr_at(1).get(&value), Some(&"two".to_string()));
}
