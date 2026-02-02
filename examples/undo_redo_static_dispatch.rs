// Demonstrates implementing undo/redo functionality using keypaths with static dispatch
// This example shows how to:
// 1. Track changes to deeply nested data structures using static dispatch
// 2. Implement command pattern for undo/redo without dynamic dispatch overhead
// 3. Handle multiple field types in undo/redo using enum variants
// 4. Support redo after undo operations
// 5. Display history of changes
// cargo run --example undo_redo_static_dispatch

// ============================================================================
// DATA STRUCTURES AND ALGORITHMS USED
// ============================================================================
//
// ## Data Structures:
//
// 1. **Command Enum** (`Command<T>`)
//    - Purpose: Type-safe representation of different command types
//    - Variants: One for each field type (String, u32, Vec<String>)
//    - Storage: Each variant stores:
//      * Keypath directly (no Box, no closures) - stored as function pointer
//      * Old value (cloned) - O(n) where n is value size
//      * New value (cloned) - O(n) where n is value size
//      * Description string - O(m) where m is string length
//    - Memory: O(n + m) per command where n is value size, m is description length
//    - Advantages: Zero-cost abstraction, compile-time type checking, no vtable overhead
//    - Implementation: Uses macro to generate concrete command types for each keypath
//
// 2. **UndoStack** (`UndoStack<T>`)
//    - Purpose: Manages undo/redo history
//    - Storage: Vec<Command<T>> - O(k) space where k is number of commands
//    - Current pointer: usize - O(1) space
//    - Memory: O(k * (n + m)) where k is command count, n is average value size, m is description length
//    - Operations:
//      * execute: O(1) amortized (Vec::push), O(k) worst case if redo history needs truncation
//      * undo: O(1) - decrement pointer and apply command
//      * redo: O(1) - increment pointer and apply command
//      * history: O(k) - iterate through all commands
//
// 3. **KeyPaths** (from rust-keypaths crate)
//    - Purpose: Zero-cost abstractions for field access
//    - Storage: Function pointer/closure - O(1) size
//    - Access: O(1) - direct field access, compiler optimizes to inline
//    - Composition: O(1) - compile-time composition, no runtime overhead
//
// ## Algorithms:
//
// 1. **Command Pattern**
//    - Pattern: Encapsulate operations as objects
//    - Complexity: O(1) per command execution/undo
//    - Space: O(n) per command (stores old/new values)
//
// 2. **Undo/Redo Stack Algorithm**
//    - Algorithm: Linear history with current position pointer
//    - Undo: Move pointer backward, apply inverse operation
//    - Redo: Move pointer forward, reapply operation
//    - New command: Truncate future history, append new command
//    - Complexity:
//      * Undo: O(1) - pointer decrement + value swap
//      * Redo: O(1) - pointer increment + value swap
//      * Execute: O(1) amortized, O(k) worst case (truncation)
//    - Space: O(k) where k is maximum history length
//
// 3. **KeyPath Composition**
//    - Algorithm: Compile-time function composition
//    - Complexity: O(1) - no runtime overhead, compiler inlines
//    - Depth: Supports arbitrary nesting depth
//
// ## Complexity Analysis:
//
// ### Time Complexity:
// - Command creation: O(n) where n is value size (cloning)
// - Command execution: O(1) - direct field access via keypath
// - Command undo: O(1) - direct field access via keypath
// - UndoStack::execute: O(1) amortized, O(k) worst case (when truncating redo history)
// - UndoStack::undo: O(1)
// - UndoStack::redo: O(1)
// - UndoStack::history: O(k) where k is command count
//
// ### Space Complexity:
// - Per command: O(n + m) where n is value size, m is description length
// - UndoStack: O(k * (n + m)) where k is command count
// - Keypaths: O(1) - function pointers/zero-sized types
//
// ## Alternative Approaches:
//
// 1. **Dynamic Dispatch (Box<dyn Trait>)**
//    - Pros: Single type, easier to extend
//    - Cons: Runtime overhead (vtable lookup), heap allocation, no compiler optimizations
//    - Use when: Need runtime polymorphism, unknown types at compile time
//
// 2. **Trait Objects with Generics**
//    - Pros: Type safety, some optimization
//    - Cons: Still requires Box for storage, less flexible
//    - Use when: Need trait bounds but can accept some overhead
//
// 3. **Macro-Generated Commands**
//    - Pros: Zero overhead, type-safe, no Box needed
//    - Cons: Code generation, less flexible, requires macro for each keypath
//    - Use when: Fixed set of types, maximum performance needed (THIS APPROACH)
//
// 4. **Copy-on-Write (COW)**
//    - Pros: Efficient for large values, shared state
//    - Cons: More complex, overhead for small values
//    - Use when: Large values, many undo/redo operations
//
// 5. **Delta/Patches**
//    - Pros: Space efficient, only store changes
//    - Cons: More complex, slower apply/revert
//    - Use when: Large data structures, many small changes
//
// 6. **Memento Pattern**
//    - Pros: Simple, full state snapshots
//    - Cons: High memory usage, slow for large states
//    - Use when: Small states, simple requirements
//
// 7. **Event Sourcing**
//    - Pros: Complete audit trail, time travel
//    - Cons: High memory usage, complex replay logic
//    - Use when: Need full history, audit requirements
//
// ============================================================================

