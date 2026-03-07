//! Integration test: `then_pin_future` composition for #[pin] Future fields.
//!
//! Uses Kp-derived `{field}_pin_future_kp()` with `Kp::then_pin_future` to await
//! pinned futures ergonomically, in the style of `then_async` for async locks.

#![cfg(all(feature = "tokio", feature = "pin_project"))]

use std::future::Future;
use std::pin::Pin;

use key_paths_derive::Kp;
use pin_project::pin_project;
use rust_key_paths::{Kp, KpType};

#[pin_project]
#[derive(Kp)]
struct WithPinnedBoxFuture {
    #[pin]
    fut: Pin<Box<dyn Future<Output = i32> + Send>>,
}

#[pin_project]
#[derive(Kp)]
struct Wrapper {
    inner: WithPinnedBoxFuture,
}

#[tokio::test]
async fn test_then_pin_future_identity() {
    use std::future::ready;

    let mut data = WithPinnedBoxFuture {
        fut: Box::pin(ready(42)),
    };

    // Identity Kp to the struct, then_pin_future awaits the #[pin] Future field
    let identity_kp: KpType<WithPinnedBoxFuture, WithPinnedBoxFuture> =
        Kp::new(|x: &WithPinnedBoxFuture| Some(x), |x: &mut WithPinnedBoxFuture| Some(x));
    let kp = identity_kp.then_pin_future(WithPinnedBoxFuture::fut_pin_future_kp());

    let result = kp.get_mut(&mut data).await;
    assert_eq!(result, Some(42));
}

#[tokio::test]
async fn test_then_pin_future_go_deeper() {
    use std::future::ready;

    let mut data = Wrapper {
        inner: WithPinnedBoxFuture {
            fut: Box::pin(ready(99)),
        },
    };

    // Navigate to inner field (sync), then await its #[pin] Future
    let kp = Wrapper::inner()
        .then_pin_future(WithPinnedBoxFuture::fut_pin_future_kp());

    let result = kp.get_mut(&mut data).await;
    assert_eq!(result, Some(99));
}

#[tokio::test]
async fn test_then_pin_future_get_optional_or_else() {
    use std::future::ready;

    let mut data = WithPinnedBoxFuture {
        fut: Box::pin(ready(21)),
    };

    let identity_kp: KpType<WithPinnedBoxFuture, WithPinnedBoxFuture> =
        Kp::new(|x: &WithPinnedBoxFuture| Some(x), |x: &mut WithPinnedBoxFuture| Some(x));
    let kp = identity_kp.then_pin_future(WithPinnedBoxFuture::fut_pin_future_kp());

    // get_optional
    assert!(kp.get_optional(None::<&WithPinnedBoxFuture>).await.is_none());
    assert_eq!(kp.get_optional(Some(&data)).await, None); // get returns None for pin future
    assert_eq!(kp.get_mut_optional(None::<&mut WithPinnedBoxFuture>).await, None);
    assert_eq!(kp.get_mut_optional(Some(&mut data)).await, Some(21));

    // get_or_else / get_mut_or_else
    assert_eq!(kp.get_or_else(None, || 0).await, 0);
    assert_eq!(kp.get_or_else(Some(&data), || 0).await, 0); // get is None so fallback
    assert_eq!(kp.get_mut_or_else(None, || 100).await, 100);

    // get_mut_or_else with Some uses a fresh future (previous get_mut consumed the first)
    let mut data2 = WithPinnedBoxFuture {
        fut: Box::pin(ready(77)),
    };
    assert_eq!(kp.get_mut_or_else(Some(&mut data2), || 100).await, 77);
}
