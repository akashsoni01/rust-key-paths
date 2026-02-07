use key_paths_derive::Kp;
use rust_key_paths::KpType;

#[derive(Kp)]
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
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    // Test accessing name field
    let name_kp = Person::name();
    let name_value = name_kp.get(&person);
    assert_eq!(name_value, Some(&"Alice".to_string()));

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
    name_kp.get_mut(&mut person).map(|name| *name = "Robert".to_string());
    assert_eq!(person.name, "Robert");
}

#[test]
fn test_keypath_composition() {
    let company = Company {
        name: "Tech Corp".to_string(),
        employees: vec![
            Person {
                name: "Alice".to_string(),
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

    // employees() returns container (Vec); employees_at() returns first element
    let employees_kp = Company::employees();
    let employees = employees_kp.get(&company);
    assert_eq!(employees.as_ref().map(|e| e.len()), Some(2));

    let first_employee_kp = Company::employees_at();
    let first_employee = first_employee_kp.get(&company);
    assert_eq!(first_employee.map(|e| &e.name), Some(&"Alice".to_string()));
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
