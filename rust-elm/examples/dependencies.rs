//! Environment / dependency injection — live vs test UUID + RNG.
//!
//! ```bash
//! cargo run -p rust-elm --example dependencies
//! ```

use rust_elm::{DependencyKey, Environment, RngDep, RngKey, UuidDep, UuidKey};

fn main() {
    let live = Environment::live();
    let test = Environment::test();

    let live_uuid = live.require::<UuidDep>().unwrap().0.next();
    let test_uuid = test.require::<UuidDep>().unwrap().0.next();
    let test_uuid2 = Environment::test()
        .require::<UuidDep>()
        .unwrap()
        .0
        .next();

    let live_rng = live.require::<RngDep>().unwrap().0.next_u64();
    let test_rng = test.require::<RngDep>().unwrap().0.next_u64();

    println!("live uuid: {live_uuid}");
    println!("test uuid: {test_uuid} (repeatable: {test_uuid2})");
    println!("live rng: {live_rng}");
    println!("test rng: {test_rng}");
    let _ = (UuidKey::live(), RngKey::live());
}
