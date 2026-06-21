//! **Sync effect recursion** — same paginated fetch as [`recursion`], driven by
//! [`ExhaustiveTestStore`] instead of a Tokio runtime.
//!
//! Pattern: `init` boots `fetch_page(0)`; each `PageLoaded` schedules `fetch_page(n + 1)`
//! until `has_more` is false. Effects run synchronously via `receive` / `boot`.
//!
//! ```bash
//! cargo run -p rust-elm --example sync_recursion --release
//! ```

use rust_elm::{allow_state_clones, Cmd, Effect, ExhaustiveTestStore};

const PAGES: &[&str] = &["alpha", "beta", "gamma", "delta"];

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct App {
    items: Vec<String>,
    pages_fetched: u32,
    done: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Action {
    PageLoaded {
        page: u32,
        chunk: Vec<String>,
        has_more: bool,
    },
}

fn init() -> (App, Cmd<Action>) {
    (App::default(), Cmd::single(fetch_page(0)))
}

fn fetch_page(page: u32) -> Effect<Action> {
    Effect::from_fn(move || {
        Box::pin(async move {
            if page as usize >= PAGES.len() {
                Ok(Action::PageLoaded {
                    page,
                    chunk: vec![],
                    has_more: false,
                })
            } else {
                Ok(Action::PageLoaded {
                    page,
                    chunk: vec![PAGES[page as usize].to_string()],
                    has_more: page as usize + 1 < PAGES.len(),
                })
            }
        })
    })
}

fn update(app: &mut App, action: Action) -> Cmd<Action> {
    match action {
        Action::PageLoaded {
            page,
            chunk,
            has_more,
        } => {
            app.pages_fetched = page + 1;
            app.items.extend(chunk);
            if has_more {
                Cmd::single(fetch_page(page + 1))
            } else {
                app.done = true;
                Cmd::none()
            }
        }
    }
}

fn expected_page_loaded(page: u32) -> Action {
    Action::PageLoaded {
        page,
        chunk: vec![PAGES[page as usize].to_string()],
        has_more: page as usize + 1 < PAGES.len(),
    }
}

fn main() {
    let (state, init_cmd) = init();
    let mut store = ExhaustiveTestStore::new(state, update);
    store.boot(init_cmd);

    for page in 0..PAGES.len() {
        store.receive(expected_page_loaded(page as u32));
    }
    store.finish();

    allow_state_clones(1, || {
        println!("pages fetched: {}", store.state.pages_fetched);
        println!("items: {:?}", store.state.items);
    });

    assert!(store.state.done);
    assert_eq!(store.state.items, ["alpha", "beta", "gamma", "delta"]);
    assert_eq!(store.state.pages_fetched, PAGES.len() as u32);

    println!("sync_recursion example OK");
}
