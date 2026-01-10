use keypaths_proc::{Casepaths, Keypaths};

#[derive(Debug, Clone, Keypaths)]
struct User {
    id: u32,
    name: String,
}

#[derive(Debug, Casepaths)]
#[All]
enum Status {
    Active(User),
    Inactive,
}

fn main() {
    let status = Status::Active(User {
        id: 1,
        name: "Ada".into(),
    });

    let kp_active = Status::active_r();
    let active_name = Status::active_r().then(User::name_r().to_optional());
    if let Some(name) = active_name.get(&status) {
        println!("Active name = {:?}", name);
    }

    let mut status2 = Status::Active(User {
        id: 2,
        name: "Bob".into(),
    });
    let kp_active_w = Status::active_w();
    if let Some(user) = kp_active_w.get_mut(&mut status2) {
        user.name.push_str("_edited");
    }
    println!("Status2 = {:?}", status2);
    
    // Embedding via readable enum - use the generated embed function
    let embedded = Status::active_embed(User {
        id: 3,
        name: "Cleo".into(),
    });
    println!("Embedded = {:?}", embedded);
}
