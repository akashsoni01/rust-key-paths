use key_paths_core::EnumKeyPath;
use key_paths_core::enum_keypath;

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

#[derive(Debug)]
enum Status {
    Active(User),
    Inactive(()),
}

#[derive(Debug)]
enum SomeOtherStatus {
    Active(String),
    Inactive,
}

fn main() {
    // ---------- EnumPath ----------
    let cp = enum_keypath!(Status::Active(User));
    let cp2 = enum_keypath!(Status::Inactive(()));

    let cp3 = enum_keypath!(SomeOtherStatus::Active(String));
    if let Some(x) = cp3.extract(&SomeOtherStatus::Active("Hello".to_string())) {
        println!("Active: {:?}", x);
    }

    let cp4 = enum_keypath!(SomeOtherStatus::Inactive);
    if let Some(x) = cp4.extract(&SomeOtherStatus::Inactive) {
        println!("Inactive: {:?}", x);
    }

    let status = Status::Active(User {
        id: 42,
        name: "Charlie".to_string(),
    });

    if let Some(u) = cp.extract(&status) {
        println!("Extracted user: {:?}", u);
    }

    let new_status = cp.embed(User {
        id: 99,
        name: "Diana".to_string(),
    });
    println!("Embedded back: {:?}", new_status);
}
