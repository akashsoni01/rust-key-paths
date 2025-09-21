// // use rust_key_paths::{FailableWritable, ReadKeyPath, Writable, WriteKeyPath};
// // use rust_key_paths::Compose;

// // ========== EXAMPLES ==========

// use rust_key_paths::{Compose, FailableWritable, ReadKeyPath, Writable, WriteKeyPath};

// // Example 1: Nested structs
// #[derive(Debug, Clone)]
// struct User {
//     name: String,
//     age: u32,
//     address: Address,
// }

// #[derive(Debug, Clone)]
// struct Address {
//     street: String,
//     city: String,
//     zip_code: String,
// }

// // Create keypaths for nested struct access
// fn user_name_keypath() -> Writable<User, String> {
//     Writable::new(
//         |user: &User| Some(&user.name),
//         |user: &mut User| Some(&mut user.name),
//         |user: &mut User, name: String| user.name = name,
//     )
// }

// fn user_address_keypath() -> Writable<User, Address> {
//     Writable::new(
//         |user: &User| Some(&user.address),
//         |user: &mut User| Some(&mut user.address),
//         |user: &mut User, address: Address| user.address = address,
//     )
// }

// fn address_city_keypath() -> Writable<Address, String> {
//     Writable::new(
//         |addr: &Address| Some(&addr.city),
//         |addr: &mut Address| Some(&mut addr.city),
//         |addr: &mut Address, city: String| addr.city = city,
//     )
// }

// // Example 2: Enum with variants
// #[derive(Debug, Clone)]
// enum Contact {
//     Email(String),
//     Phone(String),
//     Address(Address),
//     Unknown,
// }

// #[derive(Debug, Clone)]
// struct Profile {
//     name: String,
//     contact: Contact,
// }

// // Keypath for enum variant access (failable since variant might not match)
// fn contact_email_keypath() -> FailableWritable<Contact, String> {
//     FailableWritable::new(|contact: &mut Contact| {
//         match contact {
//             Contact::Email(email) => Some(email),
//             _ => None,
//         }
//     })
// }

// fn profile_contact_keypath() -> Writable<Profile, Contact> {
//     Writable::new(
//         |profile: &Profile| Some(&profile.contact),
//         |profile: &mut Profile| Some(&mut profile.contact),
//         |profile: &mut Profile, contact: Contact| profile.contact = contact,
//     )
// }

// // Example 3: Complex nested structure
// #[derive(Debug, Clone)]
// struct Company {
//     name: String,
//     employees: Vec<Employee>,
// }

// #[derive(Debug, Clone)]
// struct Employee {
//     id: u32,
//     profile: Profile,
//     salary: f64,
// }

// fn main() {
//     println!("=== Nested Struct Example ===");

//     let mut user = User {
//         name: "Alice".to_string(),
//         age: 30,
//         address: Address {
//             street: "123 Main St".to_string(),
//             city: "Springfield".to_string(),
//             zip_code: "12345".to_string(),
//         },
//     };

//     // Basic keypath usage
//     let name_kp = user_name_keypath();
//     if let Some(name) = name_kp.get(&user) {
//         println!("User name: {}", name);
//     }

//     name_kp.set(&mut user, "Bob".to_string());
//     println!("User after name change: {:?}", user);

//     // Composition: User -> Address -> City
//     let user_city_kp = user_address_keypath().then(address_city_keypath());

//     if let Some(city) = user_city_kp.get(&user) {
//         println!("User city: {}", city);
//     }

//     user_city_kp.set(&mut user, "Metropolis".to_string());
//     println!("User after city change: {:?}", user);

//     println!("\n=== Enum Example ===");

//     let mut profile = Profile {
//         name: "Charlie".to_string(),
//         contact: Contact::Email("charlie@example.com".to_string()),
//     };

//     let contact_kp = profile_contact_keypath();
//     let email_kp = contact_email_keypath();

//     // Compose profile -> contact -> email (failable)
//     let profile_email_kp = contact_kp.then(email_kp);

//     if let Some(email) = profile_email_kp.get(&profile) {
//         println!("Profile email: {}", email);
//     }

//     // This will work because contact is Email variant
//     profile_email_kp.set(&mut profile, "charlie.new@example.com".to_string());
//     println!("Profile after email change: {:?}", profile);

//     // // Change contact to Phone variant (email access will now fail)
//     // contact_kp.set(&mut profile, Contact::Phone("555-1234".to_string()));

//     if let Some(email) = profile_email_kp.get(&profile) {
//         println!("Profile email: {}", email);
//     } else {
//         println!("No email found (contact is now Phone variant)");
//     }

//     println!("\n=== Complex Nested Example ===");

//     let mut company = Company {
//         name: "Tech Corp".to_string(),
//         employees: vec![
//             Employee {
//                 id: 1,
//                 profile: Profile {
//                     name: "Dave".to_string(),
//                     contact: Contact::Email("dave@tech.com".to_string()),
//                 },
//                 salary: 50000.0,
//             },
//             Employee {
//                 id: 2,
//                 profile: Profile {
//                     name: "Eve".to_string(),
//                     contact: Contact::Phone("555-6789".to_string()),
//                 },
//                 salary: 60000.0,
//             },
//         ],
//     };

//     // Create keypath for first employee's email
//     let first_employee_kp = Writable::new(
//         |company: &Company| company.employees.first(),
//         |company: &mut Company| company.employees.first_mut(),
//         |company: &mut Company, employee: Employee| {
//             if !company.employees.is_empty() {
//                 company.employees[0] = employee;
//             }
//         },
//     );

//     let employee_profile_kp = Writable::new(
//         |employee: &Employee| Some(&employee.profile),
//         |employee: &mut Employee| Some(&mut employee.profile),
//         |employee: &mut Employee, profile: Profile| employee.profile = profile,
//     );

//     // Compose: Company -> first Employee -> Profile -> Contact -> Email
//     let company_first_employee_email_kp = first_employee_kp
//         .then(employee_profile_kp)
//         .then(profile_contact_keypath())
//         .then(contact_email_keypath());

//     if let Some(email) = company_first_employee_email_kp.get(&company) {
//         println!("First employee email: {}", email);
//     }

//     // This will work for the first employee (who has email)
//     company_first_employee_email_kp.set(
//         &mut company,
//         "dave.new@tech.com".to_string()
//     );

//     println!("Company after email change: {:?}", company);
// }

fn main() {}