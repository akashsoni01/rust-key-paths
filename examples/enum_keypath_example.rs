use rust_keypaths::EnumKeyPath;

#[derive(Debug, Clone)]
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
    let cp = EnumKeyPath::readable_enum(
        |user: User| Status::Active(user),
        |u: &Status| match u {
            Status::Active(e) => Some(e),
            _ => None,
        },
    );
    // let cp2 = enum_keypath!(Status::Inactive(()));
    let cp2 = EnumKeyPath::readable_enum(
        |_unit: ()| Status::Inactive(()),
        |u| match u {
            Status::Inactive(_) => Some(&()),
            _ => None,
        },
    );

    // let cp3 = enum_keypath!(SomeOtherStatus::Active(String));
    let cp3 = EnumKeyPath::readable_enum(
        |s: String| SomeOtherStatus::Active(s),
        |u| match u {
            SomeOtherStatus::Active(e) => Some(e),
            _ => None,
        },
    );
    if let Some(x) = cp3.get(&SomeOtherStatus::Active("Hello".to_string())) {
        println!("Active: {:?}", x);
    }

    // let cp4 = enum_keypath!(SomeOtherStatus::Inactive);
    let cp4 = EnumKeyPath::readable_enum(
        |_unit: ()| SomeOtherStatus::Inactive,
        |_u| None::<&()>,
    );
    if let Some(_x) = cp4.get(&SomeOtherStatus::Inactive) {
        println!("Inactive: {:?}", _x);
    }

    let status = Status::Active(User {
        id: 42,
        name: "Charlie".to_string(),
    });

    if let Some(u) = cp.get(&status) {
        println!("Extracted user: {:?}", u);
    }

    let new_status = cp.embed(User {
        id: 99,
        name: "Diana".to_string(),
    });
    println!("Embedded back: {:?}", new_status);
}
