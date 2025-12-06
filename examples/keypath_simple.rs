use keypaths_proc::Keypaths;

#[derive(Debug, Keypaths)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    hobbies: Vec<String>,
    scores: std::collections::HashMap<String, u32>,
}

fn main() {
    let person = Person {
        name: "John Doe".to_string(),
        age: 25,
        email: Some("john@example.com".to_string()),
        hobbies: vec!["reading".to_string(), "coding".to_string()],
        scores: {
            let mut map = std::collections::HashMap::new();
            map.insert("math".to_string(), 95);
            map.insert("science".to_string(), 88);
            map
        },
    };

    println!("=== Smart Keypaths Access ===");
    
    // Basic types - readable keypath
    println!("Name: {:?}", Person::name_r().get(&person));
    println!("Age: {:?}", Person::age_r().get(&person));

    // Option<T> - failable readable keypath to inner type
    if let Some(email) = Person::email_fr().get(&person) {
        println!("Email: {}", email);
    }

    // Vec<T> - failable readable keypath to first element
    if let Some(hobby) = Person::hobbies_fr().get(&person) {
        println!("First hobby: {}", hobby);
    }

    // HashMap<K,V> - readable keypath to container
    if let Some(scores) = Person::scores_r().get(&person) {
        println!("Scores: {:?}", scores);
    }

    println!("\n=== Keypaths Types ===");
    println!("name() returns: KeyPath<Person, String, impl for<\'r> Fn(&\'r Person) -> &\'r String> (readable)");
    println!("age() returns: KeyPath<Person, u32, impl for<\'r> Fn(&\'r Person) -> &\'r u32> (readable)");
    println!("email() returns: KeyPath<Person, String, impl for<\'r> Fn(&\'r Person) -> &\'r String> (failable readable)");
    println!("hobbies() returns: KeyPath<Person, String, impl for<\'r> Fn(&\'r Person) -> &\'r String> (failable readable)");
    println!("scores() returns: KeyPath<Person, HashMap<String, u32, impl for<\'r> Fn(&\'r Person) -> &\'r HashMap<String, u32>> (readable)");
}
