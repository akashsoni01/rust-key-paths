//! Ergonomic keypath macros for [rust-key-paths].
//!
//! Use with [key-paths-derive]: derive `Kp` on your types, then build keypaths with
//! dot notation. Each segment is `Type.field`; the macro expands to
//! `Type::field().then(NextType::next_field()).then(...)`.
//!
//! [rust-key-paths]: https://docs.rs/rust-key-paths
//! [key-paths-derive]: https://docs.rs/key-paths-derive
//!
//! # Examples
//!
//! ```ignore
//! use key_paths_derive::Kp;
//! use key_paths_macros::keypath;
//! use rust_key_paths::KpType;
//!
//! #[derive(Kp)]
//! struct User { name: String, age: u32 }
//!
//! let kp = keypath!(User.name);
//! let kp = keypath!(User.name);  // same with braces
//!
//! // Nested: alternate Type.field. Type.field (types required for chaining)
//! keypath!(App.user.User.name).get(&app);
//! keypath!(App.user.User.name).get_mut(&mut app);
//! ```

/// Build a keypath from a path of `Type.field` segments.
///
/// Expands to `Root::field1().then(Type2::field2()).then(...)`. Use with types
/// that implement keypath accessors (e.g. via `#[derive(Kp)]` from key-paths-derive).
///
/// Supports both `keypath!(...)` and `keypath!{...}`. For a single segment use
/// `keypath!(Type.field)`. For multiple segments you must specify the type at each
/// step: `keypath!(Type1.f1.Type2.f2.Type3.f3)` so the macro can generate
/// `Type1::f1().then(Type2::f2()).then(Type3::f3())`.
///
/// # Examples
///
/// ```ignore
/// // Single field
/// keypath!(User.name)
/// keypath!{ User.name }
///
/// // Nested path (type before each field)
/// keypath!(SomeComplexStruct.scsf.SomeOtherStruct.sosf.OneMoreStruct.omse.SomeEnum.b.DarkStruct.dsf)
///
/// // Usage with get / get_mut
/// keypath!(User.name).get(&user);
/// keypath!(User.name).get_mut(&mut user);
/// ```
#[macro_export]
macro_rules! keypath {
    // Braces: duplicate rules to avoid recursion (same patterns as parens)
    { $root:ident . $field:ident } => {
        $root::$field()
    };
    { $root:ident . $field:ident () } => {
        $root::$field()
    };
    { $root:ident . $field:ident . $($ty:ident . $f:ident).+ } => {
        $root::$field()
            $(.then($ty::$f()))+
    };
    { $root:ident . $field:ident () . $($ty:ident . $f:ident).+ } => {
        $root::$field()
            $(.then($ty::$f()))+
    };

    // Parens: single segment
    ($root:ident . $field:ident) => {
        $root::$field()
    };
    ($root:ident . $field:ident ()) => {
        $root::$field()
    };
    // Two or more segments
    ($root:ident . $field:ident . $($ty:ident . $f:ident).+) => {
        $root::$field()
            $(.then($ty::$f()))+
    };
    ($root:ident . $field:ident () . $($ty:ident . $f:ident).+) => {
        $root::$field()
            $(.then($ty::$f()))+
    };
    ($root:ident . $field:ident . $($ty:ident . $f:ident).+ () . $($rest:ident . $r:ident).*) => {
        $root::$field()
            $(.then($ty::$f()))+
            $(.then($rest::$r()))*
    };
}

/// Shorthand for `keypath!(...).get(root)`.
///
/// # Example
///
/// ```ignore
/// let name = get!(user => User.name);
/// let deep = get!(app => App.settings.Settings.theme.Theme::Dark);
/// ```
#[macro_export]
macro_rules! get {
    ($root:expr => $($path:tt)*) => {
        $crate::keypath!($($path)*).get($root)
    };
}

