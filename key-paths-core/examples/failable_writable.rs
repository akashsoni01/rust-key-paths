use key_paths_core::KeyPaths;

#[derive(Debug)]
struct Engine {
    horsepower: u32,
}
#[derive(Debug)]
struct Car {
    engine: Option<Engine>,
}
#[derive(Debug)]
struct Garage {
    car: Option<Car>,
}

fn main() {
    let mut garage = Garage {
        car: Some(Car {
            engine: Some(Engine { horsepower: 120 }),
        }),
    };

    let kp_car = KeyPaths::failable_writable(|g: &mut Garage| g.car.as_mut());
    let kp_engine = KeyPaths::failable_writable(|c: &mut Car| c.engine.as_mut());
    let kp_hp = KeyPaths::failable_writable(|e: &mut Engine| Some(&mut e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.compose(kp_engine).compose(kp_hp);

    println!("{garage:?}");
    if let Some(hp) = kp.get_mut(&mut garage) {
        *hp = 200;
    }

    println!("{garage:?}");
}
