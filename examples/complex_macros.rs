use key_paths_core::KeyPaths;
use key_paths_derive::{Casepaths, Keypaths};

#[derive(Debug, Keypaths)]
struct Profile {
    display_name: String,
    age: u32,
}

#[derive(Debug, Keypaths)]
struct User {
    id: u64,
    profile: Option<Profile>,
    tags: Vec<String>,
}

#[derive(Debug, Keypaths)]
struct DbConfig(u16, String); // (port, url)

#[derive(Debug, Keypaths)]
struct Settings {
    theme: String,
    db: Option<DbConfig>,
}

#[derive(Debug, Casepaths)]
enum Connection {
    Disconnected,
    Connecting(u32),
    Connected(String),
}

#[derive(Debug, Casepaths)]
enum Status {
    Active(User),
    Inactive,
    Pending(u32),
}

#[derive(Debug, Keypaths)]
struct App {
    users: Vec<User>,
    settings: Option<Settings>,
    connection: Connection,
    name: String,
}

fn main() {
    let mut app = App {
        users: vec![
            User {
                id: 1,
                profile: Some(Profile {
                    display_name: "Ada".into(),
                    age: 31,
                }),
                tags: vec!["admin".into(), "founder".into()],
            },
            User {
                id: 2,
                profile: None,
                tags: vec!["guest".into()],
            },
        ],
        settings: Some(Settings {
            theme: "dark".into(),
            db: Some(DbConfig(5432, "postgres://localhost".into())),
        }),
        connection: Connection::Connecting(42),
        name: "MegaApp".into(),
    };

    // 1) Read a nested optional field via failable readable compose
    let first_user_profile_name = App::users_r()
        .compose(KeyPaths::failable_readable(|v: &Vec<User>| v.first()))
        .compose(User::profile_fr())
        .compose(Profile::display_name_r());
    println!(
        "first_user_profile_name = {:?}",
        first_user_profile_name.get(&app)
    );

    // 2) Mutate nested Option chain via failable writable
    let settings_fw = App::settings_fw();
    let db_fw = Settings::db_fw();
    let db_port_w = DbConfig::f0_w();
    if let Some(settings) = settings_fw.get_mut(&mut app) {
        if let Some(db) = db_fw.get_mut(settings) {
            if let Some(port) = db_port_w.get_mut(db) {
                *port += 1;
            }
        }
    }
    println!(
        "db after bump = {:?}",
        app.settings.as_ref().and_then(|s| s.db.as_ref())
    );

    // 3) Compose writable + enum case (prism) to mutate only when connected
    app.connection = Connection::Connected("10.0.0.1".into());
    let connected_case = Connection::connected_case_w();
    // compose requires a keypath from App -> Connection first
    let app_connection_w = App::connection_w();
    let app_connected_ip = app_connection_w.compose(connected_case);
    if let Some(ip) = app_connected_ip.get_mut(&mut app) {
        ip.push_str(":8443");
    }
    println!("app.connection = {:?}", app.connection);

    // 4) Enum readable case path for state without payload
    app.connection = Connection::Disconnected;
    let disc = Connection::disconnected_case_r();
    println!("is disconnected? {:?}", disc.get(&app.connection).is_some());

    // 5) Iterate immutably and mutably via derived vec keypaths
    let users_r = App::users_r();
    if let Some(mut iter) = users_r.iter::<User>(&app) {
        if let Some(u0) = iter.next() {
            println!("first user id = {}", u0.id);
        }
    }
    let users_w = App::users_w();
    if let Some(iter) = users_w.iter_mut::<User>(&mut app) {
        for u in iter {
            u.tags.push("seen".into());
        }
    }
    println!("users after tag = {:?}", app.users);

    // 6) Compose across many levels: first user -> profile -> age (if present) and increment
    let first_user_fr = KeyPaths::failable_readable(|v: &Vec<User>| v.first());
    let profile_fr = User::profile_fr();
    let age_w = Profile::age_w();
    if let Some(u0) = first_user_fr.get(&app.users) {
        // borrow helper
        let mut app_ref = &mut app.users[0];
        if let Some(p) = profile_fr.get_mut(&mut app_ref) {
            if let Some(age) = age_w.get_mut(p) {
                *age += 1;
            }
        }
    }
    println!("first user after bday = {:?}", app.users.first());

    // 7) Embed: build a Connected from payload
    let connected_r = Connection::connected_case_r();
    let new_conn = connected_r.embed("192.168.0.1".to_string());
    println!("embedded = {:?}", new_conn);

    // 8) Additional enum with casepaths: Status
    let mut st = Status::Active(User {
        id: 99,
        profile: None,
        tags: vec![],
    });
    let st_active = Status::active_case_r();
    let st_active_name = st_active.compose(User::id_r());
    println!("status active user id = {:?}", st_active_name.get(&st));

    let st_pending = Status::pending_case_w();
    st = Status::Pending(5);
    if let Some(v) = st_pending.get_mut(&mut st) {
        *v += 1;
    }
    println!("status after pending increment = {:?}", st);
}
