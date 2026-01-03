// pub use key_paths_core::*;
pub use rust_keypaths::*;

use rust_keypaths::{
    KeyPath as RustKeyPath,
    OptionalKeyPath as RustOptionalKeyPath,
    WritableKeyPath as RustWritableKeyPath,
    WritableOptionalKeyPath as RustWritableOptionalKeyPath,
    PartialKeyPath as RustPartialKeyPath,
    PartialOptionalKeyPath as RustPartialOptionalKeyPath,
    PartialWritableKeyPath as RustPartialWritableKeyPath,
    PartialWritableOptionalKeyPath as RustPartialWritableOptionalKeyPath,
    AnyKeyPath as RustAnyKeyPath,
    AnyWritableKeyPath as RustAnyWritableKeyPath,
};

// ========== Basic KeyPath Enum ==========

/// Enum for basic keypath types (KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath).
/// 
/// Provides syntactic sugar for functions accepting any basic keypath type.
/// 
/// # Example
/// 
/// ```rust,ignore
/// fn process_keypath<Root>(kp: KP<Root>) {
///     match kp {
///         KP::KeyPath(k) => { /* handle KeyPath */ },
///         KP::OptionalKeyPath(k) => { /* handle OptionalKeyPath */ },
///         KP::WritableKeyPath(k) => { /* handle WritableKeyPath */ },
///         KP::WritableOptionalKeyPath(k) => { /* handle WritableOptionalKeyPath */ },
///     }
/// }
/// ```
pub enum KP<Root> {
    /// A readable keypath that always succeeds.
    KeyPath(RustPartialKeyPath<Root>),
    
    /// A readable keypath that may return `None`.
    OptionalKeyPath(RustPartialOptionalKeyPath<Root>),
    
    /// A writable keypath that always succeeds.
    WritableKeyPath(RustPartialWritableKeyPath<Root>),
    
    /// A writable keypath that may return `None`.
    WritableOptionalKeyPath(RustPartialWritableOptionalKeyPath<Root>),
}

impl<Root> KP<Root> {
    /// Convert from a concrete `rust_keypaths::KeyPath`.
    pub fn from_keypath<Value, F>(kp: RustKeyPath<Root, Value, F>) -> Self
    where
        F: for<'r> Fn(&'r Root) -> &'r Value + 'static,
        Root: 'static,
        Value: std::any::Any + 'static,
    {
        Self::KeyPath(kp.to_partial())
    }

    /// Convert from a concrete `rust_keypaths::OptionalKeyPath`.
    pub fn from_optional_keypath<Value, F>(kp: RustOptionalKeyPath<Root, Value, F>) -> Self
    where
        F: for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static,
        Root: 'static,
        Value: std::any::Any + 'static,
    {
        Self::OptionalKeyPath(kp.to_partial())
    }

    /// Convert from a concrete `rust_keypaths::WritableKeyPath`.
    pub fn from_writable_keypath<Value, F>(kp: RustWritableKeyPath<Root, Value, F>) -> Self
    where
        F: for<'r> Fn(&'r mut Root) -> &'r mut Value + 'static,
        Root: 'static,
        Value: std::any::Any + 'static,
    {
        Self::WritableKeyPath(kp.to_partial())
    }

    /// Convert from a concrete `rust_keypaths::WritableOptionalKeyPath`.
    pub fn from_writable_optional_keypath<Value, F>(kp: RustWritableOptionalKeyPath<Root, Value, F>) -> Self
    where
        F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static,
        Root: 'static,
        Value: std::any::Any + 'static,
    {
        Self::WritableOptionalKeyPath(kp.to_partial())
    }
}

// From implementations for easier conversion
impl<Root, Value, F> From<RustKeyPath<Root, Value, F>> for KP<Root>
where
    F: for<'r> Fn(&'r Root) -> &'r Value + 'static,
    Root: 'static,
    Value: std::any::Any + 'static,
{
    fn from(kp: RustKeyPath<Root, Value, F>) -> Self {
        Self::from_keypath(kp)
    }
}

impl<Root, Value, F> From<RustOptionalKeyPath<Root, Value, F>> for KP<Root>
where
    F: for<'r> Fn(&'r Root) -> Option<&'r Value> + 'static,
    Root: 'static,
    Value: std::any::Any + 'static,
{
    fn from(kp: RustOptionalKeyPath<Root, Value, F>) -> Self {
        Self::from_optional_keypath(kp)
    }
}

