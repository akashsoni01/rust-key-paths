//! Declarative macros for update dispatch and test message enums.

/// Match all variants of an action enum in `update` (exhaustive dispatch helper).
#[macro_export]
macro_rules! total_update {
    ($state:expr, $action:expr, {
        $($variant:pat => $body:expr,)*
        _ => $default:expr
    }) => {{
        match $action {
            $($variant => $body,)*
            _ => $default,
        }
    }};
}

/// Generate a trivial action for property tests.
#[macro_export]
macro_rules! arbitrary_msg {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            pub fn all() -> &'static [$name] {
                &[$(Self::$variant,)*]
            }
        }
    };
}

/// Build a [`CombineReducers`] tuple from reducer fn pointers.
#[macro_export]
macro_rules! reducers {
    ($($r:expr),+ $(,)?) => {
        $crate::reducer::CombineReducers((
            $( $crate::reducer::coerce_fn($r) ),+
        ))
    };
}
