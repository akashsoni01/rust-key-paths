use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Clone, Keypaths)]
struct Person {
    #[Readable]
    name: Option<String>,
    #[Writable]
    age: i32,
    #[Owned]
    nickname: Option<String>,
}

#[test]
fn test_attribute_scoped_keypaths() {
    let mut person = Person {
        name: Some("Alice".to_string()),
        age: 30,
        nickname: Some("Ace".to_string()),
    };
    
    let name_r: KeyPaths<Person, Option<String>> = Person::name_r();
    let name_fr: KeyPaths<Person, String> = Person::name_fr();
    let readable_value = name_r
        .get(&person)
        .and_then(|opt| opt.as_ref());
    assert_eq!(readable_value, Some(&"Alice".to_string()));

    let failable_read = name_fr.get(&person);
    assert_eq!(failable_read, Some(&"Alice".to_string()));

    let age_w: KeyPaths<Person, i32> = Person::age_w();
    if let Some(age_ref) = age_w.get_mut(&mut person) {
        *age_ref += 1;
    }
    assert_eq!(person.age, 31);

    let age_fw: KeyPaths<Person, i32> = Person::age_fw();
    if let Some(age_ref) = age_fw.get_mut(&mut person) {
        *age_ref += 1;
    }
    assert_eq!(person.age, 32);

    let nickname_fo: KeyPaths<Person, String> = Person::nickname_fo();
    let owned_value = nickname_fo.get_failable_owned(person.clone());
    assert_eq!(owned_value, Some("Ace".to_string()));

    let nickname_o: KeyPaths<Person, Option<String>> = Person::nickname_o();
    let owned_direct = nickname_o.get_owned(person);
    assert_eq!(owned_direct, Some("Ace".to_string()));
}

