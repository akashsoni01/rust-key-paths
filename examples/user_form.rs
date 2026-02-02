// // Demonstrates using rust-key-paths for generic form processing
// // This example shows how to:
// // 1. Define form fields using keypaths for type-safe access
// // 2. Build composable validators for nested fields
// // 3. Process forms generically across any data model
// // 4. Use keypaths for direct nested field access
// // cargo run --example user_form

// use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
// use keypaths_proc::Kp;

// #[derive(Debug, Clone, Kp)]
// #[All]
// struct UserProfile {
//     name: String,
//     email: String,
//     settings: UserSettings,
// }

// #[derive(Debug, Clone, Kp)]
// #[All]
// struct UserSettings {
//     notifications_enabled: bool,
//     theme: String,
// }

// // Form field definition using keypaths
// struct FormField<T: 'static, F: 'static> {
//     path: KeyPath<T, F, impl for<'r> Fn(&'r T) -> &'r F>,
//     label: &'static str,
//     validator: fn(&F) -> Result<(), String>,
// }

// // Define form schema once
// fn create_profile_form() -> Vec<FormField<UserProfile, String>> {
//     vec![
//         FormField {
//             path: UserProfile::name_w(),
//             label: "Full Name",
//             validator: |s| {
//                 if s.len() > 2 {
//                     Ok(())
//                 } else {
//                     Err("Name too short".into())
//                 }
//             },
//         },
//         FormField {
//             path: UserProfile::email_w(),
//             label: "Email Address",
//             validator: |s| {
//                 if s.contains('@') {
//                     Ok(())
//                 } else {
//                     Err("Invalid email".into())
//                 }
//             },
//         },
//         FormField {
//             path: UserProfile::settings_w().to_optional().then(UserSettings::theme_w()),
//             label: "Theme",
//             validator: |_s| Ok(()),
//         },
//     ]
// }

// // Generic form processor works with ANY model
// fn process_form<T, F>(
//     data: &mut T,
//     fields: &[FormField<T, F>],
//     inputs: Vec<F>,
// ) -> Result<(), Vec<String>>
// where
//     F: Clone,
// {
//     let mut errors = Vec::new();

//     for (field, input) in fields.iter().zip(inputs) {
//         match (field.validator)(&input) {
//             Ok(_) => {
//                 if let Some(target) = field.path.get_mut(data) {
//                     *target = input;
//                 }
//             }
//             Err(e) => errors.push(format!("{}: {}", field.label, e)),
//         }
//     }

//     if errors.is_empty() {
//         Ok(())
//     } else {
//         Err(errors)
//     }
// }

// // Usage
// fn main() {
//     let mut profile = UserProfile {
//         name: "".to_string(),
//         email: "".to_string(),
//         settings: UserSettings {
//             notifications_enabled: true,
//             theme: "light".to_string(),
//         },
//     };

//     println!("Initial profile: {:#?}", profile);

//     let form = create_profile_form();
//     let user_inputs = vec![
//         "John Doe".to_string(),
//         "john@example.com".to_string(),
//         "dark".to_string(),
//     ];

//     match process_form(&mut profile, &form, user_inputs) {
//         Ok(_) => println!("\n✓ Form processed successfully!"),
//         Err(errors) => {
//             println!("\n✗ Form validation errors:");
//             for error in errors {
//                 println!("  - {}", error);
//             }
//         }
//     }

//     println!("\nUpdated profile: {:#?}", profile);

//     // Demonstrate validation errors
//     println!("\n--- Testing validation errors ---");
//     let mut profile2 = UserProfile {
//         name: "".to_string(),
//         email: "".to_string(),
//         settings: UserSettings {
//             notifications_enabled: true,
//             theme: "light".to_string(),
//         },
//     };

//     let invalid_inputs = vec![
//         "Jo".to_string(),               // Too short
//         "not-an-email".to_string(),     // No @ symbol
//         "dark".to_string(),
//     ];

//     match process_form(&mut profile2, &form, invalid_inputs) {
//         Ok(_) => println!("Form processed successfully!"),
//         Err(errors) => {
//             println!("Form validation errors:");
//             for error in errors {
//                 println!("  - {}", error);
//             }
//         }
//     }

//     println!("\nProfile2 (with errors): {:#?}", profile2);

//     // Demonstrate the power of keypaths: accessing nested fields directly
//     println!("\n--- Direct keypath access demonstration ---");
//     let theme_path = UserProfile::settings_w().to_optional().then(UserSettings::theme_w());

//     let theme = theme_path.get_mut(&mut profile);
//     {
//         println!("Current theme: {}", theme);
//         *theme = "midnight".to_string();
//         println!("Changed theme to: {}", theme);
//     }

//     // Access boolean field through composed keypath
//     let notifications_path = UserProfile::settings_w().to_optional().then(UserSettings::notifications_enabled_w());
//     let enabled = notifications_path.get_mut(&mut profile);
//     {
//         println!("Notifications enabled: {}", enabled);
//         *enabled = false;
//         println!("Toggled notifications to: {}", enabled);
//     }

//     println!("\nFinal profile: {:#?}", profile);
// }

fn main() {}
