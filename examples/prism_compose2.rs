use rust_keypaths::WritableOptionalKeyPath;

// Example usage (SOUND: User actually owns Address)
#[derive(Debug)]
struct Address {
    city: String,
    zip: String,
}

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    address: Address,
}

#[derive(Debug)]
pub enum Product {
    Book(Book),
    Electronics(Electronics),
    Apparel,
}

#[derive(Debug)]
pub struct Book {
    title: String,
    price: f64,
}

#[derive(Debug)]
pub struct Electronics {
    name: String,
    price: f64,
    warranty: u32,
}

#[derive(Debug)]
pub struct Inventory {
    items: Vec<Product>,
    shipping_cost: f64,
}

// Add a helper method to the Product enum for easy display of price
impl Product {
    fn price(&self) -> Option<&f64> {
        match self {
            Product::Book(b) => Some(&b.price),
            Product::Electronics(e) => Some(&e.price),
            _ => None,
        }
    }
}

fn main() {
    // invalid syntx as there is nothing in Apparel to read or write.
    // let kp:KeyPaths<Product, ()> = KeyPaths::writable_enum(
    //     |v| Product::Apparel,
    //     |p: &Product| match p {
    //         Product::Apparel => Some(&()),
    //         _ => None,
    //     },
    //     |p: &mut Product| match p {
    //         Product::Apparel => Some(&mut ()),
    //         _ => None,
    //     },
    // );

    let mut inventory = Inventory {
        items: vec![
            Product::Book(Book {
                title: "The Rust Programming Language".to_string(),
                price: 50.0,
            }),
            Product::Electronics(Electronics {
                name: "Smartphone".to_string(),
                price: 699.99,
                warranty: 1,
            }),
            Product::Apparel,
        ],
        shipping_cost: 5.0,
    };

    // Create writable enum keypath for Electronics variant
    let electronics_path = WritableOptionalKeyPath::writable_enum(
        |v| Product::Electronics(v),
        |p: &Product| match p {
            Product::Electronics(electronics) => Some(electronics),
            _ => None,
        },
        |p: &mut Product| match p {
            Product::Electronics(electronics) => Some(electronics),
            _ => None,
        },
    );

    let price_path = WritableOptionalKeyPath::new(|e: &mut Electronics| Some(&mut e.price));

    // Product -> Electronics -> price
    let product_to_price = electronics_path.then(price_path);

    // Apply the composed KeyPath
    if let Some(price) = product_to_price.get_mut(&mut inventory.items[1]) {
        println!("Original smartphone price: ${}", price);
        *price = 649.99;
        println!("New smartphone price: ${:?}", inventory.items[1].price());
    } else {
        println!("Path not found for this product.");
    }

    // Product -> Book -> price
    // Now, try on a product that doesn't match the path
    if let Some(_) = product_to_price.get_mut(&mut inventory.items[0]) {
        // This won't be executed
    } else {
        println!("Path not found for the book.");
    }
}
