use key_paths_derive::Keypaths;
use std::sync::Arc;
use parking_lot::RwLock;

// Deeply nested data structures for demonstration
#[derive(Keypaths, Clone, Debug)]
struct Address {
    street: String,
    city: String,
    country: String,
    coordinates: Option<Coordinates>,
}

#[derive(Keypaths, Clone, Debug)]
struct Coordinates {
    latitude: f64,
    longitude: f64,
}

#[derive(Keypaths, Clone, Debug)]
struct Contact {
    email: String,
    phone: Option<String>,
    address: Address,
}

#[derive(Keypaths, Clone, Debug)]
struct Department {
    name: String,
    budget: u64,
    manager_id: Option<u32>, // Use ID instead of direct reference to avoid recursion
}

#[derive(Keypaths, Clone, Debug)]
struct Employee {
    id: u32,
    name: String,
    contact: Contact,
    department_id: Option<u32>, // Use ID instead of direct reference to avoid recursion
    salary: u64,
}

#[derive(Keypaths, Clone, Debug)]
struct Company {
    name: String,
    headquarters: Address,
    employees: Vec<Employee>,
    departments: Vec<Department>,
}

#[derive(Keypaths, Clone, Debug)]
struct Organization {
    company: Company,
    subsidiaries: Vec<Company>,
    global_contact: Contact,
}

