//! ISO 20022 **PAIN.001** API payload as app state — field validation via reusable keypath rules.
//!
//! - **State**: full `Pain001` message + validation errors
//! - **Actions**: load samples, patch fields through keypaths, mandatory check, validate, submit
//! - **Validation**: mandatory fail-fast (`validate_mandatory`) then full accumulation
//!   (`validate`) — see [`book/validation.md`](../book/validation.md)
//!
//! ```bash
//! cargo run -p rust-elm --example pain
//! ```

#[path = "pain/model.rs"]
mod model;
#[path = "pain/keypaths.rs"]
mod keypaths;
#[path = "pain/validation.rs"]
mod validation;
#[path = "pain/sample.rs"]
mod sample;

use model::Pain001;
use keypaths::pain_message_id;
use sample::{sample_invalid, sample_valid};
use validation::{FieldError, Validate};

use key_paths_derive::Kp;
use rust_elm::{Cmd, Environment, Reduce, ReducerProgram, Runtime, Sub};

#[derive(Clone, Debug, Default, PartialEq, Kp)]
struct PainAppState {
    payload: Pain001,
    errors: Vec<FieldError>,
    submitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PainAction {
    LoadValid,
    LoadInvalid,
    CheckMandatory,
    Validate,
    SetMessageId(String),
    Submit,
}

fn init() -> (PainAppState, Cmd<PainAction>) {
    (
        PainAppState {
            payload: sample_valid(),
            ..Default::default()
        },
        Cmd::none(),
    )
}

fn pain_reducer(state: &mut PainAppState, action: PainAction) -> Cmd<PainAction> {
    match action {
        PainAction::LoadValid => {
            state.payload = sample_valid();
            state.errors.clear();
            state.submitted = false;
        }
        PainAction::LoadInvalid => {
            state.payload = sample_invalid();
            state.errors.clear();
            state.submitted = false;
        }
        PainAction::CheckMandatory => {
            state.errors = match state.payload.validate_mandatory() {
                Ok(()) => Vec::new(),
                Err(err) => vec![err],
            };
            state.submitted = false;
        }
        PainAction::Validate => {
            state.errors = state.payload.validate();
        }
        PainAction::SetMessageId(id) => {
            pain_message_id().get_mut(&mut state.payload).map(|msg| *msg = id);
            state.errors.clear();
            state.submitted = false;
        }
        PainAction::Submit => {
            state.errors = state.payload.validate();
            state.submitted = state.errors.is_empty();
        }
    }
    Cmd::none()
}

fn subscriptions(_: &PainAppState) -> Sub<PainAction> {
    Sub::none()
}

fn print_report(label: &str, state: &PainAppState) {
    println!("\n--- {label} ---");
    println!(
        "MsgId = {:?}",
        pain_message_id().get(&state.payload)
    );
    println!("errors: {}", state.errors.len());
    for err in &state.errors {
        println!("  {}: {}", err.path, err.message);
    }
    println!("submitted: {}", state.submitted);
}

fn dispatch(store: &rust_elm::Store<PainAppState, PainAction>, action: PainAction) {
    let _ = store.send(action).finish();
}

fn main() {
    println!("=== PAIN.001 ISO 20022 — elm state + keypath validation ===");

    let program = ReducerProgram::new(Reduce::new(pain_reducer), init, subscriptions);
    let runtime = Runtime::from_reducer_program(program, Environment::new(), 16);
    let store = runtime.store();

    dispatch(&store, PainAction::Validate);
    print_report("valid sample", &store.state());
    assert!(store.state().errors.is_empty());

    dispatch(&store, PainAction::LoadInvalid);
    dispatch(&store, PainAction::CheckMandatory);
    print_report("invalid sample — mandatory check (fail-fast)", &store.state());
    {
        let s = store.state();
        assert_eq!(s.errors.len(), 1, "mandatory check returns a single error");
        assert_eq!(s.errors[0].path, "GrpHdr/MsgId");
    }

    dispatch(&store, PainAction::Validate);
    print_report("invalid sample — full validate (short-circuits on mandatory)", &store.state());
    assert!(!store.state().errors.is_empty());

    dispatch(&store, PainAction::SetMessageId("MSG-FIXED-2024".into()));
    dispatch(&store, PainAction::Validate);
    let after_fix = store.state();
    assert!(
        !after_fix
            .errors
            .iter()
            .any(|e| e.path == "GrpHdr/MsgId"),
        "MsgId error should clear after SetMessageId"
    );

    dispatch(&store, PainAction::Submit);
    print_report("submit (still invalid)", &store.state());
    assert!(!store.state().submitted);

    dispatch(&store, PainAction::LoadValid);
    dispatch(&store, PainAction::Submit);
    print_report("submit valid", &store.state());
    assert!(store.state().submitted);

    runtime.shutdown();
    println!("\npain example OK");
}
