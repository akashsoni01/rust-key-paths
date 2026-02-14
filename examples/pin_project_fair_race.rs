//! Complex example: FairRaceFuture – fair alternation between two futures
//!
//! **Use case:** When racing two async operations (e.g. two API calls, two network
//! requests), a naive `select!` can starve one future if the other keeps waking.
//! FairRaceFuture alternates which future is polled first, giving both a fair chance.
//!
//! Demonstrates:
//! - #[pin] on Future fields (pin_project pattern)
//! - Implementing Future with pin_project projections
//! - Using Kp-derived accessors for introspection (fair flag, field access)
//!
//! Run: `cargo run --example pin_project_fair_race --features "pin_project,tokio"`

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use key_paths_derive::Kp;
use pin_project::pin_project;
use tokio::time::{sleep, Duration};

/// Races two futures with fair polling: alternates which future gets polled first
/// to avoid starvation. Completes with the first result.
#[pin_project]
#[derive(Kp)]
pub struct FairRaceFuture<F1, F2>
where
    F1: Future,
    F2: Future<Output = F1::Output>,
{
    /// If true, poll fut1 first; otherwise poll fut2 first. Toggled each poll.
    pub fair: bool,
    #[pin]
    fut1: F1,
    #[pin]
    fut2: F2,
}

impl<F1, F2> FairRaceFuture<F1, F2>
where
    F1: Future,
    F2: Future<Output = F1::Output>,
{
    pub fn new(fut1: F1, fut2: F2) -> Self {
        Self {
            fair: true,
            fut1,
            fut2,
        }
    }
}

impl<F1, F2> Future for FairRaceFuture<F1, F2>
where
    F1: Future,
    F2: Future<Output = F1::Output>,
{
    type Output = F1::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        // Use the fair flag to alternate: whichever we poll second gets priority next time
        if *this.fair {
            *this.fair = false;
            if let Poll::Ready(v) = this.fut1.as_mut().poll(cx) {
                return Poll::Ready(v);
            }
            if let Poll::Ready(v) = this.fut2.poll(cx) {
                return Poll::Ready(v);
            }
        } else {
            *this.fair = true;
            if let Poll::Ready(v) = this.fut2.as_mut().poll(cx) {
                return Poll::Ready(v);
            }
            if let Poll::Ready(v) = this.fut1.poll(cx) {
                return Poll::Ready(v);
            }
        }
        Poll::Pending
    }
}

/// Wraps a future with a label for demo output.
async fn labeled_sleep(ms: u64, label: &'static str) -> String {
    sleep(Duration::from_millis(ms)).await;
    format!("{} completed", label)
}

#[tokio::main]
async fn main() {
    println!("=== FairRaceFuture: Fair alternation between two futures ===\n");

    // Scenario: Two tasks, one fast (50ms) and one slow (150ms).
    // With fair polling, both get CPU time; the fast one still wins the race.
    let fast = labeled_sleep(50, "fast");
    let slow = labeled_sleep(150, "slow");

    let mut race = FairRaceFuture::new(fast, slow);

    // Introspection via Kp: inspect the fair flag before polling
    let fair_kp = FairRaceFuture::<_, _>::fair();
    println!("  Initial fair flag: {:?}", fair_kp.get(&race));

    // Run the race – the 50ms future should complete first
    let result = race.await;
    println!("  Race result: {:?}", result);
    assert_eq!(result, "fast completed");

    // Same setup but reverse order – fair flag affects first poll
    let fast2 = labeled_sleep(30, "A");
    let slow2 = labeled_sleep(100, "B");
    let mut race2 = FairRaceFuture::new(fast2, slow2);

    // Toggle fair to demonstrate it affects polling order
    if let Some(f) = FairRaceFuture::fair().get_mut(&mut race2) {
        *f = false; // Start by polling fut2 first this time
    }
    let result2 = race2.await;
    println!("  Second race (fair=false initially): {:?}", result2);
    assert_eq!(result2, "A completed"); // A is still faster

    // Keypath introspection: inspect fair flag before the race
    let mut race3 = FairRaceFuture::new(
        labeled_sleep(10, "left"),
        labeled_sleep(20, "right"),
    );
    let fair_kp = FairRaceFuture::<_, _>::fair();
    println!("\n  Before race, fair flag: {:?}", fair_kp.get(&race3));
    let result3 = race3.await;
    println!("  Third race result: {:?}", result3);

    println!("\n=== FairRaceFuture example completed ===");
}
