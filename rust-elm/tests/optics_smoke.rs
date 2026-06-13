use key_paths_derive::Kp;
use rust_elm::optics::{extract, wrap_action};

#[derive(Debug, Kp, Clone, PartialEq)]
struct Dashboard {
    panel: Option<Panel>,
    title: String,
}

#[derive(Debug, Kp, Clone, PartialEq)]
struct Panel {
    items: Vec<Item>,
}

#[derive(Debug, Kp, Clone, PartialEq)]
struct Item {
    value: i32,
}

#[derive(Debug, Kp, Clone, PartialEq)]
enum DashAction {
    Panel(PanelAction),
    Refresh,
}

#[derive(Debug, Kp, Clone, PartialEq)]
enum PanelAction {
    Select(i32),
}

#[test]
fn nested_option_panel_smoke() {
    let mut dash = Dashboard {
        panel: Some(Panel {
            items: vec![Item { value: 1 }, Item { value: 2 }],
        }),
        title: "main".into(),
    };

    let panel_kp = Dashboard::panel();
    if let Some(panel) = panel_kp.get_mut(&mut dash) {
        panel.items[0].value = 99;
    }
    assert_eq!(dash.panel.as_ref().unwrap().items[0].value, 99);

    let title_kp = Dashboard::title();
    assert_eq!(title_kp.get(&dash).map(|s| s.as_str()), Some("main"));
}

#[test]
fn enum_action_prism_extract_and_wrap() {
    let action = DashAction::Panel(PanelAction::Select(3));
    let kp = DashAction::panel();
    let inner = extract(&kp, &action).expect("variant");
    assert!(matches!(inner, PanelAction::Select(3)));

    let wrapped = wrap_action(DashAction::Panel, PanelAction::Select(5));
    assert_eq!(wrapped, DashAction::Panel(PanelAction::Select(5)));
}

#[test]
fn box_and_option_chain_round_trip() {
    #[derive(Debug, Kp, PartialEq)]
    struct Root {
        inner: Option<Box<Leaf>>,
    }

    #[derive(Debug, Kp, PartialEq)]
    struct Leaf {
        flag: bool,
    }

    let mut root = Root {
        inner: Some(Box::new(Leaf { flag: false })),
    };
    let kp = Root::inner().then(Leaf::flag());
    assert_eq!(kp.get(&root).copied(), Some(false));
    if let Some(flag) = kp.get_mut(&mut root) {
        *flag = true;
    }
    assert_eq!(root.inner.unwrap().flag, true);
}

#[test]
fn panel_items_field_accessible() {
    let mut dash = Dashboard {
        panel: Some(Panel {
            items: vec![Item { value: 10 }],
        }),
        title: "t".into(),
    };
    let panel_kp = Dashboard::panel();
    assert!(panel_kp.get(&dash).is_some());
    dash.panel.as_mut().unwrap().items[0].value = 20;
    assert_eq!(dash.panel.unwrap().items[0].value, 20);
}

#[test]
fn counter_value_then_composition() {
    #[derive(Debug, Kp, Clone, PartialEq)]
    struct AppState {
        counter: Option<CounterState>,
    }

    #[derive(Debug, Kp, Clone, PartialEq)]
    struct CounterState {
        value: i32,
    }

    let mut state = AppState {
        counter: Some(CounterState { value: 1 }),
    };
    let value_kp = AppState::counter().then(CounterState::value());
    assert_eq!(value_kp.get(&state).copied(), Some(1));
    if let Some(v) = value_kp.get_mut(&mut state) {
        *v = 99;
    }
    assert_eq!(state.counter.unwrap().value, 99);
}
