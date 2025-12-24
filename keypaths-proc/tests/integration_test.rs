use keypaths_proc::Keypaths;
use rust_keypaths::KeyPath;


#[derive(Clone, Keypaths)]
#[Writable]
struct Person {
    #[Readable]
    name: Option<String>,
    // #[Writable]
    age: i32,
    #[Owned]
    nickname: Option<String>,
    #[Readable]
    title: String,
}

#[test]
fn test_attribute_scoped_keypaths() {
    let mut person = Person {
        name: Some("Alice".to_string()),
        age: 30,
        nickname: Some("Ace".to_string()),
        title: "Engineer".to_string(),
    };
    let name_r = Person::name_fr();
    let name_fr = Person::name_fr();
    let title_r = Person::title_r();
    let readable_value = name_r
        .get(&person);
    assert_eq!(readable_value, Some(&"Alice".to_string()));

    let failable_read = name_fr.get(&person);
    assert_eq!(failable_read, Some(&"Alice".to_string()));

    let title_value = title_r.get(&person);
    assert_eq!(title_value, &"Engineer".to_string());

    let age_w = Person::age_w();
    let mut age_ref = age_w.get_mut(&mut person);
    *age_ref += 1;
    assert_eq!(person.age, 31);

    let age_fw = Person::age_fw();
    if let Some(age_ref) = age_fw.get_mut(&mut person) {
        *age_ref += 1;
    }
    assert_eq!(person.age, 32);

    let nickname_fo = Person::nickname_fo();
    let owned_value = nickname_fo.get(&person).cloned();
    assert_eq!(owned_value, Some("Ace".to_string()));

    let nickname_o = Person::nickname_o();
    let owned_direct = nickname_o.get(&person).clone();
    assert_eq!(owned_direct, Some("Ace".to_string()));
}