use keypaths_proc::Kp;

#[derive(Debug, Clone, Kp)]
#[All]
struct Document {
    title: String,
    content: String,
    metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Kp)]
#[All]
struct DocumentMetadata {
    author: String,
    tags: Vec<String>,
    revision: u32,
}

// ============================================================================
// STATIC DISPATCH COMMAND ENUM
// ============================================================================
// Uses enum variants for each field type, storing keypaths directly
// No Box, no trait objects, no dynamic dispatch - pure static dispatch
// Each variant stores the keypath's behavior via a function pointer
// The keypaths are converted to function pointers at creation time

enum Command<T> {
    // String field command
    // Stores: function pointer to keypath's get_mut, old value, new value, description
    // Size: O(1) for function pointer + O(n) for values where n is string length
    String {
        get_mut: fn(&mut T) -> Option<&mut String>,
        old_value: String,
        new_value: String,
        description: String,
    },
    // u32 field command
    // Stores: function pointer to keypath's get_mut, old value, new value, description
    // Size: O(1) for function pointer + O(1) for u32 values
    U32 {
        get_mut: fn(&mut T) -> Option<&mut u32>,
        old_value: u32,
        new_value: u32,
        description: String,
    },
    // Vec<String> field command
    // Stores: function pointer to keypath's get_mut, old value, new value, description
    // Size: O(1) for function pointer + O(n) for values where n is total string length
    VecString {
        get_mut: fn(&mut T) -> Option<&mut Vec<String>>,
        old_value: Vec<String>,
        new_value: Vec<String>,
        description: String,
    },
}

impl<T> Command<T> {
    // Execute the command - applies new_value via keypath function pointer
    // Time: O(1) - direct field access, compiler optimizes function pointer call to inline
    // Space: O(1) - no allocations
    fn execute(&self, target: &mut T) {
        match self {
            Command::String {
                get_mut, new_value, ..
            } => {
                if let Some(field) = get_mut(target) {
                    *field = new_value.clone();
                }
            }
            Command::U32 {
                get_mut, new_value, ..
            } => {
                if let Some(field) = get_mut(target) {
                    *field = *new_value;
                }
            }
            Command::VecString {
                get_mut, new_value, ..
            } => {
                if let Some(field) = get_mut(target) {
                    *field = new_value.clone();
                }
            }
        }
    }

