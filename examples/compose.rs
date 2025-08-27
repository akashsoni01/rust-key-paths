use key_paths_core::FailableReadableKeyPath;

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

#[derive(Debug)]
struct City {
    garage: Option<Garage>,
}

fn main() {
    let city = City {
        garage: Some(Garage {
            car: Some(Car {
                engine: Some(Engine { horsepower: 250 }),
            }),
        }),
    };

    let city_hp2 = FailableReadableKeyPath::new(|c: &City| {
        c.garage
            .as_ref()
            .and_then(|g| g.car.as_ref())
            .and_then(|car| car.engine.as_ref())
            .and_then(|e| Some(&e.horsepower)) // ✅ removed the extra Some(...)
    });

    println!("Horsepower = {:?}", (city_hp2.get)(&city));

    // compose example ----
    // compose keypath together

    let city_garage = FailableReadableKeyPath::new(|c: &City| c.garage.as_ref());
    let garage_car = FailableReadableKeyPath::new(|g: &Garage| g.car.as_ref());
    let car_engine = FailableReadableKeyPath::new(|c: &Car| c.engine.as_ref());
    let engine_hp = FailableReadableKeyPath::new(|e: &Engine| Some(&e.horsepower));

    let city_hp = city_garage
        .compose(garage_car)
        .compose(car_engine)
        .compose(engine_hp);

    println!("Horsepower = {:?}", (city_hp.get)(&city));
}
