// Demonstrates binding UI fields to data model properties using keypaths
// This example shows how to:
// 1. Create a generic form binding system without hardcoded access patterns
// 2. Support multiple field types (String, bool, numbers)
// 3. Implement two-way data binding (read and write)
// 4. Build reusable form validators
// 5. Track field-level changes
// cargo run --example form_binding

use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Debug, Clone, Keypaths)]
struct UserProfile {
    name: String,
    email: String,
    age: u32,
    settings: UserSettings,
}

#[derive(Debug, Clone, Keypaths)]
struct UserSettings {
    notifications_enabled: bool,
    theme: String,
    font_size: u32,
}

// Generic form field that binds to any field type
struct FormField<T: 'static, F: 'static> {
    read_path: KeyPaths<T, F>,
    write_path: KeyPaths<T, F>,
    label: &'static str,
    field_name: &'static str,
    validator: fn(&F) -> Result<(), String>,
}

impl<T, F> FormField<T, F>
where
    F: Clone + std::fmt::Display,
{
    fn new(
        read_path: KeyPaths<T, F>,
        write_path: KeyPaths<T, F>,
        label: &'static str,
        field_name: &'static str,
        validator: fn(&F) -> Result<(), String>,
    ) -> Self {
        Self {
            read_path,
            write_path,
            label,
            field_name,
            validator,
        }
    }

    // Read current value from the model
    fn read(&self, model: &T) -> Option<F> {
        self.read_path.get(model).cloned()
    }

    // Write new value to the model
    fn write(&self, model: &mut T, value: F) -> Result<(), String> {
        // Validate first
        (self.validator)(&value)?;

        // Then write
        if let Some(target) = self.write_path.get_mut(model) {
            *target = value;
            Ok(())
        } else {
            Err(format!("Failed to write to field '{}'", self.field_name))
        }
    }

    // Validate without writing
    fn validate(&self, value: &F) -> Result<(), String> {
        (self.validator)(value)
    }
}

// Form binding system that manages multiple fields
struct FormBinding<T: 'static> {
    string_fields: Vec<FormField<T, String>>,
    bool_fields: Vec<FormField<T, bool>>,
    u32_fields: Vec<FormField<T, u32>>,
}

impl<T> FormBinding<T> {
    fn new() -> Self {
        Self {
            string_fields: Vec::new(),
            bool_fields: Vec::new(),
            u32_fields: Vec::new(),
        }
    }

    fn add_string_field(&mut self, field: FormField<T, String>) {
        self.string_fields.push(field);
    }

    fn add_bool_field(&mut self, field: FormField<T, bool>) {
        self.bool_fields.push(field);
    }

    fn add_u32_field(&mut self, field: FormField<T, u32>) {
        self.u32_fields.push(field);
    }

    // Display current form state
    fn display(&self, model: &T)
    where
        T: std::fmt::Debug,
    {
        println!("Current Form State:");
        println!("─────────────────────────────────────");

        for field in &self.string_fields {
            if let Some(value) = field.read(model) {
                println!("  {}: '{}'", field.label, value);
            }
        }

        for field in &self.u32_fields {
            if let Some(value) = field.read(model) {
                println!("  {}: {}", field.label, value);
            }
        }

        for field in &self.bool_fields {
            if let Some(value) = field.read(model) {
                println!("  {}: {}", field.label, if value { "Yes" } else { "No" });
            }
        }

        println!("─────────────────────────────────────");
    }

    // Update a string field by name
    fn update_string(
        &self,
        model: &mut T,
        field_name: &str,
        value: String,
    ) -> Result<(), String> {
        for field in &self.string_fields {
            if field.field_name == field_name {
                return field.write(model, value);
            }
        }
        Err(format!("Field '{}' not found", field_name))
    }

    // Update a bool field by name
    fn update_bool(&self, model: &mut T, field_name: &str, value: bool) -> Result<(), String> {
        for field in &self.bool_fields {
            if field.field_name == field_name {
                return field.write(model, value);
            }
        }
        Err(format!("Field '{}' not found", field_name))
    }

    // Update a u32 field by name
    fn update_u32(&self, model: &mut T, field_name: &str, value: u32) -> Result<(), String> {
        for field in &self.u32_fields {
            if field.field_name == field_name {
                return field.write(model, value);
            }
        }
        Err(format!("Field '{}' not found", field_name))
    }

    // Validate all fields
    fn validate_all(&self, model: &T) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for field in &self.string_fields {
            if let Some(value) = field.read(model) {
                if let Err(e) = field.validate(&value) {
                    errors.push(format!("{}: {}", field.label, e));
                }
            }
        }

        for field in &self.u32_fields {
            if let Some(value) = field.read(model) {
                if let Err(e) = field.validate(&value) {
                    errors.push(format!("{}: {}", field.label, e));
                }
            }
        }

