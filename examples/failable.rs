use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};

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

    let kp_car = OptionalKeyPath::new(|g: &Garage| g.car.as_ref());
    let kp_engine = OptionalKeyPath::new(|c: &Car| c.engine.as_ref());
    let kp_hp = OptionalKeyPath::new(|e: &Engine| Some(&e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.then(kp_engine).then(kp_hp);

    let kp2 = OptionalKeyPath::new(|g: &Garage| {
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
