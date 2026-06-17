//! Ecommerce shop on [`RwRuntime`] — same domain as [`ecommerce`](ecommerce.rs), read-heavy store API.
//!
//! Showcases:
//! - **`RwRuntime`** — `Arc<RwLock<ShopState>>` instead of `Mutex`
//! - **`RwStore::read_store()`** — concurrent readers (`with_read`, `read_binding`)
//! - **`RwStore`** — dispatch, `ScopedRwStore`, `send`
//! - **Zero-copy reads** — reader threads borrow via `ReadStateBinding` (no `state()` clone)
//!
//! ```bash
//! cargo run -p rust-elm --example rw_ecommerce
//! ```
//!
//! See [`book/rw_ecommerce.md`](../book/rw_ecommerce.md) for architecture diagrams.

#[path = "ecommerce/deps.rs"]
mod deps;
#[path = "ecommerce/shop.rs"]
mod shop;

use shop::*;
use rust_elm::{Environment, ReducerProgram, RuntimeConfig, RwRuntime};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Spawn N reader threads that poll catalog query via `store.read_store()` (concurrent `read()` locks).
fn spawn_catalog_readers(
    store: rust_elm::RwStore<ShopState, ShopAction>,
    stop: Arc<AtomicBool>,
    read_count: Arc<AtomicU32>,
    threads: usize,
) -> Vec<thread::JoinHandle<()>> {
    (0..threads)
        .map(|i| {
            let read_store = store.read_store();
            let stop = Arc::clone(&stop);
            let read_count = Arc::clone(&read_count);
            thread::Builder::new()
                .name(format!("catalog-reader-{i}"))
                .spawn(move || {
                    let binding = read_store.read_binding().project(ShopState::catalog());
                    while !stop.load(Ordering::Relaxed) {
                        if let Some(q) = binding.with_read(|c| c.query.clone()) {
                            if !q.is_empty() {
                                read_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        thread::sleep(Duration::from_millis(8));
                    }
                })
                .expect("spawn catalog reader")
        })
        .collect()
}

fn run_rw_shop(label: &str, env: Environment) {
    println!("\n========== {label} (RwRuntime) ==========");

    let program = ReducerProgram::new(shop_reducer(), init, subscriptions);
    let runtime: RwRuntime<ShopState, ShopAction> =
        RwRuntime::from_reducer_program(program, env, RuntimeConfig::new(64));

    let store = runtime.rw_store();

    let stop_readers = Arc::new(AtomicBool::new(false));
    let catalog_reads = Arc::new(AtomicU32::new(0));
    let reader_handles = spawn_catalog_readers(
        store.clone(),
        Arc::clone(&stop_readers),
        Arc::clone(&catalog_reads),
        4,
    );

    thread::sleep(Duration::from_millis(600));
    let (boot_user, boot_origin) = store.read_store().with_read(|s| {
        (s.session.user.clone(), s.session.http_origin.clone())
    });
    println!(
        "{label} boot: user={boot_user:?} origin={boot_origin:?}"
    );

    let cart_scope = store.scope(cart_lens(), ShopAction::cart_cp());

    store.dispatch(ShopAction::Global(GlobalAction::SignIn(
        "alex@shop.example".into(),
    )));
    thread::sleep(Duration::from_millis(600));

    store.dispatch(ShopAction::Global(GlobalAction::SeedWishlist));
    store.dispatch(ShopAction::Catalog(CatalogAction::SetQuery(
        "hoodie".into(),
    )));
    thread::sleep(Duration::from_millis(120));

    let _ = store
        .send(ShopAction::Catalog(CatalogAction::Browse(
            BrowseAction::Product(ProductAction::Detail(DetailAction::Load {
                sku: "hoodie-blue".into(),
            })),
        )))
        .finish();

    let detail_name = store.read_store().with_read(|s| {
        s.catalog
            .detail
            .as_ref()
            .and_then(|d| d.name.clone())
    });
    println!("{label} catalog loaded: {detail_name:?}");

    store.dispatch(ShopAction::Catalog(CatalogAction::Browse(
        BrowseAction::Product(ProductAction::Detail(DetailAction::AddToCart)),
    )));
    thread::sleep(Duration::from_millis(80));

    cart_scope.dispatch(CartAction::Line(0, CartLineAction::Inc));
    store.dispatch(ShopAction::Cart(CartAction::Line(0, CartLineAction::Inc)));

    store.dispatch(ShopAction::Global(GlobalAction::StartCheckout));
    store.dispatch(ShopAction::Checkout(CheckoutAction::Pay));
    thread::sleep(Duration::from_millis(400));

    store.dispatch(ShopAction::WishlistRow(
        0,
        WishlistAction::AddSku("hoodie-blue".into()),
    ));

    stop_readers.store(true, Ordering::Relaxed);
    for handle in reader_handles {
        let _ = handle.join();
    }

    let reads = catalog_reads.load(Ordering::Relaxed);
    let (lines, paid, metrics) = store.read_store().with_read(|s| {
        (
            s.cart.lines.len(),
            s.checkout.as_ref().map(|c| c.paid),
            s.metrics.clone(),
        )
    });

    println!("\n--- {label} final (zero-copy read) ---");
    println!("concurrent catalog reader polls (non-empty query): {reads}");
    println!("cart lines: {lines}");
    println!("checkout paid: {paid:?}");
    println!("subscription metrics: {metrics:?}");

    runtime.shutdown();
}

fn main() {
    run_rw_shop("live", deps::shop_environment_live());
    run_rw_shop("mock", deps::shop_environment_mock());
    println!("\nrw_ecommerce example OK");
}
