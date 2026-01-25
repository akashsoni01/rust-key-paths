// Demonstrates using rust-key-paths for building dynamic queries
// This example shows how to:
// 1. Build type-safe query filters using keypaths
// 2. Chain multiple predicates together
// 3. Execute queries on in-memory data
// 4. Access nested fields in query predicates
// cargo run --example query_builder

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::Kp;

#[derive(Debug, Clone, Kp)]
#[All]
struct Product {
    name: String,
    price: f64,
    details: ProductDetails,
}

#[derive(Debug, Clone, Kp)]
#[All]
struct ProductDetails {
    category: String,
    in_stock: bool,
    rating: f64,
}

// Query builder using keypaths
// This provides a fluent API for building complex queries over collections
// Key benefits:
// - Type-safe field access through keypaths
// - Composable filters that can be chained
// - Works with nested fields seamlessly
// - Zero runtime overhead after compilation
struct Query<T: 'static> {
    filters: Vec<Box<dyn Fn(&T) -> bool>>,
}

impl<T: 'static> Query<T> {
    fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    // Add a filter predicate using a keypath
    // The keypath provides type-safe access to the field,
    // and the predicate defines the filtering logic
    // Note: Use readable keypaths (_r) for queries since we only need read access
    fn where_<F, P>(mut self, path: KeyPath<T, F, P>, predicate: impl Fn(&F) -> bool + 'static) -> Self
    where
        F: 'static,
        P: for<'r> Fn(&'r T) -> &'r F + 'static,
    {
        let path_rc = std::rc::Rc::new(path);
        let path_clone = path_rc.clone();
        self.filters.push(Box::new(move |item| {
            predicate(path_clone.get(item))
        }));
        self
    }

    // Add a filter predicate using an optional keypath
    // This handles cases where the field might not exist (Option, nested fields, etc.)
    fn where_optional<F, P>(mut self, path: OptionalKeyPath<T, F, P>, predicate: impl Fn(&F) -> bool + 'static) -> Self
    where
        F: 'static,
        P: for<'r> Fn(&'r T) -> Option<&'r F> + 'static,
    {
        let path_rc = std::rc::Rc::new(path);
        let path_clone = path_rc.clone();
        self.filters.push(Box::new(move |item| {
            path_clone.get(item).map_or(false, |val| predicate(val))
        }));
        self
    }

    // Execute the query and return matching items
    fn execute<'a>(&self, items: &'a [T]) -> Vec<&'a T> {
        items
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .collect()
    }

    // Execute the query and return mutable references
    fn execute_mut<'a>(&self, items: &'a mut [T]) -> Vec<&'a mut T> {
        items
            .iter_mut()
            .filter(|item| self.filters.iter().all(|f| f(&**item)))
            .collect()
    }

    // Count matching items without allocating
    fn count(&self, items: &[T]) -> usize {
        items
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .count()
    }
}

// Helper function to create sample products
fn create_sample_products() -> Vec<Product> {
    vec![
        Product {
            name: "Laptop".to_string(),
            price: 999.99,
            details: ProductDetails {
                category: "Electronics".to_string(),
                in_stock: true,
                rating: 4.5,
            },
        },
        Product {
            name: "Mouse".to_string(),
            price: 29.99,
            details: ProductDetails {
                category: "Electronics".to_string(),
                in_stock: false,
                rating: 4.2,
            },
        },
        Product {
            name: "Keyboard".to_string(),
            price: 79.99,
            details: ProductDetails {
                category: "Electronics".to_string(),
                in_stock: true,
                rating: 4.7,
            },
        },
        Product {
            name: "Desk Chair".to_string(),
            price: 249.99,
            details: ProductDetails {
                category: "Furniture".to_string(),
                in_stock: true,
                rating: 4.3,
            },
        },
        Product {
            name: "Monitor".to_string(),
            price: 349.99,
            details: ProductDetails {
                category: "Electronics".to_string(),
                in_stock: true,
                rating: 4.8,
            },
        },
        Product {
            name: "Desk Lamp".to_string(),
            price: 39.99,
            details: ProductDetails {
                category: "Furniture".to_string(),
                in_stock: false,
                rating: 3.9,
            },
        },
        Product {
            name: "USB Cable".to_string(),
            price: 12.99,
            details: ProductDetails {
                category: "Electronics".to_string(),
                in_stock: true,
                rating: 4.1,
            },
        },
    ]
}