        for field in &self.bool_fields {
            if let Some(value) = field.read(model) {
                if let Err(e) = field.validate(&value) {
                    errors.push(format!("{}: {}", field.label, e));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Create the form binding for UserProfile
fn create_user_profile_form() -> FormBinding<UserProfile> {
    let mut form = FormBinding::new();

    // String field: name
    form.add_string_field(FormField::new(
        UserProfile::name_r(),
        UserProfile::name_w(),
        "Full Name",
        "name",
        |s| {
            if s.len() >= 2 {
                Ok(())
            } else {
                Err("Name must be at least 2 characters".into())
            }
        },
    ));

    // String field: email
    form.add_string_field(FormField::new(
        UserProfile::email_r(),
        UserProfile::email_w(),
        "Email Address",
        "email",
        |s| {
            if s.contains('@') && s.contains('.') {
                Ok(())
            } else {
                Err("Invalid email format".into())
            }
        },
    ));

    // Number field: age
    form.add_u32_field(FormField::new(
        UserProfile::age_r(),
        UserProfile::age_w(),
        "Age",
        "age",
        |&age| {
            if age >= 13 && age <= 120 {
                Ok(())
            } else {
                Err("Age must be between 13 and 120".into())
            }
        },
    ));

    // String field: theme (nested)
    form.add_string_field(FormField::new(
        UserProfile::settings_r().then(UserSettings::theme_r()),
        UserProfile::settings_w().then(UserSettings::theme_w()),
        "Theme",
        "theme",
        |s| {
            if ["light", "dark", "auto"].contains(&s.as_str()) {
                Ok(())
            } else {
                Err("Theme must be 'light', 'dark', or 'auto'".into())
            }
        },
    ));

    // Number field: font_size (nested)
    form.add_u32_field(FormField::new(
        UserProfile::settings_r().then(UserSettings::font_size_r()),
        UserProfile::settings_w().then(UserSettings::font_size_w()),
        "Font Size",
        "font_size",
        |&size| {
            if (10..=24).contains(&size) {
                Ok(())
            } else {
                Err("Font size must be between 10 and 24".into())
            }
        },
    ));

    // Bool field: notifications (nested)
    form.add_bool_field(FormField::new(
        UserProfile::settings_r()
            .then(UserSettings::notifications_enabled_r()),
        UserProfile::settings_w()
            .then(UserSettings::notifications_enabled_w()),
        "Notifications",
        "notifications",
        |_| Ok(()), // No validation needed for bool
    ));

    form
}

fn main() {
    println!("=== Form Binding Demo ===\n");

    // Create initial user profile
    let mut profile = UserProfile {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 28,
        settings: UserSettings {
            notifications_enabled: true,
            theme: "dark".to_string(),
            font_size: 14,
        },
    };

    // Create form binding
    let form = create_user_profile_form();

    // Display initial state
    println!("=== Initial State ===");
    form.display(&profile);

    // Validate initial state
    println!("\n=== Validating Initial State ===");
    match form.validate_all(&profile) {
        Ok(_) => println!("✓ All fields valid"),
        Err(errors) => {
            println!("✗ Validation errors:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Update various fields through the binding system
    println!("\n=== Updating Fields ===");

    // Update name
    match form.update_string(&mut profile, "name", "Alice Johnson".to_string()) {
        Ok(_) => println!("✓ Updated name successfully"),
        Err(e) => println!("✗ Failed to update name: {}", e),
    }

    // Update email
    match form.update_string(&mut profile, "email", "alice.johnson@example.com".to_string()) {
        Ok(_) => println!("✓ Updated email successfully"),
        Err(e) => println!("✗ Failed to update email: {}", e),
    }

    // Update age
    match form.update_u32(&mut profile, "age", 29) {
        Ok(_) => println!("✓ Updated age successfully"),
        Err(e) => println!("✗ Failed to update age: {}", e),
    }

    // Update theme (nested field)
    match form.update_string(&mut profile, "theme", "light".to_string()) {
        Ok(_) => println!("✓ Updated theme successfully"),
        Err(e) => println!("✗ Failed to update theme: {}", e),
    }

    // Update font size (nested field)
    match form.update_u32(&mut profile, "font_size", 16) {
        Ok(_) => println!("✓ Updated font size successfully"),
        Err(e) => println!("✗ Failed to update font size: {}", e),
    }

    // Update notifications (nested field)
    match form.update_bool(&mut profile, "notifications", false) {
        Ok(_) => println!("✓ Updated notifications successfully"),
        Err(e) => println!("✗ Failed to update notifications: {}", e),
    }

    // Display updated state
    println!("\n=== Updated State ===");
    form.display(&profile);

    // Try invalid updates
    println!("\n=== Testing Validation ===");

    // Try to set invalid name
    match form.update_string(&mut profile, "name", "A".to_string()) {
        Ok(_) => println!("✓ Updated name successfully"),
        Err(e) => println!("✗ Failed to update name: {}", e),
    }

    // Try to set invalid email
    match form.update_string(&mut profile, "email", "not-an-email".to_string()) {
        Ok(_) => println!("✓ Updated email successfully"),
        Err(e) => println!("✗ Failed to update email: {}", e),
    }

    // Try to set invalid age
    match form.update_u32(&mut profile, "age", 5) {
        Ok(_) => println!("✓ Updated age successfully"),
        Err(e) => println!("✗ Failed to update age: {}", e),
    }

    // Try to set invalid theme
    match form.update_string(&mut profile, "theme", "rainbow".to_string()) {
        Ok(_) => println!("✓ Updated theme successfully"),
        Err(e) => println!("✗ Failed to update theme: {}", e),
    }

    // Try to set invalid font size
    match form.update_u32(&mut profile, "font_size", 50) {
        Ok(_) => println!("✓ Updated font size successfully"),
        Err(e) => println!("✗ Failed to update font size: {}", e),
    }

    // Final state (should be unchanged due to validation errors)
    println!("\n=== Final State (After Invalid Updates) ===");
    form.display(&profile);

    // Demonstrate two-way binding by reading values
    println!("\n=== Two-Way Binding Demo ===");
    println!("Reading values through form binding:");

    for field in &form.string_fields {
        if let Some(value) = field.read(&profile) {
            println!("  {}: '{}'", field.label, value);
        }
    }

    for field in &form.u32_fields {
        if let Some(value) = field.read(&profile) {
            println!("  {}: {}", field.label, value);
        }
    }

    for field in &form.bool_fields {
        if let Some(value) = field.read(&profile) {
            println!("  {}: {}", field.label, value);
        }
    }

    println!("\n✓ Form binding demo complete!");
}

