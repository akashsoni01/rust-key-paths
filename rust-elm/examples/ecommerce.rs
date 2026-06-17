//! End-to-end ecommerce store — demonstrates real-world composition of rust-elm.
#![allow(clippy::collapsible_if)]
//!
//! Features showcased:
//! - **4-level actions**: `Shop → Catalog → Browse → Product → Detail`
//! - **Child state variety**: scoped catalog/cart, optional checkout (`IfLetReducer`),
//!   identified wishlists (`ForEachReducer`), cart lines (`IdentifiedVec`)
//! - **Derived optics**: `Kp` / `Cp` keypaths for state and action scoping
//! - **Reducer stack**: 6-way `CombineReducers` + `CatchReducer` + scoped / ifLet / forEach children
//! - **Runtime**: bus-driven update loop, Tokio effect interpreter, `ScopedStore`, state subscription
//! - **Effects**: `StoreTask`, cancel-on-dismiss; **subscriptions**: tick, stream, websocket, map_msg, batch
//!
//! - **Environment / DI**: live vs mock HTTP + date + WebSocket deps via [`Environment::with`]
//! - **Panic demo**: `GlobalAction::TriggerPanic` (action path) and
//!   `GlobalAction::RunEffectPanicDemo` (effect path) — see [`book/ecommerce.md`](../book/ecommerce.md)
//!
//! ```bash
//! cargo run -p rust-elm --example ecommerce          # live + mock scenarios
//! ```

#[path = "ecommerce/deps.rs"]
mod deps;
#[path = "ecommerce/shop.rs"]
mod shop;

use shop::*;
use rust_elm::{Environment, ReducerProgram, Runtime, RuntimeConfig};
use std::time::Duration;

fn run_shop(label: &str, env: Environment) {
    println!("\n========== {label} environment ==========");

    let program = ReducerProgram::new(shop_reducer(), init, subscriptions);
    let runtime: Runtime<ShopState, ShopAction> =
        Runtime::from_reducer_program(program, env, RuntimeConfig::new(64));
    let store: rust_elm::Store<ShopState, ShopAction> = runtime.store();

    std::thread::sleep(Duration::from_millis(600));
    let boot = store.state();
    println!(
        "{label} boot: origin={:?} time={:?} ws={:?}",
        boot.session.http_origin, boot.session.server_time, boot.session.checkout_ws_url
    );

    let cart_store = store.scope(cart_lens(), ShopAction::cart_cp());

    store.dispatch(ShopAction::Global(GlobalAction::SignIn(
        "alex@shop.example".into(),
    )));
    std::thread::sleep(Duration::from_millis(600));

    let after_sign_in = store.state();
    println!(
        "{label} after sign-in: origin={:?} time={:?}",
        after_sign_in.session.http_origin, after_sign_in.session.server_time
    );

    println!("{label}: dispatching TriggerPanic (intentional)…");
    store.dispatch(ShopAction::Global(GlobalAction::TriggerPanic));
    std::thread::sleep(Duration::from_millis(50));
    let after_panic = store.state();
    println!(
        "{label} after action panic: panic_survived={} user={:?} (app continues)",
        after_panic.session.panic_survived,
        after_panic.session.user
    );

    println!("{label}: dispatching RunEffectPanicDemo (effect-body + effect→reducer panic)…");
    store.dispatch(ShopAction::Global(GlobalAction::RunEffectPanicDemo));
    std::thread::sleep(Duration::from_millis(150));
    let after_effect_panic = store.state();
    println!(
        "{label} after effect panic: body_armed={} effect_panic_survived={} user={:?} (app continues)",
        after_effect_panic.session.effect_body_armed,
        after_effect_panic.session.effect_panic_survived,
        after_effect_panic.session.user
    );

    store.dispatch(ShopAction::Global(GlobalAction::SeedWishlist));

    let mut catalog_sub = store.subscribe_state();

    store.dispatch(ShopAction::Catalog(CatalogAction::SetQuery(
        "hoodie".into(),
    )));
    std::thread::sleep(Duration::from_millis(80));

    let _ = store
        .send(ShopAction::Catalog(CatalogAction::Browse(
            BrowseAction::Product(ProductAction::Detail(DetailAction::Load {
                sku: "hoodie-blue".into(),
            })),
        )))
        .finish();

    if let Some(snapshot) = catalog_sub.wait_next(Duration::from_secs(2)) {
        if let Some(detail) = snapshot.catalog.detail.as_ref() {
            println!("catalog loaded: {:?} {:?}", detail.name, detail.price_cents);
        }
    }

    store.dispatch(ShopAction::Catalog(CatalogAction::Browse(
        BrowseAction::Product(ProductAction::Detail(DetailAction::AddToCart)),
    )));
    std::thread::sleep(Duration::from_millis(100));

    cart_store.dispatch(CartAction::Line(0, CartLineAction::Inc));
    store.dispatch(ShopAction::Cart(CartAction::Line(0, CartLineAction::Inc)));

    std::thread::sleep(Duration::from_millis(50));

    store.dispatch(ShopAction::Global(GlobalAction::StartCheckout));
    store.dispatch(ShopAction::Checkout(CheckoutAction::Pay));
    std::thread::sleep(Duration::from_millis(400));

    store.dispatch(ShopAction::WishlistRow(
        0,
        WishlistAction::AddSku("hoodie-blue".into()),
    ));

    let final_state = store.state();
    println!("\n--- {label} final ---");
    println!("user: {:?}", final_state.session.user);
    println!("panic_survived: {}", final_state.session.panic_survived);
    println!("effect_body_armed: {}", final_state.session.effect_body_armed);
    println!("effect_panic_survived: {}", final_state.session.effect_panic_survived);
    println!(
        "env: origin={:?} time={:?} ws={:?}",
        final_state.session.http_origin,
        final_state.session.server_time,
        final_state.session.checkout_ws_url
    );
    println!("cart lines: {}", final_state.cart.lines.len());
    println!(
        "checkout paid: {:?}",
        final_state.checkout.as_ref().map(|c| c.paid)
    );
    println!("wishlists: {}", final_state.wishlists.len());
    println!("subscription metrics: {:?}", final_state.metrics);

    runtime.shutdown();
}

fn main() {
    run_shop("live", deps::shop_environment_live());
    run_shop("mock", deps::shop_environment_mock());
    println!("\necommerce example OK");
}
