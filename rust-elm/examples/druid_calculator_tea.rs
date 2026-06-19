//! Calculator on [`TeaRuntime`] / [`TeaStore`] + [Druid](https://github.com/linebender/druid).
//!
//! ```bash
//! cargo run -p rust-elm --example druid_calculator_tea
//! ```

#[path = "druid_calculator/ui.rs"]
mod ui;

use std::sync::{Arc, Mutex};

use druid::{Data, Lens};
use rust_elm::{Environment, Program, RuntimeConfig, TeaRuntime, TeaStore};

use ui::{init, launch, subscriptions, update, CalcAction, CalcModel, CalcState};

struct TeaRt {
    _runtime: TeaRuntime<CalcState, CalcAction>,
    store: TeaStore<CalcState, CalcAction>,
    sub: rust_elm::TeaStateSubscriber<CalcState>,
}

#[derive(Clone, Data, Lens)]
struct App {
    display: String,
    #[data(ignore)]
    rt: Arc<Mutex<TeaRt>>,
}

impl CalcModel for App {
    fn dispatch(&self, action: CalcAction) {
        self.rt.lock().expect("tea rt").store.dispatch(action);
    }

    fn sync(&mut self) -> bool {
        let mut rt = self.rt.lock().expect("tea rt");
        let mut changed = false;
        while let Some(snap) = rt.sub.next() {
            let d = snap.display.clone();
            if d != self.display {
                self.display = d;
                changed = true;
            }
        }
        changed
    }

    fn display(&self) -> &str {
        &self.display
    }
}

fn main() {
    let runtime = TeaRuntime::from_program(
        Program::new(init, update, subscriptions),
        Environment::new(),
        RuntimeConfig::new(64),
    )
    .expect("TeaRuntime");
    let store = runtime.tea_store();
    let sub = store.subscribe_state().expect("subscribe");
    let display = sub
        .latest()
        .map(|s| s.display.clone())
        .unwrap_or_else(|| "0".into());

    launch(
        "rust-elm calculator (TeaStore)",
        App {
            display,
            rt: Arc::new(Mutex::new(TeaRt {
                _runtime: runtime,
                store,
                sub,
            })),
        },
    );
}
