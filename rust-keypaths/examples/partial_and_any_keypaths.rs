// This example demonstrates Partial and Any keypaths with complex nested structures
// Run with: cargo run --example partial_and_any_keypaths

use rust_keypaths::{
    AnyKeyPath, KeyPath, OptionalKeyPath, PartialKeyPath, PartialOptionalKeyPath, WritableKeyPath,
    WritableOptionalKeyPath, containers,
};
use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};
// cd /rust-key-paths/rust-keypaths && cargo check 2>&1 | tail -3
/// cd /rust-key-paths/rust-keypaths && cargo run --example partial_and_any_keypaths 2>&1 | tail -80
#[derive(Debug, Clone)]
struct Company {
    name: String,
    employees: Vec<Employee>,
    headquarters: Option<Location>,
    financials: Arc<RwLock<Financials>>,
}

#[derive(Debug, Clone)]
struct Employee {
    id: u64,
    name: String,
    department: Option<Department>,
    salary: f64,
    manager: Option<Arc<Employee>>,
}

#[derive(Debug, Clone)]
struct Department {
    name: String,
    budget: f64,
    location: Option<Location>,
}

#[derive(Debug, Clone)]
struct Location {
    city: String,
    country: String,
    coordinates: (f64, f64),
}

#[derive(Debug)]
struct Financials {
    revenue: f64,
    expenses: f64,
    profit: f64,
}