impl<Root, Value, F> From<RustWritableKeyPath<Root, Value, F>> for KP<Root>
where
    F: for<'r> Fn(&'r mut Root) -> &'r mut Value + 'static,
    Root: 'static,
    Value: std::any::Any + 'static,
{
    fn from(kp: RustWritableKeyPath<Root, Value, F>) -> Self {
        Self::from_writable_keypath(kp)
    }
}

impl<Root, Value, F> From<RustWritableOptionalKeyPath<Root, Value, F>> for KP<Root>
where
    F: for<'r> Fn(&'r mut Root) -> Option<&'r mut Value> + 'static,
    Root: 'static,
    Value: std::any::Any + 'static,
{
    fn from(kp: RustWritableOptionalKeyPath<Root, Value, F>) -> Self {
        Self::from_writable_optional_keypath(kp)
    }
}

// From implementations for PKP enum
impl<Root> From<RustPartialKeyPath<Root>> for PKP<Root> {
    fn from(kp: RustPartialKeyPath<Root>) -> Self {
        Self::PartialKeyPath(kp)
    }
}

impl<Root> From<RustPartialOptionalKeyPath<Root>> for PKP<Root> {
    fn from(kp: RustPartialOptionalKeyPath<Root>) -> Self {
        Self::PartialOptionalKeyPath(kp)
    }
}

impl<Root> From<RustPartialWritableKeyPath<Root>> for PKP<Root> {
    fn from(kp: RustPartialWritableKeyPath<Root>) -> Self {
        Self::PartialWritableKeyPath(kp)
    }
}

impl<Root> From<RustPartialWritableOptionalKeyPath<Root>> for PKP<Root> {
    fn from(kp: RustPartialWritableOptionalKeyPath<Root>) -> Self {
        Self::PartialWritableOptionalKeyPath(kp)
    }
}

// From implementations for AKP enum
impl From<RustAnyKeyPath> for AKP {
    fn from(kp: RustAnyKeyPath) -> Self {
        Self::AnyKeyPath(kp)
    }
}

impl From<RustAnyWritableKeyPath> for AKP {
    fn from(kp: RustAnyWritableKeyPath) -> Self {
        Self::AnyWritableKeyPath(kp)
    }
}

// ========== PartialKeyPath Enum ==========

/// Enum for partial keypath types (PartialKeyPath, PartialOptionalKeyPath, PartialWritableKeyPath, PartialWritableOptionalKeyPath).
/// 
/// Provides syntactic sugar for functions accepting any partial keypath type.
/// 
/// # Example
/// 
/// ```rust,ignore
/// fn process_partial_keypath<Root>(pkp: PKP<Root>) {
///     match pkp {
///         PKP::PartialKeyPath(k) => { /* handle PartialKeyPath */ },
///         PKP::PartialOptionalKeyPath(k) => { /* handle PartialOptionalKeyPath */ },
///         PKP::PartialWritableKeyPath(k) => { /* handle PartialWritableKeyPath */ },
///         PKP::PartialWritableOptionalKeyPath(k) => { /* handle PartialWritableOptionalKeyPath */ },
///     }
/// }
/// ```
pub enum PKP<Root> {
    /// Type-erased keypath with known Root but unknown Value.
    PartialKeyPath(RustPartialKeyPath<Root>),
    
    /// Type-erased optional keypath with known Root.
    PartialOptionalKeyPath(RustPartialOptionalKeyPath<Root>),
    
    /// Type-erased writable keypath with known Root.
    PartialWritableKeyPath(RustPartialWritableKeyPath<Root>),
    
    /// Type-erased writable optional keypath with known Root.
    PartialWritableOptionalKeyPath(RustPartialWritableOptionalKeyPath<Root>),
}

// ========== AnyKeyPath Enum ==========

/// Enum for fully type-erased keypath types (AnyKeyPath, AnyWritableKeyPath).
/// 
/// Provides syntactic sugar for functions accepting any fully type-erased keypath type.
/// 
/// # Example
/// 
/// ```rust,ignore
/// fn process_any_keypath(anykp: AKP) {
///     match anykp {
///         AKP::AnyKeyPath(k) => { /* handle AnyKeyPath */ },
///         AKP::AnyWritableKeyPath(k) => { /* handle AnyWritableKeyPath */ },
///     }
/// }
/// ```
pub enum AKP {
    /// Fully type-erased keypath (unknown Root and Value).
    AnyKeyPath(RustAnyKeyPath),
    
