//! egui layout — domain in [`calc_common`](../calc_common.rs). Pure Rust widgets only.

#[path = "../calc_common.rs"]
mod calc_common;

pub use calc_common::*;

use eframe::egui;

pub fn calculator_ui(ui: &mut egui::Ui, display: &str, on_action: &mut impl FnMut(CalcAction)) {
    let mut mk = |ui: &mut egui::Ui, label: &'static str, action: CalcAction| {
        if ui
            .add(
                egui::Button::new(label)
                    .min_size(egui::vec2(72.0, 52.0))
                    .fill(egui::Color32::from_gray(55)),
            )
            .clicked()
        {
            on_action(action);
        }
    };

    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(display)
                .size(32.0)
                .monospace()
                .color(egui::Color32::WHITE),
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            mk(ui, "7", CalcAction::Digit(7));
            mk(ui, "8", CalcAction::Digit(8));
            mk(ui, "9", CalcAction::Digit(9));
            mk(ui, "÷", CalcAction::Op(CalcOp::Div));
        });
        ui.horizontal(|ui| {
            mk(ui, "4", CalcAction::Digit(4));
            mk(ui, "5", CalcAction::Digit(5));
            mk(ui, "6", CalcAction::Digit(6));
            mk(ui, "×", CalcAction::Op(CalcOp::Mul));
        });
        ui.horizontal(|ui| {
            mk(ui, "1", CalcAction::Digit(1));
            mk(ui, "2", CalcAction::Digit(2));
            mk(ui, "3", CalcAction::Digit(3));
            mk(ui, "−", CalcAction::Op(CalcOp::Sub));
        });
        ui.horizontal(|ui| {
            mk(ui, "C", CalcAction::Clear);
            mk(ui, "0", CalcAction::Digit(0));
            mk(ui, ".", CalcAction::Dot);
            mk(ui, "+", CalcAction::Op(CalcOp::Add));
        });
        ui.horizontal(|ui| {
            mk(ui, "⌫", CalcAction::Backspace);
            mk(ui, "=", CalcAction::Equals);
        });
    });
}
