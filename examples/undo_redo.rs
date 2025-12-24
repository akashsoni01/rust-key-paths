// Demonstrates implementing undo/redo functionality using keypaths
// This example shows how to:
// 1. Track changes to deeply nested data structures
// 2. Implement command pattern for undo/redo
// 3. Handle multiple field types in undo/redo
// 4. Support redo after undo operations
// 5. Display history of changes
// cargo run --example undo_redo

use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::Keypaths;
use std::rc::Rc;

#[derive(Debug, Clone, Keypaths)]
#[All]
struct Document {
    title: String,
    content: String,
    metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Keypaths)]
#[All]
struct DocumentMetadata {
    author: String,
    tags: Vec<String>,
    revision: u32,
}

// Generic command pattern using keypaths
struct ChangeCommand<T: 'static, F: Clone + 'static> {
    path: Box<dyn Fn(&mut T) -> Option<&mut F>>,
    old_value: F,
    new_value: F,
    description: String,
}

impl<T, F: Clone> ChangeCommand<T, F> {
    fn execute(&self, target: &mut T) {
        if let Some(field) = (self.path)(target) {
            *field = self.new_value.clone();
        }
    }

    fn undo(&self, target: &mut T) {
        if let Some(field) = (self.path)(target) {
            *field = self.old_value.clone();
        }
    }
}

// Trait for commands that can be executed and undone
trait Command<T> {
    fn execute(&self, target: &mut T);
    fn undo(&self, target: &mut T);
    fn description(&self) -> &str;
}

impl<T, F: Clone + 'static> Command<T> for ChangeCommand<T, F> {
    fn execute(&self, target: &mut T) {
        ChangeCommand::execute(self, target)
    }

    fn undo(&self, target: &mut T) {
        ChangeCommand::undo(self, target)
    }

    fn description(&self) -> &str {
        &self.description
    }
}

// Undo/Redo stack manager
struct UndoStack<T> {
    commands: Vec<Box<dyn Command<T>>>,
    current: usize, // Points to the next position to add a command
}

impl<T> UndoStack<T> {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
            current: 0,
        }
    }

    // Execute a new command and add it to the stack
    fn execute(&mut self, target: &mut T, command: Box<dyn Command<T>>) {
        // Execute the command
        command.execute(target);

        // If we're not at the end, truncate the redo history
        if self.current < self.commands.len() {
            self.commands.truncate(self.current);
        }

        // Add the command to the stack
        self.commands.push(command);
        self.current += 1;
    }

    // Undo the last command
    fn undo(&mut self, target: &mut T) -> Result<String, String> {
        if self.current == 0 {
            return Err("Nothing to undo".into());
        }

        self.current -= 1;
        let command = &self.commands[self.current];
        let desc = command.description().to_string();
        command.undo(target);
        Ok(desc)
    }

    // Redo the last undone command
    fn redo(&mut self, target: &mut T) -> Result<String, String> {
        if self.current >= self.commands.len() {
            return Err("Nothing to redo".into());
        }

        let command = &self.commands[self.current];
        let desc = command.description().to_string();
        command.execute(target);
        self.current += 1;
        Ok(desc)
    }

    // Check if undo is available
    fn can_undo(&self) -> bool {
        self.current > 0
    }

    // Check if redo is available
    fn can_redo(&self) -> bool {
        self.current < self.commands.len()
    }

    // Get the history of commands
    fn history(&self) -> Vec<String> {
        self.commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let marker = if i < self.current {
                    "✓"
                } else {
                    " "
                };
                format!("{} {}", marker, cmd.description())
            })
            .collect()
    }
}

// Helper to create change commands for strings
fn make_string_change<T: 'static, P, R>(
    target: &T,
    path: WritableOptionalKeyPath<T, String, P>,
    read_path: OptionalKeyPath<T, String, R>,
    new_value: String,
    description: String,
) -> Box<dyn Command<T>>
where
    P: for<'r> Fn(&'r mut T) -> Option<&'r mut String> + 'static,
    R: for<'r> Fn(&'r T) -> Option<&'r String> + 'static,
{
    let old_value = read_path.get(target).map(|s| s.clone()).unwrap_or_default();
    let path_rc = Rc::new(path);
    let path_clone = path_rc.clone();
    let path_box: Box<dyn Fn(&mut T) -> Option<&mut String>> = Box::new(move |t: &mut T| {
        path_clone.get_mut(t)
    });
    Box::new(ChangeCommand {
        path: path_box,
        old_value,
        new_value,
        description,
    })
}