fn main() {
    println!("🏗️  Deep Nesting KeyPath Composition Example");
    println!("=============================================");

    // Create a complex nested structure wrapped in RwLock
    let organization = Arc::new(RwLock::new(Organization {
        company: Company {
            name: "TechCorp".to_string(),
            headquarters: Address {
                street: "123 Tech Street".to_string(),
                city: "San Francisco".to_string(),
                country: "USA".to_string(),
                coordinates: Some(Coordinates {
                    latitude: 37.7749,
                    longitude: -122.4194,
                }),
            },
            employees: vec![
                Employee {
                    id: 1,
                    name: "Alice Johnson".to_string(),
                    contact: Contact {
                        email: "alice@techcorp.com".to_string(),
                        phone: Some("+1-555-0101".to_string()),
                        address: Address {
                            street: "456 Employee Ave".to_string(),
                            city: "San Francisco".to_string(),
                            country: "USA".to_string(),
                            coordinates: Some(Coordinates {
                                latitude: 37.7849,
                                longitude: -122.4094,
                            }),
                        },
                    },
                    department_id: Some(1),
                    salary: 120_000,
                },
                Employee {
                    id: 2,
                    name: "Bob Smith".to_string(),
                    contact: Contact {
                        email: "bob@techcorp.com".to_string(),
                        phone: None,
                        address: Address {
                            street: "789 Developer Blvd".to_string(),
                            city: "San Francisco".to_string(),
                            country: "USA".to_string(),
                            coordinates: None,
                        },
                    },
                    department_id: Some(1),
                    salary: 95_000,
                },
            ],
            departments: vec![
                Department {
                    name: "Engineering".to_string(),
                    budget: 1_000_000,
                    manager_id: None,
                },
                Department {
                    name: "Marketing".to_string(),
                    budget: 500_000,
                    manager_id: None,
                },
            ],
        },
        subsidiaries: vec![
            Company {
                name: "TechCorp Europe".to_string(),
                headquarters: Address {
                    street: "456 European Ave".to_string(),
                    city: "London".to_string(),
                    country: "UK".to_string(),
                    coordinates: Some(Coordinates {
                        latitude: 51.5074,
                        longitude: -0.1278,
                    }),
                },
                employees: vec![],
                departments: vec![],
            },
        ],
        global_contact: Contact {
            email: "global@techcorp.com".to_string(),
            phone: Some("+1-555-GLOBAL".to_string()),
            address: Address {
                street: "123 Tech Street".to_string(),
                city: "San Francisco".to_string(),
                country: "USA".to_string(),
                coordinates: Some(Coordinates {
                    latitude: 37.7749,
                    longitude: -122.4194,
                }),
            },
        },
    }));

    println!("\n🎯 Natural KeyPath Composition Examples");
    println!("=======================================");

    // Example 1: Simple composition - Company name
    println!("\n1️⃣  Simple Composition - Company Name");
    println!("-------------------------------------");
    let company_name_path = Organization::company_r().then(Company::name_r());
    
    {
        let guard = organization.read();
        if let Some(name) = company_name_path.get_ref(&&*guard) {
            println!("✅ Company name: {}", name);
        }
    }

    // Example 2: Two-level composition - Headquarters city
    println!("\n2️⃣  Two-Level Composition - Headquarters City");
    println!("---------------------------------------------");
    let hq_city_path = Organization::company_r()
        .then(Company::headquarters_r())
        .then(Address::city_r());
    
    {
        let guard = organization.read();
        if let Some(city) = hq_city_path.get_ref(&&*guard) {
            println!("✅ Headquarters city: {}", city);
        }
    }

    // Example 3: Three-level composition - Headquarters coordinates
    println!("\n3️⃣  Three-Level Composition - Headquarters Coordinates");
    println!("----------------------------------------------------");
    let hq_lat_path = Organization::company_r()
        .then(Company::headquarters_r())
        .then(Address::coordinates_fr())
        .then(Coordinates::latitude_r());
    
    {
        let guard = organization.read();
        if let Some(latitude) = hq_lat_path.get_ref(&&*guard) {
            println!("✅ Headquarters latitude: {}", latitude);
        }
    }

    // Example 4: Four-level composition - Global contact email
    println!("\n4️⃣  Four-Level Composition - Global Contact Email");
    println!("------------------------------------------------");
    let global_email_path = Organization::global_contact_r()
        .then(Contact::email_r());
    
    {
        let guard = organization.read();
        if let Some(email) = global_email_path.get_ref(&&*guard) {
            println!("✅ Global contact email: {}", email);
        }
    }

    // Example 5: Five-level composition - Global contact address coordinates
    println!("\n5️⃣  Five-Level Composition - Global Contact Address Coordinates");
    println!("-------------------------------------------------------------");
    let global_coords_path = Organization::global_contact_r()
        .then(Contact::address_r())
        .then(Address::coordinates_fr())
        .then(Coordinates::latitude_r());
    
    {
        let guard = organization.read();
        if let Some(latitude) = global_coords_path.get_ref(&&*guard) {
            println!("✅ Global contact address latitude: {}", latitude);
        }
    }

    // Example 6: Working with collections - First department budget
    println!("\n6️⃣  Working with Collections - First Department Budget");
    println!("---------------------------------------------------");
    
    // Since Vec doesn't have get_r, we'll access the first department directly
    // This demonstrates how to work with collections in a natural way
    {
        let guard = organization.read();
        let org = &*guard;
        if let Some(first_dept) = org.company.departments.first() {
            let dept_budget_path = Department::budget_r();
            if let Some(budget) = dept_budget_path.get_ref(&first_dept) {
                println!("✅ First department budget: ${}", budget);
            }
        }
    }

    // Example 7: Working with employees - First employee contact
    println!("\n7️⃣  Working with Employees - First Employee Contact");
    println!("-------------------------------------------------");
    
    {
        let guard = organization.read();
        let org = &*guard;
        if let Some(first_employee) = org.company.employees.first() {
            let employee_contact_path = Employee::contact_r()
                .then(Contact::email_r());
            if let Some(email) = employee_contact_path.get_ref(&first_employee) {
                println!("✅ First employee email: {}", email);
            }
        }
    }

    // Example 8: Global contact with optional phone
    println!("\n8️⃣  Global Contact with Optional Phone");
    println!("-------------------------------------");
    let global_phone_path = Organization::global_contact_r()
        .then(Contact::phone_fr());
    
    {
        let guard = organization.read();
        if let Some(phone) = global_phone_path.get_ref(&&*guard) {
            println!("✅ Global contact phone: {}", phone);
        }
    }

    println!("\n🔄 Natural Composition Patterns");
    println!("===============================");

    // Pattern 1: Building keypaths step by step (very natural)
    println!("\n📝 Pattern 1: Step-by-Step Composition");
    println!("--------------------------------------");
    
    // Start with organization
    let org_path = Organization::company_r();
    
    // Add company level
    let company_path = org_path.then(Company::headquarters_r());
    
    // Add headquarters level
    let hq_path = company_path.then(Address::city_r());
    
    {
        let guard = organization.read();
        if let Some(city) = hq_path.get_ref(&&*guard) {
            println!("✅ Headquarters city (step-by-step): {}", city);
        }
    }

    // Pattern 2: Fluent composition (very readable)
    println!("\n📝 Pattern 2: Fluent Composition");
    println!("-------------------------------");
    
    let fluent_path = Organization::company_r()
        .then(Company::headquarters_r())
        .then(Address::country_r());
    
    {
        let guard = organization.read();
        if let Some(country) = fluent_path.get_ref(&&*guard) {
            println!("✅ Headquarters country (fluent): {}", country);
        }
    }

    // Pattern 3: Reusable intermediate keypaths
    println!("\n📝 Pattern 3: Reusable Intermediate KeyPaths");
    println!("-------------------------------------------");
    
    // Create reusable base paths
    let company_base = Organization::company_r();
    let hq_base = company_base.then(Company::headquarters_r());
    let address_base = hq_base.then(Address::coordinates_fr());
    
    // Compose different paths using the same base
    let hq_lat_path = address_base.clone().then(Coordinates::latitude_r());
    let hq_lng_path = address_base.then(Coordinates::longitude_r());
    
    {
        let guard = organization.read();
        if let Some(lat) = hq_lat_path.get_ref(&&*guard) {
            println!("✅ HQ latitude (reusable): {}", lat);
        }
        if let Some(lng) = hq_lng_path.get_ref(&&*guard) {
            println!("✅ HQ longitude (reusable): {}", lng);
        }
    }

    // Pattern 4: Working with multiple levels of Option
    println!("\n📝 Pattern 4: Multiple Levels of Option");
    println!("-------------------------------------");
    
    let optional_coords_path = Organization::company_r()
        .then(Company::headquarters_r())
        .then(Address::coordinates_fr())
        .then(Coordinates::latitude_r());
    
    {
        let guard = organization.read();
        if let Some(latitude) = optional_coords_path.get_ref(&&*guard) {
            println!("✅ HQ coordinates latitude: {}", latitude);
        } else {
            println!("✅ HQ has no coordinates");
        }
    }

    // Pattern 5: Working with collections using iteration
    println!("\n📝 Pattern 5: Working with Collections");
    println!("-------------------------------------");
    
    {
        let guard = organization.read();
        let org = &*guard;
        
        // Iterate through employees and use keypaths on each
        for (i, employee) in org.company.employees.iter().enumerate() {
            let employee_name_path = Employee::name_r();
            let employee_email_path = Employee::contact_r().then(Contact::email_r());
            
            if let Some(name) = employee_name_path.get_ref(&employee) {
                if let Some(email) = employee_email_path.get_ref(&employee) {
                    println!("✅ Employee {}: {} ({})", i + 1, name, email);
                }
            }
        }
    }

    println!("\n💡 Key Takeaways for Natural Composition");
    println!("========================================");
    println!("1. Use .then() for natural chaining of keypaths");
    println!("2. Build complex paths step-by-step for clarity");
    println!("3. Create reusable intermediate keypaths");
    println!("4. Use failable keypaths (fr/fw) for Option types");
    println!("5. Fluent composition reads like natural language");
    println!("6. Deep nesting works seamlessly with RwLock guards");
    println!("7. Each .then() call adds one level of composition");
    println!("8. KeyPaths maintain type safety through all levels");
    println!("9. For collections, access elements first, then apply keypaths");
    println!("10. Multiple Option levels are handled naturally with failable keypaths");
}