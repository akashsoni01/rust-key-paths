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
//! - **Environment / DI**: live vs mock HTTP + date deps via [`Environment::with`]
//! - **Panic demo**: `GlobalAction::TriggerPanic` — see [`book/ecommerce.md`](../book/ecommerce.md)
//!
//! ```bash
//! cargo run -p rust-elm --example ecommerce          # live + mock scenarios
//! ```

#[path = "ecommerce/deps.rs"]
mod deps;

use deps::{DateDep, HttpDep, HTTPBIN_GET};
use key_paths_derive::{Cp, Kp};
use rust_elm::{
    CatchReducer, Cmd, CombineReducers, Effect, EffectError, EffectId, Environment,
    ForEachReducer, Identifiable, IdentifiedVec, IfLetReducer, Reducer, ReducerProgram, Reduce,
    Runtime, ScopeReducer, Sub,
};
use rust_key_paths::Kp as KpPath;
use std::time::Duration;

// ── Effect ids (cancel / debounce / task identity) ─────────────────────────

const CATALOG_CANCEL: EffectId = 2001;
const CART_CANCEL: EffectId = 2002;
const CHECKOUT_CANCEL: EffectId = 2003;

const SUB_SESSION_TICK: u64 = 8001;
const SUB_CATALOG_STREAM: u64 = 8002;
const SUB_CHECKOUT_WS: u64 = 8003;
const SUB_MAPMSG_UNIT: u64 = 8004;

// ── State ────────────────────────────────────────────────────────────────────

