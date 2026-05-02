use key_paths_derive::{Akp, Kp, Pkp};
use rust_key_paths::KpType;
use std::collections::{HashMap, HashSet};

#[derive(Kp, Pkp, Akp)]
struct Person {
    name: String,
    age: i32,
    email: String,
}

#[derive(Kp)]
struct Company {
    name: String,
    employees: Vec<Person>,
}

#[test]
fn test_basic_field_access() {
    let person = Person {
        name: "Akash".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    // Test accessing name field
    let name_kp = Person::name();
    let name_value = name_kp.get(&person);
    assert_eq!(name_value, Some(&"Akash".to_string()));

    // Test accessing age field
    let age_kp = Person::age();
    let age_value = age_kp.get(&person);
    assert_eq!(age_value, Some(&30));

    // Test accessing email field
    let email_kp = Person::email();
    let email_value = email_kp.get(&person);
    assert_eq!(email_value, Some(&"alice@example.com".to_string()));
}

#[test]
fn test_mutable_access() {
    let mut person = Person {
        name: "Bob".to_string(),
        age: 25,
        email: "bob@example.com".to_string(),
    };

    // Test setting age field
    let age_kp = Person::age();
    age_kp.get_mut(&mut person).map(|age| *age = 26);
    assert_eq!(person.age, 26);

    // Test setting name field
    let name_kp = Person::name();
    name_kp
        .get_mut(&mut person)
        .map(|name| *name = "Robert".to_string());
    assert_eq!(person.name, "Robert");
}

#[test]
fn test_keypath_composition() {
    let company = Company {
        name: "Tech Corp".to_string(),
        employees: vec![
            Person {
                name: "Akash".to_string(),
                age: 30,
                email: "alice@example.com".to_string(),
            },
            Person {
                name: "Bob".to_string(),
                age: 25,
                email: "bob@example.com".to_string(),
            },
        ],
    };

    // Access company name
    let company_name_kp = Company::name();
    let company_name = company_name_kp.get(&company);
    assert_eq!(company_name, Some(&"Tech Corp".to_string()));

    // employees() returns container (Vec); employees_at(index) returns element at index
    let employees_kp = Company::employees();
    let employees = employees_kp.get(&company);
    assert_eq!(employees.as_ref().map(|e| e.len()), Some(2));

    let first_employee_kp = Company::employees_at(0);
    let first_employee = first_employee_kp.get(&company);
    assert_eq!(first_employee.map(|e| &e.name), Some(&"Akash".to_string()));
}

#[test]
fn test_type_signature() {
    // Verify that the generated methods return KpType
    let _name_kp: KpType<'static, Person, String> = Person::name();
    let _age_kp: KpType<'static, Person, i32> = Person::age();
    let _email_kp: KpType<'static, Person, String> = Person::email();
}

#[test]
fn test_multiple_structs() {
    let person = Person {
        name: "Charlie".to_string(),
        age: 35,
        email: "charlie@example.com".to_string(),
    };

    let company = Company {
        name: "Startup Inc".to_string(),
        employees: vec![],
    };

    // Both structs should have independent keypaths
    let person_name = Person::name().get(&person);
    let company_name = Company::name().get(&company);

    assert_eq!(person_name, Some(&"Charlie".to_string()));
    assert_eq!(company_name, Some(&"Startup Inc".to_string()));
}

#[test]
fn test_partial_kps() {
    let kps = Person::partial_kps();
    assert_eq!(kps.len(), 3); // name, age, email

    let person = Person {
        name: "Dave".to_string(),
        age: 40,
        email: "dave@example.com".to_string(),
    };

    // Each PKp should be able to get the value
    let name_val = kps[0].get_as::<String>(&person);
    assert_eq!(name_val, Some(&"Dave".to_string()));

    let age_val = kps[1].get_as::<i32>(&person);
    assert_eq!(age_val, Some(&40));

    let email_val = kps[2].get_as::<String>(&person);
    assert_eq!(email_val, Some(&"dave@example.com".to_string()));
}

#[test]
fn test_any_kps() {
    let kps = Person::any_kps();
    assert_eq!(kps.len(), 3); // name, age, email

    let person = Person {
        name: "Eve".to_string(),
        age: 28,
        email: "eve@example.com".to_string(),
    };

    // AKp operates on &dyn Any - use get_as for typed access
    let name_val = kps[0].get_as::<Person, String>(&person);
    assert_eq!(name_val, Some(Some(&"Eve".to_string())));

    let age_val = kps[1].get_as::<Person, i32>(&person);
    assert_eq!(age_val, Some(Some(&28)));
}

#[derive(Kp)]
struct WithOptionalCollections {
    map: Option<HashMap<String, i32>>,
    set: Option<HashSet<String>>,
}

#[test]
fn option_hash_map_at_returns_value_ref() {
    let s = WithOptionalCollections {
        map: Some(HashMap::from([("k".to_string(), 42)])),
        set: Some(HashSet::from(["x".to_string()])),
    };
    assert_eq!(
        WithOptionalCollections::map_at("k".to_string()).get(&s),
        Some(&42)
    );

    let mut s = s;
    WithOptionalCollections::map_at("k".to_string())
        .get_mut(&mut s)
        .map(|v| *v += 1);
    assert_eq!(s.map.as_ref().unwrap().get("k"), Some(&43));

    assert_eq!(
        WithOptionalCollections::set_at("x".to_string()).get(&s),
        Some(&"x".to_string())
    );
}

#[derive(Kp)]
enum OptMapEnum {
    M(Option<HashMap<u8, String>>),
}

#[test]
fn option_hash_map_enum_variant_at() {
    let e = OptMapEnum::M(Some(HashMap::from([(1u8, "a".to_string())])));
    assert_eq!(OptMapEnum::m_at(1u8).get(&e), Some(&"a".to_string()));
}

#[derive(Kp)]
struct WithOptionalVec {
    items: Option<Vec<i32>>,
}

#[test]
fn option_vec_at_index() {
    let s = WithOptionalVec {
        items: Some(vec![10, 20]),
    };
    assert_eq!(WithOptionalVec::items_at(1).get(&s), Some(&20));

    let mut s = s;
    WithOptionalVec::items_at(0)
        .get_mut(&mut s)
        .map(|x| *x = 99);
    assert_eq!(s.items, Some(vec![99, 20]));
}

#[derive(Kp)]
enum OptVecEnum {
    V(Option<Vec<u8>>),
}

#[test]
fn option_vec_enum_variant_at() {
    let e = OptVecEnum::V(Some(vec![1, 2, 3]));
    assert_eq!(OptVecEnum::v_at(2).get(&e), Some(&3u8));
}
