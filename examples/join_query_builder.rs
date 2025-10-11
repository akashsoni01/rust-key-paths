// Demonstrates JOIN operations between collections using keypaths
// This example shows how to:
// 1. Perform inner joins between collections
// 2. Perform left joins with optional results
// 3. Join on matching field values
// 4. Create multi-table queries
// 5. Use keypaths for type-safe join conditions
// cargo run --example join_query_builder

use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;
use std::collections::HashMap;

// Database schema: Users, Orders, Products
#[derive(Debug, Clone, Keypaths)]
struct User {
    id: u32,
    name: String,
    email: String,
    city: String,
}

#[derive(Debug, Clone, Keypaths)]
struct Order {
    id: u32,
    user_id: u32,
    product_id: u32,
    quantity: u32,
    total: f64,
}

#[derive(Debug, Clone, Keypaths)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    category: String,
}

// Join result types
#[derive(Debug, Clone)]
struct UserOrder {
    user_name: String,
    user_email: String,
    order_id: u32,
    quantity: u32,
    total: f64,
}

#[derive(Debug, Clone)]
struct OrderDetail {
    order_id: u32,
    user_name: String,
    product_name: String,
    quantity: u32,
    price: f64,
    total: f64,
}

#[derive(Debug, Clone)]
struct UserOrderCount {
    user_name: String,
    user_city: String,
    order_count: usize,
    total_spent: f64,
}

#[derive(Debug, Clone)]
struct CategorySales {
    category: String,
    total_orders: usize,
    total_revenue: f64,
    unique_customers: usize,
}

// Generic join builder
struct JoinQuery<'a, L: 'static, R: 'static> {
    left: &'a [L],
    right: &'a [R],
}

impl<'a, L: Clone, R: Clone> JoinQuery<'a, L, R> {
    fn new(left: &'a [L], right: &'a [R]) -> Self {
        Self { left, right }
    }

    // Inner join: returns only matching pairs
    fn inner_join<K, O, F>(&self, left_key: KeyPaths<L, K>, right_key: KeyPaths<R, K>, mapper: F) -> Vec<O>
    where
        K: Eq + std::hash::Hash + Clone + 'static,
        F: Fn(&L, &R) -> O,
    {
        // Build index for right side for O(n) lookup
        let mut right_index: HashMap<K, Vec<&R>> = HashMap::new();
        for item in self.right.iter() {
            if let Some(key) = right_key.get(item).cloned() {
                right_index.entry(key).or_insert_with(Vec::new).push(item);
            }
        }

        // Join left with indexed right
        let mut results = Vec::new();
        for left_item in self.left.iter() {
            if let Some(key) = left_key.get(left_item).cloned() {
                if let Some(right_items) = right_index.get(&key) {
                    for right_item in right_items {
                        results.push(mapper(left_item, right_item));
                    }
                }
            }
        }

        results
    }

    // Left join: returns all left items, with optional right matches
    fn left_join<K, O, F>(&self, left_key: KeyPaths<L, K>, right_key: KeyPaths<R, K>, mapper: F) -> Vec<O>
    where
        K: Eq + std::hash::Hash + Clone + 'static,
        F: Fn(&L, Option<&R>) -> O,
    {
        // Build index for right side
        let mut right_index: HashMap<K, Vec<&R>> = HashMap::new();
        for item in self.right.iter() {
            if let Some(key) = right_key.get(item).cloned() {
                right_index.entry(key).or_insert_with(Vec::new).push(item);
            }
        }

        // Join left with indexed right
        let mut results = Vec::new();
        for left_item in self.left.iter() {
            if let Some(key) = left_key.get(left_item).cloned() {
                if let Some(right_items) = right_index.get(&key) {
                    for right_item in right_items {
                        results.push(mapper(left_item, Some(right_item)));
                    }
                } else {
                    results.push(mapper(left_item, None));
                }
            } else {
                results.push(mapper(left_item, None));
            }
        }

        results
    }

    // Filter join: only matching pairs that satisfy a predicate
    fn inner_join_where<K, O, F, P>(
        &self,
        left_key: KeyPaths<L, K>,
        right_key: KeyPaths<R, K>,
        predicate: P,
        mapper: F,
    ) -> Vec<O>
    where
        K: Eq + std::hash::Hash + Clone + 'static,
        F: Fn(&L, &R) -> O,
        P: Fn(&L, &R) -> bool,
    {
        // Build index for right side
        let mut right_index: HashMap<K, Vec<&R>> = HashMap::new();
        for item in self.right.iter() {
            if let Some(key) = right_key.get(item).cloned() {
                right_index.entry(key).or_insert_with(Vec::new).push(item);
            }
        }

        // Join left with indexed right, applying predicate
        let mut results = Vec::new();
        for left_item in self.left.iter() {
            if let Some(key) = left_key.get(left_item).cloned() {
                if let Some(right_items) = right_index.get(&key) {
                    for right_item in right_items {
                        if predicate(left_item, right_item) {
                            results.push(mapper(left_item, right_item));
                        }
                    }
                }
            }
        }

        results
    }
}

