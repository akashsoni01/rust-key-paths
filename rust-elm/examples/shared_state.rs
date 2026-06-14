//! Shared<T> observable state without the full Runtime.
//!
//! ```bash
//! cargo run -p rust-elm --example shared_state
//! ```

use rust_elm::Shared;

fn main() {
    let count = Shared::new(0i32);
    let mut sub = count.subscribe();

    count.with_mut(|n| *n += 1);
    count.with_mut(|n| *n += 2);

    if let Some(v) = sub.next() {
        println!("snapshot after updates: {v}");
    }
    println!("current: {}", count.get());
}
