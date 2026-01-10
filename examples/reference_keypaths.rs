// // Demonstrates using keypaths with collections of references
// // This example shows how to:
// // 1. Use keypaths with Vec<&T> instead of Vec<T>
// // 2. Work with HashMap values that are references
// // 3. Query collections without cloning data
// // 4. Use get_ref() for reference types
// // cargo run --example reference_keypaths
// 
// use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
// use keypaths_proc::Keypaths;
// use std::collections::HashMap;
// 
// #[derive(Debug, Clone, Keypaths)]
// struct Product {
//     id: u32,
//     name: String,
//     price: f64,
//     category: String,
//     in_stock: bool,
// }
// 
// #[derive(Debug, Clone, Keypaths)]
// struct User {
//     id: u32,
//     name: String,
//     email: String,
//     is_active: bool,
// }
// 
// fn main() {
//     println!("=== Reference KeyPaths Demo ===\n");
// 
//     // Create some owned data
//     let products = vec![
//         Product {
//             id: 1,
//             name: "Laptop".to_string(),
//             price: 999.99,
//             category: "Electronics".to_string(),
//             in_stock: true,
//         },
//         Product {
//             id: 2,
//             name: "Mouse".to_string(),
//             price: 29.99,
//             category: "Electronics".to_string(),
//             in_stock: true,
//         },
//         Product {
//             id: 3,
//             name: "Keyboard".to_string(),
//             price: 79.99,
//             category: "Electronics".to_string(),
//             in_stock: false,
//         },
//         Product {
//             id: 4,
//             name: "Monitor".to_string(),
//             price: 349.99,
//             category: "Electronics".to_string(),
//             in_stock: true,
//         },
//     ];
// 
//     // Example 1: Working with Vec<&Product>
//     println!("--- Example 1: Vec<&Product> ---");
//     let product_refs: Vec<&Product> = products.iter().collect();
//     println!("Created {} product references", product_refs.len());
// 
//     // Use get_ref() to access fields from references
//     let name_path = Product::name_r();
//     for product_ref in &product_refs {
//         if let Some(name) = name_path.to_optional().get(product_ref) {
//             println!("  Product: {}", name);
//         }
//     }
// 
//     // Example 2: Filtering references using keypaths
//     println!("\n--- Example 2: Filtering References ---");
//     let price_path = Product::price_r();
//     let in_stock_path = Product::in_stock_r();
// 
//     let affordable: Vec<&&Product> = product_refs
//         .iter()
//         .filter(|&product_ref| {
//             price_path.get(product_ref).map_or(false, |&p| p < 100.0)
//                 && in_stock_path.get(product_ref).map_or(false, |&s| s)
//         })
//         .collect();
// 
//     println!("Found {} affordable products in stock:", affordable.len());
//     for product_ref in affordable {
//         let name = name_path.get(product_ref).unwrap();
//         let price = price_path.get(product_ref).unwrap();
//         println!("  • {} - ${:.2}", name, price);
//     }
// 
//     // Example 3: HashMap with references
//     println!("\n--- Example 3: HashMap with Reference Values ---");
//     let mut product_map: HashMap<u32, &Product> = HashMap::new();
//     for product in &products {
//         product_map.insert(product.id, product);
//     }
// 
//     println!("Product map has {} entries", product_map.len());
// 
//     // Access fields through references in HashMap
//     if let Some(product_ref) = product_map.get(&1) {
//         if let Some(name) = name_path.get(product_ref) {
//             println!("  Product ID 1: {}", name);
//         }
//     }
// 
//     // Example 4: Grouping references by category
//     println!("\n--- Example 4: Grouping References by Category ---");
//     let category_path = Product::category_r();
// 
//     let mut by_category: HashMap<String, Vec<&Product>> = HashMap::new();
//     for product_ref in &product_refs {
//         if let Some(category) = category_path.get(product_ref) {
//             by_category
//                 .entry(category.clone())
//                 .or_insert_with(Vec::new)
//                 .push(*product_ref);
//         }
//     }
// 
//     for (category, prods) in &by_category {
//         println!("  {}: {} products", category, prods.len());
//     }
// 
//     // Example 5: Using with Arc<RwLock<HashMap>> pattern
//     println!("\n--- Example 5: Simulated Lock-Aware Pattern ---");
//     
//     // Simulate reading from a shared HashMap
//     let shared_products: HashMap<u32, Product> = products
//         .iter()
//         .map(|p| (p.id, p.clone()))
//         .collect();
// 
//     // Collect references from the HashMap
//     let values_refs: Vec<&Product> = shared_products.values().collect();
//     println!("Collected {} references from shared HashMap", values_refs.len());
// 
//     // Query the references without cloning
//     let expensive: Vec<&&Product> = values_refs
//         .iter()
//         .filter(|&prod_ref| {
//             price_path.get(prod_ref).map_or(false, |&p| p > 200.0)
//         })
//         .collect();
// 
//     println!("Found {} expensive products:", expensive.len());
//     for prod_ref in expensive {
//         let name = name_path.get(prod_ref).unwrap();
//         let price = price_path.get(prod_ref).unwrap();
//         println!("  • {} - ${:.2}", name, price);
//     }
// 
//     // Example 6: Nested references (Vec<&Vec<&Product>>)
//     println!("\n--- Example 6: Nested References ---");
//     
//     let batches = vec![
//         products[0..2].iter().collect::<Vec<&Product>>(),
//         products[2..4].iter().collect::<Vec<&Product>>(),
//     ];
// 
//     for (i, batch) in batches.iter().enumerate() {
//         println!("  Batch {}: {} products", i + 1, batch.len());
//         for product_ref in batch {
//             if let Some(name) = name_path.get(product_ref) {
//                 println!("    - {}", name);
//             }
//         }
//     }
// 
//     // Example 7: Comparison with owned data (showing the difference)
//     println!("\n--- Example 7: Owned vs Reference Comparison ---");
//     
//     // Owned: uses .get()
//     println!("With owned data (using .get()):");
//     for product in &products {
//         if let Some(name) = name_path.get(product) {
//             println!("  • {}", name);
//         }
//     }
// 
//     // References: uses .get()
//     println!("\nWith reference data (using .get()):");
//     for product_ref in &product_refs {
//         if let Some(name) = name_path.get(product_ref) {
//             println!("  • {}", name);
//         }
//     }
// 
//     // Example 8: Working with Users and multiple field accesses
//     println!("\n--- Example 8: Multiple Field Access on References ---");
//     
//     let users = vec![
//         User {
//             id: 1,
//             name: "Alice".to_string(),
//             email: "akash@example.com".to_string(),
//             is_active: true,
//         },
//         User {
//             id: 2,
//             name: "Bob".to_string(),
//             email: "bob@example.com".to_string(),
//             is_active: false,
//         },
//         User {
//             id: 3,
//             name: "Charlie".to_string(),
//             email: "charlie@example.com".to_string(),
//             is_active: true,
//         },
//     ];
// 
//     let user_refs: Vec<&User> = users.iter().collect();
// 
//     let user_name_path = User::name_r();
//     let user_email_path = User::email_r();
//     let user_active_path = User::is_active_r();
// 
//     println!("Active users:");
//     for user_ref in &user_refs {
//         if let Some(&is_active) = user_active_path.get(user_ref) {
//             if is_active {
//                 let name = user_name_path.get(user_ref).unwrap();
//                 let email = user_email_path.get(user_ref).unwrap();
//                 println!("  • {} <{}>", name, email);
//             }
//         }
//     }
// 
//     // Example 9: Practical use case - Avoid cloning in queries
//     println!("\n--- Example 9: Performance Benefit (No Cloning) ---");
//     
//     // Without get_ref, you might be tempted to clone
//     let _cloned_products: Vec<Product> = products
//         .iter()
//         .filter(|p| p.price < 100.0)
//         .cloned()
//         .collect();
//     println!("❌ Cloned approach: Creates new copies");
// 
//     // With get_ref, work with references directly
//     let _filtered_refs: Vec<&Product> = products
//         .iter()
//         .filter(|p| p.price < 100.0)
//         .collect();
//     println!("✓ Reference approach: No cloning needed");
// 
//     println!("\n✓ Reference keypaths demo complete!");
// }
// 

fn main() {
    
}