    /// Fully type-erased writable keypath.
    AnyWritableKeyPath(RustAnyWritableKeyPath),
}

// ========== Chain KeyPath Enums ==========

/// Enum for Arc<Mutex<T>> chain keypath types.
/// 
/// Represents all chain types that traverse through `Arc<Mutex<T>>`:
/// - ArcMutexKeyPathChain
/// - ArcMutexWritableKeyPathChain
/// - ArcMutexOptionalKeyPathChain
/// - ArcMutexWritableOptionalKeyPathChain
/// 
/// The outer keypath (to the `Arc<Mutex<T>>`) is stored as a `PartialKeyPath`.
pub enum AMKP<Root> {
    /// Chain through `Arc<Mutex<T>>` with readable inner keypath.
    ArcMutexKeyPathChain(RustPartialKeyPath<Root>),
    
    /// Chain through `Arc<Mutex<T>>` with optional readable inner keypath.
    ArcMutexOptionalKeyPathChain(RustPartialKeyPath<Root>),
}

/// Enum for Arc<RwLock<T>> chain keypath types.
/// 
/// Represents all chain types that traverse through `Arc<RwLock<T>>`:
/// - ArcRwLockKeyPathChain
/// - ArcRwLockWritableKeyPathChain
/// - ArcRwLockOptionalKeyPathChain
/// - ArcRwLockWritableOptionalKeyPathChain
/// 
/// The outer keypath (to the `Arc<RwLock<T>>`) is stored as a `PartialKeyPath`.
pub enum ARKP<Root> {
    /// Chain through `Arc<RwLock<T>>` with readable inner keypath.
    ArcRwLockKeyPathChain(RustPartialKeyPath<Root>),
    
    /// Chain through `Arc<RwLock<T>>` with optional readable inner keypath.
    ArcRwLockOptionalKeyPathChain(RustPartialKeyPath<Root>),
}

/// Enum for optional Arc<Mutex<T>> chain keypath types.
/// 
/// Represents all chain types that traverse through optional `Arc<Mutex<T>>`:
/// - OptionalArcMutexKeyPathChain
/// - OptionalArcMutexWritableKeyPathChain
/// - OptionalArcMutexOptionalKeyPathChain
/// - OptionalArcMutexWritableOptionalKeyPathChain
/// 
/// The outer keypath (to the optional `Arc<Mutex<T>>`) is stored as a `PartialOptionalKeyPath`.
pub enum OAMKP<Root> {
    /// Chain through optional `Arc<Mutex<T>>` with readable inner keypath.
    OptionalArcMutexKeyPathChain(RustPartialOptionalKeyPath<Root>),
    
    /// Chain through optional `Arc<Mutex<T>>` with optional readable inner keypath.
    OptionalArcMutexOptionalKeyPathChain(RustPartialOptionalKeyPath<Root>),
}

/// Enum for optional Arc<RwLock<T>> chain keypath types.
/// 
/// Represents all chain types that traverse through optional `Arc<RwLock<T>>`:
/// - OptionalArcRwLockKeyPathChain
/// - OptionalArcRwLockWritableKeyPathChain
/// - OptionalArcRwLockOptionalKeyPathChain
/// - OptionalArcRwLockWritableOptionalKeyPathChain
/// 
/// The outer keypath (to the optional `Arc<RwLock<T>>`) is stored as a `PartialOptionalKeyPath`.
pub enum OARKP<Root> {
    /// Chain through optional `Arc<RwLock<T>>` with readable inner keypath.
    OptionalArcRwLockKeyPathChain(RustPartialOptionalKeyPath<Root>),
    
    /// Chain through optional `Arc<RwLock<T>>` with optional readable inner keypath.
    OptionalArcRwLockOptionalKeyPathChain(RustPartialOptionalKeyPath<Root>),
}

