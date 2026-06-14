//! Minimal calculator reducer — pure `reduce` without Runtime.
//!
//! ```bash
//! cargo run -p rust-elm --example calculator
//! ```

use rust_elm::{CatchReducer, Cmd, Environment, Reduce, Reducer, ReducerProgram, Runtime};

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct Calculator {
    result: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CalculatorParams {
    a: i64,
    b: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CalculatorAction {
    Add(CalculatorParams),
    Sub(CalculatorParams),
}

fn calculator_reducer(state: &mut Calculator, action: CalculatorAction) -> Cmd<CalculatorAction> {
    match action {
        CalculatorAction::Add(params) => state.result = params.a + params.b,
        CalculatorAction::Sub(params) => state.result = params.a - params.b,
    }
    Cmd::none()
}

impl Calculator {
    fn new() -> Self{
        Self{
            result: 0,
        }
    }
}
impl Calculator {
    fn init() -> (Self, Cmd<CalculatorAction>) {
        (Calculator::new(), Cmd::none())
    }
}

fn main() {
    let safe = CatchReducer::new(Reduce::new(calculator_reducer), |_: rust_elm::ReducePanic| {
        println!("paincked .........");
        Cmd::none()
    });
    let program = ReducerProgram::new(safe, init, subscriptions);
    let runtime: Runtime<Calculator, CalculatorAction> = Runtime::from_reducer_program(program, Environment::test(), 64);
    let store: rust_elm::Store<ShopState, ShopAction> = runtime.store();


    let mut calc = Calculator::default();

    safe.reduce(
        &mut calc,
        CalculatorAction::Add(CalculatorParams { a: 10, b: 5 }),
    );
    println!("10 + 5 = {}", calc.result);
    assert_eq!(calc.result, 15);

    safe.reduce(
        &mut calc,
        CalculatorAction::Sub(CalculatorParams { a: 20, b: 3 }),
    );
    println!("20 - 3 = {}", calc.result);
    assert_eq!(calc.result, 17);

    println!("calculator example OK");
}
