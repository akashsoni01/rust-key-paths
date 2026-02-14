//! Example: #[pin] field support (pin_project pattern)
//!
//! When using pin_project, mark pinned fields with #[pin]. The Kp derive generates:
//! - `field()` - regular container keypath (for &T / &mut T access)
//! - `field_pinned()` - pinned projection (returns Pin<&mut T> via this.project())
//! - For Future fields: `field_await()` - async poll helper
//!
//! **Requires:** #[pin_project] on the struct (from pin-project crate).

use std::pin::Pin;

use key_paths_derive::Kp;
use pin_project::pin_project;

#[pin_project]
#[derive(Kp)]
struct WithPinnedFields {
    name: String,
    #[pin]
    counter: i32,
}

fn main() {
    println!("=== pin_project #[pin] Field Example ===\n");

    let mut data = WithPinnedFields {
        name: "test".to_string(),
        counter: 42,
    };

    // Regular keypath for non-pinned field
    let name_kp = WithPinnedFields::name();
    assert_eq!(name_kp.get(&data), Some(&"test".to_string()));

    // Regular keypath for #[pin] field - works for Unpin types like i32
    let counter_kp = WithPinnedFields::counter();
    assert_eq!(counter_kp.get(&data), Some(&42));

    // Pinned projection - requires Pin<&mut Self>
    let pinned = Pin::new(&mut data);
    let mut counter_pin: Pin<&mut i32> = WithPinnedFields::counter_pinned(pinned);
    assert_eq!(*counter_pin.as_mut().get_mut(), 42);

    println!("  name(): {:?}", name_kp.get(&data));
    println!("  counter(): {:?}", counter_kp.get(&data));

    let mut data2 = WithPinnedFields {
        name: "test2".to_string(),
        counter: 99,
    };
    let mut counter_pin2: Pin<&mut i32> = WithPinnedFields::counter_pinned(Pin::new(&mut data2));
    println!("  counter_pinned(): {:?}", *counter_pin2.as_mut().get_mut());
    println!("\n=== pin_project example passed! ===");
}
