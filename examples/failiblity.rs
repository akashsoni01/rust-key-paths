use keypaths_proc::Keypaths;


// #[derive(Keypaths)]
struct SomeStruct{
    f1: String
}

#[derive(Keypaths)]
enum SomeEnum {
    active, 
    passive(String)
}

impl SomeEnum {
    fn active() {}
}

fn main() {
    let x = SomeEnum::active;
    let y = SomeEnum::active();
}
