use key_paths_core::FailableWritableKeyPath;

#[derive(Debug)]
struct Engine { horsepower: u32 }
#[derive(Debug)]
struct Car { engine: Option<Engine> }
#[derive(Debug)]
struct Garage { car: Option<Car> }

fn main() {
    let mut garage = Garage {
        car: Some(Car { engine: Some(Engine { horsepower: 120 }) }),
    };

    let kp_car = FailableWritableKeyPath::new(|g: &mut Garage| g.car.as_mut());
    let kp_engine = FailableWritableKeyPath::new(|c: &mut Car| c.engine.as_mut());
    let kp_hp = FailableWritableKeyPath::new(|e: &mut Engine| Some(&mut e.horsepower));

    // Compose: Garage -> Car -> Engine -> horsepower
    let kp = kp_car.compose(kp_engine).compose(kp_hp);

    if let Some(hp) = kp.try_get_mut(&mut garage) {
        *hp = 250;
    }

    println!("{garage:?}"); // Garage { car: Some(Car { engine: Some(Engine { horsepower: 250 }) }) }
}
