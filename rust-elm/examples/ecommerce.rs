//! End-to-end ecommerce store — demonstrates real-world composition of rust-elm.
//!
//! Features showcased:
//! - **4-level actions**: `Shop → Catalog → Browse → Product → Detail`
//! - **Child state variety**: scoped catalog/cart, optional checkout (`IfLetReducer`),
//!   identified wishlists (`ForEachReducer`), cart lines (`IdentifiedVec`)
//! - **Derived optics**: `Kp` / `Cp` keypaths for state and action scoping
//! - **Reducer stack**: `CatchReducer` + `CombineReducers` + `ScopeReducer` + `IfLetReducer` + `ForEachReducer`
//! - **Runtime**: bus-driven update loop, Tokio effect interpreter, `ScopedStore`, state subscription
//! - **Effects**: `StoreTask`, cancel-on-dismiss (`IfLetReducer`); see `book/architecture.md` for debounce/async patterns
//!
//! See [`../book/architecture.md`](../book/architecture.md) for threading, concurrency, and panic strategy.
//!
//! ```bash
//! cargo run -p rust-elm --example ecommerce
//! ```

use key_paths_derive::{Cp, Kp};
use rust_elm::{
    CatchReducer, Cmd, CombineReducers, EffectId, Environment, ForEachReducer,
    Identifiable, IdentifiedVec, IfLetReducer, Reducer, ReducerProgram, Reduce, Runtime, ScopeReducer,
    Sub,
};
use rust_key_paths::Kp as KpPath;
use std::time::Duration;

// ── Effect ids (cancel / debounce / task identity) ─────────────────────────

const CATALOG_CANCEL: EffectId = 2001;
const CART_CANCEL: EffectId = 2002;
const CHECKOUT_CANCEL: EffectId = 2003;

// ── State ────────────────────────────────────────────────────────────────────

type LineId = u64;
type WishlistId = u64;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct SessionState {
    user: Option<String>,
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

#[derive(Default, Clone, Debug, PartialEq, Eq, Kp)]
struct ShopState {
    session: SessionState,
    catalog: CatalogState,
    cart: CartState,
    checkout: Option<CheckoutState>,
    wishlists: IdentifiedVec<WishlistId, WishlistState>,
    next_wishlist_id: WishlistId,
}

// ── Actions (4 nested levels under Catalog) ──────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum ShopAction {
    Global(GlobalAction),
    Catalog(CatalogAction),
    Cart(CartAction),
    Checkout(CheckoutAction),
    WishlistRow(WishlistId, WishlistAction),
    RemoveWishlist(WishlistId),
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum GlobalAction {
    SignIn(String),
    StartCheckout,
    SeedWishlist,
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum CatalogAction {
    SetQuery(String),
    Browse(BrowseAction),
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum BrowseAction {
    Product(ProductAction),
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum ProductAction {
    Detail(DetailAction),
}

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

#[derive(Clone, Debug, PartialEq, Eq, Kp)]
enum CartLineAction {
    Inc,
    Dec,
}

#[derive(Clone, Debug, PartialEq, Eq, Kp, Cp)]
enum CheckoutAction {
    Dismiss,
    Pay,
    PaymentDone,
}

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

// ── Child reducers ───────────────────────────────────────────────────────────

fn global_reducer(state: &mut ShopState, action: ShopAction) -> Cmd<ShopAction> {
    // Cross-scope bridge: product detail "add to cart" updates cart in the same reduce turn.
    if let ShopAction::Catalog(CatalogAction::Browse(BrowseAction::Product(
        ProductAction::Detail(DetailAction::AddToCart),
    ))) = &action
    {
        if let Some(sku) = state.catalog.detail.as_ref().map(|d| d.sku.clone()) {
            let _ = cart_reducer(
                &mut state.cart,
                CartAction::AddLine { sku, qty: 1 },
            );
        }
        return Cmd::none();
    }

    let ShopAction::Global(action) = action else {
        return Cmd::none();
    };
    match action {
        GlobalAction::SignIn(user) => {
            state.session.user = Some(user);
            Cmd::none()
        }
        GlobalAction::StartCheckout => {
            if state.checkout.is_none() && !state.cart.lines.is_empty() {
                state.checkout = Some(CheckoutState::default());
            }
            Cmd::none()
        }
        GlobalAction::SeedWishlist => {
            let id = state.next_wishlist_id;
            state.next_wishlist_id += 1;
            state.wishlists.insert(WishlistState {
                id,
                name: format!("List #{id}"),
                skus: vec![],
            });
            Cmd::none()
        }
    }
}

fn catalog_reducer(state: &mut CatalogState, action: CatalogAction) -> Cmd<CatalogAction> {
    match action {
        CatalogAction::SetQuery(q) => {
            state.query = q;
            Cmd::none()
        }
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
        DetailAction::AddToCart => Cmd::none(),
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

// ── Root reducer stack ───────────────────────────────────────────────────────

fn shop_reducer() -> impl Reducer<State = ShopState, Action = ShopAction> {
    CatchReducer::new(
        CombineReducers((
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
        )),
        |panic| {
            eprintln!("shop reducer panic (state kept): {panic:?}");
            Cmd::none()
        },
    )
}

fn init() -> (ShopState, Cmd<ShopAction>) {
    (ShopState::default(), Cmd::none())
}

fn subscriptions(_: &ShopState) -> Sub<ShopAction> {
    Sub::none()
}

// ── Demo scenario ────────────────────────────────────────────────────────────

fn main() {
    let program = ReducerProgram::new(shop_reducer(), init, subscriptions);
    let runtime = Runtime::from_reducer_program(program, Environment::new(), 64);
    let store = runtime.store();

    // Scoped cart UI — dispatches only `CartAction` on the parent bus.
    let cart_store = store.scope(cart_lens(), ShopAction::cart_cp());

    store.dispatch(ShopAction::Global(GlobalAction::SignIn(
        "alex@shop.example".into(),
    )));
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
    std::thread::sleep(Duration::from_millis(150));

    store.dispatch(ShopAction::WishlistRow(
        0,
        WishlistAction::AddSku("hoodie-blue".into()),
    ));

    let final_state = store.state();
    println!("\n=== final shop state ===");
    println!("user: {:?}", final_state.session.user);
    println!("cart lines: {}", final_state.cart.lines.len());
    println!(
        "checkout paid: {:?}",
        final_state.checkout.as_ref().map(|c| c.paid)
    );
    println!("wishlists: {}", final_state.wishlists.len());

    runtime.shutdown();
    println!("ecommerce example OK");
}
