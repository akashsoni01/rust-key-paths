use key_paths_core::{FailableReadableKeyPath};

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

    let kp_car = FailableReadableKeyPath::new(|g: &Garage| g.car.as_ref());
    let kp_engine = FailableReadableKeyPath::new(|c: &Car| c.engine.as_ref());
    let kp_hp = FailableReadableKeyPath::new(|e: &Engine| Some(&e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.compose(kp_engine).compose(kp_hp);

    let kp2 = FailableReadableKeyPath::new(|g: &Garage| {
        g.car
            .as_ref()
            .and_then(|c| c.engine.as_ref())
            .and_then(|e| Some(&e.horsepower))
    });

    if let Some(hp) = kp.try_get(&garage) {
        println!("{hp:?}");
    }

    if let Some(hp) = kp2.try_get(&garage) {
        println!("{hp:?}");
    }

    println!("{garage:?}");
}
