//! Shared shop state, reducers, and subscriptions for ecommerce examples.

use super::deps::{DateDep, HttpDep, WebDep, HTTPBIN_GET};
use rust_key_paths::Kp as KpPath;
use key_paths_derive::{Cp, Kp};
use rust_elm::{
    CatchReducer, Cmd, CombineReducers, Effect, EffectError, EffectId,
    ForEachReducer, Identifiable, IdentifiedVec, IfLetReducer, Reducer, Reduce,
    ScopeReducer, Sub,
};
use std::time::Duration;

// ── Effect ids (cancel / debounce / task identity) ─────────────────────────

pub const CATALOG_CANCEL: EffectId = 2001;
pub const CART_CANCEL: EffectId = 2002;
pub const CHECKOUT_CANCEL: EffectId = 2003;

pub const SUB_SESSION_TICK: u64 = 8001;
pub const SUB_CATALOG_STREAM: u64 = 8002;
pub const SUB_CHECKOUT_WS: u64 = 8003;
pub const SUB_MAPMSG_UNIT: u64 = 8004;

// ── State ────────────────────────────────────────────────────────────────────

pub type LineId = u64;
pub type WishlistId = u64;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct SessionState {
    pub user: Option<String>,
    pub http_origin: Option<String>,
    pub server_time: Option<String>,
    /// Resolved from [`WebDep`] during env probe — drives checkout `Sub::websocket`.
    pub checkout_ws_url: Option<&'static str>,
    /// Set `true` immediately before the demo action panic — proves partial state is kept.
    pub panic_survived: bool,
    /// Set `true` when an effect-delivered action panics in the reducer (`CatchReducer`).
    pub effect_panic_survived: bool,
    /// Set `true` when the effect panic sequence arms, before the effect-body `panic!`.
    pub effect_body_armed: bool,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ProductDetailState {
    pub sku: String,
    pub name: Option<String>,
    pub price_cents: Option<u32>,
    pub loading: bool,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CatalogState {
    pub query: String,
    pub detail: Option<ProductDetailState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CartLine {
    pub id: LineId,
    pub sku: String,
    pub qty: u32,
}

impl Identifiable for CartLine {
    type Id = LineId;
    fn id(&self) -> LineId {
        self.id
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CartState {
    pub lines: IdentifiedVec<LineId, CartLine>,
    pub next_line_id: LineId,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CheckoutState {
    pub payment_pending: bool,
    pub paid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WishlistState {
    pub id: WishlistId,
    pub name: String,
    pub skus: Vec<String>,
}

impl Identifiable for WishlistState {
    type Id = WishlistId;
    fn id(&self) -> WishlistId {
        self.id
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionMetrics {
    pub session_ticks: u32,
    pub catalog_stream_pulses: u32,
    pub checkout_ws_pulses: u32,
    pub map_pings: u32,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Kp)]
pub struct ShopState {
    pub session: SessionState,
    pub catalog: CatalogState,
    pub cart: CartState,
    pub checkout: Option<CheckoutState>,
    pub wishlists: IdentifiedVec<WishlistId, WishlistState>,
    pub next_wishlist_id: WishlistId,
    pub metrics: SubscriptionMetrics,
}

// ── Actions (4 nested levels under Catalog) ──────────────────────────────────

#[allow(dead_code)] // demo-only variants for API illustration
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum ShopAction {
    Global(GlobalAction),
    Catalog(CatalogAction),
    Cart(CartAction),
    Checkout(CheckoutAction),
    WishlistRow(WishlistId, WishlistAction),
    RemoveWishlist(WishlistId),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum GlobalAction {
    SignIn(String),
    StartCheckout,
    SeedWishlist,
    SessionTick,
    SubscriptionPing,
    EnvProbe,
    /// Intentionally panics after mutating `session.panic_survived` (demo only).
    TriggerPanic,
    /// Runs [`effect_panic_demo`] — effect-body panic + effect-delivered reducer panic.
    RunEffectPanicDemo,
    /// Step 1 of effect panic sequence — arms before effect-body `panic!`.
    ArmEffectBodyPanic,
    /// Step 2 — dispatched from an effect; panics in reducer (`CatchReducer` recovers).
    TriggerEffectReducerPanic,
    EnvLoaded {
        origin: String,
        server_time: String,
        checkout_ws_url: &'static str,
    },
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum CatalogAction {
    SetQuery(String),
    Browse(BrowseAction),
    StreamPulse,
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum BrowseAction {
    Product(ProductAction),
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum ProductAction {
    Detail(DetailAction),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum DetailAction {
    Load { sku: String },
    Loaded { name: String, price_cents: u32 },
    AddToCart,
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum CartAction {
    AddLine { sku: String, qty: u32 },
    Line(LineId, CartLineAction),
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp)]
pub enum CartLineAction {
    Inc,
    Dec,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
pub enum CheckoutAction {
    Dismiss,
    Pay,
    PaymentDone,
    WsPulse,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Kp)]
pub enum WishlistAction {
    Rename(String),
    AddSku(String),
}

// ── Helpers ──────────────────────────────────────────────────────────────────

pub fn mock_catalog(sku: &str) -> (&'static str, u32) {
    match sku {
        "hoodie-blue" => ("Blue Hoodie", 4999),
        "mug-rust" => ("Rust Mug", 1299),
        _ => ("Unknown SKU", 999),
    }
}

pub fn env_probe_effect() -> Effect<ShopAction> {
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
        let ws = match env.require::<WebDep>() {
            Ok(w) => w,
            Err(e) => {
                return Box::pin(async move { Err(EffectError::EnvMissing(e.name)) });
            }
        };
        let checkout_ws_url = ws.0.checkout_status_url();

        Box::pin(async move {
            let bin = http
                .0
                .get_request(HTTPBIN_GET.to_string())
                .await
                .map_err(EffectError::Other)?;
            let now = date.0.current().await;
            println!("date = {now:?}");
            println!("checkout ws url = {checkout_ws_url}");

            Ok(ShopAction::Global(GlobalAction::EnvLoaded {
                origin: bin.origin,
                server_time: now.to_rfc3339(),
                checkout_ws_url,
            }))
        })
    })
}

/// Effect panic demo — three steps in one [`Effect::sequence`]:
///
/// 1. Dispatch `ArmEffectBodyPanic` (reducer mutates state; no panic).
/// 2. Dispatch `TriggerEffectReducerPanic` (reducer panics; root [`CatchReducer`] recovers).
/// 3. `panic!` inside the effect interpreter task (Tokio absorbs; runtime continues).
///
/// Effect-returned actions are reduced on the bus thread inside [`CatchReducer`], same as UI
/// dispatches. Effect-body panics do not reach the reducer — only the async task ends.
pub fn effect_panic_demo() -> Effect<ShopAction> {
    Effect::sequence([
        Effect::from_fn(|| {
            Box::pin(async { Ok(ShopAction::Global(GlobalAction::ArmEffectBodyPanic)) })
        }),
        Effect::from_fn(|| {
            Box::pin(async {
                Ok(ShopAction::Global(GlobalAction::TriggerEffectReducerPanic))
            })
        }),
        Effect::from_fn(|| {
            Box::pin(async { panic!("ecommerce demo: intentional effect-body panic") })
        }),
    ])
}

// ── Child reducers ───────────────────────────────────────────────────────────

/// Subscription pulses arrive on child action paths; metrics live on root state.
pub fn subscription_metrics_reducer(state: &mut ShopState, action: ShopAction) -> Cmd<ShopAction> {
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
pub fn detail_cart_bridge_reducer(state: &mut ShopState, action: ShopAction) -> Cmd<ShopAction> {
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

pub fn global_reducer(state: &mut ShopState, action: ShopAction) -> Cmd<ShopAction> {
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
        ShopAction::Global(GlobalAction::RunEffectPanicDemo) => Cmd::single(effect_panic_demo()),
        ShopAction::Global(GlobalAction::ArmEffectBodyPanic) => {
            state.session.effect_body_armed = true;
            Cmd::none()
        }
        ShopAction::Global(GlobalAction::TriggerEffectReducerPanic) => {
            state.session.effect_panic_survived = true;
            panic!(
                "ecommerce demo: intentional reducer panic from effect-delivered action"
            );
        }
        ShopAction::Global(GlobalAction::TriggerPanic) => {
            state.session.panic_survived = true;
            panic!("ecommerce demo: intentional reducer panic (state.panic_survived kept)");
        }
        ShopAction::Global(GlobalAction::EnvLoaded {
            origin,
            server_time,
            checkout_ws_url,
        }) => {
            state.session.http_origin = Some(origin);
            state.session.server_time = Some(server_time);
            state.session.checkout_ws_url = Some(checkout_ws_url);
            Cmd::none()
        }
        _ => Cmd::none(),
    }
}

pub fn checkout_and_wishlist_reducer() -> impl Reducer<State = ShopState, Action = ShopAction> {
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

pub fn shop_reducer() -> impl Reducer<State = ShopState, Action = ShopAction> {
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

pub fn catalog_reducer(state: &mut CatalogState, action: CatalogAction) -> Cmd<CatalogAction> {
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

pub fn detail_reducer(state: &mut CatalogState, action: DetailAction) -> Cmd<CatalogAction> {
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

pub fn cart_reducer(state: &mut CartState, action: CartAction) -> Cmd<CartAction> {
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
            println!("this is fking working......");
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

pub fn checkout_reducer(state: &mut CheckoutState, action: CheckoutAction) -> Cmd<CheckoutAction> {
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

pub fn wishlist_reducer(state: &mut WishlistState, action: WishlistAction) -> Cmd<WishlistAction> {
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

pub fn wishlists_vec(state: &mut ShopState) -> &mut IdentifiedVec<WishlistId, WishlistState> {
    &mut state.wishlists
}

pub fn embed_wishlist(id: WishlistId, action: WishlistAction) -> ShopAction {
    ShopAction::WishlistRow(id, action)
}

pub fn extract_wishlist(action: ShopAction) -> Option<(WishlistId, WishlistAction)> {
    match action {
        ShopAction::WishlistRow(id, child) => Some((id, child)),
        _ => None,
    }
}

pub fn extract_remove_wishlist(action: ShopAction) -> Option<WishlistId> {
    match action {
        ShopAction::RemoveWishlist(id) => Some(id),
        _ => None,
    }
}

pub fn wishlist_cancel_id(id: WishlistId) -> EffectId {
    5000 + id
}

pub fn checkout_dismiss(action: ShopAction) -> bool {
    matches!(action, ShopAction::Checkout(CheckoutAction::Dismiss))
}

pub fn checkout_clear(state: &mut ShopState) {
    state.checkout = None;
}

pub type CartStateKp = KpPath<
    ShopState,
    CartState,
    &'static ShopState,
    &'static CartState,
    &'static mut ShopState,
    &'static mut CartState,
    for<'b> fn(&'b ShopState) -> Option<&'b CartState>,
    for<'b> fn(&'b mut ShopState) -> Option<&'b mut CartState>,
>;

pub type CheckoutStateKp = KpPath<
    ShopState,
    CheckoutState,
    &'static ShopState,
    &'static CheckoutState,
    &'static mut ShopState,
    &'static mut CheckoutState,
    for<'b> fn(&'b ShopState) -> Option<&'b CheckoutState>,
    for<'b> fn(&'b mut ShopState) -> Option<&'b mut CheckoutState>,
>;

pub fn cart_lens() -> CartStateKp {
    fn get(s: &ShopState) -> Option<&CartState> {
        Some(&s.cart)
    }
    fn get_mut(s: &mut ShopState) -> Option<&mut CartState> {
        Some(&mut s.cart)
    }
    KpPath::new(get, get_mut)
}

pub fn checkout_lens() -> CheckoutStateKp {
    fn get(s: &ShopState) -> Option<&CheckoutState> {
        s.checkout.as_ref()
    }
    fn get_mut(s: &mut ShopState) -> Option<&mut CheckoutState> {
        s.checkout.as_mut()
    }
    KpPath::new(get, get_mut)
}

pub fn init() -> (ShopState, Cmd<ShopAction>) {
    (
        ShopState::default(),
        Cmd::single(env_probe_effect()),
    )
}

pub fn sub_session_tick() -> ShopAction {
    ShopAction::Global(GlobalAction::SessionTick)
}

pub fn sub_catalog_pulse() -> ShopAction {
    ShopAction::Catalog(CatalogAction::StreamPulse)
}

pub fn sub_checkout_ws() -> ShopAction {
    ShopAction::Checkout(CheckoutAction::WsPulse)
}

pub fn sub_unit() {}

pub fn sub_map_ping(_: ()) -> ShopAction {
    ShopAction::Global(GlobalAction::SubscriptionPing)
}

/// State-driven subscriptions — all five `Sub` varieties.
pub fn subscriptions(state: &ShopState) -> Sub<ShopAction> {
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
        if let Some(url) = state.session.checkout_ws_url {
            subs.push(Sub::websocket(
                SUB_CHECKOUT_WS,
                url,
                Duration::from_millis(200),
                sub_checkout_ws,
            ));
        }
    }

    subs.push(Sub::map_msg(
        Sub::tick(SUB_MAPMSG_UNIT, Duration::from_millis(100), sub_unit),
        sub_map_ping,
    ));

    Sub::batch(subs)
}
