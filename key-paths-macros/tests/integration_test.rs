use key_paths_macros::Keypath;
use key_paths_core::KeyPaths;

#[derive(Keypath)]
struct Person {
    name: Option<String>,
    result_name: Result<String, String>,
    age: i32,
}

#[test]
fn test_keypath_generation() {
    let person = Person {
        name: Some("Akash".to_string()),
        result_name: Ok("Bob".to_string()),
        age: 30,
    };

    // Test that generated keypath methods work
    let name_keypath = Person::name();
    let age_keypath = Person::age();
    let name_result = Person::result_name();

    // Verify we can read values using the keypaths
    // For failable_readable, get() returns Option<&Value>
    let name_value = name_keypath.get(&person);
    let age_value = age_keypath.get(&person);
    let name_result = name_result.get(&person);

    assert_eq!(name_value, Some(&"Akash".to_string()));
    assert_eq!(age_value, Some(&30));
    assert_eq!(name_result, Some(&"Bob".to_string()));
    
    // Verify the keypaths are the correct type
    let _: KeyPaths<Person, String> = name_keypath;
    let _: KeyPaths<Person, i32> = age_keypath;
    let _: KeyPaths<Person, String> = name_keypath;
}

