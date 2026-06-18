//! Ecommerce shop on [`TeaRuntime`] — true Elm Architecture with channel-delivered model.
//!
//! Showcases:
//! - **`TeaRuntime`** — model owned only on reducer thread (no shared `Mutex` / `RwLock` / `ArcSwap`)
//! - **`TeaStore::subscribe_state()`** — reducer **pushes** `Arc<ShopState>` after each update
//! - **`TeaViewStore`** — reader threads receive snapshots via channel (no lock on read path)
//! - **`TeaStore`** — dispatch, `ScopedTeaStore`, `send`
//!
//! ```bash
//! cargo run -p rust-elm --example tea_ecommerce
//! ```
//!
//! See [`book/tea_ecommerce.md`](../book/tea_ecommerce.md) for architecture diagrams.

#[path = "ecommerce/deps.rs"]
mod deps;
#[path = "ecommerce/shop.rs"]
mod shop;

use shop::*;
use rust_elm::{start_tea_reducer_runtime, Environment, ReducerProgram, RuntimeConfig, TeaRuntime};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Spawn N reader threads that receive catalog query updates via pushed model snapshots.
fn spawn_catalog_readers(
    view: rust_elm::TeaViewStore<ShopState, ShopAction>,
    stop: Arc<AtomicBool>,
    read_count: Arc<AtomicU32>,
    threads: usize,
) -> Vec<thread::JoinHandle<()>> {
    (0..threads)
        .map(|i| {
            let view = view.clone();
            let stop = Arc::clone(&stop);
            let read_count = Arc::clone(&read_count);
            thread::Builder::new()
                .name(format!("catalog-tea-view-{i}"))
                .spawn(move || {
                    let mut sub = match view.subscribe_state() {
                        Ok(sub) => sub,
                        Err(err) => panic!("subscribe_state failed: {err}"),
                    };
                    while !stop.load(Ordering::Relaxed) {
                        if let Some(snap) = sub.wait_next(Duration::from_millis(50)) {
                            let q = snap.catalog.query.clone();
                            if !q.is_empty() {
                                read_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                })
                .expect("spawn catalog tea view reader")
        })
        .collect()
}

fn run_tea_shop(label: &str, env: Environment) {
    println!("\n========== {label} (TeaRuntime) ==========");

    let program = ReducerProgram::new(shop_reducer(), init, subscriptions);
    let runtime: TeaRuntime<ShopState, ShopAction> =
        start_tea_reducer_runtime(program, env, RuntimeConfig::new(64));

    let store = runtime.tea_store();
    let view = store.view_store();

    let stop_readers = Arc::new(AtomicBool::new(false));
    let catalog_reads = Arc::new(AtomicU32::new(0));
    let reader_handles = spawn_catalog_readers(
        view.clone(),
        Arc::clone(&stop_readers),
        Arc::clone(&catalog_reads),
        4,
    );

    thread::sleep(Duration::from_millis(600));
    let (boot_user, boot_origin) = match view.with_snapshot(|s| {
        (s.session.user.clone(), s.session.http_origin.clone())
    }) {
        Ok(v) => v,
        Err(err) => panic!("snapshot read failed: {err}"),
    };
    println!("{label} boot: user={boot_user:?} origin={boot_origin:?}");

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

    let detail_name = view
        .with_snapshot(|s| {
            s.catalog
                .detail
                .as_ref()
                .and_then(|d| d.name.clone())
        })
        .ok();
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
    let (lines, paid, metrics) = view
        .with_snapshot(|s| {
            (
                s.cart.lines.len(),
                s.checkout.as_ref().map(|c| c.paid),
                s.metrics.clone(),
            )
        })
        .unwrap_or((0, None, Default::default()));

    println!("\n--- {label} final (channel-delivered model) ---");
    println!("catalog snapshot pushes observed (non-empty query): {reads}");
    println!("cart lines: {lines}");
    println!("checkout paid: {paid:?}");
    println!("subscription metrics: {metrics:?}");

    runtime.shutdown();
}

fn main() {
    run_tea_shop("live", deps::shop_environment_live());
    run_tea_shop("mock", deps::shop_environment_mock());
    println!("\ntea_ecommerce example OK");
}
