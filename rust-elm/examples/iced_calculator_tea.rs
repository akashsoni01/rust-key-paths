//! Calculator GUI on [`TeaRuntime`] / [`TeaStore`] + [iced](https://github.com/iced-rs/iced).
//!
//! Architecture:
//! - **Reducer thread** (`TeaRuntime`) owns [`CalcState`]; each button dispatches a [`CalcAction`].
//! - **UI thread** (iced) holds a cached model updated from [`TeaStateSubscriber`] (channel-pushed
//!   `Arc<CalcState>` — no lock on the live model).
//! - iced `subscription` polls the tea subscriber ~60fps; clicks call `TeaStore::dispatch`.
//!
//! ```bash
//! cargo run -p rust-elm --example iced_calculator_tea
//! ```

use std::time::Duration;

use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Subscription, Task, Theme};

use rust_elm::{Cmd, Environment, Program, RuntimeConfig, Sub, TeaRuntime, TeaStore};

#[derive(Clone, Debug, PartialEq)]
struct CalcState {
    display: String,
    lhs: Option<f64>,
    op: Option<CalcOp>,
    fresh_rhs: bool,
}

impl Default for CalcState {
    fn default() -> Self {
        Self {
            display: "0".into(),
            lhs: None,
            op: None,
            fresh_rhs: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CalcOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CalcAction {
    Digit(u8),
    Dot,
    Op(CalcOp),
    Equals,
    Clear,
    Backspace,
}

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

fn init() -> (CalcState, Cmd<CalcAction>) {
    (CalcState::default(), Cmd::none())
}

fn parse_display(display: &str) -> Option<f64> {
    if display.is_empty() || display == "-" {
        return None;
    }
    display.parse().ok()
}

fn format_number(n: f64) -> String {
    if (n - n.round()).abs() < f64::EPSILON {
        format!("{}", n.round() as i64)
    } else {
        format!("{n:.8}").trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn apply_op(lhs: f64, rhs: f64, op: CalcOp) -> Option<f64> {
    match op {
        CalcOp::Add => Some(lhs + rhs),
        CalcOp::Sub => Some(lhs - rhs),
        CalcOp::Mul => Some(lhs * rhs),
        CalcOp::Div if rhs.abs() > f64::EPSILON => Some(lhs / rhs),
        CalcOp::Div => None,
    }
}

fn update(state: &mut CalcState, action: CalcAction) -> Cmd<CalcAction> {
    match action {
        CalcAction::Clear => {
            *state = CalcState::default();
        }
        CalcAction::Backspace => {
            state.display.pop();
            if state.display.is_empty() {
                state.display.push('0');
            }
        }
        CalcAction::Digit(d) => {
            if d <= 9 {
                if state.fresh_rhs || state.display == "0" {
                    state.display = d.to_string();
                    state.fresh_rhs = false;
                } else {
                    state.display.push((b'0' + d) as char);
                }
            }
        }
        CalcAction::Dot => {
            if state.fresh_rhs {
                state.display = "0.".into();
                state.fresh_rhs = false;
            } else if !state.display.contains('.') {
                state.display.push('.');
            }
        }
        CalcAction::Op(op) => {
            if let Some(current) = parse_display(&state.display) {
                if let (Some(lhs), Some(pending)) = (state.lhs, state.op) {
                    if !state.fresh_rhs {
                        if let Some(result) = apply_op(lhs, current, pending) {
                            state.display = format_number(result);
                            state.lhs = Some(result);
                        } else {
                            state.display = "Error".into();
                            state.lhs = None;
                            state.op = None;
                            state.fresh_rhs = true;
                            return Cmd::none();
                        }
                    }
                } else {
                    state.lhs = Some(current);
                }
            }
            state.op = Some(op);
            state.fresh_rhs = true;
        }
        CalcAction::Equals => {
            if let (Some(lhs), Some(op)) = (state.lhs, state.op) {
                if let Some(rhs) = parse_display(&state.display) {
                    if let Some(result) = apply_op(lhs, rhs, op) {
                        state.display = format_number(result);
                        state.lhs = None;
                        state.op = None;
                        state.fresh_rhs = true;
                    } else {
                        state.display = "Error".into();
                        state.lhs = None;
                        state.op = None;
                        state.fresh_rhs = true;
                    }
                }
            }
        }
    }
    Cmd::none()
}

fn subscriptions(_: &CalcState) -> Sub<CalcAction> {
    Sub::none()
}

fn boot() -> (App, Task<Message>) {
    let runtime = TeaRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(64),
    )
    .expect("TeaRuntime bootstrap");
    let store = runtime.tea_store();
    let tea_sub = store
        .subscribe_state()
        .expect("TeaStateSubscriber");
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

fn view(app: &App) -> Element<'_, Message> {
    let display = text(app.model.display.clone())
        .size(36)
        .width(Length::Fill);

    let row1 = row![
        calc_button("7", Message::Press(CalcAction::Digit(7))),
        calc_button("8", Message::Press(CalcAction::Digit(8))),
        calc_button("9", Message::Press(CalcAction::Digit(9))),
        calc_button("÷", Message::Press(CalcAction::Op(CalcOp::Div))),
    ]
    .spacing(8);

    let row2 = row![
        calc_button("4", Message::Press(CalcAction::Digit(4))),
        calc_button("5", Message::Press(CalcAction::Digit(5))),
        calc_button("6", Message::Press(CalcAction::Digit(6))),
        calc_button("×", Message::Press(CalcAction::Op(CalcOp::Mul))),
    ]
    .spacing(8);

    let row3 = row![
        calc_button("1", Message::Press(CalcAction::Digit(1))),
        calc_button("2", Message::Press(CalcAction::Digit(2))),
        calc_button("3", Message::Press(CalcAction::Digit(3))),
        calc_button("−", Message::Press(CalcAction::Op(CalcOp::Sub))),
    ]
    .spacing(8);

    let row4 = row![
        calc_button("C", Message::Press(CalcAction::Clear)),
        calc_button("0", Message::Press(CalcAction::Digit(0))),
        calc_button(".", Message::Press(CalcAction::Dot)),
        calc_button("+", Message::Press(CalcAction::Op(CalcOp::Add))),
    ]
    .spacing(8);

    let row5 = row![
        calc_button("⌫", Message::Press(CalcAction::Backspace)),
        calc_button("=", Message::Press(CalcAction::Equals)),
    ]
    .spacing(8);

    let content = column![display, row1, row2, row3, row4, row5].spacing(12);

    container(content)
        .padding(20)
        .width(Length::Fixed(320.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn calc_button<'a>(label: &'a str, message: Message) -> Element<'a, Message> {
    button(text(label).width(Length::Fill).center())
        .width(Length::Fixed(64.0))
        .height(Length::Fixed(48.0))
        .on_press(message)
        .into()
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
