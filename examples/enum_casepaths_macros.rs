use key_paths_core::KeyPaths;
use key_paths_derive::{Casepaths, Keypaths};

#[derive(Debug, Clone, Keypaths)]
struct User {
    id: u32,
    name: String,
}

#[derive(Debug, Casepaths)]
enum Status {
    Active(User),
    Inactive,
}

fn main() {
    let status = Status::Active(User {
        id: 1,
        name: "Ada".into(),
    });

    let kp_active = Status::active_case_r();
    let active_name = Status::active_case_r().compose(User::name_r());
    println!("Active name = {:?}", active_name.get(&status));

    let mut status2 = Status::Active(User {
        id: 2,
        name: "Bob".into(),
    });
    let kp_active_w = Status::active_case_w();
    if let Some(user) = kp_active_w.get_mut(&mut status2) {
        user.name.push_str("_edited");
    }
    println!("Status2 = {:?}", status2);

    // Embedding via readable enum
    let embedded = kp_active.embed(User {
        id: 3,
        name: "Cleo".into(),
    });
    println!("Embedded = {:?}", embedded);
}
