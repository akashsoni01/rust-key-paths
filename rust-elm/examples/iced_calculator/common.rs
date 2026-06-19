//! Iced layout — domain in [`calc_common`](../calc_common.rs).

#[path = "../calc_common.rs"]
mod calc_common;

pub use calc_common::*;

use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

pub fn calculator_view<M>(
    display: impl Into<String>,
    map: impl Fn(CalcAction) -> M + Copy + 'static,
) -> Element<'static, M>
where
    M: Clone + 'static,
{
    let mk = |label: &'static str, action: CalcAction| -> Element<'static, M> {
        button(text(label).width(Length::Fill).center())
            .width(Length::Fixed(64.0))
            .height(Length::Fixed(48.0))
            .on_press(map(action))
            .into()
    };
    container(
        column![
            text(display.into()).size(36).width(Length::Fill),
            row([mk("7", CalcAction::Digit(7)), mk("8", CalcAction::Digit(8)), mk("9", CalcAction::Digit(9)), mk("÷", CalcAction::Op(CalcOp::Div))].into_iter()).spacing(8),
            row([mk("4", CalcAction::Digit(4)), mk("5", CalcAction::Digit(5)), mk("6", CalcAction::Digit(6)), mk("×", CalcAction::Op(CalcOp::Mul))].into_iter()).spacing(8),
            row([mk("1", CalcAction::Digit(1)), mk("2", CalcAction::Digit(2)), mk("3", CalcAction::Digit(3)), mk("−", CalcAction::Op(CalcOp::Sub))].into_iter()).spacing(8),
            row([mk("C", CalcAction::Clear), mk("0", CalcAction::Digit(0)), mk(".", CalcAction::Dot), mk("+", CalcAction::Op(CalcOp::Add))].into_iter()).spacing(8),
            row([mk("⌫", CalcAction::Backspace), mk("=", CalcAction::Equals)].into_iter()).spacing(8),
        ]
        .spacing(12),
    )
    .padding(20)
    .width(Length::Fixed(320.0))
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}
