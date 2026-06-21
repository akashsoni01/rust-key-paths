//! **Effect recursion** — each async step dispatches an action; the reducer schedules the next step.
//!
//! Pattern: paginated load — `PageLoaded` triggers `fetch_page(n + 1)` until `has_more` is false.
//! No recursive `update` calls; recursion lives in the **action → effect → action** loop.
//!
//! ```bash
//! cargo run -p rust-elm --example recursion --release
//! ```

use rust_elm::{start_runtime, Cmd, Effect, Environment, Program, RuntimeConfig, Sub};
use std::time::Duration;

const PAGES: &[&str] = &["alpha", "beta", "gamma", "delta"];

#[derive(Default, Debug)]
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
            tokio::time::sleep(Duration::from_millis(15)).await;
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
                // Recurse: schedule the next page (tail-call style via Cmd, not Rust recursion).
                Cmd::single(fetch_page(page + 1))
            } else {
                app.done = true;
                Cmd::none()
            }
        }
    }
}

fn subscriptions(_: &App) -> Sub<Action> {
    Sub::none()
}

fn main() {
    let runtime = start_runtime(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(64),
    );

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        let app = runtime.state.lock();
        if app.done {
            let pages = app.pages_fetched;
            let items = app.items.clone();
            drop(app);
            println!("pages fetched: {pages}");
            println!("items: {items:?}");
            assert_eq!(items, ["alpha", "beta", "gamma", "delta"]);
            assert_eq!(pages, PAGES.len() as u32);
            runtime.shutdown();
            println!("recursion example OK");
            return;
        }
        drop(app);
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for paginated load");
}