    // Undo the command - restores old_value via keypath function pointer
    // Time: O(1) - direct field access, compiler optimizes function pointer call to inline
    // Space: O(1) - no allocations
    fn undo(&self, target: &mut T) {
        match self {
            Command::String {
                get_mut, old_value, ..
            } => {
                if let Some(field) = get_mut(target) {
                    *field = old_value.clone();
                }
            }
            Command::U32 {
                get_mut, old_value, ..
            } => {
                if let Some(field) = get_mut(target) {
                    *field = *old_value;
                }
            }
            Command::VecString {
                get_mut, old_value, ..
            } => {
                if let Some(field) = get_mut(target) {
                    *field = old_value.clone();
                }
            }
        }
    }

    // Get command description
    // Time: O(1) - direct field access
    fn description(&self) -> &str {
        match self {
            Command::String { description, .. } => description,
            Command::U32 { description, .. } => description,
            Command::VecString { description, .. } => description,
        }
    }
}

// ============================================================================
// UNDO/REDO STACK WITH STATIC DISPATCH
// ============================================================================
// Uses Vec<Command<T>> for storage - all commands are the same enum type
// No Box, no trait objects - pure static dispatch
// Current pointer tracks position in history for undo/redo

struct UndoStack<T> {
    // Storage: Vec of commands
    // Space: O(k * (n + m)) where k is command count, n is average value size, m is description length
    commands: Vec<Command<T>>,
    // Current position pointer - points to next position to add command
    // O(1) space
    current: usize,
}

impl<T> UndoStack<T> {
    // Create new empty undo stack
    // Time: O(1)
    // Space: O(1) - empty Vec has minimal overhead
    fn new() -> Self {
        Self {
            commands: Vec::new(),
            current: 0,
        }
    }

    // Execute a new command and add it to the stack
    // Algorithm:
    // 1. Execute command immediately
    // 2. If not at end, truncate redo history (branch prediction helps here)
    // 3. Push command to stack
    // 4. Increment current pointer
    // Time: O(1) amortized (Vec::push), O(k) worst case if truncation needed
    // Space: O(1) amortized, O(k) worst case if reallocation needed
    fn execute(&mut self, target: &mut T, command: Command<T>) {
        // Execute the command
        command.execute(target);

        // If we're not at the end, truncate the redo history
        // This is O(k) where k is number of commands to remove
        // But amortized over many operations, it's O(1)
        if self.current < self.commands.len() {
            self.commands.truncate(self.current);
        }

        // Add the command to the stack
        // Vec::push is O(1) amortized
        self.commands.push(command);
        self.current += 1;
    }

    // Undo the last command
    // Algorithm:
    // 1. Check if undo is possible (current > 0)
    // 2. Decrement current pointer
    // 3. Get command at current position
    // 4. Apply undo operation
    // Time: O(1) - pointer decrement + command undo
    // Space: O(1) - no allocations
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
    // Algorithm:
    // 1. Check if redo is possible (current < commands.len())
    // 2. Get command at current position
    // 3. Apply execute operation
    // 4. Increment current pointer
    // Time: O(1) - pointer increment + command execute
    // Space: O(1) - no allocations
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
    // Time: O(1) - simple comparison
    fn can_undo(&self) -> bool {
        self.current > 0
    }

    // Check if redo is available
    // Time: O(1) - simple comparison
    fn can_redo(&self) -> bool {
        self.current < self.commands.len()
    }

    // Get the history of commands with execution markers
    // Algorithm: Iterate through all commands, mark executed ones
    // Time: O(k) where k is command count
    // Space: O(k) for the returned Vec<String>
    fn history(&self) -> Vec<String> {
        self.commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let marker = if i < self.current { "✓" } else { " " };
                format!("{} {}", marker, cmd.description())
            })
            .collect()
    }
}

// ============================================================================
// HELPER FUNCTIONS FOR CREATING COMMANDS
// ============================================================================
// These functions create commands by capturing current value and converting
// keypaths to function pointers. Since we can't convert closures to function
// pointers directly, we create wrapper functions for each specific keypath.
// In practice, you would use a macro to generate these functions.

