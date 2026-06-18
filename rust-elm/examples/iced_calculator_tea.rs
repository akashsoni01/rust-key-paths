//! Calculator GUI on [`TeaRuntime`] / [`TeaStore`] + [iced](https://github.com/iced-rs/iced).
//!
//! - **Reducer thread** owns [`CalcState`]; clicks `TeaStore::dispatch`.
//! - **UI thread** mirrors model from [`TeaStateSubscriber`] (channel-pushed `Arc<CalcState>`).
//!
//! Compare with [`iced_calculator_rw`](iced_calculator_rw.rs) (`RwStore` — borrow in `view`).
//!
//! ```bash
//! cargo run -p rust-elm --example iced_calculator_tea
//! ```

#[path = "iced_calc_common.rs"]
mod common;

use std::time::Duration;

use common::{calculator_view, init, subscriptions, update, CalcAction, CalcState};
use iced::{Subscription, Task, Theme};

use rust_elm::{Environment, Program, RuntimeConfig, TeaRuntime, TeaStore};

#[derive(Clone, Debug)]
enum Message {
    Press(CalcAction),
    TeaPoll,
}

struct App {
    runtime: Option<TeaRuntime<CalcState, CalcAction>>,
    store: TeaStore<CalcState, CalcAction>,
    tea_sub: rust_elm::TeaStateSubscriber<CalcState>,
    model: CalcState,
}

fn boot() -> (App, Task<Message>) {
    let runtime = TeaRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(64),
    )
    .expect("TeaRuntime bootstrap");
    let store = runtime.tea_store();
    let tea_sub = store.subscribe_state().expect("TeaStateSubscriber");
    let model = tea_sub
        .latest()
        .map(|s| s.as_ref().clone())
        .unwrap_or_default();

    (
        App {
            runtime: Some(runtime),
            store,
            tea_sub,
            model,
        },
        Task::none(),
    )
}

fn handle_message(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Press(action) => {
            app.store.dispatch(action);
            Task::none()
        }
        Message::TeaPoll => {
            while let Some(snap) = app.tea_sub.next() {
                app.model = snap.as_ref().clone();
            }
            Task::none()
        }
    }
}

fn view(app: &App) -> iced::Element<'_, Message> {
    calculator_view(&app.model.display, Message::Press)
}

fn tea_poll_subscription(_app: &App) -> Subscription<Message> {
    iced::time::every(Duration::from_millis(16)).map(|_| Message::TeaPoll)
}

fn main() -> iced::Result {
    iced::application(
        "rust-elm calculator (TeaStore)",
        handle_message,
        view,
    )
    .subscription(tea_poll_subscription)
    .theme(|_| Theme::Dark)
    .window_size((360.0, 480.0))
    .centered()
    .run_with(boot)
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown();
        }
    }
}
