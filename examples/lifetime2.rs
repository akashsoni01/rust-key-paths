struct GetC<F>
where
    F: Fn(&Test) -> Option<&String>,
{
    f: F,
}
struct Test {
    a: String,
}
impl Test {
    fn a_get() -> GetC<impl Fn(&Test) -> Option<&String>> {
        GetC {
            f: |t: &Test| Some(&t.a),
        }
    }
}

fn main() {
    let mut ins = Test {
        a: String::from("test"),
    };
    let c = Test::a_get();

    println!("size = {:?}", size_of_val(&c));
    if let Some(s) = (c.f)(&ins) {
        println!("s = {:?}", s);
    }
    if let Some(s) = (c.f)(&ins) {
        println!("s = {:?}", s);
    }
    let mut_borrowed = &mut ins;
}