type LineId = u64;
type WishlistId = u64;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct SessionState {
    user: Option<String>,
    http_origin: Option<String>,
    server_time: Option<String>,
    /// Set `true` immediately before the demo panic — proves partial state is kept.
    panic_survived: bool,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct ProductDetailState {
    sku: String,
    name: Option<String>,
    price_cents: Option<u32>,
    loading: bool,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct CatalogState {
    query: String,
    detail: Option<ProductDetailState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CartLine {
    id: LineId,
    sku: String,
    qty: u32,
}

impl Identifiable for CartLine {
    type Id = LineId;
    fn id(&self) -> LineId {
        self.id
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct CartState {
    lines: IdentifiedVec<LineId, CartLine>,
    next_line_id: LineId,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct CheckoutState {
    payment_pending: bool,
    paid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WishlistState {
    id: WishlistId,
    name: String,
    skus: Vec<String>,
}

impl Identifiable for WishlistState {
    type Id = WishlistId;
    fn id(&self) -> WishlistId {
        self.id
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct SubscriptionMetrics {
    session_ticks: u32,
    catalog_stream_pulses: u32,
    checkout_ws_pulses: u32,
    map_pings: u32,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Kp)]
struct ShopState {
    session: SessionState,
    catalog: CatalogState,
    cart: CartState,
    checkout: Option<CheckoutState>,
    wishlists: IdentifiedVec<WishlistId, WishlistState>,
    next_wishlist_id: WishlistId,
    metrics: SubscriptionMetrics,
}

// ── Actions (4 nested levels under Catalog) ──────────────────────────────────

#[allow(dead_code)] // demo-only variants for API illustration
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum ShopAction {
    Global(GlobalAction),
    Catalog(CatalogAction),
    Cart(CartAction),
    Checkout(CheckoutAction),
    WishlistRow(WishlistId, WishlistAction),
    RemoveWishlist(WishlistId),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum GlobalAction {
    SignIn(String),
    StartCheckout,
    SeedWishlist,
    SessionTick,
    SubscriptionPing,
    EnvProbe,
    /// Intentionally panics after mutating `session.panic_survived` (demo only).
    TriggerPanic,
    EnvLoaded {
        origin: String,
        server_time: String,
    },
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum CatalogAction {
    SetQuery(String),
    Browse(BrowseAction),
    StreamPulse,
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum BrowseAction {
    Product(ProductAction),
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum ProductAction {
    Detail(DetailAction),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum DetailAction {
    Load { sku: String },
    Loaded { name: String, price_cents: u32 },
    AddToCart,
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum CartAction {
    AddLine { sku: String, qty: u32 },
    Line(LineId, CartLineAction),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp)]
enum CartLineAction {
    Inc,
    Dec,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum CheckoutAction {
    Dismiss,
    Pay,
    PaymentDone,
    WsPulse,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp)]
enum WishlistAction {
    Rename(String),
    AddSku(String),
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn mock_catalog(sku: &str) -> (&'static str, u32) {
    match sku {
        "hoodie-blue" => ("Blue Hoodie", 4999),
        "mug-rust" => ("Rust Mug", 1299),
        _ => ("Unknown SKU", 999),
    }
}

fn env_probe_effect() -> Effect<ShopAction> {
    Effect::from_env_fn(|env| {
        let http = match env.require::<HttpDep>() {
            Ok(h) => h,
            Err(e) => {
                return Box::pin(async move { Err(EffectError::EnvMissing(e.name)) });
            }
        };
        let date = match env.require::<DateDep>() {
            Ok(d) => d,
            Err(e) => {
                return Box::pin(async move { Err(EffectError::EnvMissing(e.name)) });
            }
        };

        Box::pin(async move {
            let bin = http
                .0
                .get_request(HTTPBIN_GET.to_string())
                .await
                .map_err(EffectError::Other)?;
            let now = date.0.current().await;
            println!("date = {now:?}");

            Ok(ShopAction::Global(GlobalAction::EnvLoaded {
                origin: bin.origin,
                server_time: now.to_rfc3339(),
            }))
        })
    })
}

// ── Child reducers ───────────────────────────────────────────────────────────

/// Subscription pulses arrive on child action paths; metrics live on root state.
fn subscription_metrics_reducer(state: &mut ShopState, action: ShopAction) -> Cmd<ShopAction> {
    match action {
        ShopAction::Catalog(CatalogAction::StreamPulse) => {
            state.metrics.catalog_stream_pulses += 1;
            println!(
                "subscription_metrics: catalog_stream_pulses = {:?}",
                state.metrics.catalog_stream_pulses
            );
            Cmd::none()
        }
        ShopAction::Checkout(CheckoutAction::WsPulse) => {
            state.metrics.checkout_ws_pulses += 1;
            println!(
                "subscription_metrics: checkout_ws_pulses = {:?}",
                state.metrics.checkout_ws_pulses
            );
            Cmd::none()
        }
        _ => Cmd::none(),
    }
}

/// Cross-scope bridge: product detail "add to cart" updates cart in the same reduce turn.
fn detail_cart_bridge_reducer(state: &mut ShopState, action: ShopAction) -> Cmd<ShopAction> {
    match action {
        ShopAction::Catalog(CatalogAction::Browse(BrowseAction::Product(
            ProductAction::Detail(DetailAction::AddToCart),
        ))) => {
            if let Some(sku) = state.catalog.detail.as_ref().map(|d| d.sku.clone()) {
                let _ = cart_reducer(
                    &mut state.cart,
                    CartAction::AddLine { sku, qty: 1 },
                );
            }
            Cmd::none()
        }
        _ => Cmd::none(),
    }
}

fn global_reducer(state: &mut ShopState, action: ShopAction) -> Cmd<ShopAction> {
    match action {
        ShopAction::Global(GlobalAction::SignIn(user)) => {
            state.session.user = Some(user);
            Cmd::single(env_probe_effect())
        }
        ShopAction::Global(GlobalAction::StartCheckout) => {
            if state.checkout.is_none() && !state.cart.lines.is_empty() {
                state.checkout = Some(CheckoutState::default());
            }
            Cmd::none()
        }
        ShopAction::Global(GlobalAction::SeedWishlist) => {
            let id = state.next_wishlist_id;
            state.next_wishlist_id += 1;
            state.wishlists.insert(WishlistState {
                id,
                name: format!("List #{id}"),
                skus: vec![],
            });
            Cmd::none()
        }
        ShopAction::Global(GlobalAction::SessionTick) => {
            state.metrics.session_ticks += 1;
            Cmd::none()
        }
        ShopAction::Global(GlobalAction::SubscriptionPing) => {
            state.metrics.map_pings += 1;
            Cmd::none()
        }
        ShopAction::Global(GlobalAction::EnvProbe) => Cmd::single(env_probe_effect()),
        ShopAction::Global(GlobalAction::TriggerPanic) => {
            state.session.panic_survived = true;
            panic!("ecommerce demo: intentional reducer panic (state.panic_survived kept)");
        }
        ShopAction::Global(GlobalAction::EnvLoaded { origin, server_time }) => {
            state.session.http_origin = Some(origin);
            state.session.server_time = Some(server_time);
            Cmd::none()
        }
        _ => Cmd::none(),
    }
}

fn checkout_and_wishlist_reducer() -> impl Reducer<State = ShopState, Action = ShopAction> {
    CombineReducers((
        IfLetReducer::new(
            checkout_lens(),
            ShopAction::checkout_cp(),
            checkout_dismiss,
            checkout_clear,
            CHECKOUT_CANCEL,
            Reduce::new(checkout_reducer),
        ),
        ForEachReducer::new(
            wishlists_vec,
            embed_wishlist,
            extract_wishlist,
            extract_remove_wishlist,
            wishlist_cancel_id,
            Reduce::new(wishlist_reducer),
        ),
    ))
}

// ── Root reducer stack ───────────────────────────────────────────────────────

fn shop_reducer() -> impl Reducer<State = ShopState, Action = ShopAction> {
    CatchReducer::new(
        CombineReducers((
            Reduce::new(subscription_metrics_reducer),
            Reduce::new(detail_cart_bridge_reducer),
            Reduce::new(global_reducer),
            ScopeReducer::new(
                ShopState::catalog(),
                ShopAction::catalog_cp(),
                CATALOG_CANCEL,
                Reduce::new(catalog_reducer),
            ),
            ScopeReducer::new(
                ShopState::cart(),
                ShopAction::cart_cp(),
                CART_CANCEL,
                Reduce::new(cart_reducer),
            ),
            checkout_and_wishlist_reducer(),
        )),
        |panic| {
            eprintln!("shop reducer panic (state kept): {panic:?}");
            Cmd::none()
        },
    )
}

fn catalog_reducer(state: &mut CatalogState, action: CatalogAction) -> Cmd<CatalogAction> {
    match action {
        CatalogAction::SetQuery(q) => {
            state.query = q;
            Cmd::none()
        }
        CatalogAction::StreamPulse => Cmd::none(), // metrics: subscription_metrics_reducer
        CatalogAction::Browse(BrowseAction::Product(ProductAction::Detail(detail_action))) => {
            detail_reducer(state, detail_action)
        }
    }
}

fn detail_reducer(state: &mut CatalogState, action: DetailAction) -> Cmd<CatalogAction> {
    match action {
        DetailAction::Load { sku } => {
            let (name, price_cents) = mock_catalog(&sku);
            state.detail = Some(ProductDetailState {
                sku,
                name: Some(name.to_string()),
                price_cents: Some(price_cents),
                loading: false,
            });
            Cmd::none()
        }
        DetailAction::Loaded { name, price_cents } => {
            if let Some(detail) = state.detail.as_mut() {
                detail.name = Some(name);
                detail.price_cents = Some(price_cents);
                detail.loading = false;
            }
            Cmd::none()
        }
        DetailAction::AddToCart => Cmd::none(), // bridge: detail_cart_bridge_reducer
    }
}

fn cart_reducer(state: &mut CartState, action: CartAction) -> Cmd<CartAction> {
    match action {
        CartAction::AddLine { sku, qty } => {
            if let Some(line) = state.lines.iter_mut().find(|l| l.sku == sku) {
                line.qty += qty;
            } else {
                let id = state.next_line_id;
                state.next_line_id += 1;
                state.lines.insert(CartLine { id, sku, qty });
            }
            Cmd::none()
        }
        CartAction::Line(id, line_action) => {
            let Some(line) = state.lines.get_mut(id) else {
                return Cmd::none();
            };
            match line_action {
                CartLineAction::Inc => line.qty += 1,
                CartLineAction::Dec => {
                    if line.qty > 1 {
                        line.qty -= 1;
                    } else {
                        state.lines.remove(id);
                    }
                }
            }
            Cmd::none()
        }
    }
}

fn checkout_reducer(state: &mut CheckoutState, action: CheckoutAction) -> Cmd<CheckoutAction> {
    match action {
        CheckoutAction::Dismiss => Cmd::none(),
        CheckoutAction::Pay => {
            state.paid = true;
            Cmd::none()
        }
        CheckoutAction::PaymentDone => Cmd::none(),
        CheckoutAction::WsPulse => Cmd::none(), // metrics: subscription_metrics_reducer
    }
}

fn wishlist_reducer(state: &mut WishlistState, action: WishlistAction) -> Cmd<WishlistAction> {
    match action {
        WishlistAction::Rename(name) => {
            state.name = name;
            Cmd::none()
        }
        WishlistAction::AddSku(sku) => {
            if !state.skus.contains(&sku) {
                state.skus.push(sku);
            }
            Cmd::none()
        }
    }
}

// ── Wishlist ForEach wiring ──────────────────────────────────────────────────

fn wishlists_vec(state: &mut ShopState) -> &mut IdentifiedVec<WishlistId, WishlistState> {
    &mut state.wishlists
}

fn embed_wishlist(id: WishlistId, action: WishlistAction) -> ShopAction {
    ShopAction::WishlistRow(id, action)
}

fn extract_wishlist(action: ShopAction) -> Option<(WishlistId, WishlistAction)> {
    match action {
        ShopAction::WishlistRow(id, child) => Some((id, child)),
        _ => None,
    }
}

fn extract_remove_wishlist(action: ShopAction) -> Option<WishlistId> {
    match action {
        ShopAction::RemoveWishlist(id) => Some(id),
        _ => None,
    }
}

fn wishlist_cancel_id(id: WishlistId) -> EffectId {
    5000 + id
}

fn checkout_dismiss(action: ShopAction) -> bool {
    matches!(action, ShopAction::Checkout(CheckoutAction::Dismiss))
}

fn checkout_clear(state: &mut ShopState) {
    state.checkout = None;
}

fn cart_lens() -> impl rust_elm::optics::StateLens<ShopState, CartState> + Clone {
    fn get(s: &ShopState) -> Option<&CartState> {
        Some(&s.cart)
    }
    fn get_mut(s: &mut ShopState) -> Option<&mut CartState> {
        Some(&mut s.cart)
    }
    KpPath::new(get, get_mut)
}

fn checkout_lens() -> impl rust_elm::optics::StateLens<ShopState, CheckoutState> + Clone {
    fn get(s: &ShopState) -> Option<&CheckoutState> {
        s.checkout.as_ref()
    }
    fn get_mut(s: &mut ShopState) -> Option<&mut CheckoutState> {
        s.checkout.as_mut()
    }
    KpPath::new(get, get_mut)
}

fn init() -> (ShopState, Cmd<ShopAction>) {
    (
        ShopState::default(),
        Cmd::single(env_probe_effect()),
    )
}

fn sub_session_tick() -> ShopAction {
    ShopAction::Global(GlobalAction::SessionTick)
}

fn sub_catalog_pulse() -> ShopAction {
    ShopAction::Catalog(CatalogAction::StreamPulse)
}

fn sub_checkout_ws() -> ShopAction {
    ShopAction::Checkout(CheckoutAction::WsPulse)
}

fn sub_unit() {}

fn sub_map_ping(_: ()) -> ShopAction {
    ShopAction::Global(GlobalAction::SubscriptionPing)
}

/// State-driven subscriptions — all five `Sub` varieties.
fn subscriptions(state: &ShopState) -> Sub<ShopAction> {
    let mut subs = Vec::new();

    if state.session.user.is_some() {
        subs.push(Sub::tick(
            SUB_SESSION_TICK,
            Duration::from_millis(120),
            sub_session_tick,
        ));
    }

    if !state.catalog.query.is_empty() {
        subs.push(Sub::stream(
            SUB_CATALOG_STREAM,
            "catalog_search",
            Duration::from_millis(150),
            sub_catalog_pulse,
        ));
    }

    if state.checkout.is_some() {
        subs.push(Sub::websocket(
            SUB_CHECKOUT_WS,
            "wss://pay.example/status",
            Duration::from_millis(200),
            sub_checkout_ws,
        ));
    }

    subs.push(Sub::map_msg(
        Sub::tick(SUB_MAPMSG_UNIT, Duration::from_millis(100), sub_unit),
        sub_map_ping,
    ));

    Sub::batch(subs)
}

// ── Demo scenario ────────────────────────────────────────────────────────────

fn run_shop(label: &str, env: Environment) {
    println!("\n========== {label} environment ==========");

    let program = ReducerProgram::new(shop_reducer(), init, subscriptions);
    let runtime = Runtime::from_reducer_program(program, env, 64);
    let store = runtime.store();

    std::thread::sleep(Duration::from_millis(600));
    let boot = store.state();
    println!(
        "{label} boot: origin={:?} time={:?}",
        boot.session.http_origin, boot.session.server_time
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
        "{label} after panic: panic_survived={} user={:?} (app continues)",
        after_panic.session.panic_survived,
        after_panic.session.user
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
    println!(
        "env: origin={:?} time={:?}",
        final_state.session.http_origin, final_state.session.server_time
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
