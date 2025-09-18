use key_paths_core::KeyPaths;
use key_paths_derive::Casepaths;

#[derive(Debug, Casepaths)]
enum Payment {
    Cash { amount: u32 },
    Card { number: String, cvv: String },
}

fn main() {
    // let kp = KeyPaths::Prism {
    //     extract: Rc::new(|p: &Payment| match p {
    //         Payment::Cash { amount } => Some(amount),
    //         _ => None,
    //     }),
    //     // embed: Rc::new(|v| Payment::Cash { amount: v }),
    //     embed: Rc::new(|v| Payment::Cash { amount: v.clone() }),
    // };
    let kp = KeyPaths::writable_enum(
        |v| Payment::Cash { amount: v },
        |p: &Payment| match p {
            Payment::Cash { amount } => Some(amount),
            _ => None,
        },
        |p: &mut Payment| match p {
            Payment::Cash { amount } => Some(amount),
            _ => None,
        },
    );

    let mut p = Payment::Cash { amount: 10 };

    println!("{:?}", p);

    if let Some(v) = kp.get_mut(&mut p) {
        *v = 34
    }
    // kp.get_mut(&mut p); // this will return none as kp is readable

    println!("{:?}", p);
}
