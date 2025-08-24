use key_paths_core::Readable;
use key_paths_core::Writable;
use key_paths_core::WritableKeyPath;
use key_paths_core::ReadableKeyPath;
use key_paths_core::readable_keypath;
use key_paths_core::writable_keypath;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    let mut users = vec![
        User { name: "Akash".into(), age: 25 },
        User { name: "Soni".into(), age: 30 },
        User { name: "Neha".into(), age: 20 },
    ];


    // Read-only keypath
    // let name_key = ReadableKeyPath::new(|u: &User| &u.name);
    let name_key = readable_keypath!(User, name);

    // Writable keypath
    // let age_key = WritableKeyPath::new(
    //     |u: &User| &u.age,
    //     |u: &mut User| &mut u.age,
    // );
    let age_key = writable_keypath!(User, age);


    println!("Names:");
    for name in name_key.iter(&users) {
        println!("{}", name);
    }

    println!("Ages before:");
    for age in age_key.iter(&users) {
        println!("{}", age);
    }

    // Mutate agesiter
    for age in age_key.iter_mut(&mut users) {
        *age += 1;
    }

    println!("Ages after:");
    for age in age_key.iter(&users) {
        println!("{}", age);
    }



}
