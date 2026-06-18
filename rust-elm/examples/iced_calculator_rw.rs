//! Calculator GUI on [`RwRuntime`] / [`RwStore`] + [iced](https://github.com/iced-rs/iced).
//!
//! - **Shared state** in `Arc<RwLock<CalcState>>`; reducer mutates in place.
//! - **UI thread** reads display via `read_store().with_read` each `view`.
//!
//! [`RwRuntime`] shuts down automatically on drop when the iced window closes.
//!
//! Compare with [`iced_calculator_tea`](iced_calculator_tea.rs) (channel-pushed snapshots).
//!
//! ```bash
//! cargo run -p rust-elm --example iced_calculator_rw
//! ```

#[path = "iced_calculator/common.rs"]
mod common;

use common::{calculator_view, init, subscriptions, update, CalcAction, CalcState};
use iced::{Task, Theme};

use rust_elm::{Environment, Program, RuntimeConfig, RwRuntime, RwStore};

struct App {
    _runtime: RwRuntime<CalcState, CalcAction>,
    store: RwStore<CalcState, CalcAction>,
}

fn boot() -> (App, Task<CalcAction>) {
    let runtime = RwRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(64),
    )
    .expect("RwRuntime bootstrap");

    (
        App {
            store: runtime.rw_store(),
            _runtime: runtime,
        },
        Task::none(),
    )
}

fn handle_message(app: &mut App, action: CalcAction) -> Task<CalcAction> {
    app.store.dispatch(action);
    Task::none()
}

fn view(app: &App) -> iced::Element<'_, CalcAction> {
    calculator_view(
        app.store
            .read_store()
            .with_read(|state| state.display.clone()),
        |a| a,
    )
}

fn main() -> iced::Result {
    iced::application(
        "rust-elm calculator (RwStore)",
        handle_message,
        view,
    )
    .theme(|_| Theme::Dark)
    .window_size((360.0, 480.0))
    .centered()
    .run_with(boot)
}
