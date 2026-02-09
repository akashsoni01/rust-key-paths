use key_paths_derive::Kp;

#[derive(Debug, Clone, Kp)]
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
    // Derive-generated keypaths for struct fields
    let user_name_kp = User::name();
    let user_id_kp = User::id();

    let user = User {
        id: 7,
        name: "Ada".into(),
    };
    println!("user.name via kp = {:?}", user_name_kp.get(&user));
    println!("user.id via kp = {:?}", user_id_kp.get(&user));

    // Enum keypaths using core enum helpers
    let status_active_user = EnumKeyPath::readable_enum(
        |u: User| Status::Active(u),
        |s: &Status| match s {
            Status::Active(u) => Some(u),
            _ => None,
        },
    );

    let status_inactive_unit = EnumKeyPath::readable_enum(
        |u: ()| Status::Inactive(u),
        |s: &Status| match s {
            Status::Inactive(u) => Some(u),
            _ => None,
        },
    );

    let some_other_active = EnumKeyPath::readable_enum(
        |v: String| SomeOtherStatus::Active(v),
        |s: &SomeOtherStatus| match s {
            SomeOtherStatus::Active(v) => Some(v),
            _ => None,
        },
    );

    let status = Status::Active(User {
        id: 42,
        name: "Grace".into(),
    });

    if let Some(u) = status_active_user.get(&status) {
        println!("Extracted user: {:?}", u);
    }

    // Compose enum kp with derived struct field kp (consumes the keypath)
    let active_user_name = EnumKeyPath::readable_enum(
        |u: User| Status::Active(u),
        |s: &Status| match s {
            Status::Active(u) => Some(u),
            _ => None,
        },
    )
    .to_optional()
    .then(User::name().to_optional());

    println!("Active user name = {:?}", active_user_name.get(&status));

    let embedded = status_active_user.embed(User {
        id: 99,
        name: "Lin".into(),
    });
    println!("Embedded back: {:?}", embedded);

    let greeting = SomeOtherStatus::Active("Hello".to_string());
    if let Some(x) = some_other_active.get(&greeting) {
        println!("SomeOtherStatus::Active: {:?}", x);
    }

    let inactive = Status::Inactive(());
    if let Some(x) = status_inactive_unit.get(&inactive) {
        println!("Status::Inactive: {:?}", x);
    }
}
