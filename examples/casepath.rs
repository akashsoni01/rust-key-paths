//! Four-level nested action enums: how casepaths combine for extract and embed.
//!
//! ```bash
//! cargo run --example casepath
//! ```

use key_paths_derive::{Cp, Kp};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Kp, Cp)]
enum RootAction {
    App(AppAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Kp, Cp)]
enum AppAction {
    Panel(PanelAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Kp, Cp)]
enum PanelAction {
    Widget(WidgetAction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Kp, Cp)]
enum WidgetAction {
    Tap,
    Submit,
}

fn main() {
    // ── Compose casepaths with `.then()` / `.chain()` (Swift CasePaths `append`) ──
    //
    // One composed casepath handles both extract and embed:
    let to_widget = RootAction::app_cp()
        .then(AppAction::panel_cp())
        .then(PanelAction::widget_cp());

    let root = RootAction::App(AppAction::Panel(PanelAction::Widget(WidgetAction::Tap)));

    assert_eq!(to_widget.get_ref(&root), Some(&WidgetAction::Tap));

    // Embed the leaf through the same chain — no nested `.embed(.embed(...))`:
    let rebuilt = to_widget.embed(WidgetAction::Submit);
    assert_eq!(
        rebuilt,
        RootAction::App(AppAction::Panel(PanelAction::Widget(WidgetAction::Submit)))
    );
    assert_eq!(to_widget.get_ref(&rebuilt), Some(&WidgetAction::Submit));

    // Extract-only keypath chain (same shape, no embed):
    let read_only = RootAction::app()
        .then(AppAction::panel())
        .then(PanelAction::widget());
    assert_eq!(read_only.get(&root), Some(&WidgetAction::Tap));

    println!("4-level casepath OK");
    println!("  compose: RootAction::app_cp().then(...).chain(...)");
    println!("  extract: composed.get_ref(&root)");
    println!("  embed:   composed.embed(WidgetAction::Submit)");
}
