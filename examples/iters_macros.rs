use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
#[All]
struct Garage {
    cars: Vec<String>,
}

fn main() {
    let cars_kp = Garage::cars_r();
    let mut g = Garage {
        cars: vec!["BMW".into(), "Tesla".into(), "Audi".into()],
    };

    // Immutable iteration
    if let Some(iter) = cars_kp.iter::<String>(&g) {
        for c in iter {
            println!("car: {}", c);
        }
    }

    // Mutable iteration via writable keypath
    let cars_kp_mut = Garage::cars_w();
    if let Some(iter) = cars_kp_mut.iter_mut::<String>(&mut g) {
        for c in iter {
            c.push_str(" 🚗");
        }
    }

    println!("{:?}", g.cars);
}
