use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Clone, Keypaths)]
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
    let user_name_kp = User::name_r();
    let user_id_kp = User::id_r();

    let user = User { id: 7, name: "Ada".into() };
    println!("user.name via kp = {:?}", user_name_kp.get(&user));
    println!("user.id via kp = {:?}", user_id_kp.get(&user));

    // Enum keypaths using core enum helpers
    let status_active_user = KeyPaths::readable_enum(Status::Active, |s| match s {
        Status::Active(u) => Some(u),
        _ => None,
    });

    let status_inactive_unit = KeyPaths::readable_enum(Status::Inactive, |s| match s {
        Status::Inactive(u) => Some(u),
        _ => None,
    });

    let some_other_active = KeyPaths::readable_enum(SomeOtherStatus::Active, |s| match s {
        SomeOtherStatus::Active(v) => Some(v),
        _ => None,
    });

    let status = Status::Active(User { id: 42, name: "Grace".into() });

    if let Some(u) = status_active_user.get(&status) {
        println!("Extracted user: {:?}", u);
    }

    // Compose enum kp with derived struct field kp (consumes the keypath)
    let active_user_name = KeyPaths::readable_enum(Status::Active, |s| match s {
        Status::Active(u) => Some(u),
        _ => None,
    })
    .compose(User::name_r());

    println!("Active user name = {:?}", active_user_name.get(&status));

    let embedded = status_active_user.embed(User { id: 99, name: "Lin".into() });
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


