use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Keypaths)]
struct Engine {
    horsepower: u32,
}

#[derive(Debug, Keypaths)]
struct Car {
    engine: Option<Engine>,
}

#[derive(Debug, Keypaths)]
struct Garage {
    car: Option<Car>,
}

#[derive(Debug, Keypaths)]
struct City {
    garage: Option<Garage>,
}

fn main() {
    let mut city = City {
        garage: Some(Garage {
            car: Some(Car {
                engine: Some(Engine { horsepower: 120 }),
            }),
        }),
    };

    // Failable readable chain via derive-generated methods on Option fields
    let city_hp = City::garage_fr()
        .compose(Garage::car_fr())
        .compose(Car::engine_fr())
        .compose(Engine::horsepower_r());

    println!("Horsepower (read) = {:?}", city_hp.get(&city));

    // Failable writable chain via derive-generated methods
    let garage_fw = City::garage_fw();
    let car_fw = Garage::car_fw();
    let engine_fw = Car::engine_fw();
    let hp_w = Engine::horsepower_w();

    if let Some(garage) = garage_fw.get_mut(&mut city) {
        if let Some(car) = car_fw.get_mut(garage) {
            if let Some(engine) = engine_fw.get_mut(car) {
                if let Some(hp) = hp_w.get_mut(engine) {
                    *hp += 30;
                }
            }
        }
    }

    println!("City after hp increment = {:?}", city);

    // Demonstrate short-circuiting when any Option is None
    let mut city2 = City { garage: None };
    println!("Missing chain get = {:?}", city_hp.get(&city2));
    if let Some(garage) = garage_fw.get_mut(&mut city2) {
        // won't run
        let _ = garage;
    } else {
        println!("No garage to mutate");
    }
}


