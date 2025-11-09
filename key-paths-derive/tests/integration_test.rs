use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;
use std::sync::{Arc, RwLock};

#[derive(Clone, Keypaths)]
struct Person {
    name: Option<String>,
    #[Writable]
    age: i32,
    #[Owned]
    nickname: Option<String>,
    title: String,
    inner: Arc<RwLock<InnerStruct>>,
}

impl Person {
    fn new() -> Self {
        Person {
            name: Some("Alice".to_string()),
            age: 30,
            nickname: Some("Ace".to_string()),
            title: "Engineer".to_string(),
            inner: Arc::new(RwLock::new(InnerStruct {
                inner: Arc::new(RwLock::new(InnerInnerStruct {
                    name: "Alice".to_string(),
                })),
            })),
        }
    }
}

#[derive(Clone, Keypaths)]
#[All]
struct InnerStruct {
    inner: Arc<RwLock<InnerInnerStruct>>,
}

#[derive(Clone, Keypaths)]
#[All]
struct InnerInnerStruct {
    name: String,
}

#[test]
fn test_attribute_scoped_keypaths() {
    let mut person = Person::new();
    let name_r: KeyPaths<Person, Option<String>> = Person::name();
    let name_fr: KeyPaths<Person, String> = Person::name_fr();
    let title_r: KeyPaths<Person, String> = Person::title();
    let readable_value = name_r.get(&person).and_then(|opt| opt.as_ref());
    assert_eq!(readable_value, Some(&"Alice".to_string()));

    let failable_read = name_fr.get(&person);
    assert_eq!(failable_read, Some(&"Alice".to_string()));

    let title_value = title_r.get(&person);
    assert_eq!(title_value, Some(&"Engineer".to_string()));

    let age_w: KeyPaths<Person, i32> = Person::age();
    if let Some(age_ref) = age_w.get_mut(&mut person) {
        *age_ref += 1;
    }
    assert_eq!(person.age, 31);

    if let Some(age_ref) = age_w.get_mut(&mut person) {
        *age_ref += 1;
    }
    assert_eq!(person.age, 32);

    let nickname_fo: KeyPaths<Person, String> = Person::nickname_fo();
    let owned_value = nickname_fo.get_failable_owned(person.clone());
    assert_eq!(owned_value, Some("Ace".to_string()));

    let nickname_owned: KeyPaths<Person, Option<String>> = Person::nickname();
    let owned_direct = nickname_owned.get_owned(person);
    assert_eq!(owned_direct, Some("Ace".to_string()));

    // let kp = Person::inner_fr();
    // let kp = InnerStruct::inner_r();
    // let kp = InnerInnerStruct::name_fr();
    //
    let inner_arc_key = InnerStruct::inner().for_arc_rwlock();
    let name_arc_key = InnerInnerStruct::name().for_arc_rwlock();

    let person = Person::new();
    let inner_arc = Person::inner().get(&person).cloned().unwrap();
    let inner_inner_arc = inner_arc_key.clone().get_failable_owned(inner_arc).unwrap();
    let owned_result = name_arc_key.get_failable_owned(inner_inner_arc);

    assert_eq!(owned_result, Some("Alice".to_string()));
}
