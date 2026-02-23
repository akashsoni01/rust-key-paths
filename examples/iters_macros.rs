use key_paths_derive::Kp;

#[derive(Debug, Kp)]
struct Garage {
    cars: Vec<String>,
}

fn main() {
    let cars_kp = Garage::cars();
    let mut g = Garage {
        cars: vec!["BMW".into(), "Tesla".into(), "Audi".into()],
    };

    // Immutable iteration
    if let Some(iter) = cars_kp.get(&g) {
        for c in iter {
            println!("car: {}", c);
        }
    }

    // Mutable iteration via writable keypath
    let cars_kp_mut = Garage::cars();
    if let Some(iter) = cars_kp_mut.get_mut(&mut g) {
        for c in iter {
            c.push_str(" 🚗");
        }
    }

    println!("{:?}", g.cars);
}
