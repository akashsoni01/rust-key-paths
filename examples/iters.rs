use rust_keypaths::{KeyPath, WritableKeyPath};

struct Garage {
    cars: Vec<String>,
}

fn main() {
    let kp = KeyPath::new(|g: &Garage| &g.cars);
    let mut g = Garage {
        cars: vec!["BMW".into(), "Tesla".into(), "Audi".into()],
    };

    // Immutable iteration
    if let Some(iter) = kp.iter::<String>(&g) {
    if let Some(iter) = kp.iter(&g) {
        for c in iter {
            println!("car: {}", c);
        }
    }

    // Mutable iteration
    let kp_mut = WritableKeyPath::new(|g: &mut Garage| &mut g.cars);
    if let Some(iter) = kp_mut.iter_mut(&mut g) {
        for c in iter {
            c.push_str(" 🚗");
        }
    }

    println!("{:?}", g.cars);
}
