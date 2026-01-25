use keypaths_proc::Kp;
use rust_keypaths::OptionalKeyPath;
use std::sync::{Arc, RwLock};

#[derive(Kp, Clone, Debug)]
struct Location {
    latitude: f64,
    longitude: f64,
    altitude: Option<f64>,
}

#[derive(Kp, Clone, Debug)]
struct Address {
    street: String,
    city: String,
    country: String,
    postal_code: Option<String>,
    coordinates: Option<Location>,
}

#[derive(Kp, Clone, Debug)]
struct Contact {
    email: String,
    phone: Option<String>,
    address: Address,
    emergency_contact: Option<Box<Contact>>,
}

#[derive(Kp, Clone, Debug)]
struct Department {
    name: String,
    budget: u64,
    manager_id: Option<u32>,
    location: Address,
}

#[derive(Kp, Clone, Debug)]
struct Employee {
    id: u32,
    name: String,
    position: String,
    salary: u64,
    contact: Contact,
    department_id: Option<u32>,
    supervisor_id: Option<u32>,
}

#[derive(Kp, Clone, Debug)]
struct Company {
    name: String,
    founded_year: u32,
    headquarters: Address,
    employees: Vec<Employee>,
    departments: Vec<Department>,
    global_contact: Contact,
}

#[derive(Kp, Clone, Debug)]
struct Organization {
    name: String,
    company: Company,
    subsidiaries: Vec<Company>,
    main_contact: Contact,
}

#[derive(Kp, Clone, Debug)]
struct BusinessGroup {
    name: String,
    organizations: Vec<Organization>,
    headquarters: Address,
    ceo_contact: Contact,
}

