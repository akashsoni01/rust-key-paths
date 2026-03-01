//! Integration test: keypath! with rust_key_paths and key_paths_derive.

use key_paths_derive::Kp;
use key_paths_macros::{get, get_mut, keypath, set};

#[derive(Kp, Debug)]
struct Person {
    name: String,
    age: u32,
}

#[derive(Kp, Debug)]
struct App {
    person: Person,
}

#[test]
fn test_keypath_get_set() {
    let mut person = Person {
        name: "Akash".to_string(),
        age: 30,
    };

    let kp = keypath!(Person.name);
    assert_eq!(kp.get(&person), Some(&"Akash".to_string()));

    let name = get!(&person => Person.name);
    assert_eq!(name, Some(&"Akash".to_string()));

    set!(&mut person => (Person.name) = "Bob".to_string());
    assert_eq!(person.name, "Bob");

    let m = get_mut!(&mut person => Person.age);
    if let Some(a) = m {
        *a = 31;
    }
    assert_eq!(person.age, 31);
}

#[test]
fn test_keypath_braces_and_chain() {
    let mut app = App {
        person: Person {
            name: "Carol".to_string(),
            age: 25,
        },
    };

    let kp_braces = keypath! { App.person.Person.name };
    assert_eq!(kp_braces.get(&app), Some(&"Carol".to_string()));
    drop(kp_braces);

    keypath!(App.person.Person.name).get_mut(&mut app).map(|s| *s = "Dave".to_string());
    assert_eq!(app.person.name, "Dave");
}
