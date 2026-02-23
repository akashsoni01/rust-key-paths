//! Example: parallel collection operations with [key-paths-iter] and the [Kp] derive macro.
//!
//! Uses `#[derive(Kp)]` to get a keypath to a `Vec<Item>` field, then runs Rayon-powered
//! parallel map, filter, count, sort, and for_each over the collection.

use key_paths_derive::Kp;
use key_paths_iter::query_par::ParallelCollectionKeyPath;

#[derive(Kp, Debug)]
struct Company {
    name: String,
    employees: Vec<Employee>,
}

#[derive(Debug, Clone)]
struct Employee {
    name: String,
    role: String,
    salary: u32,
    active: bool,
}

fn main() {
    let company = Company {
        name: "Acme Corp".to_string(),
        employees: (0..5000000)
            .map(|i| Employee {
                name: format!("Employee_{}", i),
                role: if i % 3 == 0 {
                    "Engineer"
                } else if i % 3 == 1 {
                    "Designer"
                } else {
                    "Manager"
                }
                .to_string(),
                salary: 50_000 + (i % 100) * 1000,
                active: i % 5 != 0,
            })
            .collect(),
    };

    // Keypath from derive: KpType<'static, Company, Vec<Employee>>
    let employees_kp = Company::employees();

    println!("=== key-paths-iter parallel (Kp derive) ===\n");
    println!("Company: {} ({} employees)\n", company.name, company.employees.len());

    // Parallel map: extract salaries
    let salaries = employees_kp.par_map(&company, |e| e.salary);
    println!("Collected {} salaries (par_map)", salaries.len());

    // Parallel filter: active engineers
    let active_engineers = employees_kp.par_filter(&company, |e| e.active && e.role == "Engineer");
    println!("Active engineers: {} (par_filter)", active_engineers.len());

    // Parallel count by predicate
    let high_earners = employees_kp.par_count_by(&company, |e| e.salary >= 100_000);
    println!("High earners (salary >= 100k): {} (par_count_by)", high_earners);

    // Parallel any / all
    let has_manager = employees_kp.par_any(&company, |e| e.role == "Manager");
    let all_have_salary = employees_kp.par_all(&company, |e| e.salary > 0);
    println!("Has manager: {}, all have salary: {} (par_any, par_all)", has_manager, all_have_salary);

    // Parallel min/max by key
    let min_sal = employees_kp.par_min_by_key(&company, |e| e.salary).map(|e| e.salary);
    let max_sal = employees_kp.par_max_by_key(&company, |e| e.salary).map(|e| e.salary);
    println!("Min salary: {:?}, max salary: {:?} (par_min_by_key, par_max_by_key)", min_sal, max_sal);

    // Parallel partition: active vs inactive
    let (active, inactive) = employees_kp.par_partition(&company, |e| e.active);
    println!("Active: {}, inactive: {} (par_partition)", active.len(), inactive.len());

    // Parallel group by role
    let by_role = employees_kp.par_group_by(&company, |e| e.role.clone());
    println!("Grouped by role: {} roles (par_group_by)", by_role.len());
    for (role, group) in &by_role {
        println!("  {}: {} employees", role, group.len());
    }

    // Parallel sort by key (returns owned Vec<Employee>)
    let sorted_by_salary = employees_kp.par_sort_by_key(&company, |e| e.salary);
    println!("\nSorted by salary (par_sort_by_key): {} items", sorted_by_salary.len());
    if let (Some(first), Some(last)) = (sorted_by_salary.first(), sorted_by_salary.last()) {
        println!("  First: {} ({}), last: {} ({})", first.name, first.salary, last.name, last.salary);
    }

    println!("\nDone.");
}
