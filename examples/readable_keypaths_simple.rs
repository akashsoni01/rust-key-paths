use key_paths_derive::ReadableKeypaths;

#[derive(Debug, ReadableKeypaths)]
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

    // Basic readable keypaths
    println!("Name: {:?}", Person::name_r().get(&person));
    println!("Age: {:?}", Person::age_r().get(&person));

    // Failable readable keypaths
    if let Some(email) = Person::email_fr().get(&person) {
        println!("Email: {}", email);
    }

    if let Some(hobby) = Person::hobbies_fr().get(&person) {
        println!("First hobby: {}", hobby);
    }

    if let Some(score) = Person::scores_fr("math".to_string()).get(&person) {
        println!("Math score: {}", score);
    }

    // Indexed access for Vec
    if let Some(hobby) = Person::hobbies_fr_at(1).get(&person) {
        println!("Second hobby: {}", hobby);
    }
}
