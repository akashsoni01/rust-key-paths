use rust_key_paths::{ReadKeyPath, Writable, WriteKeyPath};
use rust_key_paths::Compose;

// Example usage (SOUND: User actually owns Address)
#[derive(Debug)]
struct Address {
    city: String,
    zip: String,
}

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    address: Address,
}


fn main() {
    // field keypaths
    let user_name = Writable::new(
        |user: &User| Some(&user.name),
        |user: &mut User| Some(&mut user.name),
        |user: &mut User, name: String| user.name = name,
    );

    let user_address = Writable::new(
        |user: &User| Some(&user.address),
        |user: &mut User| Some(&mut user.address),
        |user: &mut User, addr: Address| user.address = addr,
    );

    let address_city = Writable::new(
        |addr: &Address| Some(&addr.city),
        |addr: &mut Address| Some(&mut addr.city),
        |addr: &mut Address, city: String| addr.city = city,
    );

    // Compose: User -> Address -> city
    let user_city = user_address.then::<Writable<Address, String>>(address_city);

    let mut user = User {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            city: "Default".to_string(),
            zip: "00000".to_string(),
        },
    };

    // Read usage
    if let Some(name) = user_name.get(&user) {
        println!("User name: {}", name);
    }

    // Write usage
    if let Some(name_mut) = user_name.get_mut(&mut user) {
        *name_mut = "Bob".to_string();
    }
    user_name.set(&mut user, "Charlie".to_string());
    println!("User after set: {:?}", user);

    // Composed read
    if let Some(city) = user_city.get(&user) {
        println!("User city: {}", city);
    }

    // Composed write
    if let Some(city_mut) = user_city.get_mut(&mut user) {
        *city_mut = "Wonderland".to_string();
    }
    println!("User after city change: {:?}", user);
}
