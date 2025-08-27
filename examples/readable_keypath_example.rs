use key_paths_core::{ReadableKeyPath, readable_keypath};

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    let users = vec![
        User {
            name: "Akash".into(),
            age: 25,
        },
        User {
            name: "Soni".into(),
            age: 30,
        },
        User {
            name: "Neha".into(),
            age: 20,
        },
    ];

    // Read-only keypath
    // let name_key = ReadableKeyPath::new(|u: &User| &u.name);
    let name_key = readable_keypath!(User, name);

    // Writable keypath
    // let age_key = WritableKeyPath::new(
    //     |u: &User| &u.age,
    //     |u: &mut User| &mut u.age,
    // );
    // let age_key = writable_keypath!(User, age);

    println!("Names:");
    for name in name_key.iter(&users) {
        println!("{}", name);
    }
}
