use keypaths_proc::Kp;

#[derive(Clone, Kp)]
#[All]
struct Person {
    name: Option<String>,
    age: i32,
    nickname: Option<String>,
    title: String,
}

#[test]
fn test_attribute_scoped_keypaths() {
    let mut person = Person {
        name: Some("Akash".to_string()),
        age: 30,
        nickname: Some("Ace".to_string()),
        title: "Engineer".to_string(),
    };

    let name_r = Person::name_r();
    let readable_value = name_r.get(&person);
    assert_eq!(readable_value.as_ref(), Some(&"Akash".to_string()));

    let title_r = Person::title_r();
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

    let nickname_o = Person::nickname_o();
    let owned_direct = nickname_o.get(&person).clone();
    assert_eq!(owned_direct, Some("Ace".to_string()));
}
