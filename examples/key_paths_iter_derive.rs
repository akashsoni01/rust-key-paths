//! Example: [key-paths-iter] query builder with keypaths from the [Kp] derive macro.
//!
//! Uses `#[derive(Kp)]` to get keypaths to `Vec<Item>` fields, then `.query()` to filter,
//! limit, and execute over the collection.

use key_paths_derive::Kp;
use key_paths_iter::QueryableCollectionStatic;

#[derive(Kp, Debug)]
struct Company {
    name: String,
    employees: Vec<Employee>,
}

#[derive(Debug)]
struct Employee {
    name: String,
    role: String,
    active: bool,
    years: u32,
}

fn main() {
    let company = Company {
        name: "Acme Corp".to_string(),
        employees: vec![
            Employee {
                name: "Alice".to_string(),
                role: "Engineer".to_string(),
                active: true,
                years: 5,
            },
            Employee {
                name: "Bob".to_string(),
                role: "Designer".to_string(),
                active: false,
                years: 3,
            },
            Employee {
                name: "Charlie".to_string(),
                role: "Engineer".to_string(),
                active: true,
                years: 8,
            },
            Employee {
                name: "Diana".to_string(),
                role: "Manager".to_string(),
                active: true,
                years: 2,
            },
        ],
    };

    // Keypath from derive: Company::employees() returns KpType<'static, Company, Vec<Employee>>
    let employees_kp = Company::employees();

    println!("=== key-paths-iter with Kp derive ===\n");
    println!("Company: {}\n", company.name);

    // Query: active employees with 3+ years, limit 2
    let results = employees_kp
        .query()
        .filter(|e| e.active)
        .filter(|e| e.years >= 3)
        .limit(2)
        .execute(&company);

    println!("Active employees with 3+ years (limit 2):");
    for e in results {
        println!("  - {} ({}), {} years", e.name, e.role, e.years);
    }

    // Count active employees
    let active_count = employees_kp.query().filter(|e| e.active).count(&company);
    println!("\nActive employees count: {}", active_count);

    // Any engineer?
    let has_engineer = employees_kp
        .query()
        .filter(|e| e.role == "Engineer")
        .exists(&company);
    println!("Has engineer: {}", has_engineer);

    // First active employee
    if let Some(first) = employees_kp.query().filter(|e| e.active).first(&company) {
        println!("First active: {}", first.name);
    }

    println!("\nDone.");
}
