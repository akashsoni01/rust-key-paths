use key_paths_derive::Keypath;

#[derive(Debug, Keypath)]
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

    println!("=== Smart Keypath Access ===");
    
    // Basic types - readable keypath
    println!("Name: {:?}", Person::name().get(&person));
    println!("Age: {:?}", Person::age().get(&person));

    // Option<T> - failable readable keypath to inner type
    if let Some(email) = Person::email().get(&person) {
        println!("Email: {}", email);
    }

    // Vec<T> - failable readable keypath to first element
    if let Some(hobby) = Person::hobbies().get(&person) {
        println!("First hobby: {}", hobby);
    }

    // HashMap<K,V> - readable keypath to container
    if let Some(scores) = Person::scores().get(&person) {
        println!("Scores: {:?}", scores);
    }

    println!("\n=== Keypath Types ===");
    println!("name() returns: KeyPaths<Person, String> (readable)");
    println!("age() returns: KeyPaths<Person, u32> (readable)");
    println!("email() returns: KeyPaths<Person, String> (failable readable)");
    println!("hobbies() returns: KeyPaths<Person, String> (failable readable)");
    println!("scores() returns: KeyPaths<Person, HashMap<String, u32>> (readable)");
}
