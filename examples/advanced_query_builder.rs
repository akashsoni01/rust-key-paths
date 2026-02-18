// Demonstrates advanced query builder with SQL-like operations using keypaths
// This example shows how to:
// 1. Select specific fields (projection)
// 2. Order results by fields (ascending/descending)
// 3. Group by fields with aggregations
// 4. Limit and paginate results
// 5. Compute aggregates (count, sum, avg, min, max)
// 6. Chain complex queries
// cargo run --example advanced_query_builder

// use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
// use keypaths_proc::Kp;
use std::collections::HashMap;
use key_paths_derive::Kp;
use rust_key_paths::KpDynamic;

#[derive(Debug, Clone, Kp)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    category: String,
    stock: u32,
    rating: f64,
}

// Query builder with advanced SQL-like operations
struct Query<'a, T: 'static> {
    data: &'a [T],
    filters: Vec<Box<dyn Fn(&T) -> bool + 'a>>,
}

impl<'a, T: 'static + Clone> Query<'a, T> {
    fn new(data: &'a [T]) -> Self {
        Self {
            data,
            filters: Vec::new(),
        }
    }

    // Add a filter predicate (KpDynamic::get returns Option<&F>)
    fn where_<F>(mut self, path: KpDynamic<T, F>, predicate: impl Fn(&F) -> bool + 'static) -> Self
    where
        F: 'static,
    {
        self.filters.push(Box::new(move |item: &T| {
            path.get(item).is_some_and(&predicate)
        }));
        self
    }

    // Execute and get all matching items
    fn all(&self) -> Vec<&T> {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .collect()
    }

    // Get first matching item
    fn first(&self) -> Option<&T> {
        self.data
            .iter()
            .find(|item| self.filters.iter().all(|f| f(item)))
    }

    // Count matching items
    fn count(&self) -> usize {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .count()
    }

    // Limit results
    fn limit(&self, n: usize) -> Vec<&T> {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .take(n)
            .collect()
    }

    // Skip and limit (pagination)
    fn skip<'b>(&'b self, offset: usize) -> QueryWithSkip<'a, 'b, T> {
        QueryWithSkip {
            query: self,
            offset,
        }
    }

    // Order by a field (ascending) - for types that implement Ord (KpDynamic::get returns Option<&F>)
    fn order_by<F>(&self, path: KpDynamic<T, F>) -> Vec<T>
    where
        F: Ord + Clone + 'static,
    {
        let mut results: Vec<T> = self
            .data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .cloned()
            .collect();
        results.sort_by(|a, b| path.get(a).cmp(&path.get(b)));
        results
    }

    // Order by a field (descending) - for types that implement Ord
    fn order_by_desc<F>(&self, path: KpDynamic<T, F>) -> Vec<T>
    where
        F: Ord + Clone + 'static,
    {
        let mut results: Vec<T> = self
            .data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .cloned()
            .collect();
        results.sort_by(|a, b| path.get(b).cmp(&path.get(a)));
        results
    }

    // Order by a float field (ascending) - for f64
    fn order_by_float(&self, path: KpDynamic<T, f64>) -> Vec<T> {
        let mut results: Vec<T> = self
            .data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .cloned()
            .collect();
        results.sort_by(|a, b| {
            let a_val = path.get(a).copied().unwrap_or(0.0);
            let b_val = path.get(b).copied().unwrap_or(0.0);
            a_val.partial_cmp(&b_val).unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    // Order by a float field (descending) - for f64
    fn order_by_float_desc(&self, path: KpDynamic<T, f64>) -> Vec<T> {
        let mut results: Vec<T> = self
            .data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .cloned()
            .collect();
        results.sort_by(|a, b| {
            let a_val = path.get(a).copied().unwrap_or(0.0);
            let b_val = path.get(b).copied().unwrap_or(0.0);
            b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    // Select/project a single field from results
    fn select<F>(&self, path: KpDynamic<T, F>) -> Vec<F>
    where
        F: Clone + 'static,
    {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .filter_map(|item| path.get(item).cloned())
            .collect()
    }

    // Group by a field
    fn group_by<F>(&self, path: KpDynamic<T, F>) -> HashMap<F, Vec<T>>
    where
        F: Eq + std::hash::Hash + Clone + 'static,
    {
        let mut groups: HashMap<F, Vec<T>> = HashMap::new();
        for item in self.data.iter() {
            if self.filters.iter().all(|f| f(item)) {
                if let Some(key) = path.get(item).cloned() {
                    groups.entry(key).or_default().push(item.clone());
                }
            }
        }
        groups
    }

    // Aggregate functions
    fn sum<F>(&self, path: KpDynamic<T, F>) -> F
    where
        F: Clone + std::ops::Add<Output = F> + Default + 'static,
    {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .filter_map(|item| path.get(item).cloned())
            .fold(F::default(), |acc, val| acc + val)
    }

    fn avg(&self, path: KpDynamic<T, f64>) -> Option<f64> {
        let items: Vec<f64> = self
            .data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .filter_map(|item| path.get(item).copied())
            .collect();
        if items.is_empty() {
            None
        } else {
            Some(items.iter().sum::<f64>() / items.len() as f64)
        }
    }

    fn min<F>(&self, path: KpDynamic<T, F>) -> Option<F>
    where
        F: Ord + Clone + 'static,
    {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .filter_map(|item| path.get(item).cloned())
            .min()
    }

    fn max<F>(&self, path: KpDynamic<T, F>) -> Option<F>
    where
        F: Ord + Clone + 'static,
    {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .filter_map(|item| path.get(item).cloned())
            .max()
    }

    fn min_float(&self, path: KpDynamic<T, f64>) -> Option<f64> {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .filter_map(|item| path.get(item).copied())
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    fn max_float(&self, path: KpDynamic<T, f64>) -> Option<f64> {
        self.data
            .iter()
            .filter(|item| self.filters.iter().all(|f| f(item)))
            .filter_map(|item| path.get(item).copied())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    // Check if any items match
    fn exists(&self) -> bool {
        self.data
            .iter()
            .any(|item| self.filters.iter().all(|f| f(item)))
    }
}

// Helper struct for pagination
struct QueryWithSkip<'a, 'b, T: 'static> {
    query: &'b Query<'a, T>,
    offset: usize,
}

impl<'a, 'b, T: 'static> QueryWithSkip<'a, 'b, T> {
    fn limit(&self, n: usize) -> Vec<&'a T> {
        self.query
            .data
            .iter()
            .filter(|item| self.query.filters.iter().all(|f| f(item)))
            .skip(self.offset)
            .take(n)
            .collect()
    }
}

// Helper function to create sample products
fn create_product_catalog() -> Vec<Product> {
    vec![
        Product {
            id: 1,
            name: "Laptop Pro".to_string(),
            price: 1299.99,
            category: "Electronics".to_string(),
            stock: 15,
            rating: 4.8,
        },
        Product {
            id: 2,
            name: "Wireless Mouse".to_string(),
            price: 29.99,
            category: "Electronics".to_string(),
            stock: 50,
            rating: 4.5,
        },
        Product {
            id: 3,
            name: "Mechanical Keyboard".to_string(),
            price: 129.99,
            category: "Electronics".to_string(),
            stock: 30,
            rating: 4.7,
        },
        Product {
            id: 4,
            name: "Office Chair".to_string(),
            price: 299.99,
            category: "Furniture".to_string(),
            stock: 20,
            rating: 4.6,
        },
        Product {
            id: 5,
            name: "Standing Desk".to_string(),
            price: 499.99,
            category: "Furniture".to_string(),
            stock: 10,
            rating: 4.9,
        },
        Product {
            id: 6,
            name: "USB-C Hub".to_string(),
            price: 49.99,
            category: "Electronics".to_string(),
            stock: 100,
            rating: 4.3,
        },
        Product {
            id: 7,
            name: "Monitor 27\"".to_string(),
            price: 349.99,
            category: "Electronics".to_string(),
            stock: 25,
            rating: 4.7,
        },
        Product {
            id: 8,
            name: "Desk Lamp".to_string(),
            price: 39.99,
            category: "Furniture".to_string(),
            stock: 40,
            rating: 4.2,
        },
        Product {
            id: 9,
            name: "Webcam HD".to_string(),
            price: 79.99,
            category: "Electronics".to_string(),
            stock: 35,
            rating: 4.4,
        },
        Product {
            id: 10,
            name: "Bookshelf".to_string(),
            price: 149.99,
            category: "Furniture".to_string(),
            stock: 15,
            rating: 4.5,
        },
    ]
}

fn main() {
    println!("=== Advanced Query Builder Demo ===\n");

    let products = create_product_catalog();
    println!("Total products in catalog: {}\n", products.len());

    // Query 1: Select all product names
    println!("--- Query 1: Select All Product Names ---");
    let names = Query::new(&products).select(Product::name().into());
    println!("Product names ({}):", names.len());
    for name in &names {
        println!("  • {}", name);
    }

    // Query 2: Order by price (ascending)
    println!("\n--- Query 2: Products Ordered by Price (Ascending) ---");
    let ordered = Query::new(&products).order_by_float(Product::price().into());
    for product in ordered.iter().take(5) {
        println!("  • {} - ${:.2}", product.name, product.price);
    }

    // Query 3: Order by rating (descending)
    println!("\n--- Query 3: Top-Rated Products (Descending) ---");
    let top_rated = Query::new(&products).order_by_float_desc(Product::rating().into());
    for product in top_rated.iter().take(5) {
        println!("  • {} - Rating: {:.1}", product.name, product.rating);
    }

    // Query 4: Group by category
    println!("\n--- Query 4: Products Grouped by Category ---");
    let by_category = Query::new(&products).group_by(Product::category().into());
    for (category, items) in &by_category {
        println!("  {}: {} products", category, items.len());
        for item in items {
            println!("    - {} (${:.2})", item.name, item.price);
        }
    }

    // Query 5: Aggregations - Electronics statistics
    println!("\n--- Query 5: Electronics Category Statistics ---");
    let electronics_query =
        Query::new(&products).where_(Product::category().into(), |cat| cat == "Electronics");

    println!("  Count: {}", electronics_query.count());
    println!(
        "  Total Value: ${:.2}",
        electronics_query.sum(Product::price().into())
    );
    println!(
        "  Average Price: ${:.2}",
        electronics_query.avg(Product::price().into()).unwrap_or(0.0)
    );
    println!(
        "  Min Price: ${:.2}",
        electronics_query
            .min_float(Product::price().into())
            .unwrap_or(0.0)
    );
    println!(
        "  Max Price: ${:.2}",
        electronics_query
            .max_float(Product::price().into())
            .unwrap_or(0.0)
    );
    println!(
        "  Total Stock: {}",
        electronics_query.sum(Product::stock().into())
    );

    // Query 6: Complex filtering with ordering
    println!("\n--- Query 6: Electronics Under $200, Ordered by Rating ---");
    let affordable_electronics = Query::new(&products)
        .where_(Product::category().into(), |cat| cat == "Electronics")
        .where_(Product::price().into(), |&price| price < 200.0)
        .order_by_float_desc(Product::rating().into());

    for product in &affordable_electronics {
        println!(
            "  • {} - ${:.2} - Rating: {:.1}",
            product.name, product.price, product.rating
        );
    }

    // Query 7: Limit results
    println!("\n--- Query 7: First 3 Products ---");
    let query7 = Query::new(&products);
    let first_three = query7.limit(3);
    for product in &first_three {
        println!("  • {} (ID: {})", product.name, product.id);
    }

    // Query 8: Pagination
    println!("\n--- Query 8: Pagination (Page 2, 3 items per page) ---");
    let query8 = Query::new(&products);
    let page_2 = query8.skip(3).limit(3);
    for product in &page_2 {
        println!("  • {} (ID: {})", product.name, product.id);
    }

    // Query 9: First matching item
    println!("\n--- Query 9: Find First Product Over $1000 ---");
    let query9 = Query::new(&products).where_(Product::price().into(), |&price| price > 1000.0);
    let expensive = query9.first();

    if let Some(product) = expensive {
        println!("  Found: {} - ${:.2}", product.name, product.price);
    } else {
        println!("  No products found over $1000");
    }

    // Query 10: Check existence
    println!("\n--- Query 10: Check if Any Furniture Exists ---");
    let has_furniture = Query::new(&products)
        .where_(Product::category().into(), |cat| cat == "Furniture")
        .exists();
    println!("  Furniture available: {}", has_furniture);

    // Query 11: Multiple aggregations by group
    println!("\n--- Query 11: Category Statistics ---");
    let grouped = Query::new(&products).group_by(Product::category().into());

    for (category, items) in &grouped {
        let cat_query = Query::new(items);
        println!("\n  {} Statistics:", category);
        println!("    Products: {}", items.len());
        println!("    Total Value: ${:.2}", cat_query.sum(Product::price().into()));
        println!(
            "    Avg Price: ${:.2}",
            cat_query.avg(Product::price().into()).unwrap_or(0.0)
        );
        println!("    Total Stock: {}", cat_query.sum(Product::stock().into()));
        println!(
            "    Avg Rating: {:.2}",
            cat_query.avg(Product::rating().into()).unwrap_or(0.0)
        );
    }

    // Query 12: Complex multi-stage query
    println!("\n--- Query 12: Top 3 Highly-Rated Products (Rating > 4.5) by Price ---");
    let top_products = Query::new(&products)
        .where_(Product::rating().into(), |&rating| rating > 4.5)
        .order_by_float_desc(Product::price().into());

    for (i, product) in top_products.iter().take(3).enumerate() {
        println!(
            "  {}. {} - ${:.2} - Rating: {:.1}",
            i + 1,
            product.name,
            product.price,
            product.rating
        );
    }

    // Query 13: Select multiple fields (simulated with tuples)
    println!("\n--- Query 13: Select Name and Price for Electronics ---");
    let query13 = Query::new(&products).where_(Product::category().into(), |cat| cat == "Electronics");
    let electronics = query13.all();

    for product in electronics {
        println!("  • {} - ${:.2}", product.name, product.price);
    }

    // Query 14: Stock analysis
    println!("\n--- Query 14: Low Stock Alert (Stock < 20) ---");
    let low_stock = Query::new(&products)
        .where_(Product::stock().into(), |&stock| stock < 20)
        .order_by(Product::stock().into());

    for product in &low_stock {
        println!("  ⚠️  {} - Only {} in stock", product.name, product.stock);
    }

    // Query 15: Price range query with multiple conditions
    println!("\n--- Query 15: Mid-Range Products ($50-$300) with Good Ratings (>4.5) ---");
    let mid_range = Query::new(&products)
        .where_(Product::price().into(), |&price| price >= 50.0 && price <= 300.0)
        .where_(Product::rating().into(), |&rating| rating > 4.5)
        .order_by_float(Product::price().into());

    for product in &mid_range {
        println!(
            "  • {} - ${:.2} - Rating: {:.1} - Stock: {}",
            product.name, product.price, product.rating, product.stock
        );
    }

    // Query 16: Revenue calculation
    println!("\n--- Query 16: Potential Revenue by Category ---");
    let by_category = Query::new(&products).group_by(Product::category().into());

    for (category, items) in &by_category {
        let revenue: f64 = items.iter().map(|p| p.price * p.stock as f64).sum();
        println!("  {}: ${:.2}", category, revenue);
    }

    println!("\n✓ Advanced query builder demo complete!");
}
