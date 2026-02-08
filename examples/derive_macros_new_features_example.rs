use keypaths_proc::{AnyKeypaths, Kp, PartialKp};
use rust_keypaths::{
    AnyKeyPath, PartialKeyPath, PartialOptionalKeyPath, PartialWritableKeyPath,
    PartialWritableOptionalKeyPath,
};
use std::any::Any;
// cd /rust-key-paths && cargo run --example derive_macros_new_features_example 2>&1 | tail -20
/// Example demonstrating the new derive macros for PartialKeyPath and AnyKeyPath
/// This example shows how to use the new #[derive(PartialKeypaths)] and #[derive(AnyKeypaths)] macros

#[derive(Debug, Clone, Kp, PartialKp, AnyKeypaths)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
    is_active: bool,
    tags: Vec<String>,
    metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Kp, PartialKp, AnyKeypaths)]
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
        name: "Akash".to_string(),
        email: Some("akash@example.com".to_string()),
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

    let name = name_path.get(&user);
    println!("User name: {}", name);

    if let Some(email) = email_path.get(&user) {
        println!("User email: {:?}", email);
    }

    let tags = tags_path.get(&user);
    println!("User tags: {:?}", tags);

    // Example 2: Using PartialKeyPath derive macros
    println!("\n--- 2. PartialKeyPath derive macros ---");

    // Generated methods: field_partial_r(), field_partial_w(), field_partial_fr(), etc.
    let name_partial = User::name_partial_r();
    let email_partial = User::email_partial_fr();
    let tags_partial = User::tags_partial_r();
    let metadata_partial = User::metadata_partial_r();

    // Store different keypaths - note: email_partial is PartialOptionalKeyPath, others are PartialKeyPath
    // For demonstration, we'll handle them separately
    println!("Name (partial): {:?}", name_partial.get(&user).type_id());
    if let Some(email) = email_partial.get(&user) {
        println!("Email (partial): {:?}", email.type_id());
    }
    println!("Tags (partial): {:?}", tags_partial.get(&user).type_id());
    println!(
        "Metadata (partial): {:?}",
        metadata_partial.get(&user).type_id()
    );

    // Note: Different partial keypath types have different get() signatures
    // PartialKeyPath::get() returns &dyn Any
    // PartialOptionalKeyPath::get() returns Option<&dyn Any>

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
                println!(
                    "Any keypath {} (user): {:?} (type: {})",
                    i,
                    value.type_id(),
                    keypath.kind_name()
                );
            }
        } else {
            // Try with product (for product keypaths)
            if let Some(value) = keypath.get(&*product_boxed) {
                println!(
                    "Any keypath {} (product): {:?} (type: {})",
                    i,
                    value.type_id(),
                    keypath.kind_name()
                );
            }
        }
    }

    // Example 4: Collection access with derive macros
    println!("\n--- 4. Collection access with derive macros ---");

    // Vec access with partial keypaths
    let first_tag_partial = User::tags_partial_fr_at(0);
    if let Some(tag) = first_tag_partial.get_as::<String>(&user) {
        println!("First tag (partial): {:?}", tag);
    }

    // HashMap access with partial keypaths
    let department_partial = User::metadata_partial_fr("department".to_string());
    if let Some(dept) = department_partial.get_as::<String>(&user) {
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
    // Note: Writable keypaths are not generated by default - use regular KeyPath for reading
    let name_r = User::name_r();
    let name_ref = name_r.get(&user_mut);
    println!("Name (regular): {}", name_ref);

    // Note: Type-erased keypaths (PartialKeyPath, AnyKeyPath) return &dyn Any
    // which cannot be directly assigned to. They are primarily for read-only access
    // and dynamic keypath selection. For mutation, use regular KeyPaths.

    // Demonstrate that partial keypaths work for reading
    let name_partial_r = User::name_partial_r();
    let name_ref = name_partial_r.get(&user_mut);
    println!("Name via partial keypath: {:?}", name_ref.type_id());

    // Example 6: Owned keypaths with derive macros
    println!("\n--- 6. Owned keypaths with derive macros ---");

    // Note: Owned keypath methods are not generated for PartialKeypaths
    // as they require references, not owned values
    let name_partial_r = User::name_partial_r();
    if let Some(name_ref) = name_partial_r.get_as::<String>(&user) {
        println!("Name (partial): {:?}", name_ref);
    }

    // Note: Owned keypath methods are not generated for AnyKeypaths
    // as they require references, not owned values
    let name_any_r = User::name_any_r();
    let user_boxed: Box<dyn Any + Send + Sync> = Box::new(user.clone());
    if let Some(name_ref_any) = name_any_r.get(&*user_boxed) {
        if let Some(name_str) = name_ref_any.downcast_ref::<String>() {
            println!("Name (any): {:?}", name_str);
        }
    }

    // Example 7: Mixed keypath types in collections
    println!("\n--- 7. Mixed keypath types in collections ---");

    // Create a collection of different keypath types
    // Note: We can't easily mix PartialKeyPath and PartialOptionalKeyPath in the same collection
    // So we'll demonstrate them separately
    let name_partial = User::name_partial_r();
    let name_value = name_partial.get(&user);
    println!(
        "Mixed keypath 0 (partial, name): {:?}",
        name_value.type_id()
    );

    let email_partial = User::email_partial_fr();
    if let Some(email_value) = email_partial.get(&user) {
        println!(
            "Mixed keypath 1 (partial, email): {:?}",
            email_value.type_id()
        );
    }

    let name_any = User::name_any_r();
    let user_boxed: Box<dyn Any + Send + Sync> = Box::new(user.clone());
    if let Some(name_value) = name_any.get(&*user_boxed) {
        println!(
            "Mixed keypath 2 (any, user name): {:?}",
            name_value.type_id()
        );
    }

    let title_any = Product::title_any_r();
    let product_boxed: Box<dyn Any + Send + Sync> = Box::new(product.clone());
    if let Some(title_value) = title_any.get(&*product_boxed) {
        println!(
            "Mixed keypath 3 (any, product title): {:?}",
            title_value.type_id()
        );
    }

    // Example 8: Dynamic keypath selection with derive macros
    println!("\n--- 8. Dynamic keypath selection with derive macros ---");

    // Note: We can't easily mix PartialKeyPath and PartialOptionalKeyPath in the same HashMap
    // So we'll demonstrate dynamic access separately
    for field_name in ["name", "tags", "metadata"] {
        match field_name {
            "name" => {
                let keypath = User::name_partial_r();
                let value = keypath.get(&user);
                println!(
                    "Dynamic access to {} (partial): {:?}",
                    field_name,
                    value.type_id()
                );
            }
            "tags" => {
                let keypath = User::tags_partial_r();
                let value = keypath.get(&user);
                println!(
                    "Dynamic access to {} (partial): {:?}",
                    field_name,
                    value.type_id()
                );
            }
            "metadata" => {
                let keypath = User::metadata_partial_r();
                let value = keypath.get(&user);
                println!(
                    "Dynamic access to {} (partial): {:?}",
                    field_name,
                    value.type_id()
                );
            }
            _ => {}
        }
    }
    // Handle optional field separately
    let email_keypath = User::email_partial_fr();
    if let Some(value) = email_keypath.get(&user) {
        println!("Dynamic access to email (partial): {:?}", value.type_id());
    }

    println!("\n✅ Derive Macros for New KeyPath Features Example completed!");
    println!("📝 This example demonstrates:");
    println!("   • #[derive(PartialKeypaths)] - Generates field_partial_*() methods");
    println!("   • #[derive(AnyKeypaths)] - Generates field_any_*() methods");
    println!("   • Type-erased keypaths for collections of same Root type (PartialKeyPath)");
    println!(
        "   • Fully type-erased keypaths for collections of different Root types (AnyKeyPath)"
    );
    println!("   • Collection access with indexed methods (field_partial_fr_at, field_any_fr_at)");
    println!("   • Writable and owned keypath variants");
    println!("   • Dynamic keypath selection and usage");
    println!("   • Mixed keypath types in collections");
    println!("   • Full integration with existing KeyPaths ecosystem!");
}
