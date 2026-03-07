//! Keypath `zip`, `zip_with`, and `zip_with_kp!`: combine keypaths on the same root.
//!
//! Like [Option::zip](Option::zip): get returns `Some((a, b))` when both keypaths succeed.
//! `zip_with` applies a transform to the pair. The macro `zip_with_kp!` zips 2–6 keypaths
//! and runs a closure on the tuple in one step.
//!
//! Run with: `cargo run --example kp_zip_example`

use rust_key_paths::{zip_with_kp, Kp, KpType};

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    city: String,
}

fn main() {
    let user = User {
        name: "Akash".to_string(),
        age: 30,
        city: "NYC".to_string(),
    };

    // Keypaths for zip / zip_with (consumed by .zip() and .zip_with())
    let name_kp: KpType<User, String> =
        Kp::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
    let age_kp: KpType<User, u32> =
        Kp::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));

    // zip: access both fields at once
    let zipped = name_kp.zip(age_kp);
    let (name, age) = zipped.get(&user).unwrap();
    assert_eq!(*name, "Akash");
    assert_eq!(*age, 30);
    println!("zip: name = {}, age = {}", name, age);

    // zip_with: combine into a single value
    let name_kp2: KpType<User, String> =
        Kp::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
    let age_kp2: KpType<User, u32> =
        Kp::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));
    let full_info = name_kp2.zip_with(age_kp2, |name: &String, age: &u32| {
        format!("{} (age {})", name, age)
    });
    let info = full_info.get(&user).unwrap();
    assert_eq!(info, "Akash (age 30)");
    println!("zip_with: {}", info);

    // zip_with_kp!: multi-field aggregation (2, 3, or more keypaths)
    let name_kp_m: KpType<User, String> =
        Kp::new(|u: &User| Some(&u.name), |u: &mut User| Some(&mut u.name));
    let age_kp_m: KpType<User, u32> =
        Kp::new(|u: &User| Some(&u.age), |u: &mut User| Some(&mut u.age));
    let city_kp: KpType<User, String> =
        Kp::new(|u: &User| Some(&u.city), |u: &mut User| Some(&mut u.city));
    let summary = zip_with_kp!(
        &user,
        |(name, age, city)| format!("{}, {} from {}", name, age, city) =>
        name_kp_m,
        age_kp_m,
        city_kp
    );
    assert_eq!(summary, Some("Akash, 30 from NYC".to_string()));
    println!("zip_with_kp!: {}", summary.as_deref().unwrap());

    println!("kp_zip_example OK");
}