// Usage: Build type-safe queries
fn main() {
    println!("=== Query Builder Demo ===\n");

    let products = create_sample_products();

    println!("Total products in database: {}\n", products.len());

    // Query 1: Electronics, in stock, price < 1000, rating > 4.0
    println!("--- Query 1: Premium Electronics in Stock ---");
    let query1 = Query::new()
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::category_r().to_optional()),
            |cat| cat == "Electronics",
        )
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::in_stock_r().to_optional()),
            |&in_stock| in_stock,
        )
        .where_(Product::price_r(), |&price| price < 1000.0)
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::rating_r().to_optional()),
            |&rating| rating > 4.0,
        );

    let results1 = query1.execute(&products);
    println!("Found {} products:", results1.len());
    for product in results1 {
        println!(
            "  - {} (${:.2}) - Rating: {:.1} - In Stock: {}",
            product.name, product.price, product.details.rating, product.details.in_stock
        );
    }

    // Query 2: Budget items under $50
    println!("\n--- Query 2: Budget Items Under $50 ---");
    let query2 = Query::new().where_(Product::price_r(), |&price| price < 50.0);

    let results2 = query2.execute(&products);
    println!("Found {} products:", results2.len());
    for product in results2 {
        println!(
            "  - {} (${:.2}) - {}",
            product.name, product.price, product.details.category
        );
    }

    // Query 3: Out of stock items
    println!("\n--- Query 3: Out of Stock Items ---");
    let query3 = Query::new().where_optional(
        Product::details_r().to_optional().then(ProductDetails::in_stock_r().to_optional()),
        |&in_stock| !in_stock,
    );

    let results3 = query3.execute(&products);
    println!("Found {} products:", results3.len());
    for product in results3 {
        println!("  - {} (${:.2})", product.name, product.price);
    }

    // Query 4: Highly rated furniture (rating >= 4.0)
    println!("\n--- Query 4: Highly Rated Furniture ---");
    let query4 = Query::new()
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::category_r().to_optional()),
            |cat| cat == "Furniture",
        )
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::rating_r().to_optional()),
            |&rating| rating >= 4.0,
        );

    let results4 = query4.execute(&products);
    println!("Found {} products:", results4.len());
    for product in results4 {
        println!(
            "  - {} (${:.2}) - Rating: {:.1}",
            product.name, product.price, product.details.rating
        );
    }

    // Query 5: Count products by category
    println!("\n--- Query 5: Products by Category ---");
    let electronics_query = Query::new().where_optional(
        Product::details_r().to_optional().then(ProductDetails::category_r().to_optional()),
        |cat| cat == "Electronics",
    );

    let furniture_query = Query::new().where_optional(
        Product::details_r().to_optional().then(ProductDetails::category_r().to_optional()),
        |cat| cat == "Furniture",
    );

    println!("Electronics: {} products", electronics_query.count(&products));
    println!("Furniture: {} products", furniture_query.count(&products));

    // Query 6: Complex query - Mid-range products
    println!("\n--- Query 6: Mid-Range Products ($30-$300) with Good Ratings ---");
    let query6 = Query::new()
        .where_(Product::price_r(), |&price| price >= 30.0 && price <= 300.0)
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::rating_r().to_optional()),
            |&rating| rating >= 4.0,
        )
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::in_stock_r().to_optional()),
            |&in_stock| in_stock,
        );

    let results6 = query6.execute(&products);
    println!("Found {} products:", results6.len());
    for product in results6 {
        println!(
            "  - {} (${:.2}) - {} - Rating: {:.1}",
            product.name, product.price, product.details.category, product.details.rating
        );
    }

    // Demonstrate mutable query results
    println!("\n--- Applying Discount to Electronics Over $100 ---");
    let mut products_mut = products.clone();

    let discount_query = Query::new()
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::category_r().to_optional()),
            |cat| cat == "Electronics",
        )
        .where_(Product::price_r(), |&price| price > 100.0);

    let to_discount = discount_query.execute_mut(&mut products_mut);
    println!("Applying 10% discount to {} products:", to_discount.len());

    for product in to_discount {
        let old_price = product.price;
        product.price *= 0.9; // 10% discount
        println!(
            "  - {}: ${:.2} -> ${:.2}",
            product.name, old_price, product.price
        );
    }

    // Verify the changes
    println!("\n--- Verification: Electronics Over $100 (After Discount) ---");
    let verify_query = Query::new()
        .where_optional(
            Product::details_r().to_optional().then(ProductDetails::category_r().to_optional()),
            |cat| cat == "Electronics",
        )
        .where_(Product::price_r(), |&price| price > 100.0);

    let after_discount = verify_query.execute(&products_mut);
    println!("Products still over $100: {}", after_discount.len());
    for product in after_discount {
        println!("  - {} (${:.2})", product.name, product.price);
    }

    println!("\n✓ Query builder demo complete!");
}

