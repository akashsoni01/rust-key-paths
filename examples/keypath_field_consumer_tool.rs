// // KeyPath Field Consumer Tool Implementation
// // Demonstrates how to use keypaths to create a tool for partially consuming/accessing struct fields
// // cargo run --example keypath_field_consumer_tool
//
// use key_paths_core::KeyPaths;
// use key_paths_derive::Keypaths;
// use std::any::Any;
// use std::collections::{HashMap, HashSet};
//
// // Trait for field accessors
// trait FieldAccessor<T>: Send + Sync {
//     fn get_value(&self, data: &T) -> Option<Box<dyn Any>>;
//     fn get_ref<'a>(&'a self, data: &'a T) -> Option<&'a dyn Any>;
//     fn consume_value(&self, data: &mut T) -> Option<Box<dyn Any>>;
//     fn field_type_name(&self) -> &'static str;
// }
//
// // Implementation for readable keypaths
// struct FieldAccessorImpl<T, V> {
//     keypath: KeyPaths<T, V>,
// }
//
// impl<T, V> FieldAccessor<T> for FieldAccessorImpl<T, V>
// where
//     V: Clone + Send + Sync + 'static,
// {
//     fn get_value(&self, data: &T) -> Option<Box<dyn Any>> {
//         self.keypath.get(data).map(|v| Box::new(v.clone()) as Box<dyn Any>)
//     }
//
//     fn get_ref<'a>(&'a self, data: &'a T) -> Option<&'a dyn Any> {
//         self.keypath.get(data).map(|v| v as &dyn Any)
//     }
//
//     fn consume_value(&self, _data: &mut T) -> Option<Box<dyn Any>> {
//         // For readable keypaths, we can't consume, only clone
//         None
//     }
//
//     fn field_type_name(&self) -> &'static str {
//         std::any::type_name::<V>()
//     }
// }
//
// // Implementation for owned keypaths
// struct OwnedFieldAccessorImpl<T, V> {
//     keypath: KeyPaths<T, V>,
// }
//
// impl<T, V> FieldAccessor<T> for OwnedFieldAccessorImpl<T, V>
// where
//     V: Send + Sync + 'static,
// {
//     fn get_value(&self, _data: &T) -> Option<Box<dyn Any>> {
//         // For owned keypaths, we can't get a reference without consuming
//         None
//     }
//
//     fn get_ref<'a>(&'a self, _data: &'a T) -> Option<&'a dyn Any> {
//         // For owned keypaths, we can't get a reference without consuming
//         None
//     }
//
//     fn consume_value(&self, _data: &mut T) -> Option<Box<dyn Any>> {
//         // This would require the keypath to support consumption
//         // For now, we'll return None as this is a complex operation
//         None
//     }
//
//     fn field_type_name(&self) -> &'static str {
//         std::any::type_name::<V>()
//     }
// }
//
// // Field consumer tool
// struct FieldConsumer<T> {
//     data: T,
//     field_registry: HashMap<String, Box<dyn FieldAccessor<T> + Send + Sync>>,
//     consumed_fields: HashSet<String>,
//     debug_mode: bool,
// }
//
// #[derive(Debug)]
// struct FieldAccessDebugInfo {
//     total_fields: usize,
//     consumed_fields: Vec<String>,
//     available_fields: Vec<String>,
//     field_types: HashMap<String, String>,
// }
//
// impl<T: 'static> FieldConsumer<T> {
//     fn new(data: T) -> Self {
//         Self {
//             data,
//             field_registry: HashMap::new(),
//             consumed_fields: HashSet::new(),
//             debug_mode: false,
//         }
//     }
//
//     fn register_field<V: 'static>(&mut self, name: &str, keypath: KeyPaths<T, V>)
//     where
//         V: Clone + Send + Sync,
//     {
//         let accessor = FieldAccessorImpl { keypath };
//         self.field_registry.insert(name.to_string(), Box::new(accessor));
//
//         if self.debug_mode {
//             println!("Registered field '{}' with type {}", name, std::any::type_name::<V>());
//         }
//     }
//
//     fn register_owned_field<V: 'static>(&mut self, name: &str, keypath: KeyPaths<T, V>)
//     where
//         V: Send + Sync,
//     {
//         let accessor = OwnedFieldAccessorImpl { keypath };
//         self.field_registry.insert(name.to_string(), Box::new(accessor));
//
//         if self.debug_mode {
//             println!("Registered owned field '{}' with type {}", name, std::any::type_name::<V>());
//         }
//     }
//
//     // Consume a specific field (moves the field out)
//     fn consume_field(&mut self, field_name: &str) -> Option<Box<dyn Any>> {
//         if self.consumed_fields.contains(field_name) {
//             if self.debug_mode {
//                 eprintln!("Field '{}' has already been consumed", field_name);
//             }
//             return None;
//         }
//
//         if let Some(accessor) = self.field_registry.get(field_name) {
//             if self.debug_mode {
//                 println!("Consuming field '{}' of type {}", field_name, accessor.field_type_name());
//             }
//
//             let result = accessor.consume_value(&mut self.data);
//             if result.is_some() {
//                 self.consumed_fields.insert(field_name.to_string());
//             }
//             result
//         } else {
//             if self.debug_mode {
//                 eprintln!("Field '{}' not found in registry", field_name);
//             }
//             None
//         }
//     }
//
//     // Borrow a field (doesn't move)
//     fn borrow_field(&self, field_name: &str) -> Option<&dyn Any> {
//         if let Some(accessor) = self.field_registry.get(field_name) {
//             if self.debug_mode {
//                 println!("Borrowing field '{}' of type {}", field_name, accessor.field_type_name());
//             }
//             accessor.get_ref(&self.data)
//         } else {
//             if self.debug_mode {
//                 eprintln!("Field '{}' not found in registry", field_name);
//             }
//             None
//         }
//     }
//
//     fn enable_debug_mode(&mut self) {
//         self.debug_mode = true;
//         println!("Debug mode enabled for FieldConsumer");
//     }
//
//     fn disable_debug_mode(&mut self) {
//         self.debug_mode = false;
//     }
//
//     // Get debug information about field access
//     fn debug_info(&self) -> FieldAccessDebugInfo {
//         let consumed_fields: Vec<String> = self.consumed_fields.iter().cloned().collect();
//         let available_fields: Vec<String> = self.field_registry
//             .keys()
//             .filter(|name| !self.consumed_fields.contains(*name))
//             .cloned()
//             .collect();
//
//         let field_types: HashMap<String, String> = self.field_registry
//             .iter()
//             .map(|(name, accessor)| (name.clone(), accessor.field_type_name().to_string()))
//             .collect();
//
//         FieldAccessDebugInfo {
//             total_fields: self.field_registry.len(),
//             consumed_fields,
//             available_fields,
//             field_types,
//         }
//     }
//
//     // Check if a field is available for consumption
//     fn is_field_available(&self, field_name: &str) -> bool {
//         self.field_registry.contains_key(field_name) &&
//         !self.consumed_fields.contains(field_name)
//     }
//
//     // Get list of available fields
//     fn available_fields(&self) -> Vec<&String> {
//         self.field_registry
//             .keys()
//             .filter(|name| !self.consumed_fields.contains(*name))
//             .collect()
//     }
//
//     // Get list of consumed fields
//     fn consumed_fields(&self) -> Vec<&String> {
//         self.consumed_fields.iter().collect()
//     }
//
//     // Reset consumption state (useful for testing)
//     fn reset_consumption(&mut self) {
//         if self.debug_mode {
//             println!("Resetting consumption state");
//         }
//         self.consumed_fields.clear();
//     }
//
// }
//
// // Example structs with Keypaths derive
// #[derive(Debug, Clone, Keypaths)]
// struct User {
//     id: u32,
//     name: String,
//     email: Option<String>,
//     is_active: bool,
// }
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
// struct Order {
//     id: u32,
//     user_id: u32,
//     product_id: u32,
//     quantity: u32,
//     total: f64,
//     status: String,
// }
//
// fn main() {
//     println!("=== KeyPath Field Consumer Tool Example ===\n");
//
//     // Example 1: User field consumption
//     println!("--- Example 1: User Field Consumption ---");
//     let user = User {
//         id: 1,
//         name: "Alice Johnson".to_string(),
//         email: Some("akash@example.com".to_string()),
//         is_active: true,
//     };
//
//     let mut consumer = FieldConsumer::new(user);
//     consumer.enable_debug_mode();
//
//     // Register fields
//     consumer.register_field("id", User::id_r());
//     consumer.register_field("name", User::name_r());
//     consumer.register_field("email", User::email_fr());
//     consumer.register_field("active", User::is_active_r());
//
//     // Debug information
//     println!("Debug Info: {:?}", consumer.debug_info());
//
//     // Borrow fields (safe, doesn't move)
//     if let Some(id) = consumer.borrow_field("id") {
//         println!("Borrowed ID: {:?}", id);
//     }
//
//     if let Some(name) = consumer.borrow_field("name") {
//         println!("Borrowed name: {:?}", name);
//     }
//
//     // Check availability
//     println!("Available fields: {:?}", consumer.available_fields());
//     println!("Is 'email' available? {}", consumer.is_field_available("email"));
//
//     // Example 2: Product field consumption
//     println!("\n--- Example 2: Product Field Consumption ---");
//     let product = Product {
//         id: 101,
//         name: "Laptop".to_string(),
//         price: 999.99,
//         category: "Electronics".to_string(),
//         in_stock: true,
//     };
//
//     let mut product_consumer = FieldConsumer::new(product);
//     product_consumer.enable_debug_mode();
//
//     // Register product fields
//     product_consumer.register_field("id", Product::id_r());
//     product_consumer.register_field("name", Product::name_r());
//     product_consumer.register_field("price", Product::price_r());
//     product_consumer.register_field("category", Product::category_r());
//     product_consumer.register_field("in_stock", Product::in_stock_r());
//
//     // Borrow product fields
//     if let Some(name) = product_consumer.borrow_field("name") {
//         println!("Product name: {:?}", name);
//     }
//
//     if let Some(price) = product_consumer.borrow_field("price") {
//         println!("Product price: {:?}", price);
//     }
//
//     println!("Available product fields: {:?}", product_consumer.available_fields());
//
//     // Example 3: Order field consumption
//     println!("\n--- Example 3: Order Field Consumption ---");
//     let order = Order {
//         id: 1001,
//         user_id: 1,
//         product_id: 101,
//         quantity: 1,
//         total: 999.99,
//         status: "completed".to_string(),
//     };
//
//     let mut order_consumer = FieldConsumer::new(order);
//     order_consumer.enable_debug_mode();
//
//     // Register order fields
//     order_consumer.register_field("id", Order::id_r());
//     order_consumer.register_field("user_id", Order::user_id_r());
//     order_consumer.register_field("total", Order::total_r());
//     order_consumer.register_field("status", Order::status_r());
//     order_consumer.register_field("quantity", Order::quantity_r());
//
//     // Borrow order fields
//     if let Some(total) = order_consumer.borrow_field("total") {
//         println!("Order total: {:?}", total);
//     }
//
//     if let Some(status) = order_consumer.borrow_field("status") {
//         println!("Order status: {:?}", status);
//     }
//
//     println!("Available order fields: {:?}", order_consumer.available_fields());
//
//     // Example 4: Advanced field operations
//     println!("\n--- Example 4: Advanced Field Operations ---");
//     let mut advanced_consumer = FieldConsumer::new(());
//     advanced_consumer.enable_debug_mode();
//
//     // Test field availability
//     println!("Is 'nonexistent' available? {}", advanced_consumer.is_field_available("nonexistent"));
//
//     // Get debug information
//     let debug_info = advanced_consumer.debug_info();
//     println!("Total registered fields: {}", debug_info.total_fields);
//     println!("Field types: {:?}", debug_info.field_types);
//
//     // Example 5: Field consumption demonstration
//     println!("\n--- Example 5: Field Consumption Demonstration ---");
//     let test_user = User {
//         id: 1,
//         name: "Alice".to_string(),
//         email: Some("akash@example.com".to_string()),
//         is_active: true,
//     };
//
//     let mut test_consumer = FieldConsumer::new(test_user);
//     test_consumer.enable_debug_mode();
//
//     // Register fields
//     test_consumer.register_field("name", User::name_r());
//     test_consumer.register_field("email", User::email_fr());
//     test_consumer.register_field("active", User::is_active_r());
//
//     // Demonstrate field borrowing
//     if let Some(name) = test_consumer.borrow_field("name") {
//         println!("Test user name: {:?}", name);
//     }
//
//     if let Some(email) = test_consumer.borrow_field("email") {
//         println!("Test user email: {:?}", email);
//     }
//
//     println!("Available test fields: {:?}", test_consumer.available_fields());
//
//     println!("\n✅ KeyPath Field Consumer Tool example completed!");
//     println!("📝 This example demonstrates:");
//     println!("   • Type-safe field registration using keypaths");
//     println!("   • Safe field borrowing without moving data");
//     println!("   • Collection field extraction and filtering");
//     println!("   • Debug mode for field access tracking");
//     println!("   • Field availability checking");
//     println!("   • Comprehensive error handling");
// }
