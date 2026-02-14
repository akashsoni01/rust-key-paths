//! Pin-project keypath support: #[pin] Future field await.
//!
//! Enables composing `fut_await` with [crate::Kp::then_pin_future], e.g.:
//!   `kp.then_pin_future(WithPinnedBoxFuture::fut_pin_future_kp())`
//!
//! Requires: `#[pin_project]` `#[derive(Kp)]` struct with `#[pin] fut: Pin<Box<dyn Future>>`

use async_trait::async_trait;
use std::pin::Pin;

use crate::async_lock::SyncKeyPathLike;

/// Trait for awaiting a #[pin] Future field via Pin<&mut S>. Implemented by [pin_future_await_kp!]
/// and by the Kp derive for `{field}_pin_future_kp()` methods.
#[async_trait(?Send)]
pub trait PinFutureAwaitLike<S, Output> {
    /// Await the pinned future. Requires `Pin<&mut S>` (use via [crate::Kp::then_pin_future]).
    async fn get_await(&self, this: Pin<&mut S>) -> Option<Output>;
}

/// Async keypath that awaits a #[pin] Future field. Use [pin_future_await_kp!] or the derived
/// `{field}_pin_future_kp()` to construct.
#[derive(Clone)]
pub struct PinFutureAwaitKp<S, Output, L>
where
    L: Clone,
{
    inner: L,
    _p: std::marker::PhantomData<fn() -> (S, Output)>,
}

impl<S, Output, L> PinFutureAwaitKp<S, Output, L>
where
    L: PinFutureAwaitLike<S, Output> + Clone,
{
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            _p: std::marker::PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<S, Output, L> PinFutureAwaitLike<S, Output> for PinFutureAwaitKp<S, Output, L>
where
    L: PinFutureAwaitLike<S, Output> + Sync + Clone,
{
    async fn get_await(&self, this: Pin<&mut S>) -> Option<Output> {
        self.inner.get_await(this).await
    }
}

/// Macro to create a [PinFutureAwaitKp] from a derived `field_await` function.
///
/// # Example
/// ```ignore
/// #[pin_project]
/// #[derive(Kp)]
/// struct WithPinnedBoxFuture {
///     #[pin] fut: Pin<Box<dyn Future<Output = i32> + Send>>,
/// }
///
/// let kp = Kp::identity::<WithPinnedBoxFuture>()
///     .then_pin_future(pin_future_await_kp!(WithPinnedBoxFuture, fut_await -> i32));
/// let result = kp.get_mut(&mut data).await;  // Option<i32>
/// ```
#[macro_export]
macro_rules! pin_future_await_kp {
    ($ty:ty, $method:ident -> $output:ty) => {{
        #[derive(Clone, Copy)]
        struct KpImpl;
        #[::async_trait::async_trait(?Send)]
        impl $crate::pin::PinFutureAwaitLike<$ty, $output> for KpImpl {
            async fn get_await(&self, this: std::pin::Pin<&mut $ty>) -> Option<$output> {
                <$ty>::$method(this).await
            }
        }
        $crate::pin::PinFutureAwaitKp::new(KpImpl)
    }};
}

/// Keypath that chains a sync [crate::Kp] with a [PinFutureAwaitKp]. Use [crate::Kp::then_pin_future] to create.
/// Enables: `kp.then_pin_future(...).get_mut(&mut root).await` to await #[pin] Future fields.
#[derive(Clone)]
pub struct KpThenPinFuture<R, S, Output, Root, MutRoot, Value, MutValue, First, Second> {
    pub(crate) first: First,
    pub(crate) second: Second,
    pub(crate) _p: std::marker::PhantomData<(R, S, Output, Root, MutRoot, Value, MutValue)>,
}

impl<R, S, Output, Root, MutRoot, Value, MutValue, First, Second>
    KpThenPinFuture<R, S, Output, Root, MutRoot, Value, MutValue, First, Second>
where
    S: Unpin,
    Output: 'static,
    MutValue: std::borrow::BorrowMut<S>,
    First: SyncKeyPathLike<Root, Value, MutRoot, MutValue>,
    Second: PinFutureAwaitLike<S, Output>,
{
    /// Get through the chain. For pin futures, [get] returns `None` (await requires mutable access).
    pub async fn get(&self, _root: Root) -> Option<Output> {
        None
    }

    /// Get mutable through the chain: sync navigate to &mut S, then await the pinned future.
    pub async fn get_mut(&self, root: MutRoot) -> Option<Output> {
        let mut mut_value = self.first.sync_get_mut(root)?;
        let s: &mut S = mut_value.borrow_mut();
        self.second.get_await(Pin::new(s)).await
    }
}
