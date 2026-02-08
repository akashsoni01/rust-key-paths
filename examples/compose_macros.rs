use keypaths_proc::Kp;

#[derive(Debug, Kp)]
#[All]
struct Engine {
    horsepower: u32,
}

#[derive(Debug, Kp)]
#[All]
struct Car {
    engine: Option<Engine>,
}

#[derive(Debug, Kp)]
#[All]
struct Garage {
    car: Option<Car>,
}

#[derive(Debug, Kp)]
#[All]
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

    // Compose using derive-generated failable readable methods
    let city_hp = City::garage_fr()
        .then(Garage::car_fr())
        .then(Car::engine_fr())
        .then(Engine::horsepower_fr());

    println!("Horsepower = {:?}", city_hp.get(&city));

    // Demonstrate writable/failable-writable compose
    let mut city2 = City {
        garage: Some(Garage {
            car: Some(Car {
                engine: Some(Engine { horsepower: 100 }),
            }),
        }),
    };

    let garage_fw = City::garage_fw();
    let car_fw = Garage::car_fw();
    let engine_fw = Car::engine_fw();
    let hp_fw = Engine::horsepower_fw();

    if let Some(garage) = garage_fw.get_mut(&mut city2) {
        if let Some(car) = car_fw.get_mut(garage) {
            if let Some(engine) = engine_fw.get_mut(car) {
                if let Some(hp) = hp_fw.get_mut(engine) {
                    *hp += 23;
                }
            }
        }
    }

    println!("Updated city2 = {:?}", city2);
}
