use key_paths_macros::Keypath;
use key_paths_core::KeyPaths;

#[derive(Keypath)]
struct Person {
    name: String,
    age: i32,
}

#[test]
fn test_keypath_generation() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    // Test that generated keypath methods work
    let name_keypath = Person::name();
    let age_keypath = Person::age();

    // Verify we can read values using the keypaths
    // For failable_readable, get() returns Option<&Value>
    let name_value = name_keypath.get(&person);
    let age_value = age_keypath.get(&person);

    assert_eq!(name_value, Some(&"Alice".to_string()));
    assert_eq!(age_value, Some(&30));
    
    // Verify the keypaths are the correct type
    let _: KeyPaths<Person, String> = name_keypath;
    let _: KeyPaths<Person, i32> = age_keypath;
}