/// Shorthand for `keypath!(...).get_mut(root)` and optionally set a value.
///
/// # Examples
///
/// ```ignore
/// get_mut!(user => User.name);                    // returns Option<&mut String>
/// set!(user => User.name = "Alice".to_string());  // set value
/// ```
#[macro_export]
macro_rules! get_mut {
    ($root:expr => $($path:tt)*) => {
        $crate::keypath!($($path)*).get_mut($root)
    };
}

/// Set a value through a keypath: `keypath!(...).get_mut(root).map(|x| *x = value)`.
///
/// Path must be in parentheses so the macro can tell where the path ends and the value begins.
///
/// # Example
///
/// ```ignore
/// set!(user => (User.name) = "Alice".to_string());
/// set!(instance => (SomeComplexStruct.scsf.SomeOtherStruct.sosf.OneMoreStruct.omse.SomeEnum.b.DarkStruct.dsf) = "new".to_string());
/// ```
#[macro_export]
macro_rules! set {
    ($root:expr => ($($path:tt)*) = $value:expr) => {
        $crate::keypath!($($path)*).get_mut($root).map(|x| *x = $value)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal types with keypath-style methods (no derive in this crate)
    struct User {
        name: String,
        age: u32,
    }
    struct Address {
        city: String,
    }
    struct App {
        user: User,
        address: Address,
    }

    impl User {
        fn name() -> rust_key_paths::KpType<'static, User, String> {
            rust_key_paths::Kp::new(
                |u: &User| Some(&u.name),
                |u: &mut User| Some(&mut u.name),
            )
        }
        fn age() -> rust_key_paths::KpType<'static, User, u32> {
            rust_key_paths::Kp::new(
                |u: &User| Some(&u.age),
                |u: &mut User| Some(&mut u.age),
            )
        }
    }
    impl Address {
        fn city() -> rust_key_paths::KpType<'static, Address, String> {
            rust_key_paths::Kp::new(
                |a: &Address| Some(&a.city),
                |a: &mut Address| Some(&mut a.city),
            )
        }
    }
    impl App {
        fn user() -> rust_key_paths::KpType<'static, App, User> {
            rust_key_paths::Kp::new(
                |a: &App| Some(&a.user),
                |a: &mut App| Some(&mut a.user),
            )
        }
        fn address() -> rust_key_paths::KpType<'static, App, Address> {
            rust_key_paths::Kp::new(
                |a: &App| Some(&a.address),
                |a: &mut App| Some(&mut a.address),
            )
        }
    }

    #[test]
    fn keypath_single() {
        let user = User {
            name: "Alice".to_string(),
            age: 30,
        };
        let kp = keypath!(User.name);
        assert_eq!(kp.get(&user), Some(&"Alice".to_string()));
        let kp_braces = keypath! { User.name };
        assert_eq!(kp_braces.get(&user), Some(&"Alice".to_string()));
    }

    #[test]
    fn keypath_chain() {
        let app = App {
            user: User {
                name: "Bob".to_string(),
                age: 25,
            },
            address: Address {
                city: "NYC".to_string(),
            },
        };
        let kp = keypath!(App.user.User.name);
        assert_eq!(kp.get(&app), Some(&"Bob".to_string()));
        let kp_city = keypath!(App.address.Address.city);
        assert_eq!(kp_city.get(&app), Some(&"NYC".to_string()));
    }

    #[test]
    fn get_macro() {
        let user = User {
            name: "Carol".to_string(),
            age: 40,
        };
        let name = get!(&user => User.name);
        assert_eq!(name, Some(&"Carol".to_string()));
    }

    #[test]
    fn set_macro() {
        let mut user = User {
            name: "Dave".to_string(),
            age: 22,
        };
        set!(&mut user => (User.name) = "David".to_string());
        assert_eq!(user.name, "David");
    }

    #[test]
    fn get_mut_macro() {
        let mut user = User {
            name: "Eve".to_string(),
            age: 28,
        };
        let m = get_mut!(&mut user => User.name);
        if let Some(n) = m {
            *n = "Eva".to_string();
        }
        assert_eq!(user.name, "Eva");
    }
}
