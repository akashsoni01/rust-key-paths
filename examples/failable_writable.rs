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
    let mut garage = Garage {
        car: Some(Car {
            engine: Some(Engine { horsepower: 120 }),
        }),
    };

    let kp_car = WritableOptionalKeyPath::new(|g: &mut Garage| g.car.as_mut());
    let kp_engine = WritableOptionalKeyPath::new(|c: &mut Car| c.engine.as_mut());
    let kp_hp = WritableOptionalKeyPath::new(|e: &mut Engine| Some(&mut e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.then(kp_engine).then(kp_hp);

    println!("{garage:?}");
    let hp = kp.get_mut(&mut garage);
    {
        *hp = 200;
    }

    println!("{garage:?}");
}
