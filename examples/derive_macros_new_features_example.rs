use key_paths_derive::{Keypaths, PartialKeypaths, AnyKeypaths};
use key_paths_core::{KeyPaths, PartialKeyPath, AnyKeyPath};
use std::any::Any;

/// Example demonstrating the new derive macros for PartialKeyPath and AnyKeyPath
/// This example shows how to use the new #[derive(PartialKeypaths)] and #[derive(AnyKeypaths)] macros

#[derive(Debug, Clone, Keypaths, PartialKeypaths, AnyKeypaths)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
    is_active: bool,
    tags: Vec<String>,
    metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Keypaths, PartialKeypaths, AnyKeypaths)]
struct Product {
    id: u64,
    title: String,
    price: f64,
    in_stock: bool,
    categories: Vec<String>,
}

fn main() {
    println!("=== Derive Macros for New KeyPath Features ===\n");

    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: Some("alice@example.com".to_string()),
        is_active: true,
        tags: vec!["premium".to_string(), "verified".to_string()],
        metadata: std::collections::HashMap::from([
            ("department".to_string(), "engineering".to_string()),
            ("level".to_string(), "senior".to_string()),
        ]),
    };

    let product = Product {
        id: 101,
        title: "Rust Programming Book".to_string(),
        price: 49.99,
        in_stock: true,
        categories: vec!["programming".to_string(), "books".to_string()],
    };

    // Example 1: Using regular KeyPaths (existing functionality)
    println!("--- 1. Regular KeyPaths (existing functionality) ---");
    
    let name_path = User::name_r();
    let email_path = User::email_fr();
    let tags_path = User::tags_r();

    if let Some(name) = name_path.get(&user) {
        println!("User name: {}", name);
    }

    if let Some(email) = email_path.get(&user) {
        println!("User email: {:?}", email);
    }

    if let Some(tags) = tags_path.get(&user) {
        println!("User tags: {:?}", tags);
    }

    // Example 2: Using PartialKeyPath derive macros
    println!("\n--- 2. PartialKeyPath derive macros ---");
    
    // Generated methods: field_partial_r(), field_partial_w(), field_partial_fr(), etc.
    let name_partial = User::name_partial_r();
    let email_partial = User::email_partial_fr();
    let tags_partial = User::tags_partial_r();
    let metadata_partial = User::metadata_partial_r();

    // Store different keypaths in the same collection (type-erased Value)
    let partial_keypaths: Vec<PartialKeyPath<User>> = vec![
        name_partial,
        email_partial,
        tags_partial,
        metadata_partial,
    ];

    // Use partial keypaths with type erasure
    for (i, keypath) in partial_keypaths.iter().enumerate() {
        if let Some(value) = keypath.get(&user) {
            println!("Partial keypath {}: {:?} (type: {})", i, value, keypath.kind_name());
        }
    }

    // Example 3: Using AnyKeyPath derive macros
    println!("\n--- 3. AnyKeyPath derive macros ---");
    
    // Generated methods: field_any_r(), field_any_w(), field_any_fr(), etc.
    let user_name_any = User::name_any_r();
    let user_email_any = User::email_any_fr();
    let product_title_any = Product::title_any_r();
    let product_price_any = Product::price_any_r();

    // Store different keypaths from different types in the same collection (fully type-erased)
    let any_keypaths: Vec<AnyKeyPath> = vec![
        user_name_any,
        user_email_any,
        product_title_any,
        product_price_any,
    ];

    // Use any keypaths with full type erasure
    for (i, keypath) in any_keypaths.iter().enumerate() {
        // We need to box the data to use with AnyKeyPath
        let user_boxed: Box<dyn Any + Send + Sync> = Box::new(user.clone());
        let product_boxed: Box<dyn Any + Send + Sync> = Box::new(product.clone());
        
        // Try with user first (for user keypaths)
        if i < 2 {
            if let Some(value) = keypath.get(&*user_boxed) {
                println!("Any keypath {} (user): {:?} (type: {})", i, value, keypath.kind_name());
            }
        } else {
            // Try with product (for product keypaths)
            if let Some(value) = keypath.get(&*product_boxed) {
                println!("Any keypath {} (product): {:?} (type: {})", i, value, keypath.kind_name());
            }
        }
    }

    // Example 4: Collection access with derive macros
    println!("\n--- 4. Collection access with derive macros ---");
    
    // Vec access with partial keypaths
    let first_tag_partial = User::tags_partial_fr_at(0);
    if let Some(tag) = first_tag_partial.get(&user) {
        println!("First tag (partial): {:?}", tag);
    }

    // HashMap access with partial keypaths
    let department_partial = User::metadata_partial_fr("department".to_string());
    if let Some(dept) = department_partial.get(&user) {
        println!("Department (partial): {:?}", dept);
    }

    // Vec access with any keypaths
    let first_tag_any = User::tags_any_fr_at(0);
    let user_boxed: Box<dyn Any + Send + Sync> = Box::new(user.clone());
    if let Some(tag) = first_tag_any.get(&*user_boxed) {
        println!("First tag (any): {:?}", tag);
    }

    // Example 5: Writable keypaths with derive macros
    println!("\n--- 5. Writable keypaths with derive macros ---");
    
    let mut user_mut = user.clone();
    
    // Using regular writable keypaths (not type-erased)
    let name_w = User::name_w();
    if let Some(name_ref) = name_w.get_mut(&mut user_mut) {
        *name_ref = "Alice Updated".to_string();
        println!("Updated name (regular): {}", name_ref);
    }

    // Note: Type-erased keypaths (PartialKeyPath, AnyKeyPath) return &dyn Any
    // which cannot be directly assigned to. They are primarily for read-only access
    // and dynamic keypath selection. For mutation, use regular KeyPaths.
    
    // Demonstrate that partial keypaths work for reading
    let name_partial_r = User::name_partial_r();
    if let Some(name_ref) = name_partial_r.get(&user_mut) {
        println!("Name via partial keypath: {:?}", name_ref);
    }

    // Example 6: Owned keypaths with derive macros
    println!("\n--- 6. Owned keypaths with derive macros ---");
    
    // Using partial owned keypaths
    let name_partial_o = User::name_partial_o();
    let owned_name = name_partial_o.get_owned(user.clone());
    println!("Owned name (partial): {:?}", owned_name);

    // Using any owned keypaths
    let name_any_o = User::name_any_o();
    let user_boxed: Box<dyn Any + Send + Sync> = Box::new(user.clone());
    let owned_name_any = name_any_o.get_owned(user_boxed);
    println!("Owned name (any): {:?}", owned_name_any);

    // Example 7: Mixed keypath types in collections
    println!("\n--- 7. Mixed keypath types in collections ---");
    
    // Create a collection of different keypath types
    let mixed_keypaths: Vec<Box<dyn Any>> = vec![
        Box::new(User::name_partial_r()),
        Box::new(User::email_partial_fr()),
        Box::new(User::name_any_r()),  // Use User keypath instead of Product
        Box::new(Product::title_any_r()),
    ];

    // Process mixed keypaths
    for (i, keypath_box) in mixed_keypaths.iter().enumerate() {
        if let Some(partial_keypath) = keypath_box.downcast_ref::<PartialKeyPath<User>>() {
            if let Some(value) = partial_keypath.get(&user) {
                println!("Mixed keypath {} (partial): {:?}", i, value);
            }
        } else if let Some(any_keypath) = keypath_box.downcast_ref::<AnyKeyPath>() {
            // Use the correct data type for each keypath
            if i == 2 { // User::name_any_r()
                let user_boxed: Box<dyn Any + Send + Sync> = Box::new(user.clone());
                if let Some(value) = any_keypath.get(&*user_boxed) {
                    println!("Mixed keypath {} (any, user): {:?}", i, value);
                }
            } else if i == 3 { // Product::title_any_r()
                let product_boxed: Box<dyn Any + Send + Sync> = Box::new(product.clone());
                if let Some(value) = any_keypath.get(&*product_boxed) {
                    println!("Mixed keypath {} (any, product): {:?}", i, value);
                }
            }
        }
    }

    // Example 8: Dynamic keypath selection with derive macros
    println!("\n--- 8. Dynamic keypath selection with derive macros ---");
    
    let partial_keypath_map: std::collections::HashMap<String, PartialKeyPath<User>> = [
        ("name".to_string(), User::name_partial_r()),
        ("email".to_string(), User::email_partial_fr()),
        ("tags".to_string(), User::tags_partial_r()),
        ("metadata".to_string(), User::metadata_partial_r()),
    ].iter().cloned().collect();

    // Dynamically select and use partial keypaths
    for field_name in ["name", "email", "tags", "metadata"] {
        if let Some(keypath) = partial_keypath_map.get(field_name) {
            if let Some(value) = keypath.get(&user) {
                println!("Dynamic access to {} (partial): {:?}", field_name, value);
            }
        }
    }

    println!("\n✅ Derive Macros for New KeyPath Features Example completed!");
    println!("📝 This example demonstrates:");
    println!("   • #[derive(PartialKeypaths)] - Generates field_partial_*() methods");
    println!("   • #[derive(AnyKeypaths)] - Generates field_any_*() methods");
    println!("   • Type-erased keypaths for collections of same Root type (PartialKeyPath)");
    println!("   • Fully type-erased keypaths for collections of different Root types (AnyKeyPath)");
    println!("   • Collection access with indexed methods (field_partial_fr_at, field_any_fr_at)");
    println!("   • Writable and owned keypath variants");
    println!("   • Dynamic keypath selection and usage");
    println!("   • Mixed keypath types in collections");
    println!("   • Full integration with existing KeyPaths ecosystem!");
}
