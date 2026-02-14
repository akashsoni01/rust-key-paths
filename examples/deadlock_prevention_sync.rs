//! Deadlock prevention with keypaths: sync/parallel execution.
//!
//! Demonstrates:
//! 1. **Eager Lock Release** - Lock, read, release; then lock next. Prevents deadlock.
//! 2. **Lock Ordering** - Acquire locks in consistent order (by account ID).
//! 3. **Snapshot Pattern** - Copy data out, release lock, process without holding locks.
//!
//! Run: `cargo run --example deadlock_prevention_sync --features parking_lot`

#![cfg(feature = "parking_lot")]

use key_paths_derive::Kp;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Kp)]
struct Account {
    balance: i32,
    id: u64,
}

#[derive(Clone, Kp)]
struct Bank {
    account1: Arc<parking_lot::Mutex<Account>>,
    account2: Arc<parking_lot::Mutex<Account>>,
}

fn main() {
    println!("=== Deadlock Prevention with Keypaths (Parallel) ===\n");

    let bank = Arc::new(Bank {
        account1: Arc::new(parking_lot::Mutex::new(Account {
            balance: 1000,
            id: 1,
        })),
        account2: Arc::new(parking_lot::Mutex::new(Account {
            balance: 2000,
            id: 2,
        })),
    });

    // --- 1. Eager Lock Release ---
    println!("--- 1. Eager Lock Release (no deadlock) ---");
    let bank1 = bank.clone();
    let bank2 = bank.clone();

    let handle1 = thread::spawn(move || {
        // Lock acc1, read balance, RELEASE (scope ends)
        let balance1 = {
            let acc1 = Bank::account1_lock().get(&*bank1).unwrap();
            acc1.balance
        };
        thread::sleep(Duration::from_millis(5));
        // Now lock acc2 independently (use set() — works with &Bank, get_mut needs &mut Bank)
        Bank::account2_lock()
            .set(&*bank1, |acc| acc.balance += balance1)
            .unwrap();
        let acc2_bal = Bank::account2_lock().get(&*bank1).unwrap().balance;
        println!("  Thread 1: Transfer 1->2 done, acc2.balance = {}", acc2_bal);
    });

    let handle2 = thread::spawn(move || {
        let balance2 = {
            let acc2 = Bank::account2_lock().get(&*bank2).unwrap();
            acc2.balance
        };
        thread::sleep(Duration::from_millis(5));
        Bank::account1_lock()
            .set(&*bank2, |acc| acc.balance += balance2)
            .unwrap();
        let acc1_bal = Bank::account1_lock().get(&*bank2).unwrap().balance;
        println!("  Thread 2: Transfer 2->1 done, acc1.balance = {}", acc1_bal);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
    println!("  ✓ No deadlock (eager release)\n");

    // --- 2. Lock Ordering ---
    println!("--- 2. Lock Ordering (consistent order by ID) ---");
    let bank3 = bank.clone();
    let bank4 = bank.clone();

    let handle3 = thread::spawn(move || {
        // Always acquire lower ID first — update in consistent order
        let id1 = Bank::account1_lock().get(&*bank3).unwrap().id;
        let id2 = Bank::account2_lock().get(&*bank3).unwrap().id;
        if id1 < id2 {
            Bank::account1_lock().set(&*bank3, |acc| acc.balance += 10).unwrap();
            Bank::account2_lock().set(&*bank3, |acc| acc.balance -= 10).unwrap();
        } else {
            Bank::account2_lock().set(&*bank3, |acc| acc.balance -= 10).unwrap();
            Bank::account1_lock().set(&*bank3, |acc| acc.balance += 10).unwrap();
        }
        println!("  Thread 3: Ordered transfer done");
    });

    let handle4 = thread::spawn(move || {
        let id1 = Bank::account1_lock().get(&*bank4).unwrap().id;
        let id2 = Bank::account2_lock().get(&*bank4).unwrap().id;
        if id1 < id2 {
            Bank::account1_lock().set(&*bank4, |acc| acc.balance -= 5).unwrap();
            Bank::account2_lock().set(&*bank4, |acc| acc.balance += 5).unwrap();
        } else {
            Bank::account2_lock().set(&*bank4, |acc| acc.balance += 5).unwrap();
            Bank::account1_lock().set(&*bank4, |acc| acc.balance -= 5).unwrap();
        }
        println!("  Thread 4: Ordered transfer done");
    });

    handle3.join().unwrap();
    handle4.join().unwrap();
    println!("  ✓ No deadlock (lock ordering)\n");

    // --- 3. Snapshot Pattern ---
    println!("--- 3. Snapshot Pattern ---");
    let bank5 = bank.clone();

    let handle5 = thread::spawn(move || {
        let b1 = Bank::account1_lock().get(&*bank5).map(|a| a.balance);
        let b2 = Bank::account2_lock().get(&*bank5).map(|a| a.balance);
        if let (Some(b1), Some(b2)) = (b1, b2) {
            println!("  Thread 5: Snapshot acc1={}, acc2={}", b1, b2);
            Bank::account2_lock()
                .set(&*bank5, |acc| acc.balance += 100)
                .unwrap();
        }
    });

    handle5.join().unwrap();
    println!("  ✓ Snapshot done\n");

    println!("=== All parallel patterns completed ===");
}
