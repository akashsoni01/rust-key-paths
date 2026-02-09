use std::sync::{Arc, Mutex, RwLock};
use key_paths_derive::Kp;
use rust_key_paths::async_lock::SyncKeyPathLike;

#[derive(Kp, Clone, Debug)]
struct SomeOtherStruct {
    value: String,
    count: u32,
}

#[derive(Kp, Clone, Debug)]
struct SomeStruct {
    field1: Arc<std::sync::RwLock<SomeOtherStruct>>,
    field2: Arc<std::sync::Mutex<SomeOtherStruct>>,
}

fn main() {
    println!("🔧 Arc<Sync> Derive Macro Support Example");
    println!("=========================================");

    // Create test data
    let some_struct = SomeStruct {
        field1: Arc::new(RwLock::new(SomeOtherStruct {
            value: "Hello from RwLock".to_string(),
            count: 42,
        })),
        field2: Arc::new(Mutex::new(SomeOtherStruct {
            value: "Hello from Mutex".to_string(),
            count: 24,
        })),
    };

    println!("\n🎯 Testing Arc<RwLock<T>> Field Access");
    println!("-------------------------------------");

    // Test Arc<RwLock<T>> field access
    let field1_path = SomeStruct::field1();
    if let Some(field1_ref) = field1_path.get(&some_struct) {
        println!("✅ Arc<RwLock<SomeOtherStruct>> field accessible: {:?}", field1_ref);
    }

    // Test Arc<Mutex<T>> field access
    let field2_path = SomeStruct::field2();
    if let Some(field2_ref) = field2_path.get(&some_struct) {
        println!("✅ Arc<Mutex<SomeOtherStruct>> field accessible: {:?}", field2_ref);
    }

    println!("\n🎯 Testing with WithContainer Trait");
    println!("----------------------------------");

    // Test with WithContainer trait for no-clone access
    let value_path = SomeOtherStruct::value();
    let count_path = SomeOtherStruct::count();

    println!("\n🎯 Testing Read-Only Composition");
    println!("--------------------------------");

    // Create a more complex nested structure for composition
    #[derive(Kp, Clone, Debug)]
    struct Company {
        name: String,
        departments: Vec<Department>,
    }

    #[derive(Kp, Clone, Debug)]
    struct Department {
        name: String,
        manager: Arc<std::sync::RwLock<Employee>>,
        budget: u64,
    }

    #[derive(Kp, Clone, Debug)]
    struct Employee {
        name: String,
        salary: u32,
        contact: Arc<std::sync::Mutex<Contact>>,
    }

    #[derive(Kp, Clone, Debug)]
    struct Contact {
        email: String,
        phone: String,
    }

    // Create test data
    let company = Company {
        name: "TechCorp".to_string(),
        departments: vec![
            Department {
                name: "Engineering".to_string(),
                manager: Arc::new(RwLock::new(Employee {
                    name: "Akash Johnson".to_string(),
                    salary: 120000,
                    contact: Arc::new(Mutex::new(Contact {
                        email: "akash@techcorp.com".to_string(),
                        phone: "+1-555-0123".to_string(),
                    })),
                })),
                budget: 500000,
            },
            Department {
                name: "Marketing".to_string(),
                manager: Arc::new(RwLock::new(Employee {
                    name: "Bob Smith".to_string(),
                    salary: 95000,
                    contact: Arc::new(Mutex::new(Contact {
                        email: "bob@techcorp.com".to_string(),
                        phone: "+1-555-0456".to_string(),
                    })),
                })),
                budget: 200000,
            },
        ],
    };

    // Example 1: Simple composition - Company name
    let company_name_path = Company::name();
    if let Some(name) = company_name_path.get(&company) {
        println!("✅ Company name: {}", name);
    }

    // Example 2: Composition through Vec - First department name
    // We need to access the Vec element directly since KeyPaths doesn't have get_r
    if let Some(first_dept) = company.departments.first() {
        let dept_name_path = Department::name();
        if let Some(dept_name) = dept_name_path.get(&first_dept) {
            println!("✅ First department: {}", dept_name);
        }
    }

    // Example 3: Deep composition - Manager name through Arc<RwLock>
    // Get the Arc<RwLock<Employee>> first, then use with_rwlock
    if let Some(first_dept) = company.departments.first() {
        let manager_arc_path = Department::manager();
        if let Some(manager_arc) = manager_arc_path.get(&first_dept) {
            let employee_name_path = Employee::name();
        }
    }

    // Example 4: Even deeper composition - Contact email through Arc<Mutex>
    if let Some(first_dept) = company.departments.first() {
        let manager_arc_path = Department::manager();
        if let Some(manager_arc) = manager_arc_path.get(&first_dept) {
            // Get the contact Arc<Mutex<Contact>> from the employee
            let contact_arc_path = Employee::contact();
        }
    }
    
}