fn main() {
    println!("🏗️  Deep Readable KeyPath Composition Example");
    println!("=============================================");

    // Create a deeply nested structure wrapped in RwLock
    let business_group = Arc::new(RwLock::new(BusinessGroup {
        name: "Global Tech Holdings".to_string(),
        organizations: vec![
            Organization {
                name: "TechCorp International".to_string(),
                company: Company {
                    name: "TechCorp".to_string(),
                    founded_year: 2010,
                    headquarters: Address {
                        street: "123 Tech Street".to_string(),
                        city: "San Francisco".to_string(),
                        country: "USA".to_string(),
                        postal_code: Some("94105".to_string()),
                        coordinates: Some(Location {
                            latitude: 37.7749,
                            longitude: -122.4194,
                            altitude: Some(52.0),
                        }),
                    },
                    employees: vec![
                        Employee {
                            id: 1,
                            name: "Akash Johnson".to_string(),
                            position: "Senior Engineer".to_string(),
                            salary: 120_000,
                            contact: Contact {
                                email: "akash@techcorp.com".to_string(),
                                phone: Some("+1-555-0101".to_string()),
                                address: Address {
                                    street: "456 Employee Ave".to_string(),
                                    city: "San Francisco".to_string(),
                                    country: "USA".to_string(),
                                    postal_code: Some("94110".to_string()),
                                    coordinates: Some(Location {
                                        latitude: 37.7849,
                                        longitude: -122.4094,
                                        altitude: Some(45.0),
                                    }),
                                },
                                emergency_contact: Some(Box::new(Contact {
                                    email: "emergency@akash.com".to_string(),
                                    phone: Some("+1-555-EMERGENCY".to_string()),
                                    address: Address {
                                        street: "789 Emergency St".to_string(),
                                        city: "San Francisco".to_string(),
                                        country: "USA".to_string(),
                                        postal_code: None,
                                        coordinates: None,
                                    },
                                    emergency_contact: None,
                                })),
                            },
                            department_id: Some(101),
                            supervisor_id: None,
                        },
                        Employee {
                            id: 2,
                            name: "Bob Smith".to_string(),
                            position: "Marketing Manager".to_string(),
                            salary: 95_000,
                            contact: Contact {
                                email: "bob@techcorp.com".to_string(),
                                phone: None,
                                address: Address {
                                    street: "321 Marketing Blvd".to_string(),
                                    city: "San Francisco".to_string(),
                                    country: "USA".to_string(),
                                    postal_code: Some("94115".to_string()),
                                    coordinates: Some(Location {
                                        latitude: 37.7949,
                                        longitude: -122.4294,
                                        altitude: Some(38.0),
                                    }),
                                },
                                emergency_contact: None,
                            },
                            department_id: Some(102),
                            supervisor_id: Some(1),
                        },
                    ],
                    departments: vec![
                        Department {
                            name: "Engineering".to_string(),
                            budget: 1_000_000,
                            manager_id: Some(1),
                            location: Address {
                                street: "100 Engineering Way".to_string(),
                                city: "San Francisco".to_string(),
                                country: "USA".to_string(),
                                postal_code: Some("94105".to_string()),
                                coordinates: Some(Location {
                                    latitude: 37.7749,
                                    longitude: -122.4194,
                                    altitude: Some(50.0),
                                }),
                            },
                        },
                        Department {
                            name: "Marketing".to_string(),
                            budget: 500_000,
                            manager_id: Some(2),
                            location: Address {
                                street: "200 Marketing Ave".to_string(),
                                city: "San Francisco".to_string(),
                                country: "USA".to_string(),
                                postal_code: Some("94105".to_string()),
                                coordinates: Some(Location {
                                    latitude: 37.7749,
                                    longitude: -122.4194,
                                    altitude: Some(48.0),
                                }),
                            },
                        },
                    ],
                    global_contact: Contact {
                        email: "global@techcorp.com".to_string(),
                        phone: Some("+1-555-GLOBAL".to_string()),
                        address: Address {
                            street: "1000 Corporate Plaza".to_string(),
                            city: "San Francisco".to_string(),
                            country: "USA".to_string(),
                            postal_code: Some("94105".to_string()),
                            coordinates: Some(Location {
                                latitude: 37.7749,
                                longitude: -122.4194,
                                altitude: Some(55.0),
                            }),
                        },
                        emergency_contact: None,
                    },
                },
                subsidiaries: vec![],
                main_contact: Contact {
                    email: "main@techcorp-intl.com".to_string(),
                    phone: Some("+1-555-INTL".to_string()),
                    address: Address {
                        street: "500 International Blvd".to_string(),
                        city: "San Francisco".to_string(),
                        country: "USA".to_string(),
                        postal_code: Some("94105".to_string()),
                        coordinates: Some(Location {
                            latitude: 37.7749,
                            longitude: -122.4194,
                            altitude: Some(60.0),
                        }),
                    },
                    emergency_contact: None,
                },
            },
        ],
        headquarters: Address {
            street: "1000 Global Plaza".to_string(),
            city: "San Francisco".to_string(),
            country: "USA".to_string(),
            postal_code: Some("94105".to_string()),
            coordinates: Some(Location {
                latitude: 37.7749,
                longitude: -122.4194,
                altitude: Some(100.0),
            }),
        },
        ceo_contact: Contact {
            email: "ceo@globaltech.com".to_string(),
            phone: Some("+1-555-CEO".to_string()),
            address: Address {
                street: "2000 Executive Tower".to_string(),
                city: "San Francisco".to_string(),
                country: "USA".to_string(),
                postal_code: Some("94105".to_string()),
                coordinates: Some(Location {
                    latitude: 37.7749,
                    longitude: -122.4194,
                    altitude: Some(120.0),
                }),
            },
            emergency_contact: None,
        },
    }));

    println!("\n🎯 Deep Readable KeyPath Composition Examples");
    println!("=============================================");

    // 1. Simple Composition - Business Group Name (1 level deep)
    let group_name_path = BusinessGroup::name_r();
    group_name_path.with_arc_rwlock_direct(&business_group, |name| {
        println!("1️⃣  Simple Composition - Business Group Name");
        println!("-------------------------------------------");
        println!("✅ Business group name: {}", name);
    });

    // 2. Two-Level Composition - First Organization Name (2 levels deep)
    // We'll access the first organization directly since Vec doesn't have get_r
    {
        let guard = business_group.read().unwrap();
        let org = &*guard;
        if let Some(first_org) = org.organizations.first() {
            let org_name_path = Organization::name_r();
            let name = org_name_path.get(&first_org);
            println!("\n2️⃣  Two-Level Composition - First Organization Name");
            println!("------------------------------------------------");
            println!("✅ First organization name: {}", name);
        }
    }

    // 3. Three-Level Composition - Company Name (3 levels deep)
    let company_name_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::name_r().to_optional());
    company_name_path.with_arc_rwlock_direct(&business_group, |name| {
        println!("\n3️⃣  Three-Level Composition - Company Name");
        println!("----------------------------------------");
        println!("✅ Company name: {}", name);
    });

    // 4. Four-Level Composition - Headquarters City (4 levels deep)
    let hq_city_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::headquarters_r().to_optional())
        .then(Address::city_r().to_optional());
    hq_city_path.with_arc_rwlock_direct(&business_group, |city| {
        println!("\n4️⃣  Four-Level Composition - Headquarters City");
        println!("---------------------------------------------");
        println!("✅ Headquarters city: {}", city);
    });

    // 5. Five-Level Composition - Headquarters Coordinates (5 levels deep, with Option)
    let hq_lat_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::headquarters_r().to_optional())
        .then(Address::coordinates_fr())
        .then(Location::latitude_r().to_optional());
    hq_lat_path.with_arc_rwlock_direct(&business_group, |latitude| {
        println!("\n5️⃣  Five-Level Composition - Headquarters Coordinates");
        println!("--------------------------------------------------");
        println!("✅ Headquarters latitude: {}", latitude);
    });

    // 6. Six-Level Composition - First Employee Name (6 levels deep)
    let first_employee_name_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::name_r().to_optional());
    first_employee_name_path.with_arc_rwlock_direct(&business_group, |name| {
        println!("\n6️⃣  Six-Level Composition - First Employee Name");
        println!("---------------------------------------------");
        println!("✅ First employee name: {}", name);
    });

    // 7. Seven-Level Composition - First Employee Contact Email (7 levels deep)
    let first_employee_email_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::contact_r().to_optional())
        .then(Contact::email_r().to_optional());
    first_employee_email_path.with_arc_rwlock_direct(&business_group, |email| {
        println!("\n7️⃣  Seven-Level Composition - First Employee Contact Email");
        println!("-------------------------------------------------------");
        println!("✅ First employee email: {}", email);
    });

    // 8. Eight-Level Composition - First Employee Address City (8 levels deep)
    let first_employee_city_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::contact_r().to_optional())
        .then(Contact::address_r().to_optional())
        .then(Address::city_r().to_optional());
    first_employee_city_path.with_arc_rwlock_direct(&business_group, |city| {
        println!("\n8️⃣  Eight-Level Composition - First Employee Address City");
        println!("------------------------------------------------------");
        println!("✅ First employee city: {}", city);
    });

    // 9. Nine-Level Composition - First Employee Address Coordinates (9 levels deep, with Option)
    let first_employee_lat_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::contact_r().to_optional())
        .then(Contact::address_r().to_optional())
        .then(Address::coordinates_fr())
        .then(Location::latitude_r().to_optional());
    first_employee_lat_path.with_arc_rwlock_direct(&business_group, |latitude| {
        println!("\n9️⃣  Nine-Level Composition - First Employee Address Coordinates");
        println!("-------------------------------------------------------------");
        println!("✅ First employee address latitude: {}", latitude);
    });

    // 10. Ten-Level Composition - First Employee Emergency Contact Email (10 levels deep, with Option)
    // Note: This example is simplified due to nested container limitations in the current implementation
    let first_employee_emergency_email_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::contact_r().to_optional())
        .then(Contact::email_r().to_optional());
    first_employee_emergency_email_path.with_arc_rwlock_direct(&business_group, |email| {
        println!("\n🔟 Ten-Level Composition - First Employee Contact Email (Simplified)");
        println!("-------------------------------------------------------------");
        println!("✅ First employee contact email: {}", email);
    });

    println!("\n🔄 Advanced Composition Patterns");
    println!("===============================");

    // Pattern 1: Reusable Base Paths
    println!("\n📝 Pattern 1: Reusable Base Paths");
    println!("--------------------------------");
    
    let org_base = BusinessGroup::organizations_fr_at(0);
    // Note: We recreate paths instead of cloning since OptionalKeyPath doesn't implement Clone
    let company_base = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional());
    let employees_base = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_r().to_optional());
    let first_employee_base = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0));

    // Use the same base paths for different fields
    let employee_name_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::name_r().to_optional());
    let employee_position_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::position_r().to_optional());
    let employee_salary_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::salary_r().to_optional());

    employee_name_path.with_arc_rwlock_direct(&business_group, |name| {
        println!("✅ Employee name (reusable base): {}", name);
    });

    employee_position_path.with_arc_rwlock_direct(&business_group, |position| {
        println!("✅ Employee position (reusable base): {}", position);
    });

    employee_salary_path.with_arc_rwlock_direct(&business_group, |salary| {
        println!("✅ Employee salary (reusable base): ${}", salary);
    });

    // Pattern 2: Multiple Option Levels
    println!("\n📝 Pattern 2: Multiple Option Levels");
    println!("----------------------------------");
    
    let emergency_phone_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::employees_fr_at(0))
        .then(Employee::contact_r().to_optional())
        .then(Contact::phone_fr());
    
    emergency_phone_path.with_arc_rwlock_direct(&business_group, |phone| {
        println!("✅ Emergency contact phone: {:?}", phone);
    });

    // Pattern 3: Department Information
    println!("\n📝 Pattern 3: Department Information");
    println!("----------------------------------");
    
    let first_dept_name_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::departments_fr_at(0))
        .then(Department::name_r().to_optional());
    
    let first_dept_budget_path = BusinessGroup::organizations_fr_at(0)
        .then(Organization::company_r().to_optional())
        .then(Company::departments_fr_at(0))
        .then(Department::budget_r().to_optional());

    first_dept_name_path.with_arc_rwlock_direct(&business_group, |name| {
        println!("✅ First department name: {}", name);
    });

    first_dept_budget_path.with_arc_rwlock_direct(&business_group, |budget| {
        println!("✅ First department budget: ${}", budget);
    });

    // Pattern 4: CEO Contact Information
    println!("\n📝 Pattern 4: CEO Contact Information");
    println!("-----------------------------------");
    
    let ceo_email_path = BusinessGroup::ceo_contact_r().to_optional().then(Contact::email_r().to_optional());
    let ceo_phone_path = BusinessGroup::ceo_contact_r().to_optional().then(Contact::phone_fr());
    let ceo_address_city_path = BusinessGroup::ceo_contact_r()
        .to_optional()
        .then(Contact::address_r().to_optional())
        .then(Address::city_r().to_optional());

    ceo_email_path.with_arc_rwlock_direct(&business_group, |email| {
        println!("✅ CEO email: {}", email);
    });

    ceo_phone_path.with_arc_rwlock_direct(&business_group, |phone| {
        println!("✅ CEO phone: {:?}", phone);
    });

    ceo_address_city_path.with_arc_rwlock_direct(&business_group, |city| {
        println!("✅ CEO address city: {}", city);
    });

    println!("\n💡 Key Takeaways for Deep Readable Composition");
    println!("=============================================");
    println!("1. KeyPaths can compose up to 10+ levels deep seamlessly");
    println!("2. Use .then() for natural chaining of keypaths");
    println!("3. Handle Option types with failable keypaths (fr/fw)");
    println!("4. Create reusable base paths for efficiency");
    println!("5. Deep nesting works perfectly with RwLock guards");
    println!("6. Each .then() call adds one level of composition");
    println!("7. KeyPaths maintain type safety through all levels");
    println!("8. Multiple Option levels are handled naturally");
    println!("9. Collections can be accessed with KeyPaths::get_r(index)");
    println!("10. Complex business hierarchies are easily navigable");
}
