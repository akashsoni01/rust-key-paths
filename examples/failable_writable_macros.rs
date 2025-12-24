use keypaths_proc::Keypaths;

#[derive(Debug, Keypaths)]
#[Writable]
struct Engine {
    horsepower: u32,
}

#[derive(Debug, Keypaths)]
#[Writable]
struct Car {
    engine: Option<Engine>,
}

#[derive(Debug, Keypaths)]
#[Writable]
struct Garage {
    car: Option<Car>,
}

#[derive(Debug, Keypaths)]
#[Writable]
struct City {
    garage: Option<Garage>,
}

fn main() {
    // Start with everything present
    let mut city = City {
        garage: Some(Garage {
            car: Some(Car {
                engine: Some(Engine { horsepower: 180 }),
            }),
        }),
    };

    // Build a failable-writable chain using derive-generated methods
    let garage_fw = City::garage_fw();
    let car_fw = Garage::car_fw();
    let engine_fw = Car::engine_fw();
    let hp_fw = Engine::horsepower_fw();

    // Mutate through the entire chain (only if each Option is Some)
    if let Some(garage) = garage_fw.get_mut(&mut city) {
        if let Some(car) = car_fw.get_mut(garage) {
            if let Some(engine) = engine_fw.get_mut(car) {
                if let Some(hp) = hp_fw.get_mut(engine) {
                    *hp += 20;
                }
            }
        }
    }

    println!("City after failable_writable chain mutation = {:?}", city);

    // Show short-circuit: with a missing link, nothing happens
    let mut city_missing = City { garage: None };
    if let Some(garage) = garage_fw.get_mut(&mut city_missing) {
        if let Some(car) = car_fw.get_mut(garage) {
            if let Some(engine) = engine_fw.get_mut(car) {
                if let Some(hp) = hp_fw.get_mut(engine) {
                    *hp += 1000; // won't be reached
                }
            }
        }
    }
    println!("City with missing path unchanged = {:?}", city_missing);
}
