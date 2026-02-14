//! Deadlock prevention with keypaths: async/parallel execution (Tokio).
//!
//! Demonstrates the same patterns as deadlock_prevention_sync, but with:
//! - Arc<tokio::sync::Mutex<T>> instead of parking_lot::Mutex
//! - tokio::spawn for concurrent tasks
//! - .get().await, .set().await for async lock access
//!
//! Run: `cargo run --example deadlock_prevention_async --features tokio`

#![cfg(feature = "tokio")]

use key_paths_derive::Kp;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Clone, Kp)]
struct Account {
    balance: i32,
    id: u64,
}

#[derive(Clone, Kp)]
struct Bank {
    account1: Arc<tokio::sync::Mutex<Account>>,
    account2: Arc<tokio::sync::Mutex<Account>>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("=== Deadlock Prevention with Keypaths (Async, Parallel) ===\n");

    let bank = Arc::new(Bank {
        account1: Arc::new(tokio::sync::Mutex::new(Account {
            balance: 1000,
            id: 1,
        })),
        account2: Arc::new(tokio::sync::Mutex::new(Account {
            balance: 2000,
            id: 2,
        })),
    });

    // --- 1. Eager Lock Release ---
    println!("--- 1. Eager Lock Release (no deadlock) ---");
    let bank1 = bank.clone();
    let bank2 = bank.clone();

    let handle1 = tokio::spawn(async move {
        // Lock acc1, read balance, RELEASE (scope ends)
        let balance1 = {
            let acc1 = Bank::account1_async().get(&*bank1).await.unwrap();
            acc1.balance
        };
        sleep(Duration::from_millis(5)).await;
        // Now lock acc2 independently (use set — works with &Bank)
        Bank::account2_async()
            .set(&*bank1, |acc| acc.balance += balance1)
            .await
            .unwrap();
        let acc2_bal = Bank::account2_async().get(&*bank1).await.unwrap().balance;
        println!("  Task 1: Transfer 1->2 done, acc2.balance = {}", acc2_bal);
    });

    let handle2 = tokio::spawn(async move {
        let balance2 = {
            let acc2 = Bank::account2_async().get(&*bank2).await.unwrap();
            acc2.balance
        };
        sleep(Duration::from_millis(5)).await;
        Bank::account1_async()
            .set(&*bank2, |acc| acc.balance += balance2)
            .await
            .unwrap();
        let acc1_bal = Bank::account1_async().get(&*bank2).await.unwrap().balance;
        println!("  Task 2: Transfer 2->1 done, acc1.balance = {}", acc1_bal);
    });

    handle1.await.unwrap();
    handle2.await.unwrap();
    println!("  ✓ No deadlock (eager release)\n");

    // --- 2. Lock Ordering ---
    println!("--- 2. Lock Ordering (consistent order by ID) ---");
    let bank3 = bank.clone();
    let bank4 = bank.clone();

    let handle3 = tokio::spawn(async move {
        let id1 = Bank::account1_async().get(&*bank3).await.unwrap().id;
        let id2 = Bank::account2_async().get(&*bank3).await.unwrap().id;
        if id1 < id2 {
            Bank::account1_async()
                .set(&*bank3, |acc| acc.balance += 10)
                .await
                .unwrap();
            Bank::account2_async()
                .set(&*bank3, |acc| acc.balance -= 10)
                .await
                .unwrap();
        } else {
            Bank::account2_async()
                .set(&*bank3, |acc| acc.balance -= 10)
                .await
                .unwrap();
            Bank::account1_async()
                .set(&*bank3, |acc| acc.balance += 10)
                .await
                .unwrap();
        }
        println!("  Task 3: Ordered transfer done");
    });

    let handle4 = tokio::spawn(async move {
        let id1 = Bank::account1_async().get(&*bank4).await.unwrap().id;
        let id2 = Bank::account2_async().get(&*bank4).await.unwrap().id;
        if id1 < id2 {
            Bank::account1_async()
                .set(&*bank4, |acc| acc.balance -= 5)
                .await
                .unwrap();
            Bank::account2_async()
                .set(&*bank4, |acc| acc.balance += 5)
                .await
                .unwrap();
        } else {
            Bank::account2_async()
                .set(&*bank4, |acc| acc.balance += 5)
                .await
                .unwrap();
            Bank::account1_async()
                .set(&*bank4, |acc| acc.balance -= 5)
                .await
                .unwrap();
        }
        println!("  Task 4: Ordered transfer done");
    });

    handle3.await.unwrap();
    handle4.await.unwrap();
    println!("  ✓ No deadlock (lock ordering)\n");

    // --- 3. Snapshot Pattern ---
    println!("--- 3. Snapshot Pattern ---");
    let bank5 = bank.clone();

    let handle5 = tokio::spawn(async move {
        let b1 = Bank::account1_async()
            .get(&*bank5)
            .await
            .map(|a| a.balance);
        let b2 = Bank::account2_async()
            .get(&*bank5)
            .await
            .map(|a| a.balance);
        if let (Some(b1), Some(b2)) = (b1, b2) {
            println!("  Task 5: Snapshot acc1={}, acc2={}", b1, b2);
            Bank::account2_async()
                .set(&*bank5, |acc| acc.balance += 100)
                .await
                .unwrap();
        }
    });

    handle5.await.unwrap();
    println!("  ✓ Snapshot done\n");

    println!("=== All async parallel patterns completed ===");
}
