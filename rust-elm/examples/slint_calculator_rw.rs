//! Calculator on [`RwRuntime`] / [`RwStore`] + [Slint](https://slint.dev).
//!
//! Same domain as [`druid_calculator_rw`](druid_calculator_rw.rs): shared [`calc_common`] reducer,
//! display synced from `read_store().with_read` on a 16 ms timer (like Druid's `SyncController`).
//!
//! ```bash
//! cargo run -p rust-elm --example slint_calculator_rw
//! ```

#[path = "calc_common.rs"]
mod calc_common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use calc_common::{init, subscriptions, update, CalcAction, CalcOp, CalcState};
use rust_elm::{Environment, Program, RuntimeConfig, RwRuntime, RwStore};
use slint::ComponentHandle;

slint::include_modules!();

struct RwRt {
    _runtime: RwRuntime<CalcState, CalcAction>,
    store: RwStore<CalcState, CalcAction>,
}

fn map_pressed(label: &str) -> Option<CalcAction> {
    match label {
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
            Some(CalcAction::Digit(label.parse().ok()?))
        }
        "." => Some(CalcAction::Dot),
        "+" => Some(CalcAction::Op(CalcOp::Add)),
        "−" | "-" => Some(CalcAction::Op(CalcOp::Sub)),
        "×" | "x" | "*" => Some(CalcAction::Op(CalcOp::Mul)),
        "÷" | "/" => Some(CalcAction::Op(CalcOp::Div)),
        "C" => Some(CalcAction::Clear),
        "⌫" => Some(CalcAction::Backspace),
        "=" => Some(CalcAction::Equals),
        _ => None,
    }
}

fn sync_display(ui: &CalculatorWindow, rt: &RwRt) {
    let display = rt.store.read_store().with_read(|s| s.display.clone());
    if ui.get_display_text().as_str() != display {
        ui.set_display_text(display.into());
    }
}

fn main() {
    let runtime = RwRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(64),
    )
    .expect("RwRuntime");

    let store = runtime.rw_store();
    let display = store.read_store().with_read(|s| s.display.clone());

    let rt = Arc::new(Mutex::new(RwRt {
        _runtime: runtime,
        store,
    }));

    let ui = CalculatorWindow::new().expect("slint ui");
    ui.set_display_text(display.into());

    let ui_weak = ui.as_weak();
    let rt_press = Arc::clone(&rt);
    ui.on_pressed(move |label| {
        let Some(action) = map_pressed(label.as_str()) else {
            return;
        };
        let Ok(guard) = rt_press.lock() else {
            return;
        };
        guard.store.dispatch(action);
        if let Some(ui) = ui_weak.upgrade() {
            sync_display(&ui, &guard);
        }
    });

    let ui_weak = ui.as_weak();
    let rt_timer = Arc::clone(&rt);
    slint::Timer::default().start(
        slint::TimerMode::Repeated,
        Duration::from_millis(16),
        move || {
            if let Some(ui) = ui_weak.upgrade() {
                if let Ok(guard) = rt_timer.lock() {
                    sync_display(&ui, &guard);
                }
            }
        },
    );

    ui.run().expect("slint run");
}
