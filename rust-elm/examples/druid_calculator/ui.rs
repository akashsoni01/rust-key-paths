//! Shared [Druid](https://github.com/linebender/druid) calculator keypad.

use std::time::Duration;

use druid::widget::prelude::*;
use druid::widget::{Controller, CrossAxisAlignment, Flex, Label, Painter};
use druid::{theme, AppLauncher, Color, Data, WidgetExt, WindowDesc};

#[path = "../calc_common.rs"]
mod calc_common;

pub use calc_common::{init, subscriptions, update, CalcAction, CalcOp, CalcState};

const TICK: Duration = Duration::from_millis(16);

pub trait CalcModel: Data {
    fn dispatch(&self, action: CalcAction);
    /// Pull store state into the model `display` field.
    fn sync(&mut self) -> bool;
    fn display(&self) -> &str;
}

struct SyncController {
    timer: Option<druid::TimerToken>,
}

impl<M: CalcModel> Controller<M, Flex<M>> for SyncController {
    fn event(
        &mut self,
        child: &mut Flex<M>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut M,
        env: &Env,
    ) {
        match event {
            Event::WindowConnected => self.timer = Some(ctx.request_timer(TICK)),
            Event::Timer(id) if self.timer.is_some_and(|t| t == *id) => {
                if data.sync() {
                    ctx.request_paint();
                }
                self.timer = Some(ctx.request_timer(TICK));
            }
            _ => {}
        }
        child.event(ctx, event, data, env);
    }
}

fn btn<M: CalcModel>(label: String, action: CalcAction, dark: bool) -> impl Widget<M> + use<M> {
    let painter = Painter::new(move |ctx, _, env| {
        let b = ctx.size().to_rect();
        let dark_bg = env.get(theme::PRIMARY_DARK);
        let light_bg = env.get(theme::BACKGROUND_LIGHT);
        ctx.fill(
            b,
            if dark { &dark_bg } else { &light_bg },
        );
        if ctx.is_hot() {
            ctx.stroke(b.inset(-0.5), &Color::WHITE, 1.0);
        }
        if ctx.is_active() {
            ctx.fill(b, &Color::rgb8(0x71, 0x71, 0x71));
        }
    });
    Label::new(label)
        .with_text_size(22.0)
        .center()
        .background(painter)
        .expand()
        .on_click(move |ctx, data: &mut M, _| {
            data.dispatch(action);
            if data.sync() {
                ctx.request_paint();
            }
        })
}

fn row4<M: CalcModel>(
    a: impl Widget<M> + 'static,
    b: impl Widget<M> + 'static,
    c: impl Widget<M> + 'static,
    d: impl Widget<M> + 'static,
) -> impl Widget<M> {
    Flex::row()
        .with_flex_child(a, 1.0)
        .with_spacer(1.0)
        .with_flex_child(b, 1.0)
        .with_spacer(1.0)
        .with_flex_child(c, 1.0)
        .with_spacer(1.0)
        .with_flex_child(d, 1.0)
}

fn build_ui<M: CalcModel + 'static>() -> impl Widget<M> {
    let display = Label::new(|data: &M, _env: &_| data.display().to_string())
        .with_text_size(32.0)
        .padding(5.0);
    let op = |l: &str, a: CalcAction| btn::<M>(l.to_string(), a, true);
    let digit = |d: u8| btn::<M>(d.to_string(), CalcAction::Digit(d), false);

    Flex::column()
        .with_flex_spacer(0.1)
        .with_child(display)
        .with_flex_spacer(0.1)
        .cross_axis_alignment(CrossAxisAlignment::End)
        .with_flex_child(
            row4(digit(7), digit(8), digit(9), op("÷", CalcAction::Op(CalcOp::Div))),
            1.0,
        )
        .with_spacer(1.0)
        .with_flex_child(
            row4(digit(4), digit(5), digit(6), op("×", CalcAction::Op(CalcOp::Mul))),
            1.0,
        )
        .with_spacer(1.0)
        .with_flex_child(
            row4(digit(1), digit(2), digit(3), op("−", CalcAction::Op(CalcOp::Sub))),
            1.0,
        )
        .with_spacer(1.0)
        .with_flex_child(
            row4(
                op("C", CalcAction::Clear),
                digit(0),
                op(".", CalcAction::Dot),
                op("+", CalcAction::Op(CalcOp::Add)),
            ),
            1.0,
        )
        .with_spacer(1.0)
        .with_flex_child(
            Flex::row()
                .with_flex_child(op("⌫", CalcAction::Backspace), 1.0)
                .with_spacer(1.0)
                .with_flex_child(op("=", CalcAction::Equals), 1.0),
            1.0,
        )
        .controller(SyncController { timer: None })
}

pub fn launch<M: CalcModel + 'static>(title: &'static str, data: M) {
    let window = WindowDesc::new(build_ui::<M>())
        .window_size((320., 420.))
        .title(title);
    AppLauncher::with_window(window)
        .launch(data)
        .expect("druid launch failed");
}