// Helper to create change commands for u32
fn make_u32_change<T: 'static, P, R>(
    target: &T,
    path: WritableOptionalKeyPath<T, u32, P>,
    read_path: OptionalKeyPath<T, u32, R>,
    new_value: u32,
    description: String,
) -> Box<dyn Command<T>>
where
    P: for<'r> Fn(&'r mut T) -> Option<&'r mut u32> + 'static,
    R: for<'r> Fn(&'r T) -> Option<&'r u32> + 'static,
{
    let old_value = read_path.get(target).copied().unwrap_or_default();
    let path_rc = Rc::new(path);
    let path_clone = path_rc.clone();
    let path_box: Box<dyn Fn(&mut T) -> Option<&mut u32>> = Box::new(move |t: &mut T| {
        path_clone.get_mut(t)
    });
    Box::new(ChangeCommand {
        path: path_box,
        old_value,
        new_value,
        description,
    })
}

// Helper to create change commands for Vec<String>
fn make_vec_string_change<T: 'static, P, R>(
    target: &T,
    path: WritableOptionalKeyPath<T, Vec<String>, P>,
    read_path: OptionalKeyPath<T, Vec<String>, R>,
    new_value: Vec<String>,
    description: String,
) -> Box<dyn Command<T>>
where
    P: for<'r> Fn(&'r mut T) -> Option<&'r mut Vec<String>> + 'static,
    R: for<'r> Fn(&'r T) -> Option<&'r Vec<String>> + 'static,
{
    let old_value = read_path.get(target).map(|v| v.clone()).unwrap_or_default();
    let path_rc = Rc::new(path);
    let path_clone = path_rc.clone();
    let path_box: Box<dyn Fn(&mut T) -> Option<&mut Vec<String>>> = Box::new(move |t: &mut T| {
        path_clone.get_mut(t)
    });
    Box::new(ChangeCommand {
        path: path_box,
        old_value,
        new_value,
        description,
    })
}