// Helper function for creating sample data
fn create_sample_data() -> (Vec<User>, Vec<Order>, Vec<Product>) {
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            city: "New York".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            city: "San Francisco".to_string(),
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            city: "New York".to_string(),
        },
        User {
            id: 4,
            name: "Diana".to_string(),
            email: "diana@example.com".to_string(),
            city: "Boston".to_string(),
        },
    ];

    let products = vec![
        Product {
            id: 101,
            name: "Laptop".to_string(),
            price: 999.99,
            category: "Electronics".to_string(),
        },
        Product {
            id: 102,
            name: "Mouse".to_string(),
            price: 29.99,
            category: "Electronics".to_string(),
        },
        Product {
            id: 103,
            name: "Desk Chair".to_string(),
            price: 199.99,
            category: "Furniture".to_string(),
        },
        Product {
            id: 104,
            name: "Monitor".to_string(),
            price: 299.99,
            category: "Electronics".to_string(),
        },
        Product {
            id: 105,
            name: "Keyboard".to_string(),
            price: 79.99,
            category: "Electronics".to_string(),
        },
    ];

    let orders = vec![
        Order {
            id: 1001,
            user_id: 1,
            product_id: 101,
            quantity: 1,
            total: 999.99,
        },
        Order {
            id: 1002,
            user_id: 1,
            product_id: 102,
            quantity: 2,
            total: 59.98,
        },
        Order {
            id: 1003,
            user_id: 2,
            product_id: 103,
            quantity: 1,
            total: 199.99,
        },
        Order {
            id: 1004,
            user_id: 2,
            product_id: 104,
            quantity: 1,
            total: 299.99,
        },
        Order {
            id: 1005,
            user_id: 3,
            product_id: 102,
            quantity: 3,
            total: 89.97,
        },
        Order {
            id: 1006,
            user_id: 1,
            product_id: 105,
            quantity: 1,
            total: 79.99,
        },
        Order {
            id: 1007,
            user_id: 3,
            product_id: 101,
            quantity: 1,
            total: 999.99,
        },
    ];

    (users, orders, products)
}

