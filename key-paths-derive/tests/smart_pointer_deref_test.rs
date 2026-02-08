use key_paths_derive::Kp;
use rust_key_paths::KpType;

#[derive(Kp)]
struct AllWrapperTypes {
    boxed_value: Box<i32>,
    rc_value: std::rc::Rc<String>,
    arc_value: std::sync::Arc<f64>,
}

#[test]
fn test_box_returns_inner_type() {
    let data = AllWrapperTypes {
        boxed_value: Box::new(42),
        rc_value: std::rc::Rc::new("test".to_string()),
        arc_value: std::sync::Arc::new(3.14),
    };

    // The keypath should return &i32, not &Box<i32>
    let box_kp = AllWrapperTypes::boxed_value();
    let value: Option<&i32> = box_kp.get(&data);
    assert_eq!(value, Some(&42));

    // Verify the type is indeed KpType<'static, AllWrapperTypes, i32>
    let _typed: KpType<'static, AllWrapperTypes, i32> = box_kp;
}

#[test]
fn test_rc_returns_inner_type() {
    let data = AllWrapperTypes {
        boxed_value: Box::new(1),
        rc_value: std::rc::Rc::new("hello".to_string()),
        arc_value: std::sync::Arc::new(1.0),
    };

    // The keypath should return &String, not &Rc<String>
    let rc_kp = AllWrapperTypes::rc_value();
    let value: Option<&String> = rc_kp.get(&data);
    assert_eq!(value.map(|s| s.as_str()), Some("hello"));

    // Verify the type
    let _typed: KpType<'static, AllWrapperTypes, String> = rc_kp;
}

#[test]
fn test_arc_returns_inner_type() {
    let data = AllWrapperTypes {
        boxed_value: Box::new(1),
        rc_value: std::rc::Rc::new("test".to_string()),
        arc_value: std::sync::Arc::new(2.71),
    };

    // The keypath should return &f64, not &Arc<f64>
    let arc_kp = AllWrapperTypes::arc_value();
    let value: Option<&f64> = arc_kp.get(&data);
    assert_eq!(value, Some(&2.71));

    // Verify the type
    let _typed: KpType<'static, AllWrapperTypes, f64> = arc_kp;
}

#[test]
fn test_box_mutable_returns_inner_type() {
    let mut data = AllWrapperTypes {
        boxed_value: Box::new(10),
        rc_value: std::rc::Rc::new("test".to_string()),
        arc_value: std::sync::Arc::new(1.0),
    };

    // Mutable access should return &mut i32, not &mut Box<i32>
    let box_kp = AllWrapperTypes::boxed_value();
    let value: Option<&mut i32> = box_kp.get_mut(&mut data);
    assert!(value.is_some());
    *value.unwrap() = 100;

    assert_eq!(*data.boxed_value, 100);
}
