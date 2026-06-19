//! Calculator domain for iced / druid examples (no UI deps).

use rust_elm::{Cmd, Sub};

#[derive(Clone, Debug, PartialEq)]
pub struct CalcState {
    pub display: String,
    pub lhs: Option<f64>,
    pub op: Option<CalcOp>,
    pub fresh_rhs: bool,
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
pub enum CalcOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalcAction {
    Digit(u8),
    Dot,
    Op(CalcOp),
    Equals,
    Clear,
    Backspace,
}

pub fn init() -> (CalcState, Cmd<CalcAction>) {
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
        format!("{n:.8}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
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

pub fn update(state: &mut CalcState, action: CalcAction) -> Cmd<CalcAction> {
    match action {
        CalcAction::Clear => *state = CalcState::default(),
        CalcAction::Backspace => {
            state.display.pop();
            if state.display.is_empty() {
                state.display.push('0');
            }
        }
        CalcAction::Digit(d) if d <= 9 => {
            if state.fresh_rhs || state.display == "0" {
                state.display = d.to_string();
                state.fresh_rhs = false;
            } else {
                state.display.push((b'0' + d) as char);
            }
        }
        CalcAction::Digit(_) => {}
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

pub fn subscriptions(_: &CalcState) -> Sub<CalcAction> {
    Sub::none()
}