fn main() {
    println!("=== Join Query Builder Demo ===\n");

    let (users, orders, products) = create_sample_data();

    println!("Database:");
    println!("  Users: {}", users.len());
    println!("  Orders: {}", orders.len());
    println!("  Products: {}\n", products.len());

    // Join 1: Inner join Users and Orders
    println!("--- Join 1: Users with Their Orders ---");
    let user_orders = JoinQuery::new(&users, &orders).inner_join(
        User::id_r(),
        Order::user_id_r(),
        |user, order| UserOrder {
            user_name: user.name.clone(),
            user_email: user.email.clone(),
            order_id: order.id,
            quantity: order.quantity,
            total: order.total,
        },
    );

    for uo in &user_orders {
        println!(
            "  • {} - Order #{} - {} items - ${:.2}",
            uo.user_name, uo.order_id, uo.quantity, uo.total
        );
    }
    println!("Total: {} user-order pairs", user_orders.len());

    // Join 2: Three-way join (Orders -> Users, Orders -> Products)
    println!("\n--- Join 2: Complete Order Details (3-Way Join) ---");
    
    // First join: Orders with Users
    let orders_with_users = JoinQuery::new(&orders, &users).inner_join(
        Order::user_id_r(),
        User::id_r(),
        |order, user| (order.clone(), user.clone()),
    );

    // Second join: (Orders+Users) with Products
    let mut order_details = Vec::new();
    for (order, user) in &orders_with_users {
        for product in &products {
            if order.product_id == product.id {
                order_details.push(OrderDetail {
                    order_id: order.id,
                    user_name: user.name.clone(),
                    product_name: product.name.clone(),
                    quantity: order.quantity,
                    price: product.price,
                    total: order.total,
                });
            }
        }
    }

    for od in &order_details {
        println!(
            "  • Order #{}: {} bought {} x {} @ ${:.2} = ${:.2}",
            od.order_id, od.user_name, od.quantity, od.product_name, od.price, od.total
        );
    }

    // Join 3: Left join to show all users (including those without orders)
    println!("\n--- Join 3: All Users with Order Count (Left Join) ---");
    
    // Build a map of user orders
    let mut user_order_map: HashMap<u32, Vec<&Order>> = HashMap::new();
    for order in &orders {
        user_order_map
            .entry(order.user_id)
            .or_insert_with(Vec::new)
            .push(order);
    }

    let user_order_stats: Vec<UserOrderCount> = users
        .iter()
        .map(|user| {
            let user_orders = user_order_map.get(&user.id);
            let order_count = user_orders.map(|v| v.len()).unwrap_or(0);
            let total_spent = user_orders
                .map(|orders| orders.iter().map(|o| o.total).sum())
                .unwrap_or(0.0);

            UserOrderCount {
                user_name: user.name.clone(),
                user_city: user.city.clone(),
                order_count,
                total_spent,
            }
        })
        .collect();

    for stat in &user_order_stats {
        if stat.order_count > 0 {
            println!(
                "  • {} ({}) - {} orders - ${:.2} total",
                stat.user_name, stat.user_city, stat.order_count, stat.total_spent
            );
        } else {
            println!("  • {} ({}) - No orders yet", stat.user_name, stat.user_city);
        }
    }

    // Join 4: Aggregated join - Category sales analysis
    println!("\n--- Join 4: Sales by Product Category ---");

    let mut category_stats: HashMap<String, (Vec<u32>, f64, std::collections::HashSet<u32>)> =
        HashMap::new();

    for order in &orders {
        if let Some(product) = products.iter().find(|p| p.id == order.product_id) {
            let entry = category_stats
                .entry(product.category.clone())
                .or_insert_with(|| (Vec::new(), 0.0, std::collections::HashSet::new()));
            entry.0.push(order.id);
            entry.1 += order.total;
            entry.2.insert(order.user_id);
        }
    }

    let mut category_sales: Vec<CategorySales> = category_stats
        .into_iter()
        .map(|(category, (orders, revenue, customers))| CategorySales {
            category,
            total_orders: orders.len(),
            total_revenue: revenue,
            unique_customers: customers.len(),
        })
        .collect();

    category_sales.sort_by(|a, b| {
        b.total_revenue
            .partial_cmp(&a.total_revenue)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for cs in &category_sales {
        println!(
            "  • {}: {} orders - ${:.2} revenue - {} customers",
            cs.category, cs.total_orders, cs.total_revenue, cs.unique_customers
        );
    }

    // Join 5: Filtered join - High value orders
    println!("\n--- Join 5: High Value Orders (>$100) with User Details ---");
    let high_value_orders = JoinQuery::new(&orders, &users).inner_join_where(
        Order::user_id_r(),
        User::id_r(),
        |order, _user| order.total > 100.0,
        |order, user| (user.name.clone(), order.id, order.total),
    );

    for (name, order_id, total) in &high_value_orders {
        println!("  • {} - Order #{} - ${:.2}", name, order_id, total);
    }

    // Join 6: Users in same city analysis
    println!("\n--- Join 6: Users from Same City ---");
    let user_pairs = JoinQuery::new(&users, &users).inner_join_where(
        User::city_r(),
        User::city_r(),
        |u1, u2| u1.id < u2.id, // Avoid duplicates and self-pairs
        |u1, u2| (u1.name.clone(), u2.name.clone(), u1.city.clone()),
    );

    for (name1, name2, city) in &user_pairs {
        println!("  • {} and {} both live in {}", name1, name2, city);
    }

    // Join 7: Product popularity
    println!("\n--- Join 7: Product Popularity Ranking ---");
    let mut product_sales: HashMap<u32, (String, usize, u32, f64)> = HashMap::new();

    for order in &orders {
        if let Some(product) = products.iter().find(|p| p.id == order.product_id) {
            let entry = product_sales
                .entry(product.id)
                .or_insert_with(|| (product.name.clone(), 0, 0, 0.0));
            entry.1 += 1; // order count
            entry.2 += order.quantity; // total quantity
            entry.3 += order.total; // total revenue
        }
    }

    let mut popularity: Vec<_> = product_sales.into_iter().collect();
    popularity.sort_by(|a, b| b.1 .1.cmp(&a.1 .1)); // sort by order count

    for (_, (name, order_count, total_qty, revenue)) in &popularity {
        println!(
            "  • {} - {} orders - {} units - ${:.2}",
            name, order_count, total_qty, revenue
        );
    }

    // Join 8: User spending by city
    println!("\n--- Join 8: Total Spending by City ---");
    let mut city_spending: HashMap<String, (f64, usize)> = HashMap::new();

    for user in &users {
        if let Some(user_orders) = user_order_map.get(&user.id) {
            let total_spent: f64 = user_orders.iter().map(|o| o.total).sum();
            let entry = city_spending
                .entry(user.city.clone())
                .or_insert_with(|| (0.0, 0));
            entry.0 += total_spent;
            entry.1 += 1;
        }
    }

    let mut city_stats: Vec<_> = city_spending.into_iter().collect();
    city_stats.sort_by(|a, b| {
        b.1 .0
            .partial_cmp(&a.1 .0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (city, (total, customer_count)) in &city_stats {
        println!(
            "  • {} - ${:.2} total - {} customers - ${:.2} avg",
            city,
            total,
            customer_count,
            total / *customer_count as f64
        );
    }

    // Statistics summary
    println!("\n=== Summary Statistics ===");
    println!("Total orders: {}", orders.len());
    println!("Total revenue: ${:.2}", orders.iter().map(|o| o.total).sum::<f64>());
    println!(
        "Average order value: ${:.2}",
        orders.iter().map(|o| o.total).sum::<f64>() / orders.len() as f64
    );
    println!("Active customers: {}", user_order_map.len());
    println!(
        "Average orders per customer: {:.1}",
        orders.len() as f64 / user_order_map.len() as f64
    );

    println!("\n✓ Join query builder demo complete!");
}