/// Enum for Arc<parking_lot::Mutex<T>> chain keypath types.
/// 
/// Represents all chain types that traverse through `Arc<parking_lot::Mutex<T>>`:
/// - ArcParkingMutexKeyPathChain
/// - ArcParkingMutexWritableKeyPathChain
/// - ArcParkingMutexOptionalKeyPathChain
/// - ArcParkingMutexWritableOptionalKeyPathChain
/// 
/// Requires the `parking_lot` feature to be enabled.
/// The outer keypath (to the `Arc<parking_lot::Mutex<T>>`) is stored as a `PartialKeyPath`.
#[cfg(feature = "parking_lot")]
pub enum APMKP<Root> {
    /// Chain through `Arc<parking_lot::Mutex<T>>` with readable inner keypath.
    ArcParkingMutexKeyPathChain(RustPartialKeyPath<Root>),
    
    /// Chain through `Arc<parking_lot::Mutex<T>>` with optional readable inner keypath.
    ArcParkingMutexOptionalKeyPathChain(RustPartialKeyPath<Root>),
}

/// Enum for Arc<parking_lot::RwLock<T>> chain keypath types.
/// 
/// Represents all chain types that traverse through `Arc<parking_lot::RwLock<T>>`:
/// - ArcParkingRwLockKeyPathChain
/// - ArcParkingRwLockWritableKeyPathChain
/// - ArcParkingRwLockOptionalKeyPathChain
/// - ArcParkingRwLockWritableOptionalKeyPathChain
/// 
/// Requires the `parking_lot` feature to be enabled.
/// The outer keypath (to the `Arc<parking_lot::RwLock<T>>`) is stored as a `PartialKeyPath`.
#[cfg(feature = "parking_lot")]
pub enum APRKP<Root> {
    /// Chain through `Arc<parking_lot::RwLock<T>>` with readable inner keypath.
    ArcParkingRwLockKeyPathChain(RustPartialKeyPath<Root>),
    
    /// Chain through `Arc<parking_lot::RwLock<T>>` with optional readable inner keypath.
    ArcParkingRwLockOptionalKeyPathChain(RustPartialKeyPath<Root>),
}

/// Enum for optional Arc<parking_lot::Mutex<T>> chain keypath types.
/// 
/// Represents all chain types that traverse through optional `Arc<parking_lot::Mutex<T>>`:
/// - OptionalArcParkingMutexKeyPathChain
/// - OptionalArcParkingMutexWritableKeyPathChain
/// - OptionalArcParkingMutexOptionalKeyPathChain
/// - OptionalArcParkingMutexWritableOptionalKeyPathChain
/// 
/// Requires the `parking_lot` feature to be enabled.
/// The outer keypath (to the optional `Arc<parking_lot::Mutex<T>>`) is stored as a `PartialOptionalKeyPath`.
#[cfg(feature = "parking_lot")]
pub enum OAPMKP<Root> {
    /// Chain through optional `Arc<parking_lot::Mutex<T>>` with readable inner keypath.
    OptionalArcParkingMutexKeyPathChain(RustPartialOptionalKeyPath<Root>),
    
    /// Chain through optional `Arc<parking_lot::Mutex<T>>` with optional readable inner keypath.
    OptionalArcParkingMutexOptionalKeyPathChain(RustPartialOptionalKeyPath<Root>),
}

/// Enum for optional Arc<parking_lot::RwLock<T>> chain keypath types.
/// 
/// Represents all chain types that traverse through optional `Arc<parking_lot::RwLock<T>>`:
/// - OptionalArcParkingRwLockKeyPathChain
/// - OptionalArcParkingRwLockWritableKeyPathChain
/// - OptionalArcParkingRwLockOptionalKeyPathChain
/// - OptionalArcParkingRwLockWritableOptionalKeyPathChain
/// 
/// Requires the `parking_lot` feature to be enabled.
/// The outer keypath (to the optional `Arc<parking_lot::RwLock<T>>`) is stored as a `PartialOptionalKeyPath`.
#[cfg(feature = "parking_lot")]
pub enum OAPRKP<Root> {
    /// Chain through optional `Arc<parking_lot::RwLock<T>>` with readable inner keypath.
    OptionalArcParkingRwLockKeyPathChain(RustPartialOptionalKeyPath<Root>),
    
    /// Chain through optional `Arc<parking_lot::RwLock<T>>` with optional readable inner keypath.
    OptionalArcParkingRwLockOptionalKeyPathChain(RustPartialOptionalKeyPath<Root>),
}
