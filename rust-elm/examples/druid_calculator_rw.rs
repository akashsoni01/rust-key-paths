//! Calculator on [`RwRuntime`] / [`RwStore`] + [Druid](https://github.com/linebender/druid).
//!
//! ```bash
//! cargo run -p rust-elm --example druid_calculator_rw
//! ```

#[path = "druid_calculator/ui.rs"]
mod ui;

use std::sync::{Arc, Mutex};

use druid::{Data, Lens};
use rust_elm::{Environment, Program, RuntimeConfig, RwRuntime, RwStore};

use ui::{init, launch, subscriptions, update, CalcAction, CalcModel, CalcState};

struct RwRt {
    _runtime: RwRuntime<CalcState, CalcAction>,
    store: RwStore<CalcState, CalcAction>,
}

#[derive(Clone, Data, Lens)]
struct App {
    display: String,
    #[data(ignore)]
    rt: Arc<Mutex<RwRt>>,
}

impl CalcModel for App {
    fn dispatch(&self, action: CalcAction) {
        self.rt.lock().expect("rw rt").store.dispatch(action);
    }

    fn sync(&mut self) -> bool {
        let rt = self.rt.lock().expect("rw rt");
        let d = rt
            .store
            .read_store()
            .with_read(|s| s.display.clone());
        if d != self.display {
            self.display = d;
            true
        } else {
            false
        }
    }

    fn display(&self) -> &str {
        &self.display
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
    let display = store
        .read_store()
        .with_read(|s| s.display.clone());

    launch(
        "rust-elm calculator (RwStore)",
        App {
            display,
            rt: Arc::new(Mutex::new(RwRt {
                _runtime: runtime,
                store,
            })),
        },
    );
}
