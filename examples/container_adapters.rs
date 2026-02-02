// Demonstrates using keypath adapters for smart pointers and containers
// This example shows how to:
// 1. Use for_arc() for Vec<Arc<T>> collections
// 2. Use for_box() for Vec<Box<T>> collections
// 3. Use for_rc() for Vec<Rc<T>> collections
// 4. Query and filter wrapped types
// 5. Compose keypaths with adapters
// cargo run --example container_adapters

use keypaths_proc::Kp;
use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Kp)]
#[All]
struct Product {
    id: u32,
    name: String,
    price: f64,
    category: String,
    in_stock: bool,
}

#[derive(Debug, Clone, Kp)]
#[All]
struct User {
    id: u32,
    name: String,
    email: String,
    age: u32,
}

fn main() {
    println!("=== Container Adapter Demo ===\n");

    // ===== Example 1: Vec<Arc<T>> =====
    println!("--- Example 1: Vec<Arc<Product>> ---");

    let products_arc: Vec<Arc<Product>> = vec![
        Arc::new(Product {
            id: 1,
            name: "Laptop".to_string(),
            price: 999.99,
            category: "Electronics".to_string(),
            in_stock: true,
        }),
        Arc::new(Product {
            id: 2,
            name: "Mouse".to_string(),
            price: 29.99,
            category: "Electronics".to_string(),
            in_stock: true,
        }),
        Arc::new(Product {
            id: 3,
            name: "Desk".to_string(),
            price: 299.99,
            category: "Furniture".to_string(),
            in_stock: false,
        }),
    ];

    // Create adapted keypaths for Arc<Product>
    let name_path_arc = Product::name_r().for_arc_root();
    let price_path_arc = Product::price_r().for_arc_root();
    let category_path_arc = Product::category_r().for_arc_root();
    let in_stock_path_arc = Product::in_stock_r().for_arc_root();

    println!("All products:");
    for product in &products_arc {
        if let Some(name) = name_path_arc.get(product) {
            if let Some(price) = price_path_arc.get(product) {
                println!("  • {} - ${:.2}", name, price);
            }
        }
    }

    // Filter Arc<Product> using adapted keypaths
    let affordable_in_stock: Vec<&Arc<Product>> = products_arc
        .iter()
        .filter(|p| {
            price_path_arc.get(p).map_or(false, |price| *price < 100.0)
                && in_stock_path_arc.get(p).map_or(false, |stock| *stock)
        })
        .collect();

    println!("\nAffordable products in stock:");
    for product in affordable_in_stock {
        if let Some(name) = name_path_arc.get(product) {
            println!("  • {}", name);
        }
    }

    // ===== Example 2: Vec<Box<T>> =====
    println!("\n--- Example 2: Vec<Box<User>> ---");

    let users_box: Vec<Box<User>> = vec![
        Box::new(User {
            id: 1,
            name: "Akash".to_string(),
            email: "akash@example.com".to_string(),
            age: 30,
        }),
        Box::new(User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 25,
        }),
        Box::new(User {
            id: 3,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
        }),
    ];

    // Create adapted keypaths for Box<User>
    let name_path_box = User::name_r().for_box_root();
    let age_path_box = User::age_r().for_box_root();
    let email_path_box = User::email_r().for_box_root();

    println!("All users:");
    for user in &users_box {
        if let Some(name) = name_path_box.get(user) {
            if let Some(age) = age_path_box.get(user) {
                println!("  • {} ({})", name, age);
            }
        }
    }

    // Filter Box<User> using adapted keypaths
    let senior_users: Vec<&Box<User>> = users_box
        .iter()
        .filter(|u| age_path_box.get(u).map_or(false, |age| *age >= 30))
        .collect();

    println!("\nUsers 30+:");
    for user in senior_users {
        if let Some(name) = name_path_box.get(user) {
            if let Some(email) = email_path_box.get(user) {
                println!("  • {} <{}>", name, email);
            }
        }
    }

    // ===== Example 3: Vec<Rc<T>> =====
    println!("\n--- Example 3: Vec<Rc<Product>> ---");

    let products_rc: Vec<Rc<Product>> = vec![
        Rc::new(Product {
            id: 4,
            name: "Monitor".to_string(),
            price: 349.99,
            category: "Electronics".to_string(),
            in_stock: true,
        }),
        Rc::new(Product {
            id: 5,
            name: "Chair".to_string(),
            price: 199.99,
            category: "Furniture".to_string(),
            in_stock: true,
        }),
    ];

    // Create adapted keypaths for Rc<Product>
    let name_path_rc = Product::name_r().for_rc_root();
    let price_path_rc = Product::price_r().for_rc_root();
    let category_path_rc = Product::category_r().for_rc_root();

    println!("Rc products:");
    for product in &products_rc {
        if let Some(name) = name_path_rc.get(product) {
            if let Some(category) = category_path_rc.get(product) {
                println!("  • {} - {}", name, category);
            }
        }
    }

    // ===== Example 4: Mutable Access with Box =====
    println!("\n--- Example 4: Mutable Access with Box ---");

    let mut users_box_mut = users_box;
    let name_path_box_w = User::name_w().for_box_root();
    let age_path_box_w = User::age_w().for_box_root();

    // Modify through Box keypath
    if let Some(user) = users_box_mut.get_mut(0) {
        let name = name_path_box_w.get_mut(user);
        println!("  Original name: {}", name);
        *name = "Akash Smith".to_string();
        println!("  Modified name: {}", name);

        let age = age_path_box_w.get_mut(user);
        *age += 1;
        println!("  Incremented age to: {}", age);
    }

    // ===== Example 5: Grouping by Category (Arc) =====
    println!("\n--- Example 5: Grouping Arc Products by Category ---");

    use std::collections::HashMap;
    let mut by_category: HashMap<String, Vec<Arc<Product>>> = HashMap::new();

    for product in products_arc {
        if let Some(category) = category_path_arc.get(&product) {
            by_category
                .entry(category.clone())
                .or_insert_with(Vec::new)
                .push(product);
        }
    }

    for (category, products) in &by_category {
        println!("  {}: {} products", category, products.len());
        for product in products {
            if let Some(name) = name_path_arc.get(product) {
                println!("    - {}", name);
            }
        }
    }

    // ===== Example 6: Complex Filtering (Rc) =====
    println!("\n--- Example 6: Complex Filtering with Rc ---");

    let expensive_electronics: Vec<&Rc<Product>> = products_rc
        .iter()
        .filter(|p| {
            let is_electronics = category_path_rc
                .get(p)
                .map_or(false, |cat| cat == "Electronics");
            let is_expensive = price_path_rc.get(p).map_or(false, |price| *price > 200.0);
            is_electronics && is_expensive
        })
        .collect();

    println!("Expensive electronics (Rc):");
    for product in expensive_electronics {
        if let Some(name) = name_path_rc.get(product) {
            if let Some(&price) = price_path_rc.get(product) {
                println!("  • {} - ${:.2}", name, price);
            }
        }
    }

    // ===== Example 7: Mixed Container Types =====
    println!("\n--- Example 7: Comparison Across Container Types ---");

    let product_arc = Arc::new(Product {
        id: 10,
        name: "Keyboard".to_string(),
        price: 79.99,
        category: "Electronics".to_string(),
        in_stock: true,
    });

    let product_box = Box::new(Product {
        id: 11,
        name: "Keyboard".to_string(),
        price: 79.99,
        category: "Electronics".to_string(),
        in_stock: true,
    });

    let product_rc = Rc::new(Product {
        id: 12,
        name: "Keyboard".to_string(),
        price: 79.99,
        category: "Electronics".to_string(),
        in_stock: true,
    });

    // All use the same underlying keypath, just adapted
    println!("Arc:  {}", name_path_arc.get(&product_arc).unwrap());
    println!(
        "Box:  {}",
        Product::name_r().for_box_root().get(&product_box).unwrap()
    );
    println!("Rc:   {}", name_path_rc.get(&product_rc).unwrap());

    // ===== Example 8: Practical Use Case - Shared State =====
    println!("\n--- Example 8: Simulating Shared State Pattern ---");

    // Common pattern: Arc for shared ownership
    let shared_products: Vec<Arc<Product>> = vec![
        Arc::new(Product {
            id: 20,
            name: "Headphones".to_string(),
            price: 149.99,
            category: "Electronics".to_string(),
            in_stock: true,
        }),
        Arc::new(Product {
            id: 21,
            name: "Webcam".to_string(),
            price: 89.99,
            category: "Electronics".to_string(),
            in_stock: false,
        }),
    ];

    // Clone Arcs (cheap - just increments reference count)
    let thread1_products = shared_products.clone();
    let thread2_products = shared_products.clone();

    // Both threads can query using adapted keypaths
    println!("Thread 1 view:");
    for product in &thread1_products {
        if let Some(name) = name_path_arc.get(product) {
            if let Some(in_stock) = Product::in_stock_r().for_arc_root().get(product) {
                println!(
                    "  • {} - {}",
                    name,
                    if *in_stock {
                        "Available"
                    } else {
                        "Out of stock"
                    }
                );
            }
        }
    }

    println!("Thread 2 view (same data):");
    let available_count = thread2_products
        .iter()
        .filter(|p| {
            Product::in_stock_r()
                .for_arc_root()
                .get(p)
                .map_or(false, |s| *s)
        })
        .count();
    println!("  Available products: {}", available_count);

    println!("\n✓ Container adapter demo complete!");
}