fn main() {
    println!("=== Undo/Redo System Demo ===\n");

    // Create initial document
    let mut doc = Document {
        title: "My Document".to_string(),
        content: "Hello, World!".to_string(),
        metadata: DocumentMetadata {
            author: "Alice".to_string(),
            tags: vec!["draft".to_string()],
            revision: 1,
        },
    };

    println!("Initial document:");
    println!("{:#?}\n", doc);

    // Create undo stack
    let mut undo_stack = UndoStack::new();

    // Change 1: Update title
    println!("--- Change 1: Update title ---");
    let cmd = make_string_change(
        &doc,
        Document::title_w().to_optional(),
        Document::title_r().to_optional(),
        "Updated Document".to_string(),
        "Change title to 'Updated Document'".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Title: {}", doc.title);

    // Change 2: Update content
    println!("\n--- Change 2: Update content ---");
    let cmd = make_string_change(
        &doc,
        Document::content_w().to_optional(),
        Document::content_r().to_optional(),
        "Hello, Rust!".to_string(),
        "Change content to 'Hello, Rust!'".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Content: {}", doc.content);

    // Change 3: Update nested author field
    println!("\n--- Change 3: Update author (nested field) ---");
    let cmd = make_string_change(
        &doc,
        Document::metadata_w().to_optional().then(DocumentMetadata::author_w().to_optional()),
        Document::metadata_r().to_optional().then(DocumentMetadata::author_r().to_optional()),
        "Bob".to_string(),
        "Change author to 'Bob'".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Author: {}", doc.metadata.author);

    // Change 4: Update revision number
    println!("\n--- Change 4: Update revision ---");
    let cmd = make_u32_change(
        &doc,
        Document::metadata_w().to_optional().then(DocumentMetadata::revision_w().to_optional()),
        Document::metadata_r().to_optional().then(DocumentMetadata::revision_r().to_optional()),
        2,
        "Increment revision to 2".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Revision: {}", doc.metadata.revision);

    // Change 5: Update tags
    println!("\n--- Change 5: Update tags ---");
    let cmd = make_vec_string_change(
        &doc,
        Document::metadata_w().to_optional().then(DocumentMetadata::tags_w().to_optional()),
        Document::metadata_r().to_optional().then(DocumentMetadata::tags_r().to_optional()),
        vec!["draft".to_string(), "reviewed".to_string()],
        "Add 'reviewed' tag".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Tags: {:?}", doc.metadata.tags);

    // Display current state
    println!("\n=== Current State (After all changes) ===");
    println!("{:#?}", doc);

    // Display history
    println!("\n=== Command History ===");
    for (i, entry) in undo_stack.history().iter().enumerate() {
        println!("{}. {}", i + 1, entry);
    }

    // Perform undo operations
    println!("\n=== Performing Undo Operations ===");

    // Undo 1
    if undo_stack.can_undo() {
        match undo_stack.undo(&mut doc) {
            Ok(desc) => println!("✓ Undone: {}", desc),
            Err(e) => println!("✗ {}", e),
        }
        println!("Tags: {:?}", doc.metadata.tags);
    }

    // Undo 2
    if undo_stack.can_undo() {
        match undo_stack.undo(&mut doc) {
            Ok(desc) => println!("\n✓ Undone: {}", desc),
            Err(e) => println!("✗ {}", e),
        }
        println!("Revision: {}", doc.metadata.revision);
    }

    // Undo 3
    if undo_stack.can_undo() {
        match undo_stack.undo(&mut doc) {
            Ok(desc) => println!("\n✓ Undone: {}", desc),
            Err(e) => println!("✗ {}", e),
        }
        println!("Author: {}", doc.metadata.author);
    }

    println!("\n=== State After 3 Undos ===");
    println!("{:#?}", doc);

    // Display updated history
    println!("\n=== Updated Command History ===");
    for (i, entry) in undo_stack.history().iter().enumerate() {
        println!("{}. {}", i + 1, entry);
    }

    // Perform redo operations
    println!("\n=== Performing Redo Operations ===");

    // Redo 1
    if undo_stack.can_redo() {
        match undo_stack.redo(&mut doc) {
            Ok(desc) => println!("✓ Redone: {}", desc),
            Err(e) => println!("✗ {}", e),
        }
        println!("Author: {}", doc.metadata.author);
    }

    // Redo 2
    if undo_stack.can_redo() {
        match undo_stack.redo(&mut doc) {
            Ok(desc) => println!("\n✓ Redone: {}", desc),
            Err(e) => println!("✗ {}", e),
        }
        println!("Revision: {}", doc.metadata.revision);
    }

    println!("\n=== State After 2 Redos ===");
    println!("{:#?}", doc);

    // Make a new change (should clear redo history)
    println!("\n=== Making New Change (clears redo history) ===");
    let cmd = make_string_change(
        &doc,
        Document::content_w().to_optional(),
        Document::content_r().to_optional(),
        "Hello, KeyPaths!".to_string(),
        "Change content to 'Hello, KeyPaths!'".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Content: {}", doc.content);

    println!("\n=== Command History (redo history cleared) ===");
    for (i, entry) in undo_stack.history().iter().enumerate() {
        println!("{}. {}", i + 1, entry);
    }

    // Demonstrate full undo to beginning
    println!("\n=== Undoing All Changes ===");
    let mut undo_count = 0;
    while undo_stack.can_undo() {
        if let Ok(desc) = undo_stack.undo(&mut doc) {
            undo_count += 1;
            println!("{}. Undone: {}", undo_count, desc);
        }
    }

    println!("\n=== State After Undoing Everything ===");
    println!("{:#?}", doc);

    // Verify we're back to the original state
    println!("\n=== Verification ===");
    println!("Title matches original: {}", doc.title == "My Document");
    println!("Content matches original: {}", doc.content == "Hello, World!");
    println!("Author matches original: {}", doc.metadata.author == "Alice");
    println!("Revision matches original: {}", doc.metadata.revision == 1);
    println!(
        "Tags match original: {}",
        doc.metadata.tags == vec!["draft".to_string()]
    );

    // Test redo all
    println!("\n=== Redoing All Changes ===");
    let mut redo_count = 0;
    while undo_stack.can_redo() {
        if let Ok(desc) = undo_stack.redo(&mut doc) {
            redo_count += 1;
            println!("{}. Redone: {}", redo_count, desc);
        }
    }

    println!("\n=== Final State (After Redo All) ===");
    println!("{:#?}", doc);

    println!("\n=== Summary ===");
    println!("Total commands in history: {}", undo_stack.commands.len());
    println!("Can undo: {}", undo_stack.can_undo());
    println!("Can redo: {}", undo_stack.can_redo());

    println!("\n✓ Undo/Redo demo complete!");
}