fn main() {
    println!("=== Partial and Any KeyPaths Complex Example ===\n");

    // Create a complex nested structure
    let mut company = Company {
        name: "TechCorp".to_string(),
        employees: vec![
            Employee {
                id: 1,
                name: "Akash".to_string(),
                department: Some(Department {
                    name: "Engineering".to_string(),
                    budget: 1000000.0,
                    location: Some(Location {
                        city: "San Francisco".to_string(),
                        country: "USA".to_string(),
                        coordinates: (37.7749, -122.4194),
                    }),
                }),
                salary: 150000.0,
                manager: None,
            },
            Employee {
                id: 2,
                name: "Bob".to_string(),
                department: Some(Department {
                    name: "Sales".to_string(),
                    budget: 500000.0,
                    location: None,
                }),
                salary: 120000.0,
                manager: Some(Arc::new(Employee {
                    id: 1,
                    name: "Akash".to_string(),
                    department: None,
                    salary: 150000.0,
                    manager: None,
                })),
            },
        ],
        headquarters: Some(Location {
            city: "New York".to_string(),
            country: "USA".to_string(),
            coordinates: (40.7128, -74.0060),
        }),
        financials: Arc::new(RwLock::new(Financials {
            revenue: 10000000.0,
            expenses: 8000000.0,
            profit: 2000000.0,
        })),
    };

    // ========== Example 1: Using PartialKeyPath (hides Value type) ==========
    println!("1. PartialKeyPath - Hides Value type, keeps Root visible:");

    // Create concrete keypath
    let name_kp = KeyPath::new(|c: &Company| &c.name);

    // Convert to PartialKeyPath using `to_partial()` or `to()`
    let partial_name = name_kp.to_partial();
    // Or: let partial_name = name_kp.to();

    // Access using type-erased interface
    let name_any = partial_name.get(&company);
    println!("  Company name (via PartialKeyPath): {:?}", name_any);

    // Downcast to specific type
    if let Some(name) = partial_name.get_as::<String>(&company) {
        println!("  Company name (downcast): {}", name);
    }

    // Check value type
    println!("  Value TypeId: {:?}", partial_name.value_type_id());
    println!(
        "  Matches String: {}",
        partial_name.value_type_id() == TypeId::of::<String>()
    );

    // ========== Example 2: Using PartialOptionalKeyPath with chaining ==========
    println!("\n2. PartialOptionalKeyPath - Chaining through nested Option types:");

    // Create keypath chain: Company -> Option<Location> -> city
    let hq_kp = OptionalKeyPath::new(|c: &Company| c.headquarters.as_ref());
    let city_kp = OptionalKeyPath::new(|l: &Location| Some(&l.city));

    // Chain them
    let hq_city_kp = hq_kp.then(city_kp);

    // Convert to PartialOptionalKeyPath
    let partial_hq_city = hq_city_kp.to_partial();

    // Access
    if let Some(city_any) = partial_hq_city.get(&company) {
        println!(
            "  Headquarters city (via PartialOptionalKeyPath): {:?}",
            city_any
        );

        // Downcast
        if let Some(Some(city)) = partial_hq_city.get_as::<String>(&company) {
            println!("  Headquarters city (downcast): {}", city);
        }
    }

    // ========== Example 3: Using AnyKeyPath (hides both Root and Value) ==========
    println!("\n3. AnyKeyPath - Hides both Root and Value types:");

    // Create keypaths for different types
    let company_name_kp = OptionalKeyPath::new(|c: &Company| Some(&c.name));
    let employee_name_kp = OptionalKeyPath::new(|e: &Employee| Some(&e.name));

    // Convert to AnyKeyPath - can now store in same collection
    let any_company_name = company_name_kp.to_any();
    let any_employee_name = employee_name_kp.to_any();

    // Store in a collection without knowing exact types
    let any_keypaths: Vec<AnyKeyPath> = vec![any_company_name.clone(), any_employee_name.clone()];

    println!(
        "  Stored {} keypaths in Vec<AnyKeyPath>",
        any_keypaths.len()
    );
    println!(
        "  Company name TypeId: {:?}",
        any_company_name.root_type_id()
    );
    println!(
        "  Employee name TypeId: {:?}",
        any_employee_name.root_type_id()
    );

    // Use the AnyKeyPath
    if let Some(name_any) = any_company_name.get(&company as &dyn Any) {
        println!("  Company name (via AnyKeyPath): {:?}", name_any);

        // Type-checked access
        if let Some(Some(name)) = any_company_name.get_as::<Company, String>(&company) {
            println!("  Company name (type-checked): {}", name);
        }
    }

    // ========== Example 4: Complex nested access with PartialOptionalKeyPath ==========
    println!("\n4. Complex nested access - Employee -> Department -> Location -> city:");

    // Create keypath: Company -> employees[0] -> department -> location -> city
    let employees_kp = OptionalKeyPath::new(|c: &Company| c.employees.get(0));
    let dept_kp = OptionalKeyPath::new(|e: &Employee| e.department.as_ref());
    let loc_kp = OptionalKeyPath::new(|d: &Department| d.location.as_ref());
    let city_kp = OptionalKeyPath::new(|l: &Location| Some(&l.city));

    // Chain all together
    let complex_kp = employees_kp.then(dept_kp).then(loc_kp).then(city_kp);

    // Convert to PartialOptionalKeyPath
    let partial_complex = complex_kp.to_partial();

    // Access
    if let Some(city_any) = partial_complex.get(&company) {
        println!(
            "  Employee department city (via PartialOptionalKeyPath): {:?}",
            city_any
        );

        // Downcast
        if let Some(Some(city)) = partial_complex.get_as::<String>(&company) {
            println!("  Employee department city (downcast): {}", city);
        }
    }

    // ========== Example 5: Writable Partial and Any keypaths ==========
    println!("\n5. Writable Partial and Any keypaths:");

    // Create writable keypath
    let salary_kp = WritableKeyPath::new(|e: &mut Employee| &mut e.salary);

    // Convert to PartialWritableKeyPath
    let partial_salary = salary_kp.to_partial();

    // Modify through type-erased interface
    if let Some(employee) = company.employees.get_mut(0) {
        if let Some(salary_any) = partial_salary.get_mut_as::<f64>(employee) {
            *salary_any = 160000.0;
            println!(
                "  Updated salary via PartialWritableKeyPath: {}",
                employee.salary
            );
        }
    }

    // Using AnyWritableKeyPath
    let salary_opt_kp = WritableOptionalKeyPath::new(|e: &mut Employee| {
        e.department.as_mut().map(|d| &mut d.budget)
    });
    let any_budget = salary_opt_kp.to_any();

    if let Some(employee) = company.employees.get_mut(0) {
        if let Some(Some(budget)) = any_budget.get_mut_as::<Employee, f64>(employee) {
            *budget = 1100000.0;
            println!(
                "  Updated budget via AnyWritableKeyPath: {}",
                employee.department.as_ref().unwrap().budget
            );
        }
    }

    // ========== Example 6: Using `from()` and `to()` methods ==========
    println!("\n6. Using `from()` and `to()` methods for conversion:");

    // Create keypath and convert using `to()` method (recommended)
    let name_kp2 = KeyPath::new(|c: &Company| &c.name);
    let _partial_name2 = name_kp2.to(); // Using `to()` alias for `to_partial()`

    // Create OptionalKeyPath and convert using `to()` method
    let hq_kp2 = OptionalKeyPath::new(|c: &Company| c.headquarters.as_ref());
    let _partial_hq = hq_kp2.to(); // Using `to()` alias for `to_partial()`

    // Create another for `to_any()` (since `to()` consumes the keypath)
    let hq_kp2_any = OptionalKeyPath::new(|c: &Company| c.headquarters.as_ref());
    let _any_hq = hq_kp2_any.to_any(); // Using `to_any()`

    // Alternative: Use `new()` static method directly (same as `from()`)
    let hq_kp3 = OptionalKeyPath::new(|c: &Company| c.headquarters.as_ref());
    let _partial_hq2 = PartialOptionalKeyPath::new(hq_kp3);
    let hq_kp4 = OptionalKeyPath::new(|c: &Company| c.headquarters.as_ref());
    let _any_hq2 = AnyKeyPath::new(hq_kp4);

    println!("  Created PartialKeyPath using `to()`");
    println!("  Created PartialOptionalKeyPath using `to()`");
    println!("  Created AnyKeyPath using `to_any()`");
    println!("  Alternative: Use `new()` static method (same as `from()`)");

    // ========== Example 7: Accessing Arc<RwLock<T>> through chain ==========
    println!("\n7. Accessing Arc<RwLock<T>> through type-erased keypath:");

    // Create keypath to financials
    let financials_kp = OptionalKeyPath::new(|c: &Company| Some(&c.financials));

    // Convert to PartialOptionalKeyPath
    let partial_financials = financials_kp.to_partial();

    // Access the Arc<RwLock<Financials>>
    if let Some(_financials_any) = partial_financials.get(&company) {
        // We know it's Arc<RwLock<Financials>>, but we can't directly downcast
        // because we need to handle the Arc and RwLock
        println!("  Financials accessed via PartialOptionalKeyPath");

        // For actual access, we'd use the concrete keypath or helper functions
        if let Some(guard) = containers::read_arc_rwlock(&company.financials) {
            println!("  Current revenue: ${:.2}", guard.revenue);
            println!("  Current profit: ${:.2}", guard.profit);
        }
    }

    // ========== Example 8: Type checking and validation ==========
    println!("\n8. Type checking and validation:");

    let kp1 = OptionalKeyPath::new(|c: &Company| Some(&c.name));
    let kp2 = OptionalKeyPath::new(|e: &Employee| Some(&e.name));

    let any1 = kp1.to_any();
    let any2 = kp2.to_any();

    // Check if keypaths are compatible
    println!(
        "  Company name keypath - Root: {:?}, Value: {:?}",
        any1.root_type_id(),
        any1.value_type_id()
    );
    println!(
        "  Employee name keypath - Root: {:?}, Value: {:?}",
        any2.root_type_id(),
        any2.value_type_id()
    );

    // Try to use wrong keypath on wrong type
    if any1.get(&company as &dyn Any).is_some() {
        println!("  ✓ Company keypath works on Company");
    }

    if any2.get(&company as &dyn Any).is_none() {
        println!("  ✓ Employee keypath correctly fails on Company");
    }

    println!("\n✅ All Partial and Any keypath examples completed!");

    // ========== Explanation: Why PhantomData? ==========
    println!("\n=== Why PhantomData? ===");
    println!("PhantomData<Root> is needed in PartialKeyPath<Root> because:");
    println!("1. The Root type is not actually stored in the struct (only used in the closure)");
    println!("2. Rust needs to know the generic type parameter for:");
    println!("   - Type checking at compile time");
    println!(
        "   - Ensuring correct usage (e.g., PartialKeyPath<User> can only be used with &User)"
    );
    println!("   - Preventing mixing different Root types");
    println!("3. Without PhantomData, Rust would complain that Root is unused");
    println!("4. PhantomData is zero-sized - it adds no runtime overhead");
    println!("\nFor AnyKeyPath, we don't need PhantomData because:");
    println!("- Both Root and Value types are completely erased");
    println!("- We store TypeId instead for runtime type checking");
    println!("- The type information is encoded in the closure's behavior, not the struct");
}
