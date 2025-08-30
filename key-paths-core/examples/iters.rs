use key_paths_core::KeyPaths;

struct Garage {
    cars: Vec<String>,
}

fn main() {
    let kp = KeyPaths::readable(|g: &Garage| &g.cars);
    let mut g = Garage {
        cars: vec!["BMW".into(), "Tesla".into(), "Audi".into()],
    };

    // Immutable iteration
    if let Some(iter) = kp.iter::<String>(&g) {
        for c in iter {
            println!("car: {}", c);
        }
    }

    // Mutable iteration
    let kp_mut = KeyPaths::writable(|g: &mut Garage| &mut g.cars);
    if let Some(iter) = kp_mut.iter_mut::<String>(&mut g) {
        for c in iter {
            c.push_str(" 🚗");
        }
    }

    println!("{:?}", g.cars);
}
