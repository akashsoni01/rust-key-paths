// use keypaths_proc::Keypaths;
// use rust_keypaths::KeyPath;
// use std::sync::{Arc};

// #[derive(Keypaths, Clone, Debug)]
// struct SomeOtherStruct {
//     value: String,
//     count: u32,
// }

// #[derive(Keypaths, Clone, Debug)]
// struct SomeStruct {
//     field1: Arc<std::sync::RwLock<SomeOtherStruct>>,
//     field2: Arc<std::sync::Mutex<SomeOtherStruct>>,
// }

// fn main() {
//     println!("🔧 Arc<Sync> Derive Macro Support Example");
//     println!("=========================================");

//     // Create test data
//     let some_struct = SomeStruct {
//         field1: Arc::new(RwLock::new(SomeOtherStruct {
//             value: "Hello from RwLock".to_string(),
//             count: 42,
//         })),
//         field2: Arc::new(Mutex::new(SomeOtherStruct {
//             value: "Hello from Mutex".to_string(),
//             count: 24,
//         })),
//     };

//     println!("\n🎯 Testing Arc<RwLock<T>> Field Access");
//     println!("-------------------------------------");

//     // Test Arc<RwLock<T>> field access
//     let field1_path = SomeStruct::field1_r();
//     if let Some(field1_ref) = field1_path.get(&some_struct) {
//         println!("✅ Arc<RwLock<SomeOtherStruct>> field accessible: {:?}", field1_ref);
//     }

//     // Test Arc<Mutex<T>> field access
//     let field2_path = SomeStruct::field2_r();
//     if let Some(field2_ref) = field2_path.get(&some_struct) {
//         println!("✅ Arc<Mutex<SomeOtherStruct>> field accessible: {:?}", field2_ref);
//     }

//     println!("\n🎯 Testing with WithContainer Trait");
//     println!("----------------------------------");

//     // Test with WithContainer trait for no-clone access
//     let value_path = SomeOtherStruct::value_r();
//     let count_path = SomeOtherStruct::count_r();

//     // Access through Arc<RwLock<T>> - we need to get the field first, then use with_rwlock
//     if let Some(arc_rwlock_field) = field1_path.get(&some_struct) {
//         value_path.clone().with_rwlock(arc_rwlock_field, |value| {
//             println!("✅ Value from Arc<RwLock<SomeOtherStruct>>: {}", value);
//         });
//         count_path.clone().with_rwlock(arc_rwlock_field, |count| {
//             println!("✅ Count from Arc<RwLock<SomeOtherStruct>>: {}", count);
//         });
//     }

//     // Access through Arc<Mutex<T>> - we need to get the field first, then use with_mutex
//     if let Some(arc_mutex_field) = field2_path.get(&some_struct) {
//         value_path.with_arc_mutex_direct(arc_mutex_field, |value| {
//             println!("✅ Value from Arc<Mutex<SomeOtherStruct>>: {}", value);
//         });
//         count_path.with_arc_mutex_direct(arc_mutex_field, |count| {
//             println!("✅ Count from Arc<Mutex<SomeOtherStruct>>: {}", count);
//         });
//     }

//     println!("\n🎯 Testing Read-Only Composition");
//     println!("--------------------------------");

//     // Create a more complex nested structure for composition
//     #[derive(Keypaths, Clone, Debug)]
//     struct Company {
//         name: String,
//         departments: Vec<Department>,
//     }

//     #[derive(Keypaths, Clone, Debug)]
//     struct Department {
//         name: String,
//         manager: Arc<RwLock<Employee>>,
//         budget: u64,
//     }

//     #[derive(Keypaths, Clone, Debug)]
//     struct Employee {
//         name: String,
//         salary: u32,
//         contact: Arc<Mutex<Contact>>,
//     }

//     #[derive(Keypaths, Clone, Debug)]
//     struct Contact {
//         email: String,
//         phone: String,
//     }

//     // Create test data
//     let company = Company {
//         name: "TechCorp".to_string(),
//         departments: vec![
//             Department {
//                 name: "Engineering".to_string(),
//                 manager: Arc::new(RwLock::new(Employee {
//                     name: "Alice Johnson".to_string(),
//                     salary: 120000,
//                     contact: Arc::new(Mutex::new(Contact {
//                         email: "akash@techcorp.com".to_string(),
//                         phone: "+1-555-0123".to_string(),
//                     })),
//                 })),
//                 budget: 500000,
//             },
//             Department {
//                 name: "Marketing".to_string(),
//                 manager: Arc::new(RwLock::new(Employee {
//                     name: "Bob Smith".to_string(),
//                     salary: 95000,
//                     contact: Arc::new(Mutex::new(Contact {
//                         email: "bob@techcorp.com".to_string(),
//                         phone: "+1-555-0456".to_string(),
//                     })),
//                 })),
//                 budget: 200000,
//             },
//         ],
//     };

//     // Example 1: Simple composition - Company name
//     let company_name_path = Company::name_r();
//     if let Some(name) = company_name_path.get(&company) {
//         println!("✅ Company name: {}", name);
//     }

//     // Example 2: Composition through Vec - First department name
//     // We need to access the Vec element directly since KeyPaths doesn't have get_r
//     if let Some(first_dept) = company.departments.first() {
//         let dept_name_path = Department::name_r();
//         if let Some(dept_name) = dept_name_path.get(&first_dept) {
//             println!("✅ First department: {}", dept_name);
//         }
//     }

