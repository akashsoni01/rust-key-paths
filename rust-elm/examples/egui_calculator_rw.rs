//! Calculator on [`RwRuntime`] / [`RwStore`] + [egui](https://github.com/emilk/egui) (pure Rust UI).
//!
//! Same domain as [`druid_calculator_rw`](druid_calculator_rw.rs). Slint requires its own
//! `.slint` markup language — there is no imperative Rust widget API — so this example uses
//! egui for a Druid-like calculator built entirely in Rust.
//!
//! ```bash
//! cargo run -p rust-elm --example egui_calculator_rw
//! ```

#[path = "egui_calculator/common.rs"]
mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{init, subscriptions, update, CalcAction, CalcState};
use eframe::egui;
use rust_elm::{Environment, Program, RuntimeConfig, RwRuntime, RwStore};

struct RwRt {
    _runtime: RwRuntime<CalcState, CalcAction>,
    store: RwStore<CalcState, CalcAction>,
}

struct CalcApp {
    rt: Arc<Mutex<RwRt>>,
    display: String,
}

impl CalcApp {
    fn sync_display(&mut self) {
        if let Ok(guard) = self.rt.lock() {
            let d = guard.store.read_store().with_read(|s| s.display.clone());
            if d != self.display {
                self.display = d;
            }
        }
    }
}

impl eframe::App for CalcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_display();

        let mut pressed = false;
        let rt = Arc::clone(&self.rt);
        let display = self.display.clone();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_gray(30)))
            .show(ctx, |ui| {
                let mut on_action = |action: CalcAction| {
                    if let Ok(guard) = rt.lock() {
                        guard.store.dispatch(action);
                    }
                    pressed = true;
                };
                common::calculator_ui(ui, &display, &mut on_action);
            });

        if pressed {
            self.sync_display();
        }

        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

fn main() -> eframe::Result<()> {
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

    eframe::run_native(
        "rust-elm calculator (RwStore)",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([360.0, 480.0]),
            ..Default::default()
        },
        Box::new(|_cc| {
            Ok(Box::new(CalcApp {
                rt,
                display,
            }))
        }),
    )
}