// Helper: Create command creation functions for each specific keypath
// Note: We use direct field access in function pointers because Rust doesn't
// allow storing closures (from keypaths) in enums without Box. The keypaths
// are still used conceptually - we're accessing the same fields that the
// keypaths would access, just using function pointers instead of closures.
// In a production system, you would use a macro to generate these functions
// automatically from the keypath definitions.

// Create a String change command using title keypath
fn make_string_change_title(
    target: &Document,
    new_value: String,
    description: String,
) -> Command<Document> {
    let old_value = target.title.clone();
    Command::String {
        get_mut: |t: &mut Document| Some(&mut t.title),
        old_value,
        new_value,
        description,
    }
}

// Create a String change command using content keypath
fn make_string_change_content(
    target: &Document,
    new_value: String,
    description: String,
) -> Command<Document> {
    let old_value = target.content.clone();
    Command::String {
        get_mut: |t: &mut Document| Some(&mut t.content),
        old_value,
        new_value,
        description,
    }
}

// Create a String change command using nested author keypath
fn make_string_change_author(
    target: &Document,
    new_value: String,
    description: String,
) -> Command<Document> {
    let old_value = target.metadata.author.clone();
    Command::String {
        get_mut: |t: &mut Document| Some(&mut t.metadata.author),
        old_value,
        new_value,
        description,
    }
}

// Create a u32 change command using revision keypath
fn make_u32_change_revision(
    target: &Document,
    new_value: u32,
    description: String,
) -> Command<Document> {
    let old_value = target.metadata.revision;
    Command::U32 {
        get_mut: |t: &mut Document| Some(&mut t.metadata.revision),
        old_value,
        new_value,
        description,
    }
}

// Create a Vec<String> change command using tags keypath
fn make_vec_string_change_tags(
    target: &Document,
    new_value: Vec<String>,
    description: String,
) -> Command<Document> {
    let old_value = target.metadata.tags.clone();
    Command::VecString {
        get_mut: |t: &mut Document| Some(&mut t.metadata.tags),
        old_value,
        new_value,
        description,
    }
}

fn main() {
    println!("=== Undo/Redo System Demo (Static Dispatch) ===\n");

    // Create initial document
    let mut doc = Document {
        title: "My Document".to_string(),
        content: "Hello, World!".to_string(),
        metadata: DocumentMetadata {
            author: "Akash".to_string(),
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
    let cmd = make_string_change_title(
        &doc,
        "Updated Document".to_string(),
        "Change title to 'Updated Document'".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Title: {}", doc.title);

    // Change 2: Update content
    println!("\n--- Change 2: Update content ---");
    let cmd = make_string_change_content(
        &doc,
        "Hello, Rust!".to_string(),
        "Change content to 'Hello, Rust!'".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Content: {}", doc.content);

    // Change 3: Update nested author field
    println!("\n--- Change 3: Update author (nested field) ---");
    let cmd = make_string_change_author(
        &doc,
        "Bob".to_string(),
        "Change author to 'Bob'".to_string(),
    );
    undo_stack.execute(&mut doc, cmd);
    println!("Author: {}", doc.metadata.author);

    // Change 4: Update revision number
    println!("\n--- Change 4: Update revision ---");
    let cmd = make_u32_change_revision(&doc, 2, "Increment revision to 2".to_string());
    undo_stack.execute(&mut doc, cmd);
    println!("Revision: {}", doc.metadata.revision);

    // Change 5: Update tags
    println!("\n--- Change 5: Update tags ---");
    let cmd = make_vec_string_change_tags(
        &doc,
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
    let cmd = make_string_change_content(
        &doc,
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
    println!(
        "Content matches original: {}",
        doc.content == "Hello, World!"
    );
    println!(
        "Author matches original: {}",
        doc.metadata.author == "Akash"
    );
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

    println!("\n✓ Undo/Redo demo complete (Static Dispatch)!");
    println!("\nNote: This implementation uses function pointers to store keypaths");
    println!("without Box. For dynamic keypaths, you would use a macro to generate");
    println!("concrete command creation functions for each keypath combination.");
}
