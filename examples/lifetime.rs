struct Test {
    a: String
}

impl Test {
    fn a_get() -> impl for<'a> Fn(&'a Test) -> Option<&'a String> {
        |t: &Test| Some(&t.a)
    }
}

fn main() {
    let mut ins = Test { a: String::from("test")};
    let c = Test::a_get();
    if let Some(s) = (c)(& ins) {
        println!("s = {:?}", s);

    }

    if let Some(s) = (c)(& ins) {
        println!("s = {:?}", s);

    }


    let mut_borrowed = &mut ins;


}

