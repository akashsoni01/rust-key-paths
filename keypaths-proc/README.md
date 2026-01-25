# 🔑 KeyPaths & CasePaths in Rust

Key paths and case paths provide a **safe, composable way to access and modify nested data** in Rust.
Inspired by **Swift’s KeyPath / CasePath** system, this feature rich crate lets you work with **struct fields** and **enum variants** as *first-class values*.

---
#### Derive Macro Generated Methods for Locks

The derive macro generates helper methods for `Arc<Mutex<T>>` and `Arc<RwLock<T>>` fields:

| Field Type | Generated Methods | Description |
|------------|-------------------|-------------|
| `Arc<Mutex<T>>` (parking_lot default) | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through parking_lot::Mutex |
| `Arc<RwLock<T>>` (parking_lot default) | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through parking_lot::RwLock |
| `Arc<std::sync::Mutex<T>>` | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through std::sync::Mutex |
| `Arc<std::sync::RwLock<T>>` | `_r()`, `_w()`, `_fr_at(kp)`, `_fw_at(kp)` | Chain through std::sync::RwLock |

```rust
use std::sync::Arc;
use parking_lot::RwLock;
use keypaths_proc::Kp;

#[derive(Kp)]
#[Writable]
struct Container {
    // This uses parking_lot::RwLock (default)
    data: Arc<RwLock<DataStruct>>,
    
    // This uses std::sync::RwLock (explicit)
    std_data: Arc<std::sync::RwLock<DataStruct>>,
}

#[derive(Kp)]
#[Writable]
struct DataStruct {
    name: String,
}

fn main() {
    let container = Container { /* ... */ };
    
    // Using generated _fr_at() for parking_lot (default)
    Container::data_fr_at(DataStruct::name_r())
        .get(&container, |value| {
            println!("Name: {}", value);
        });
    
    // Using generated _fw_at() for parking_lot (default)
    Container::data_fw_at(DataStruct::name_w())
        .get_mut(&container, |value| {
            *value = "New name".to_string();
        });
    
    // Using generated _fr_at() for std::sync::RwLock (explicit)
    Container::std_data_fr_at(DataStruct::name_r())
        .get(&container, |value| {
            println!("Name: {}", value);
        });
}
```

## 📜 License

* Mozilla Public License 2.0