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

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Races two futures with fair polling: alternates which future gets polled first
/// to avoid starvation. Completes with the first result.
#[pin_project]
#[derive(Kp)]
pub struct FairRaceFuture {
    /// If true, poll fut1 first; otherwise poll fut2 first. Toggled each poll.
    pub fair: bool,
    #[pin]
    fut1: BoxFuture<String>,
    #[pin]
    fut2: BoxFuture<String>,
}

impl FairRaceFuture {
    pub fn new<F1, F2>(fut1: F1, fut2: F2) -> Self
    where
        F1: Future<Output = String> + Send + 'static,
        F2: Future<Output = String> + Send + 'static,
    {
        Self {
            fair: true,
            fut1: Box::pin(fut1),
            fut2: Box::pin(fut2),
        }
    }
}

impl Future for FairRaceFuture {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

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

async fn labeled_sleep(ms: u64, label: &'static str) -> String {
    sleep(Duration::from_millis(ms)).await;
    format!("{} completed", label)
}

#[tokio::main]
async fn main() {
    println!("=== FairRaceFuture: Fair alternation between two futures ===\n");

    // Scenario: Two tasks, one fast (50ms) and one slow (150ms).
    let fast = labeled_sleep(50, "fast");
    let slow = labeled_sleep(150, "slow");

    let race = FairRaceFuture::new(fast, slow);

    // Introspection via Kp: inspect the fair flag before polling
    let fair_kp = FairRaceFuture::fair();
    println!("  Initial fair flag: {:?}", fair_kp.get(&race));

    let result = race.await;
    println!("  Race result: {:?}", result);
    assert_eq!(result, "fast completed");

    // Toggle fair via keypath to demonstrate mutable access
    let fast2 = labeled_sleep(30, "A");
    let slow2 = labeled_sleep(100, "B");
    let mut race2 = FairRaceFuture::new(fast2, slow2);

    if let Some(f) = FairRaceFuture::fair().get_mut(&mut race2) {
        *f = false;
    }
    let result2 = race2.await;
    println!("  Second race (fair=false initially): {:?}", result2);
    assert_eq!(result2, "A completed");

    // Keypath introspection before the race
    let race3 = FairRaceFuture::new(
        labeled_sleep(10, "left"),
        labeled_sleep(20, "right"),
    );
    println!("\n  Before race, fair flag: {:?}", fair_kp.get(&race3));
    let result3 = race3.await;
    println!("  Third race result: {:?}", result3);

    println!("\n=== FairRaceFuture example completed ===");
}