//     // Example 3: Deep composition - Manager name through Arc<RwLock>
//     // Get the Arc<RwLock<Employee>> first, then use with_rwlock
//     if let Some(first_dept) = company.departments.first() {
//         let manager_arc_path = Department::manager_r();
//         if let Some(manager_arc) = manager_arc_path.get(&first_dept) {
//             let employee_name_path = Employee::name_r();
//             employee_name_path.with_arc_rwlock_direct(manager_arc, |name| {
//                 println!("✅ Engineering manager: {}", name);
//             });
//         }
//     }

//     // Example 4: Even deeper composition - Contact email through Arc<Mutex>
//     if let Some(first_dept) = company.departments.first() {
//         let manager_arc_path = Department::manager_r();
//         if let Some(manager_arc) = manager_arc_path.get(&first_dept) {
//             // Get the contact Arc<Mutex<Contact>> from the employee
//             let contact_arc_path = Employee::contact_r();
//             let contact_arc = contact_arc_path.with_arc_rwlock_direct(manager_arc, |contact_arc| {
//                 contact_arc.clone()
//             });
//             if let Some(contact_arc) = contact_arc {
//                 let email_path = Contact::email_r();
//                 email_path.with_arc_mutex_direct(&*contact_arc, |email| {
//                     println!("✅ Engineering manager email: {}", email);
//                 });
//             }
//         }
//     }

//     // Example 5: Composition with multiple departments
//     println!("\n📊 All Department Information:");
//     for dept in &company.departments {
//         // Department name
//         let dept_name_path = Department::name_r();
//         if let Some(dept_name) = dept_name_path.get(&dept) {
//             print!("  {}: ", dept_name);
//         }

//         // Department budget
//         let budget_path = Department::budget_r();
//         if let Some(budget) = budget_path.get(&dept) {
//             print!("Budget ${} | ", budget);
//         }

//         // Manager name
//         let manager_arc_path = Department::manager_r();
//         if let Some(manager_arc) = manager_arc_path.get(&dept) {
//             let employee_name_path = Employee::name_r();
//             employee_name_path.with_arc_rwlock_direct(manager_arc, |name| {
//                 print!("Manager: {} | ", name);
//             });
//         }

//         // Manager salary
//         if let Some(manager_arc) = manager_arc_path.get(&dept) {
//             let salary_path = Employee::salary_r();
//             salary_path.with_arc_rwlock_direct(manager_arc, |salary| {
//                 print!("Salary: ${} | ", salary);
//             });
//         }

//         // Manager email
//         if let Some(manager_arc) = manager_arc_path.get(&dept) {
//             let contact_arc_path = Employee::contact_r();
//             let contact_arc = contact_arc_path.with_arc_rwlock_direct(manager_arc, |contact_arc| {
//                 contact_arc.clone()
//             });
//             if let Some(contact_arc) = contact_arc {
//                 let email_path = Contact::email_r();
//                 email_path.with_arc_mutex_direct(&*contact_arc, |email| {
//                     println!("Email: {}", email);
//                 });
//             }
//         }
//     }

//     println!("\n🎯 Testing with Aggregator Functions");
//     println!("-----------------------------------");

//     // Test with aggregator functions (requires parking_lot feature)
//     #[cfg(feature = "parking_lot")]
//     {
//         let value_arc_rwlock_path = value_path.for_arc_rwlock();
//         let count_arc_rwlock_path = count_path.for_arc_rwlock();
//         let value_arc_mutex_path = value_path.for_arc_mutex();
//         let count_arc_mutex_path = count_path.for_arc_mutex();

//         if let Some(value) = value_arc_rwlock_path.get_failable_owned(some_struct.field1.clone()) {
//             println!("✅ Value from Arc<RwLock<SomeOtherStruct>> (aggregator): {}", value);
//         }

//         if let Some(count) = count_arc_rwlock_path.get_failable_owned(some_struct.field1.clone()) {
//             println!("✅ Count from Arc<RwLock<SomeOtherStruct>> (aggregator): {}", count);
//         }

//         if let Some(value) = value_arc_mutex_path.get_failable_owned(some_struct.field2.clone()) {
//             println!("✅ Value from Arc<Mutex<SomeOtherStruct>> (aggregator): {}", value);
//         }

//         if let Some(count) = count_arc_mutex_path.get_failable_owned(some_struct.field2.clone()) {
//             println!("✅ Count from Arc<Mutex<SomeOtherStruct>> (aggregator): {}", count);
//         }
//     }

//     #[cfg(not(feature = "parking_lot"))]
//     {
//         println!("⚠️  Parking lot feature not enabled - aggregator functions not available");
//         println!("   Enable with: cargo run --example arc_sync_derive_example --features parking_lot");
//     }

//     println!("\n💡 Key Takeaways");
//     println!("================");
//     println!("1. Derive macro now supports Arc<RwLock<T>> and Arc<Mutex<T>> fields");
//     println!("2. Generated methods provide container-level access (field1_r(), field2_r())");
//     println!("3. Use WithContainer trait for no-clone access to inner values");
//     println!("4. Use aggregator functions (with parking_lot feature) for clone-based access");
//     println!("5. Arc<Mutex<T>> and Arc<RwLock<T>> don't support writable access (Arc is immutable)");
//     println!("6. Direct access to inner types requires proper lock handling");
//     println!("7. Composition works by chaining keypaths with .then() and using with_* methods");
//     println!("8. For deep nesting, access each level step by step using get_ref() and with_* methods");
// }

fn main() {}