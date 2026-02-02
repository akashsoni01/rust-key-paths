use keypaths_proc::Kp;

// #[derive(Kp)]
struct SomeStruct {
    f1: String,
}

#[derive(Kp)]
enum SomeEnum {
    active,
    passive(String),
}

impl SomeEnum {
    fn active() {}
}

fn main() {
    let x = SomeEnum::active;
    let y = SomeEnum::active();
}
