use rust_keypaths::{PartialKeyPath, AnyKeyPath};
use keypaths_proc::{Keypaths, PartialKeypaths, AnyKeypaths};
use std::sync::{Arc, Mutex, RwLock};
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Debug, Clone, Keypaths, PartialKeypaths, AnyKeypaths)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Keypaths, PartialKeypaths, AnyKeypaths)]
struct Company {
    name: String,
    employees: Vec<Person>,
    revenue: f64,
}

fn main() {
    println!("=== PartialKeyPath and AnyKeyPath Aggregator Functions Example ===\n");

    // Create sample data
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: Some("akash@example.com".to_string()),
        metadata: [("department".to_string(), "engineering".to_string())].into(),
    };

    let company = Company {
        name: "TechCorp".to_string(),
        employees: vec![person.clone()],
        revenue: 1000000.0,
    };

    // ===== PartialKeyPath Aggregator Examples =====
    println!("--- 1. PartialKeyPath Aggregator Functions ---");

    // Create a partial keypath for Person::name
    let name_partial = Person::name_partial_r();

    // Test Arc aggregator
    let person_arc = Arc::new(person.clone());
    let name_arc_partial = name_partial.clone().for_arc();
    if let Some(value) = name_arc_partial.get(&person_arc) {
        println!("Person name via Arc<Person> (partial): {:?}", value);
    }

    // Test Box aggregator
    let person_box = Box::new(person.clone());
    let name_box_partial = name_partial.clone().for_box();
    if let Some(value) = name_box_partial.get(&person_box) {
        println!("Person name via Box<Person> (partial): {:?}", value);
    }

    // Test Rc aggregator
    let person_rc = Rc::new(person.clone());
    let name_rc_partial = name_partial.clone().for_rc();
    if let Some(value) = name_rc_partial.get(&person_rc) {
        println!("Person name via Rc<Person> (partial): {:?}", value);
    }

    // Test Option aggregator
    let person_option = Some(person.clone());
    let name_option_partial = name_partial.clone().for_option();
    if let Some(value) = name_option_partial.get(&person_option) {
        println!("Person name via Option<Person> (partial): {:?}", value);
    }

    // Test Result aggregator
    let person_result: Result<Person, String> = Ok(person.clone());
    let name_result_partial = name_partial.clone().for_result::<String>();
    if let Some(value) = name_result_partial.get(&person_result) {
        println!("Person name via Result<Person, String> (partial): {:?}", value);
    }

    // Test Arc<RwLock> aggregator (owned only - requires owned keypath)
    let person_arc_rwlock = Arc::new(RwLock::new(person.clone()));
    let name_owned = Person::name_partial_o();
    let name_arc_rwlock_partial = name_owned.clone().for_arc_rwlock();
    let owned_value = name_arc_rwlock_partial.get_owned(person_arc_rwlock);
    println!("Person name via Arc<RwLock<Person>> (partial, owned): {:?}", owned_value);

    // Test Arc<Mutex> aggregator (owned only - requires owned keypath)
    let person_arc_mutex = Arc::new(Mutex::new(person.clone()));
    let name_arc_mutex_partial = name_owned.clone().for_arc_mutex();
    let owned_value = name_arc_mutex_partial.get_owned(person_arc_mutex);
    println!("Person name via Arc<Mutex<Person>> (partial, owned): {:?}", owned_value);

    // ===== AnyKeyPath Aggregator Examples =====
    println!("\n--- 2. AnyKeyPath Aggregator Functions ---");

    // Create an any keypath for Person::name
    let name_any = Person::name_any_r();

    // Test Arc aggregator
    let person_arc_boxed: Box<dyn std::any::Any + Send + Sync> = Box::new(Arc::new(person.clone()));
    let name_arc_any = name_any.clone().for_arc::<Person>();
    if let Some(value) = name_arc_any.get(&*person_arc_boxed) {
        println!("Person name via Arc<Person> (any): {:?}", value);
    }

    // Test Box aggregator
    let person_box_boxed: Box<dyn std::any::Any + Send + Sync> = Box::new(Box::new(person.clone()));
    let name_box_any = name_any.clone().for_box::<Person>();
    if let Some(value) = name_box_any.get(&*person_box_boxed) {
        println!("Person name via Box<Person> (any): {:?}", value);
    }

    // Test Rc aggregator (using Arc since Rc is not Send + Sync)
    let person_arc_boxed2: Box<dyn std::any::Any + Send + Sync> = Box::new(Arc::new(person.clone()));
    let name_arc_any2 = name_any.clone().for_arc::<Person>();
    if let Some(value) = name_arc_any2.get(&*person_arc_boxed2) {
        println!("Person name via Arc<Person> #2 (any): {:?}", value);
    }

    // Test Option aggregator
    let person_option_boxed: Box<dyn std::any::Any + Send + Sync> = Box::new(Some(person.clone()));
    let name_option_any = name_any.clone().for_option::<Person>();
    if let Some(value) = name_option_any.get(&*person_option_boxed) {
        println!("Person name via Option<Person> (any): {:?}", value);
    }

    // Test Result aggregator
    let person_result_boxed: Box<dyn std::any::Any + Send + Sync> = Box::new(Ok::<Person, String>(person.clone()));
    let name_result_any = name_any.clone().for_result::<Person, String>();
    if let Some(value) = name_result_any.get(&*person_result_boxed) {
        println!("Person name via Result<Person, String> (any): {:?}", value);
    }

    // Test Arc<RwLock> aggregator (owned only - requires owned keypath)
    let person_arc_rwlock_boxed: Box<dyn std::any::Any + Send + Sync> = Box::new(Arc::new(RwLock::new(person.clone())));
    let name_owned_any = Person::name_any_o();
    let name_arc_rwlock_any = name_owned_any.clone().for_arc_rwlock::<Person>();
    let owned_value = name_arc_rwlock_any.get_owned(person_arc_rwlock_boxed);
    println!("Person name via Arc<RwLock<Person>> (any, owned): {:?}", owned_value);

    // Test Arc<Mutex> aggregator (owned only - requires owned keypath)
    let person_arc_mutex_boxed: Box<dyn std::any::Any + Send + Sync> = Box::new(Arc::new(Mutex::new(person.clone())));
    let name_arc_mutex_any = name_owned_any.clone().for_arc_mutex::<Person>();
    let owned_value = name_arc_mutex_any.get_owned(person_arc_mutex_boxed);
    println!("Person name via Arc<Mutex<Person>> (any, owned): {:?}", owned_value);

    // ===== Mixed Container Types =====
    println!("\n--- 3. Mixed Container Types ---");

    // Create a collection of different container types
    let containers: Vec<Box<dyn std::any::Any + Send + Sync>> = vec![
        Box::new(person.clone()),
        Box::new(Arc::new(person.clone())),
        Box::new(Box::new(person.clone())),
        Box::new(Arc::new(person.clone())),
        Box::new(Some(person.clone())),
        Box::new(Ok::<Person, String>(person.clone())),
        Box::new(Arc::new(RwLock::new(person.clone()))),
        Box::new(Arc::new(Mutex::new(person.clone()))),
    ];

    // Create different keypaths
    let name_partial = Person::name_partial_r();
    let name_owned = Person::name_partial_o();
    let age_partial = Person::age_partial_r();
    let email_partial = Person::email_partial_fr();

    // Test with different aggregators
    for (i, container) in containers.iter().enumerate() {
        match i {
            0 => {
                // Direct Person
                if let Some(person_ref) = container.downcast_ref::<Person>() {
                    if let Some(value) = name_partial.get(person_ref) {
                        println!("Container {} (Person): {:?}", i, value);
                    }
                }
            }
            1 => {
                // Arc<Person>
                if let Some(arc_ref) = container.downcast_ref::<Arc<Person>>() {
                    let name_arc_partial = name_partial.clone().for_arc();
                    if let Some(value) = name_arc_partial.get(arc_ref) {
                        println!("Container {} (Arc<Person>): {:?}", i, value);
                    }
                }
            }
            2 => {
                // Box<Person>
                if let Some(box_ref) = container.downcast_ref::<Box<Person>>() {
                    let name_box_partial = name_partial.clone().for_box();
                    if let Some(value) = name_box_partial.get(box_ref) {
                        println!("Container {} (Box<Person>): {:?}", i, value);
                    }
                }
            }
            3 => {
                // Arc<Person> #2
                if let Some(arc_ref) = container.downcast_ref::<Arc<Person>>() {
                    let name_arc_partial = name_partial.clone().for_arc();
                    if let Some(value) = name_arc_partial.get(arc_ref) {
                        println!("Container {} (Arc<Person> #2): {:?}", i, value);
                    }
                }
            }
            4 => {
                // Option<Person>
                if let Some(option_ref) = container.downcast_ref::<Option<Person>>() {
                    let name_option_partial = name_partial.clone().for_option();
                    if let Some(value) = name_option_partial.get(option_ref) {
                        println!("Container {} (Option<Person>): {:?}", i, value);
                    }
                }
            }
            5 => {
                // Result<Person, String>
                if let Some(result_ref) = container.downcast_ref::<Result<Person, String>>() {
                    let name_result_partial = name_partial.clone().for_result::<String>();
                    if let Some(value) = name_result_partial.get(result_ref) {
                        println!("Container {} (Result<Person, String>): {:?}", i, value);
                    }
                }
            }
            6 => {
                // Arc<RwLock<Person>> (owned only - requires owned keypath)
                if let Some(arc_rwlock_ref) = container.downcast_ref::<Arc<RwLock<Person>>>() {
                    let name_arc_rwlock_partial = name_owned.clone().for_arc_rwlock();
                    let owned_value = name_arc_rwlock_partial.get_owned(arc_rwlock_ref.clone());
                    println!("Container {} (Arc<RwLock<Person>>, owned): {:?}", i, owned_value);
                }
            }
            7 => {
                // Arc<Mutex<Person>> (owned only - requires owned keypath)
                if let Some(arc_mutex_ref) = container.downcast_ref::<Arc<Mutex<Person>>>() {
                    let name_arc_mutex_partial = name_owned.clone().for_arc_mutex();
                    let owned_value = name_arc_mutex_partial.get_owned(arc_mutex_ref.clone());
                    println!("Container {} (Arc<Mutex<Person>>, owned): {:?}", i, owned_value);
                }
            }
            _ => {}
        }
    }

    // ===== Composition with Aggregators =====
    println!("\n--- 4. Composition with Aggregators ---");

    // Create a company with Arc<RwLock<Person>> employees
    let employee = Arc::new(RwLock::new(person.clone()));
    let company_with_arc_employees = Company {
        name: "TechCorp".to_string(),
        employees: vec![employee.read().unwrap().clone()],
        revenue: 1000000.0,
    };

    // Create keypaths for company name and first employee name
    let company_name_partial = Company::name_partial_r();
    let employee_name_partial = Person::name_partial_r();

    // Access company name directly
    if let Some(value) = company_name_partial.get(&company_with_arc_employees) {
        println!("Company name: {:?}", value);
    }

    // Access first employee name through composition
    let first_employee_partial = Company::employees_partial_fr_at(0);
    if let Some(value) = first_employee_partial.get(&company_with_arc_employees) {
        println!("First employee (type-erased): {:?}", value);
    }

    println!("\n✅ PartialKeyPath and AnyKeyPath Aggregator Functions Example completed!");
    println!("📝 This example demonstrates:");
    println!("   • PartialKeyPath aggregator functions (for_arc, for_box, for_rc, etc.)");
    println!("   • AnyKeyPath aggregator functions with type parameters");
    println!("   • Working with different container types (Arc, Box, Rc, Option, Result, etc.)");
    println!("   • Thread-safe containers (Arc<RwLock<T>>, Arc<Mutex<T>>)");
    println!("   • Mixed container types in collections");
    println!("   • Composition with aggregator functions");
    println!("   • Full integration with derive macros!");
}
