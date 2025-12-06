use std::marker::PhantomData;

#[derive(Clone)]
pub struct KeyPath<Root, Value, F>
where
    F: Fn(&Root) -> &Value,
{
    getter: F,
    _root: PhantomData<Root>,
    _value: PhantomData<Value>,
}

impl<Root, Value, F> KeyPath<Root, Value, F>
where
    F: Fn(&Root) -> &Value,
{
    pub fn new(getter: F) -> Self {
        Self {
            getter,
            _root: PhantomData,
            _value: PhantomData,
        }
    }

    pub fn get<'a>(&self, root: &'a Root) -> &'a Value {
        (self.getter)(root)
    }

    // Swift-like operator
    pub fn appending<SubValue, G>(
        &self,
        next: KeyPath<Value, SubValue, G>,
    ) -> KeyPath<Root, SubValue, impl Fn(&Root) -> &SubValue>
    where
        G: Fn(&Value) -> &SubValue,
        F: Clone,
        G: Clone,
        Value: 'static,
    {
        let first = self.getter.clone();
        let second = next.getter.clone();

        KeyPath::new(move |root| (second)((first)(root)))
    }
}

#[derive(Clone)]
pub struct OptionalKeyPath<Root, Value, F>
where
    F: Fn(&Root) -> Option<&Value>,
{
    getter: F,
    _root: PhantomData<Root>,
    _value: PhantomData<Value>,
}

impl<Root, Value, F> OptionalKeyPath<Root, Value, F>
where
    F: Fn(&Root) -> Option<&Value>,
{
    pub fn new(getter: F) -> Self {
        Self {
            getter,
            _root: PhantomData,
            _value: PhantomData,
        }
    }

    pub fn get<'a>(&self, root: &'a Root) -> Option<&'a Value> {
        (self.getter)(root)
    }

    // Swift-like optional chaining
    pub fn appending<SubValue, G>(
        &self,
        next: KeyPath<Value, SubValue, G>,
    ) -> OptionalKeyPath<Root, SubValue, impl Fn(&Root) -> Option<&SubValue>>
    where
        G: Fn(&Value) -> &SubValue,
        F: Clone,
        G: Clone,
        Value: 'static,
    {
        let first = self.getter.clone();
        let second = next.getter.clone();

        OptionalKeyPath::new(move |root| first(root).map(|value| second(value)))
    }
}

fn optional_keypaths() {
    // Helper for optional chaining
    let user_metadata = KeyPath::new(|user: &User| &user.metadata);

    // To handle Option<T>, we need a different approach
    // Let's create a special keypath that works with Option

    // Example usage
    let user_metadata = OptionalKeyPath::new(|user: &User| user.metadata.as_ref());
    let metadata_tags = KeyPath::new(|metadata: &Metadata| &metadata.tags);

    let user_metadata_tags = user_metadata.appending(metadata_tags);

    let user_with_metadata = create_sample_user();
    let user_without_metadata = User {
        metadata: None,
        ..create_sample_user()
    };

    println!(
        "User with metadata tags: {:?}",
        user_metadata_tags.get(&user_with_metadata)
    );
    println!(
        "User without metadata tags: {:?}",
        user_metadata_tags.get(&user_without_metadata)
    ); // Returns None
}

// Complex nested types similar to Swift
#[derive(Debug)]
struct User {
    id: u64,
    name: String,
    address: Address,
    preferences: Preferences,
    metadata: Option<Metadata>,
}

#[derive(Debug)]
struct Address {
    street: String,
    city: String,
    country: Country,
    coordinates: Coordinates,
}

#[derive(Debug)]
struct Country {
    code: String,
    name: String,
    currency: Currency,
}

#[derive(Debug)]
struct Currency {
    code: String,
    symbol: char,
}

#[derive(Debug)]
struct Coordinates {
    lat: f64,
    lon: f64,
}

#[derive(Debug)]
struct Preferences {
    theme: String,
    language: String,
    notifications: NotificationSettings,
}

#[derive(Debug)]
struct NotificationSettings {
    email: bool,
    push: bool,
    frequency: NotificationFrequency,
}

#[derive(Debug)]
enum NotificationFrequency {
    Immediate,
    Daily,
    Weekly,
    Never,
}

#[derive(Debug)]
struct Metadata {
    created_at: String,
    updated_at: String,
    tags: Vec<String>,
}
fn create_sample_user() -> User {
    User {
        id: 42,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "San Francisco".to_string(),
            country: Country {
                code: "US".to_string(),
                name: "United States".to_string(),
                currency: Currency {
                    code: "USD".to_string(),
                    symbol: '$',
                },
            },
            coordinates: Coordinates {
                lat: 37.7749,
                lon: -122.4194,
            },
        },
        preferences: Preferences {
            theme: "dark".to_string(),
            language: "en".to_string(),
            notifications: NotificationSettings {
                email: true,
                push: false,
                frequency: NotificationFrequency::Daily,
            },
        },
        metadata: Some(Metadata {
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-15".to_string(),
            tags: vec!["premium".to_string(), "active".to_string()],
        }),
    }
}

fn basic_keypaths() {
    let user = User {
        id: 42,
        name: "Alice".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "San Francisco".to_string(),
            country: Country {
                code: "US".to_string(),
                name: "United States".to_string(),
                currency: Currency {
                    code: "USD".to_string(),
                    symbol: '$',
                },
            },
            coordinates: Coordinates {
                lat: 37.7749,
                lon: -122.4194,
            },
        },
        preferences: Preferences {
            theme: "dark".to_string(),
            language: "en".to_string(),
            notifications: NotificationSettings {
                email: true,
                push: false,
                frequency: NotificationFrequency::Daily,
            },
        },
        metadata: Some(Metadata {
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-15".to_string(),
            tags: vec!["premium".to_string(), "active".to_string()],
        }),
    };

    // Create keypaths
    let user_name = KeyPath::new(|user: &User| &user.name);
    let user_id = KeyPath::new(|user: &User| &user.id);
    let user_address = KeyPath::new(|user: &User| &user.address);

    // Use them
    println!("User name: {}", user_name.get(&user));
    println!("User ID: {}", user_id.get(&user));
    println!("Address: {:?}", user_address.get(&user));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        basic_keypaths();
        optional_keypaths();
    }
}
