//! Example: Box and Pin keypath support
//!
//! Demonstrates keypath derivation for:
//! - `Box<T>`: derefs to inner type (field() returns KpType<Root, T>)
//! - `Pin<T>`: container access (field()) + inner access when T: Unpin (field_inner())
//! - `Pin<Box<T>>`: container access + inner access when T: Unpin

use std::pin::Pin;

use key_paths_derive::Kp;

#[derive(Debug, Kp)]
struct WithBox {
    /// Box<T> - keypath derefs to inner T directly
    boxed: Box<String>,
}

#[derive(Debug, Kp)]
struct WithPin {
    /// Pin<T> - field() gives Pin<T>, field_inner() gives T (requires T: Unpin)
    pinned: Pin<Box<String>>,
}

#[derive(Debug, Kp)]
struct WithPinDirect {
    /// Pin<T> where T is not Box - e.g. Pin<&mut T> isn't stored, but Pin<MyType> could be
    pinned_value: Pin<Box<i32>>,
}

fn main() {
    println!("=== Box and Pin Keypath Example ===\n");

    // --- Box ---
    println!("--- Box<T> ---");
    let with_box = WithBox {
        boxed: Box::new("hello".to_string()),
    };
    let box_kp = WithBox::boxed();
    assert_eq!(box_kp.get(&with_box), Some(&"hello".to_string()));
    println!("  boxed() returns KpType<WithBox, String> (derefs through Box)");
    println!("  get: {:?}", box_kp.get(&with_box));

    let mut with_box_mut = WithBox {
        boxed: Box::new("world".to_string()),
    };
    if let Some(s) = box_kp.get_mut(&mut with_box_mut) {
        *s = "modified".to_string();
    }
    println!("  after mutation: {:?}", with_box_mut.boxed);
    println!();

    // --- Pin<Box<T>> ---
    println!("--- Pin<Box<T>> ---");
    let with_pin = WithPin {
        pinned: Pin::new(Box::new("pinned".to_string())),
    };
    // Container access: pinned() returns KpType<WithPin, Pin<Box<String>>>
    let pin_container_kp = WithPin::pinned();
    let container_ref = pin_container_kp.get(&with_pin).unwrap();
    println!("  pinned() -> Pin<Box<String>>: {:?}", Pin::as_ref(container_ref).get_ref().as_str());

    // Inner access: pinned_inner() returns KpType<WithPin, String> (String: Unpin)
    let pin_inner_kp = WithPin::pinned_inner();
    assert_eq!(pin_inner_kp.get(&with_pin), Some(&"pinned".to_string()));
    println!("  pinned_inner() -> String (requires Unpin): {:?}", pin_inner_kp.get(&with_pin));

    let mut with_pin_mut = WithPin {
        pinned: Pin::new(Box::new("mutable".to_string())),
    };
    if let Some(s) = pin_inner_kp.get_mut(&mut with_pin_mut) {
        *s = "changed".to_string();
    }
    println!("  after mutation via pinned_inner: {:?}", with_pin_mut.pinned.get_ref().as_str());
    println!();

    // --- Pin<Box<i32>> ---
    println!("--- Pin<Box<i32>> ---");
    let with_pin_int = WithPinDirect {
        pinned_value: Pin::new(Box::new(42)),
    };
    let int_inner_kp = WithPinDirect::pinned_value_inner();
    assert_eq!(int_inner_kp.get(&with_pin_int), Some(&42));
    println!("  pinned_value_inner() -> i32: {:?}", int_inner_kp.get(&with_pin_int));

    let mut with_pin_int_mut = WithPinDirect {
        pinned_value: Pin::new(Box::new(100)),
    };
    if let Some(v) = int_inner_kp.get_mut(&mut with_pin_int_mut) {
        *v = 200;
    }
    println!("  after mutation: {:?}", with_pin_int_mut.pinned_value.get_ref());
    println!();

    println!("=== All Box and Pin examples passed! ===");
}
