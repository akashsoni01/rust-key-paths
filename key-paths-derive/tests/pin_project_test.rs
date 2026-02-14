//! Tests for #[pin] field support (pin_project pattern).
//! Requires: #[pin_project] #[derive(Kp)] struct S { #[pin] f: T }

use std::future::Future;
use std::pin::Pin;

use key_paths_derive::Kp;
use pin_project::pin_project;

#[pin_project]
#[derive(Kp)]
struct WithPinnedBoxFuture {
    #[pin]
    fut: Pin<Box<dyn Future<Output = i32> + Send>>,
}

#[pin_project]
#[derive(Kp)]
struct WithPinnedField {
    fair: bool,
    #[pin]
    value: i32,
}

#[test]
fn test_pinned_field_container_and_pinned_projection() {
    let data = WithPinnedField { fair: true, value: 42 };

    // Regular container access
    let kp = WithPinnedField::value();
    assert_eq!(kp.get(&data), Some(&42));

    // Mutable via Kp
    let mut data_mut = WithPinnedField { fair: false, value: 100 };
    if let Some(v) = kp.get_mut(&mut data_mut) {
        *v = 200;
    }
    assert_eq!(data_mut.value, 200);

    // Pinned projection - requires Pin<&mut Self>
    let mut data_pin = WithPinnedField { fair: true, value: 99 };
    let pinned = Pin::new(&mut data_pin);
    let projected: Pin<&mut i32> = WithPinnedField::value_pinned(pinned);
    assert_eq!(*projected.get_mut(), 99);
}

#[tokio::test]
async fn test_pinned_future_field_await() {
    use std::future::ready;

    let mut data = WithPinnedBoxFuture {
        fut: Box::pin(ready(42)),
    };
    let pinned = Pin::new(&mut data);

    // field_await: poll the pinned future through Pin<&mut Self>
    let result = WithPinnedBoxFuture::fut_await(pinned).await;
    assert_eq!(result, Some(42));
}
