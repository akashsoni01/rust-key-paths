use keypaths_proc::WritableKeypaths;

#[derive(Debug, WritableKeypaths)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
    hobbies: Vec<String>,
    scores: std::collections::HashMap<String, u32>,
}

fn main() {
    let mut person = Person {
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

    println!("=== Initial State ===");
    println!("Name: {}", person.name);
    println!("Age: {}", person.age);
    println!("Email: {:?}", person.email);
    println!("Hobbies: {:?}", person.hobbies);
    println!("Scores: {:?}", person.scores);

    // Basic writable keypaths
    if let Some(name_ref) = Person::name_w().get_mut(&mut person) {
        *name_ref = "John Smith".to_string();
        println!("\nUpdated name to: {}", name_ref);
    }

    if let Some(age_ref) = Person::age_w().get_mut(&mut person) {
        *age_ref = 26;
        println!("Updated age to: {}", age_ref);
    }

    // Failable writable keypaths
    if let Some(email_ref) = Person::email_fw().get_mut(&mut person) {
        *email_ref = "john.smith@example.com".to_string();
        println!("Updated email to: {}", email_ref);
    }

    if let Some(hobby_ref) = Person::hobbies_fw().get_mut(&mut person) {
        *hobby_ref = "gaming".to_string();
        println!("Updated first hobby to: {}", hobby_ref);
    }

    if let Some(score_ref) = Person::scores_fw("math".to_string()).get_mut(&mut person) {
        *score_ref = 98;
        println!("Updated math score to: {}", score_ref);
    }

    // Indexed access for Vec
    if let Some(hobby_ref) = Person::hobbies_fw_at(1).get_mut(&mut person) {
        *hobby_ref = "swimming".to_string();
        println!("Updated second hobby to: {}", hobby_ref);
    }

    println!("\n=== Final State ===");
    println!("Name: {}", person.name);
    println!("Age: {}", person.age);
    println!("Email: {:?}", person.email);
    println!("Hobbies: {:?}", person.hobbies);
    println!("Scores: {:?}", person.scores);
}
