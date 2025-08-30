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
    let garage = Garage {
        car: Some(Car {
            engine: Some(Engine { horsepower: 120 }),
        }),
    };

    let kp_car = KeyPaths::failable_readable(|g: &Garage| g.car.as_ref());
    let kp_engine = KeyPaths::failable_readable(|c: &Car| c.engine.as_ref());
    let kp_hp = KeyPaths::failable_readable(|e: &Engine| Some(&e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.compose(kp_engine).compose(kp_hp);

    let kp2 = KeyPaths::failable_readable(|g: &Garage| {
        g.car
            .as_ref()
            .and_then(|c| c.engine.as_ref())
            .and_then(|e| Some(&e.horsepower))
    });

    if let Some(hp) = kp.get(&garage) {
        println!("{hp:?}");
    }

    if let Some(hp) = kp2.get(&garage) {
        println!("{hp:?}");
    }

    println!("{garage:?}");
}
