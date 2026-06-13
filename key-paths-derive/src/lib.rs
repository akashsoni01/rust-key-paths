use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input, spanned::Spanned};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WrapperKind {
    None,
    Option,
    Box,
    Rc,
    Arc,
    Vec,
    HashMap,
    BTreeMap,
    HashSet,
    BTreeSet,
    VecDeque,
    LinkedList,
    BinaryHeap,
    // Error handling containers
    Result,
    // Reference counting with weak references
    Weak,
    // String and owned text
    String,
    OptionString,
    // Interior mutability (std::cell)
    Cell,
    RefCell,
    OptionCell,
    OptionRefCell,
    // Lazy init (once_cell, std::sync::OnceLock, std::sync::LazyLock)
    OnceCell,
    Lazy,
    OptionOnceCell,
    OptionLazy,
    // Marker / zero-size
    PhantomData,
    OptionPhantomData,
    // Range iterators (std::ops::Range, RangeInclusive)
    Range,
    OptionRange,
    // Nested container support
    OptionBox,
    OptionRc,
    OptionArc,
    BoxOption,
    RcOption,
    ArcOption,
    VecOption,
    OptionVec,
    VecDequeOption,
    OptionVecDeque,
    LinkedListOption,
    OptionLinkedList,
    BinaryHeapOption,
    OptionBinaryHeap,
    HashSetOption,
    OptionHashSet,
    BTreeSetOption,
    OptionBTreeSet,
    ResultOption,
    OptionResult,
    HashMapOption,
    OptionHashMap,
    BTreeMapOption,
    OptionBTreeMap,
    // Arc with synchronization primitives (default)
    StdArcMutex,
    StdArcRwLock,
    StdArcMutexOption,
    StdArcRwLockOption,
    OptionStdArcMutex,
    OptionStdArcRwLock,
    // Synchronization primitives default
    StdMutex,
    StdRwLock,
    StdMutexOption,
    StdRwLockOption,
    OptionStdMutex,
    OptionStdRwLock,
    // Synchronization primitives (parking_lot)
    Mutex,
    RwLock,
    OptionMutex,
    OptionRwLock,
    // Synchronization primitives (tokio::sync - requires tokio feature)
    TokioMutex,
    TokioRwLock,
    // parking_lot
    ArcMutex,
    ArcRwLock,
    ArcMutexOption,
    ArcRwLockOption,
    OptionArcMutex,
    OptionArcRwLock,
    MutexOption,
    RwLockOption,
    // Arc with synchronization primitives (tokio::sync - requires tokio feature)
    TokioArcMutex,
    TokioArcRwLock,
    OptionTokioArcMutex,
    OptionTokioArcRwLock,
    // arc-swap (`arc-swap` feature): derive emits `SyncKp` only — struct `field()`, enum variant `snake()` (no `field_kp()`, no `*_lock()` suffix).
    /// `arc_swap::ArcSwap<T>` or `ArcSwapAny<Arc<T>>` (bare; nested under `Arc<…>`)
    BareArcSwap,
    /// `arc_swap::ArcSwapOption<T>` or `ArcSwapAny<Option<Arc<T>>>` (bare)
    BareArcSwapOption,
    /// `Arc<arc_swap::ArcSwap<T>>` (requires rust-key-paths `arc-swap` feature)
    ArcArcSwap,
    /// `Arc<arc_swap::ArcSwapOption<T>>`
    ArcArcSwapOption,
    OptionArcArcSwap,
    OptionArcArcSwapOption,
    // Tagged types
    Tagged,
    OptionTagged,
    // Clone-on-write (std::borrow::Cow)
    Cow,
    OptionCow,
    // Reference types (&T, &str, &[T], etc.)
    Reference,
    OptionReference,
    // Atomic types (std::sync::atomic::*)
    Atomic,
    OptionAtomic,
    // Pin types
    Pin,
    PinBox,
    /// Field marked with #[pin] - plain type (pin_project pattern)
    PinnedField,
    /// Field marked with #[pin] - Future type (pin_project pattern)
    PinnedFuture,
    /// Field marked with #[pin] - Box<dyn Future> (pin_project pattern)
    PinnedBoxFuture,
}

/// Helper function to check if a type path is exactly under std::sync (not lock_api or parking_lot).
/// Requires the path to start with "std" then "sync" so we don't confuse with RwLock&lt;RawRwLock, T&gt; (lock_api).
fn is_std_sync_type(path: &syn::Path) -> bool {
    let segments: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments.len() >= 2
        && segments.get(0).map(|s| s.as_str()) == Some("std")
        && segments.get(1).map(|s| s.as_str()) == Some("sync")
}

/// Helper function to check if a type path is exactly under tokio::sync.
fn is_tokio_sync_type(path: &syn::Path) -> bool {
    let segments: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments.len() >= 2
        && segments.get(0).map(|s| s.as_str()) == Some("tokio")
        && segments.get(1).map(|s| s.as_str()) == Some("sync")
}

/// Helper function to check if a type path is under parking_lot (RwLock/Mutex from lock_api).
/// Use so we never treat parking_lot::RwLock as std::sync::RwLock.
fn is_parking_lot_type(path: &syn::Path) -> bool {
    path.segments.first().map(|s| s.ident == "parking_lot") == Some(true)
}

/// True if `path` starts with the `arc_swap` or `arcswap` crate alias (hyphenated dep `arc-swap` → `arc_swap`).
fn is_arcswap_crate_path(path: &syn::Path) -> bool {
    path.segments.first().is_some_and(|s| {
        matches!(
            s.ident.to_string().as_str(),
            "arc_swap" | "arcswap"
        )
    })
}

fn angle_bracket_first_type(args: &syn::PathArguments) -> Option<&syn::Type> {
    match args {
        syn::PathArguments::AngleBracketed(ab) => ab.args.iter().find_map(|a| match a {
            syn::GenericArgument::Type(t) => Some(t),
            _ => None,
        }),
        _ => None,
    }
}

/// Classify `ArcSwapAny<P>` payloads: `Arc<T>` → bare ArcSwap (`ArcSwap<T>`), `Option<Arc<T>>` → ArcSwapOption.
fn arc_swap_any_payload_kind(payload: &Type) -> Option<(WrapperKind, Type)> {
    let syn::Type::Path(tp) = payload else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    match seg.ident.to_string().as_str() {
        "Arc" => {
            let t = angle_bracket_first_type(&seg.arguments)?.clone();
            Some((WrapperKind::BareArcSwap, t))
        }
        "Option" => {
            let opt_inner = angle_bracket_first_type(&seg.arguments)?;
            let syn::Type::Path(tp2) = opt_inner else {
                return None;
            };
            let seg2 = tp2.path.segments.last()?;
            if seg2.ident == "Arc" {
                let t = angle_bracket_first_type(&seg2.arguments)?.clone();
                Some((WrapperKind::BareArcSwapOption, t))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// First path segment before `ArcSwap` / `ArcSwapOption` in `Arc<…>` or `Option<Arc<…>>` fields.
fn arc_swap_crate_ident_from_container_ty(ty: &syn::Type) -> syn::Ident {
    fn peel(ty: &syn::Type) -> Option<syn::Ident> {
        let syn::Type::Path(tp) = ty else {
            return None;
        };
        let seg = tp.path.segments.last()?;
        match seg.ident.to_string().as_str() {
            "Option" => {
                let inner = angle_bracket_first_type(&seg.arguments)?;
                peel(inner)
            }
            "Arc" => {
                let inner = angle_bracket_first_type(&seg.arguments)?;
                let syn::Type::Path(inner_tp) = inner else {
                    return None;
                };
                let inner_seg = inner_tp.path.segments.last()?;
                if matches!(
                    inner_seg.ident.to_string().as_str(),
                    "ArcSwap" | "ArcSwapOption" | "ArcSwapAny"
                ) {
                    inner_tp.path.segments.first().map(|s| s.ident.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    peel(ty).unwrap_or_else(|| format_ident!("arc_swap"))
}

/// Helper function to check if a type path is under std::sync::atomic (strict prefix).
fn is_std_sync_atomic_type(path: &syn::Path) -> bool {
    let segments: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments.len() >= 3
        && segments.get(0).map(|s| s.as_str()) == Some("std")
        && segments.get(1).map(|s| s.as_str()) == Some("sync")
        && segments.get(2).map(|s| s.as_str()) == Some("atomic")
}

/// Atomic type idents (no type params): AtomicBool, AtomicI8, etc.
const ATOMIC_TYPE_IDENTS: &[&str] = &[
    "AtomicBool",
    "AtomicI8",
    "AtomicI16",
    "AtomicI32",
    "AtomicI64",
    "AtomicI128",
    "AtomicIsize",
    "AtomicU8",
    "AtomicU16",
    "AtomicU32",
    "AtomicU64",
    "AtomicU128",
    "AtomicUsize",
];

fn extract_wrapper_inner_type(ty: &Type) -> (WrapperKind, Option<Type>) {
    use syn::{GenericArgument, PathArguments};

    // Handle reference types: &T, &'a str, &[T], etc.
    if let Type::Reference(tr) = ty {
        return (WrapperKind::Reference, Some((*tr.elem).clone()));
    }

    if let Type::Path(tp) = ty {
        // Check if this is explicitly a std::sync type
        let is_std_sync = is_std_sync_type(&tp.path);
        // Check if this is explicitly a tokio::sync type
        let is_tokio_sync = is_tokio_sync_type(&tp.path);

        if let Some(seg) = tp.path.segments.last() {
            let ident_str = seg.ident.to_string();

            if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                let args: Vec<_> = ab.args.iter().collect();

                // Handle map types (HashMap, BTreeMap) - they have K, V parameters
                if ident_str == "HashMap" || ident_str == "BTreeMap" {
                    if let (Some(_key_arg), Some(value_arg)) = (args.get(0), args.get(1)) {
                        if let GenericArgument::Type(inner) = value_arg {
                            // Check for nested Option in map values
                            let (inner_kind, inner_inner) = extract_wrapper_inner_type(inner);
                            match (ident_str.as_str(), inner_kind) {
                                ("HashMap", WrapperKind::Option) => {
                                    return (WrapperKind::HashMapOption, inner_inner);
                                }
                                ("BTreeMap", WrapperKind::Option) => {
                                    return (WrapperKind::BTreeMapOption, inner_inner);
                                }
                                _ => {
                                    return match ident_str.as_str() {
                                        "HashMap" => (WrapperKind::HashMap, Some(inner.clone())),
                                        "BTreeMap" => (WrapperKind::BTreeMap, Some(inner.clone())),
                                        _ => (WrapperKind::None, None),
                                    };
                                }
                            }
                        }
                    }
                }
                // Handle Cow<'a, B> - has lifetime then type parameter
                else if ident_str == "Cow" {
                    if let Some(inner) = args.iter().find_map(|arg| {
                        if let GenericArgument::Type(t) = arg {
                            Some(t.clone())
                        } else {
                            None
                        }
                    }) {
                        return (WrapperKind::Cow, Some(inner));
                    }
                }
                // Handle single-parameter container types
                else if let Some(arg) = args.get(0) {
                    if let GenericArgument::Type(inner) = arg {
                        // Check for nested containers first
                        let (inner_kind, inner_inner) = extract_wrapper_inner_type(inner);

                        // Handle nested combinations
                        match (ident_str.as_str(), inner_kind) {
                            ("Option", WrapperKind::Box) => {
                                return (WrapperKind::OptionBox, inner_inner);
                            }
                            ("Option", WrapperKind::Rc) => {
                                return (WrapperKind::OptionRc, inner_inner);
                            }
                            ("Option", WrapperKind::ArcArcSwap) => {
                                return (WrapperKind::OptionArcArcSwap, inner_inner);
                            }
                            ("Option", WrapperKind::ArcArcSwapOption) => {
                                return (WrapperKind::OptionArcArcSwapOption, inner_inner);
                            }
                            ("Option", WrapperKind::Arc) => {
                                return (WrapperKind::OptionArc, inner_inner);
                            }
                            ("Option", WrapperKind::Vec) => {
                                return (WrapperKind::OptionVec, inner_inner);
                            }
                            ("Option", WrapperKind::HashMap) => {
                                return (WrapperKind::OptionHashMap, inner_inner);
                            }
                            ("Option", WrapperKind::BTreeMap) => {
                                return (WrapperKind::OptionBTreeMap, inner_inner);
                            }
                            ("Option", WrapperKind::VecDeque) => {
                                return (WrapperKind::OptionVecDeque, inner_inner);
                            }
                            ("Option", WrapperKind::LinkedList) => {
                                return (WrapperKind::OptionLinkedList, inner_inner);
                            }
                            ("Option", WrapperKind::BinaryHeap) => {
                                return (WrapperKind::OptionBinaryHeap, inner_inner);
                            }
                            ("Option", WrapperKind::HashSet) => {
                                return (WrapperKind::OptionHashSet, inner_inner);
                            }
                            ("Option", WrapperKind::BTreeSet) => {
                                return (WrapperKind::OptionBTreeSet, inner_inner);
                            }
                            ("Option", WrapperKind::Result) => {
                                return (WrapperKind::OptionResult, inner_inner);
                            }
                            ("Option", WrapperKind::StdArcMutex) => {
                                return (WrapperKind::OptionStdArcMutex, inner_inner);
                            }
                            ("Option", WrapperKind::StdArcRwLock) => {
                                return (WrapperKind::OptionStdArcRwLock, inner_inner);
                            }
                            ("Option", WrapperKind::ArcMutex) => {
                                return (WrapperKind::OptionArcMutex, inner_inner);
                            }
                            ("Option", WrapperKind::ArcRwLock) => {
                                return (WrapperKind::OptionArcRwLock, inner_inner);
                            }
                            ("Option", WrapperKind::StdMutex) => {
                                return (WrapperKind::OptionStdMutex, inner_inner);
                            }
                            ("Option", WrapperKind::StdRwLock) => {
                                return (WrapperKind::OptionStdRwLock, inner_inner);
                            }
                            ("Option", WrapperKind::Mutex) => {
                                return (WrapperKind::OptionMutex, inner_inner);
                            }
                            ("Option", WrapperKind::RwLock) => {
                                return (WrapperKind::OptionRwLock, inner_inner);
                            }
                            ("Option", WrapperKind::TokioArcMutex) => {
                                return (WrapperKind::OptionTokioArcMutex, inner_inner);
                            }
                            ("Option", WrapperKind::TokioArcRwLock) => {
                                return (WrapperKind::OptionTokioArcRwLock, inner_inner);
                            }
                            ("Option", WrapperKind::Cow) => {
                                return (WrapperKind::OptionCow, inner_inner);
                            }
                            ("Option", WrapperKind::Tagged) => {
                                return (WrapperKind::OptionTagged, inner_inner);
                            }
                            ("Option", WrapperKind::Reference) => {
                                return (WrapperKind::OptionReference, Some(inner.clone()));
                            }
                            ("Option", WrapperKind::Atomic) => {
                                return (WrapperKind::OptionAtomic, Some(inner.clone()));
                            }
                            ("Option", WrapperKind::String) => {
                                return (WrapperKind::OptionString, None);
                            }
                            ("Option", WrapperKind::Cell) => {
                                return (WrapperKind::OptionCell, inner_inner);
                            }
                            ("Option", WrapperKind::RefCell) => {
                                return (WrapperKind::OptionRefCell, inner_inner);
                            }
                            ("Option", WrapperKind::OnceCell) => {
                                return (WrapperKind::OptionOnceCell, inner_inner);
                            }
                            ("Option", WrapperKind::Lazy) => {
                                return (WrapperKind::OptionLazy, inner_inner);
                            }
                            ("Option", WrapperKind::PhantomData) => {
                                return (WrapperKind::OptionPhantomData, inner_inner);
                            }
                            ("Option", WrapperKind::Range) => {
                                return (WrapperKind::OptionRange, inner_inner);
                            }
                            ("Pin", WrapperKind::Box) => {
                                return (WrapperKind::PinBox, inner_inner);
                            }
                            ("Box", WrapperKind::Option) => {
                                return (WrapperKind::BoxOption, inner_inner);
                            }
                            ("Rc", WrapperKind::Option) => {
                                return (WrapperKind::RcOption, inner_inner);
                            }
                            ("Arc", WrapperKind::Option) => {
                                return (WrapperKind::ArcOption, inner_inner);
                            }
                            ("Vec", WrapperKind::Option) => {
                                return (WrapperKind::VecOption, inner_inner);
                            }
                            ("VecDeque", WrapperKind::Option) => {
                                return (WrapperKind::VecDequeOption, inner_inner);
                            }
                            ("LinkedList", WrapperKind::Option) => {
                                return (WrapperKind::LinkedListOption, inner_inner);
                            }
                            ("BinaryHeap", WrapperKind::Option) => {
                                return (WrapperKind::BinaryHeapOption, inner_inner);
                            }
                            ("HashSet", WrapperKind::Option) => {
                                return (WrapperKind::HashSetOption, inner_inner);
                            }
                            ("BTreeSet", WrapperKind::Option) => {
                                return (WrapperKind::BTreeSetOption, inner_inner);
                            }
                            ("Result", WrapperKind::Option) => {
                                return (WrapperKind::ResultOption, inner_inner);
                            }
                            ("HashMap", WrapperKind::Option) => {
                                return (WrapperKind::HashMapOption, inner_inner);
                            }
                            // BTreeMapOption is handled in the map block (HashMap/BTreeMap)
                            // Mutex<Option<T>> / RwLock<Option<T>> (yields SyncKp value T)
                            // std::sync::Mutex<Option<T>> / RwLock<Option<T>>
                            ("Mutex", WrapperKind::Option) if is_std_sync_type(&tp.path) => {
                                return (WrapperKind::StdMutexOption, inner_inner);
                            }
                            ("RwLock", WrapperKind::Option) if is_std_sync_type(&tp.path) => {
                                return (WrapperKind::StdRwLockOption, inner_inner);
                            }
                            // parking_lot::Mutex<Option<T>> / RwLock<Option<T>>, and bare Mutex/RwLock assumed parking_lot
                            ("Mutex", WrapperKind::Option) => {
                                return (WrapperKind::MutexOption, inner_inner);
                            }
                            ("RwLock", WrapperKind::Option) => {
                                return (WrapperKind::RwLockOption, inner_inner);
                            }
                            // std::sync variants (when inner is StdMutex/StdRwLock)
                            ("Arc", WrapperKind::StdMutex) => {
                                return (WrapperKind::StdArcMutex, inner_inner);
                            }
                            ("Arc", WrapperKind::StdRwLock) => {
                                return (WrapperKind::StdArcRwLock, inner_inner);
                            }
                            ("Arc", WrapperKind::StdMutexOption) => {
                                return (WrapperKind::StdArcMutexOption, inner_inner);
                            }
                            ("Arc", WrapperKind::StdRwLockOption) => {
                                return (WrapperKind::StdArcRwLockOption, inner_inner);
                            }
                            // parking_lot variants (default - when inner is Mutex/RwLock without std::sync prefix)
                            ("Arc", WrapperKind::Mutex) => {
                                return (WrapperKind::ArcMutex, inner_inner);
                            }
                            ("Arc", WrapperKind::RwLock) => {
                                return (WrapperKind::ArcRwLock, inner_inner);
                            }
                            ("Arc", WrapperKind::MutexOption) => {
                                return (WrapperKind::ArcMutexOption, inner_inner);
                            }
                            ("Arc", WrapperKind::RwLockOption) => {
                                return (WrapperKind::ArcRwLockOption, inner_inner);
                            }
                            // tokio::sync variants (when inner is TokioMutex/TokioRwLock)
                            ("Arc", WrapperKind::TokioMutex) => {
                                return (WrapperKind::TokioArcMutex, inner_inner);
                            }
                            ("Arc", WrapperKind::TokioRwLock) => {
                                return (WrapperKind::TokioArcRwLock, inner_inner);
                            }
                            ("Arc", WrapperKind::BareArcSwap) => {
                                return (WrapperKind::ArcArcSwap, inner_inner);
                            }
                            ("Arc", WrapperKind::BareArcSwapOption) => {
                                return (WrapperKind::ArcArcSwapOption, inner_inner);
                            }
                            _ => {
                                // Handle single-level containers
                                // For Mutex and RwLock:
                                // - If path contains std::sync, it's std::sync (StdMutex/StdRwLock)
                                // - Otherwise, default to parking_lot (Mutex/RwLock)
                                return match ident_str.as_str() {
                                    "Option" => (WrapperKind::Option, Some(inner.clone())),
                                    "Box" => (WrapperKind::Box, Some(inner.clone())),
                                    "Rc" => (WrapperKind::Rc, Some(inner.clone())),
                                    "Arc" => (WrapperKind::Arc, Some(inner.clone())),
                                    "Vec" => (WrapperKind::Vec, Some(inner.clone())),
                                    "HashSet" => (WrapperKind::HashSet, Some(inner.clone())),
                                    "BTreeSet" => (WrapperKind::BTreeSet, Some(inner.clone())),
                                    "VecDeque" => (WrapperKind::VecDeque, Some(inner.clone())),
                                    "LinkedList" => (WrapperKind::LinkedList, Some(inner.clone())),
                                    "BinaryHeap" => (WrapperKind::BinaryHeap, Some(inner.clone())),
                                    "Result" => (WrapperKind::Result, Some(inner.clone())),
                                    // Explicit parking_lot path (lock_api::RwLock) — never treat as std
                                    "Mutex" if is_parking_lot_type(&tp.path) => {
                                        (WrapperKind::Mutex, Some(inner.clone()))
                                    }
                                    "RwLock" if is_parking_lot_type(&tp.path) => {
                                        (WrapperKind::RwLock, Some(inner.clone()))
                                    }
                                    // std::sync::Mutex and std::sync::RwLock only when path starts with std::sync
                                    "Mutex" if is_std_sync => {
                                        (WrapperKind::StdMutex, Some(inner.clone()))
                                    }
                                    "RwLock" if is_std_sync => {
                                        (WrapperKind::StdRwLock, Some(inner.clone()))
                                    }
                                    // tokio::sync::Mutex and tokio::sync::RwLock
                                    "Mutex" if is_tokio_sync => {
                                        (WrapperKind::TokioMutex, Some(inner.clone()))
                                    }
                                    "RwLock" if is_tokio_sync => {
                                        (WrapperKind::TokioRwLock, Some(inner.clone()))
                                    }
                                    // Default: parking_lot (bare Mutex/RwLock or unknown path)
                                    "Mutex" => (WrapperKind::Mutex, Some(inner.clone())),
                                    "RwLock" => (WrapperKind::RwLock, Some(inner.clone())),
                                    "ArcSwap" if is_arcswap_crate_path(&tp.path) => {
                                        (WrapperKind::BareArcSwap, Some(inner.clone()))
                                    }
                                    "ArcSwapOption" if is_arcswap_crate_path(&tp.path) => {
                                        (WrapperKind::BareArcSwapOption, Some(inner.clone()))
                                    }
                                    "ArcSwapAny" if is_arcswap_crate_path(&tp.path) => {
                                        if let Some((kind, peeled)) =
                                            arc_swap_any_payload_kind(inner)
                                        {
                                            (kind, Some(peeled))
                                        } else {
                                            (WrapperKind::None, None)
                                        }
                                    }
                                    "Weak" => (WrapperKind::Weak, Some(inner.clone())),
                                    "Tagged" => (WrapperKind::Tagged, Some(inner.clone())),
                                    "Cow" => (WrapperKind::Cow, Some(inner.clone())),
                                    "AtomicPtr" if is_std_sync_atomic_type(&tp.path) => {
                                        (WrapperKind::Atomic, None)
                                    }
                                    "Pin" => (WrapperKind::Pin, Some(inner.clone())),
                                    "Cell" => (WrapperKind::Cell, Some(inner.clone())),
                                    "RefCell" => (WrapperKind::RefCell, Some(inner.clone())),
                                    "OnceCell" | "OnceLock" => {
                                        (WrapperKind::OnceCell, Some(inner.clone()))
                                    }
                                    "Lazy" | "LazyLock" => (WrapperKind::Lazy, Some(inner.clone())),
                                    "PhantomData" => {
                                        (WrapperKind::PhantomData, Some(inner.clone()))
                                    }
                                    "Range" | "RangeInclusive" => {
                                        (WrapperKind::Range, Some(inner.clone()))
                                    }
                                    _ => (WrapperKind::None, None),
                                };
                            }
                        }
                    }
                }
            }
            // Handle atomic types with no angle bracket args (AtomicBool, AtomicI32, etc.)
            if matches!(seg.arguments, PathArguments::None) {
                if ident_str == "String" {
                    return (WrapperKind::String, None);
                }
                if is_std_sync_atomic_type(&tp.path)
                    && ATOMIC_TYPE_IDENTS.contains(&ident_str.as_str())
                {
                    return (WrapperKind::Atomic, None);
                }
            }
        }
    }
    (WrapperKind::None, None)
}

/// Check if a field has the #[pin] attribute (pin_project pattern).
fn field_has_pin_attr(field: &syn::Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| attr.path().get_ident().map(|i| i == "pin").unwrap_or(false))
}

/// Check if a type is a Future (dyn Future, impl Future, or Box<dyn Future>).
fn is_future_type(ty: &Type) -> bool {
    use syn::{GenericArgument, PathArguments, TypeParamBound};

    match ty {
        Type::TraitObject(trait_obj) => trait_obj.bounds.iter().any(|b| {
            if let TypeParamBound::Trait(t) = b {
                t.path
                    .segments
                    .last()
                    .map(|s| s.ident == "Future")
                    .unwrap_or(false)
            } else {
                false
            }
        }),
        Type::ImplTrait(impl_trait) => impl_trait.bounds.iter().any(|b| {
            if let TypeParamBound::Trait(t) = b {
                t.path
                    .segments
                    .last()
                    .map(|s| s.ident == "Future")
                    .unwrap_or(false)
            } else {
                false
            }
        }),
        Type::Path(tp) => {
            if let Some(seg) = tp.path.segments.last() {
                match seg.ident.to_string().as_str() {
                    "Box" | "Pin" => {
                        if let PathArguments::AngleBracketed(args) = &seg.arguments {
                            if let Some(GenericArgument::Type(inner)) = args.args.first() {
                                return is_future_type(inner);
                            }
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        _ => false,
    }
}

/// Extract Output type from Future trait bound (dyn Future<Output = T>, impl Future<Output = T>, etc.).
fn extract_future_output(ty: &Type) -> Option<Type> {
    use syn::{GenericArgument, PathArguments, TypeParamBound};

    let bounds = match ty {
        Type::TraitObject(t) => &t.bounds,
        Type::ImplTrait(t) => &t.bounds,
        Type::Path(tp) => {
            if let Some(seg) = tp.path.segments.last() {
                if matches!(seg.ident.to_string().as_str(), "Box" | "Pin") {
                    if let PathArguments::AngleBracketed(args) = &seg.arguments {
                        if let Some(GenericArgument::Type(inner)) = args.args.first() {
                            return extract_future_output(inner);
                        }
                    }
                }
            }
            return None;
        }
        _ => return None,
    };

    for bound in bounds {
        if let TypeParamBound::Trait(trait_bound) = bound {
            if let Some(seg) = trait_bound.path.segments.last() {
                if seg.ident == "Future" {
                    if let PathArguments::AngleBracketed(args) = &seg.arguments {
                        for arg in &args.args {
                            if let GenericArgument::AssocType(assoc) = arg {
                                if assoc.ident == "Output" {
                                    return Some(assoc.ty.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// For HashMap<K,V> or BTreeMap<K,V>, returns Some((key_ty, value_ty)).
fn extract_map_key_value(ty: &Type) -> Option<(Type, Type)> {
    use syn::{GenericArgument, PathArguments};

    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            let ident_str = seg.ident.to_string();
            if ident_str == "HashMap" || ident_str == "BTreeMap" {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    let args: Vec<_> = ab.args.iter().collect();
                    if let (Some(key_arg), Some(value_arg)) = (args.get(0), args.get(1)) {
                        if let (GenericArgument::Type(key_ty), GenericArgument::Type(value_ty)) =
                            (key_arg, value_arg)
                        {
                            return Some((key_ty.clone(), value_ty.clone()));
                        }
                    }
                }
            }
        }
    }
    None
}

/// For `Option<HashMap<K,V>>` / `Option<BTreeMap<K,V>>`, returns `Some((key_ty, value_ty))`.
fn extract_map_key_value_through_option(ty: &Type) -> Option<(Type, Type)> {
    use syn::{GenericArgument, PathArguments};

    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Option" {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    if let Some(GenericArgument::Type(inner)) = ab.args.first() {
                        return extract_map_key_value(inner);
                    }
                }
            }
        }
    }
    None
}

fn to_snake_case(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

// Full `SyncKp<…>` return types: borrow slots use `&'static` / `&'static mut`; closure slots use
// elided references in `impl Fn(…)`. Still paired with `fn …<'b>()` in generated code.
// HRTB on `G` / `S` uses `for<'c> fn(…)` so it does not clash with reference lifetimes in `SyncKp`.

fn kp_lock_ty_arc_mutex(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            #inner,
            #inner,
            &'static #root,
            &'static #lock,
            &'static #inner,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut #inner,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ArcMutexAccess<#inner>,
            impl Fn(& #inner) -> Option<& #inner>,
            impl Fn(&mut #inner) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_arc_mutex_option(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            Option<#inner>,
            #inner,
            &'static #root,
            &'static #lock,
            &'static Option<#inner>,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut Option<#inner>,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ArcMutexAccess<Option<#inner>>,
            impl Fn(& Option<#inner>) -> Option<& #inner>,
            impl Fn(&mut Option<#inner>) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_arc_rw_lock(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            #inner,
            #inner,
            &'static #root,
            &'static #lock,
            &'static #inner,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut #inner,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ArcRwLockAccess<#inner>,
            impl Fn(& #inner) -> Option<& #inner>,
            impl Fn(&mut #inner) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_arc_rw_lock_option(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            Option<#inner>,
            #inner,
            &'static #root,
            &'static #lock,
            &'static Option<#inner>,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut Option<#inner>,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ArcRwLockAccess<Option<#inner>>,
            impl Fn(& Option<#inner>) -> Option<& #inner>,
            impl Fn(&mut Option<#inner>) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_arc_arc_swap(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            #inner,
            #inner,
            &'static #root,
            &'static #lock,
            &'static #inner,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut #inner,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ArcArcSwapAccess<#inner>,
            impl Fn(& #inner) -> Option<& #inner>,
            impl Fn(&mut #inner) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_arc_arc_swap_option(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            ::std::option::Option<::std::sync::Arc<#inner>>,
            #inner,
            &'static #root,
            &'static #lock,
            &'static ::std::option::Option<::std::sync::Arc<#inner>>,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut ::std::option::Option<::std::sync::Arc<#inner>>,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ArcArcSwapOptionAccess<#inner>,
            impl Fn(& ::std::option::Option<::std::sync::Arc<#inner>>) -> Option<& #inner>,
            impl Fn(&mut ::std::option::Option<::std::sync::Arc<#inner>>) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_parking_lot_mutex(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            #inner,
            #inner,
            &'static #root,
            &'static #lock,
            &'static #inner,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut #inner,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ParkingLotMutexAccess<#inner>,
            impl Fn(& #inner) -> Option<& #inner>,
            impl Fn(&mut #inner) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_parking_lot_mutex_option(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            Option<#inner>,
            #inner,
            &'static #root,
            &'static #lock,
            &'static Option<#inner>,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut Option<#inner>,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ParkingLotMutexAccess<Option<#inner>>,
            impl Fn(& Option<#inner>) -> Option<& #inner>,
            impl Fn(&mut Option<#inner>) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_parking_lot_rw_lock(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            #inner,
            #inner,
            &'static #root,
            &'static #lock,
            &'static #inner,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut #inner,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ParkingLotRwLockAccess<#inner>,
            impl Fn(& #inner) -> Option<& #inner>,
            impl Fn(&mut #inner) -> Option<&mut #inner>,
        >
    }
}

fn kp_lock_ty_parking_lot_rw_lock_option(
    root: &impl ToTokens,
    lock: &impl ToTokens,
    inner: &impl ToTokens,
) -> proc_macro2::TokenStream {
    quote! {
        rust_key_paths::sync_kp::SyncKp<
            #root,
            #lock,
            Option<#inner>,
            #inner,
            &'static #root,
            &'static #lock,
            &'static Option<#inner>,
            &'static #inner,
            &'static mut #root,
            &'static mut #lock,
            &'static mut Option<#inner>,
            &'static mut #inner,
            impl Fn(& #root) -> Option<& #lock>,
            impl Fn(&mut #root) -> Option<&mut #lock>,
            rust_key_paths::sync_kp::ParkingLotRwLockAccess<Option<#inner>>,
            impl Fn(& Option<#inner>) -> Option<& #inner>,
            impl Fn(&mut Option<#inner>) -> Option<&mut #inner>,
        >
    }
}

/// Derive macro for generating simple keypath methods.
///
/// Generates one method per field: `StructName::field_name()` that returns a `Kp`.
/// Intelligently handles wrapper types (Option, Vec, Box, Arc, etc.) to generate appropriate keypaths.
///
/// # Example
///
/// ```ignore
/// #[derive(Kp)]
/// struct Person {
///     name: String,
///     age: i32,
///     email: Option<String>,
///     addresses: Vec<String>,
/// }
///
/// // Generates:
/// // impl Person {
/// //     pub fn name() -> Kp<...> { ... }
/// //     pub fn age() -> Kp<...> { ... }
/// //     pub fn email() -> Kp<...> { ... } // unwraps Option
/// //     pub fn addresses() -> Kp<...> { ... } // accesses first element
/// // }
/// ```
#[proc_macro_derive(Kp)]
pub fn derive_keypaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let input_span = input.span();

    let methods = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => {
                let mut tokens = proc_macro2::TokenStream::new();

                // Generate identity methods for the struct
                tokens.extend(quote! {
                    /// Returns a generic identity keypath for this type
                    #[inline(always)]
                    pub fn _identity<'a, Root, MutRoot>() -> rust_key_paths::Kp<
                        #name,
                        #name,
                        Root,
                        Root,
                        MutRoot,
                        MutRoot,
                        fn(Root) -> Option<Root>,
                        fn(MutRoot) -> Option<MutRoot>,
                    >
                    where
                        Root: std::borrow::Borrow<#name>,
                        MutRoot: std::borrow::BorrowMut<#name>,
                    {
                        rust_key_paths::Kp::new(
                            |r: Root| Some(r),
                            |r: MutRoot| Some(r)
                        )
                    }

                    // /// Returns a simple identity keypath for this type
                    // #[inline(always)]
                    // pub fn identity() -> rust_key_paths::Kp<#name, #name, &'static #name, &'static #name, &'static mut #name, &'static mut #name, impl Fn(&#name) -> Option<&#name>, impl Fn(&mut #name) -> Option<&mut #name>,> {
                    //     rust_key_paths::Kp::new(
                    //         |r: &#name| Some(r),
                    //         |r: &mut #name| Some(r)
                    //     )
                    // }
                });

                // When struct has #[pin] fields, generated code calls this.project() which must
                // be provided by #[pin_project]. If missing, user gets: no method named `project`.

                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    // Centralized keypath method names – change here to adjust for all types
                    let kp_fn = format_ident!("{}", field_ident);
                    let kp_at_fn = format_ident!("{}_at", field_ident);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    // Override kind when field has #[pin] (pin_project pattern)
                    let (kind, inner_ty) = if field_has_pin_attr(field) {
                        let pinned_kind = if let Some(output_ty) = extract_future_output(ty) {
                            if matches!(kind, WrapperKind::Box) {
                                (WrapperKind::PinnedBoxFuture, Some(output_ty))
                            } else {
                                (WrapperKind::PinnedFuture, Some(output_ty))
                            }
                        } else if is_future_type(ty) {
                            (WrapperKind::PinnedFuture, inner_ty.clone())
                        } else {
                            (WrapperKind::PinnedField, inner_ty.clone())
                        };
                        pinned_kind
                    } else {
                        (kind, inner_ty.clone())
                    };

                    match (kind, inner_ty) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            // For Option<T>, unwrap and access inner type
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionBox, Some(inner_ty)) => {
                            // For Option<Box<T>>, keypath to T: get returns Option<&T> via as_deref()
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_deref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_deref_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionRc, Some(inner_ty)) => {
                            // For Option<Rc<T>>, keypath to T: get returns Option<&T> via as_deref()
                            // Setter: as_mut() gives Option<&mut Rc<T>>, and_then(Rc::get_mut) gives Option<&mut T>
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_deref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut().and_then(std::rc::Rc::get_mut)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionArc, Some(inner_ty)) => {
                            // For Option<Arc<T>>, keypath to T: get returns Option<&T> via as_deref()
                            // Setter: as_mut() gives Option<&mut Arc<T>>, and_then(Arc::get_mut) gives Option<&mut T>
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_deref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut().and_then(std::sync::Arc::get_mut)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionHashMap, Some(inner_ty)) => {
                            if let Some((key_ty, _)) = extract_map_key_value_through_option(ty) {
                                let type_name = name.to_string();
                                let whole_fn = kp_fn.to_string();
                                let at_fn = kp_at_fn.to_string();
                                let whole_doc = format!(
                                    "Keypath to the whole optional `HashMap` (`{type_name}::{whole_fn}`). For a value at one key when the map is `Some`, use `{type_name}::{at_fn}(key)`."
                                );
                                let at_doc = format!(
                                    "Keyed access into the inner `HashMap` (`{type_name}::{at_fn}`). `get`/`get_mut` return `None` if the field is `None` or the key is missing. See also `{type_name}::{whole_fn}` for the full optional map."
                                );
                                tokens.extend(quote! {
                                    #[doc = #whole_doc]
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        )
                                    }
                                    #[doc = #at_doc]
                                    #[inline(always)]
                                    pub fn #kp_at_fn(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                    where
                                        #key_ty: Clone + std::hash::Hash + Eq + 'static,
                                    {
                                        let key2 = key.clone();
                                        rust_key_paths::Kp::new(
                                            Box::new(move |root: &#name| root.#field_ident.as_ref().and_then(|m| m.get(&key))),
                                            Box::new(move |root: &mut #name| root.#field_ident.as_mut().and_then(|m| m.get_mut(&key2))),
                                        )
                                    }
                                });
                            } else {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        )
                                    }
                                });
                            }
                        }
                        (WrapperKind::OptionBTreeMap, Some(inner_ty)) => {
                            if let Some((key_ty, _)) = extract_map_key_value_through_option(ty) {
                                let type_name = name.to_string();
                                let whole_fn = kp_fn.to_string();
                                let at_fn = kp_at_fn.to_string();
                                let whole_doc = format!(
                                    "Keypath to the whole optional `BTreeMap` (`{type_name}::{whole_fn}`). For a value at one key when the map is `Some`, use `{type_name}::{at_fn}(key)`."
                                );
                                let at_doc = format!(
                                    "Keyed access into the inner `BTreeMap` (`{type_name}::{at_fn}`). `get`/`get_mut` return `None` if the field is `None` or the key is missing. See also `{type_name}::{whole_fn}` for the full optional map."
                                );
                                tokens.extend(quote! {
                                    #[doc = #whole_doc]
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        )
                                    }
                                    #[doc = #at_doc]
                                    #[inline(always)]
                                    pub fn #kp_at_fn(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                    where
                                        #key_ty: Clone + Ord + 'static,
                                    {
                                        let key2 = key.clone();
                                        rust_key_paths::Kp::new(
                                            Box::new(move |root: &#name| root.#field_ident.as_ref().and_then(|m| m.get(&key))),
                                            Box::new(move |root: &mut #name| root.#field_ident.as_mut().and_then(|m| m.get_mut(&key2))),
                                        )
                                    }
                                });
                            } else {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        )
                                    }
                                });
                            }
                        }
                        (WrapperKind::OptionHashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }

                                /// _at: check if element exists and get reference.
                                /// HashSet does not allow mutable element access (would break hash invariant).
                                #[inline(always)]
                                pub fn #kp_at_fn(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                where
                                    #inner_ty: Clone + std::hash::Hash + Eq + 'static,
                                {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#field_ident.as_ref().and_then(|s| s.get(&key))),
                                        Box::new(move |_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionBTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }

                                /// _at: check if element exists and get reference.
                                /// BTreeSet does not allow mutable element access (would break ordering invariant).
                                #[inline(always)]
                                pub fn #kp_at_fn(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                where
                                    #inner_ty: Clone + Ord + 'static,
                                {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#field_ident.as_ref().and_then(|s| s.get(&key))),
                                        Box::new(move |_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionVec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_at_fn(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#field_ident.as_ref().and_then(|v| v.get(index))),
                                        Box::new(move |root: &mut #name| root.#field_ident.as_mut().and_then(|v| v.get_mut(index))),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionVecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_at_fn(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#field_ident.as_ref().and_then(|v| v.get(index))),
                                        Box::new(move |root: &mut #name| root.#field_ident.as_mut().and_then(|v| v.get_mut(index))),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionLinkedList, Some(_inner_ty))
                        | (WrapperKind::OptionBinaryHeap, Some(_inner_ty))
                        | (WrapperKind::OptionResult, Some(_inner_ty)) => {
                            // Keypath to the Option container (reference), like Vec/HashSet
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_at_fn(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#field_ident.get(index)),
                                        Box::new(move |root: &mut #name| root.#field_ident.get_mut(index)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            if let Some((key_ty, _)) = extract_map_key_value(ty) {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        )
                                    }
                                    #[inline(always)]
                                    pub fn #kp_at_fn(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                    where
                                        #key_ty: Clone + std::hash::Hash + Eq + 'static,
                                    {
                                        let key2 = key.clone();
                                        rust_key_paths::Kp::new(
                                            Box::new(move |root: &#name| root.#field_ident.get(&key)),
                                            Box::new(move |root: &mut #name| root.#field_ident.get_mut(&key2)),
                                        )
                                    }
                                });
                            } else {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        )
                                    }
                                });
                            }
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty))
                        | (WrapperKind::BTreeMapOption, Some(inner_ty)) => {
                            if let Some((key_ty, _)) = extract_map_key_value(ty) {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        )
                                    }
                                    #[inline(always)]
                                    pub fn #kp_at_fn(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                    where
                                        #key_ty: Clone + Ord + 'static,
                                    {
                                        let key2 = key.clone();
                                        rust_key_paths::Kp::new(
                                            Box::new(move |root: &#name| root.#field_ident.get(&key)),
                                            Box::new(move |root: &mut #name| root.#field_ident.get_mut(&key2)),
                                        )
                                    }
                                });
                            } else {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        )
                                    }
                                });
                            }
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            // For Box<T>, deref to inner type (returns &T / &mut T, not &Box<T>)
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&*root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut *root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BoxOption, Some(inner_ty)) => {
                            // For Box<Option<T>>, keypath to T: deref Box to Option<T>, then Option::as_ref/as_mut
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| (&*root.#field_ident).as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| (&mut *root.#field_ident).as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::RcOption, Some(inner_ty)) => {
                            // For Rc<Option<T>>, keypath to T: deref Rc to &Option<T>, then Option::as_ref; set = Rc::get_mut then as_mut
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| (&*root.#field_ident).as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| std::rc::Rc::get_mut(&mut root.#field_ident).and_then(std::option::Option::as_mut)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::ArcOption, Some(inner_ty)) => {
                            // For Arc<Option<T>>, keypath to T: deref Arc to &Option<T>, then Option::as_ref; set = Arc::get_mut then as_mut
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| (&*root.#field_ident).as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| std::sync::Arc::get_mut(&mut root.#field_ident).and_then(std::option::Option::as_mut)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Pin, Some(inner_ty)) => {
                            let kp_inner_fn = format_ident!("{}_inner", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_inner_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
>
>
                                where #inner_ty: std::marker::Unpin
                                {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(std::pin::Pin::as_ref(&root.#field_ident).get_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(std::pin::Pin::as_mut(&mut root.#field_ident).get_mut())),
                                    )
                                }
                            });
                        }
                        (WrapperKind::PinBox, Some(inner_ty)) => {
                            let kp_inner_fn = format_ident!("{}_inner", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_inner_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
>
>
                                where #inner_ty: std::marker::Unpin
                                {
                                    // Pin::as_ref on Pin<Box<T>> returns Pin<&T> (Box Deref target), so get_ref() already gives &T
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(std::pin::Pin::as_ref(&root.#field_ident).get_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(std::pin::Pin::as_mut(&mut root.#field_ident).get_mut())),
                                    )
                                }
                            });
                        }
                        (WrapperKind::PinnedField, _) => {
                            let kp_pinned_fn = format_ident!("{}_pinned", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                /// Pinned projection for #[pin] field. Requires #[pin_project] on struct.
                                #[inline(always)]
                                pub fn #kp_pinned_fn(this: std::pin::Pin<&mut #name>) -> std::pin::Pin<&mut #ty> {
                                    this.project().#field_ident
                                }
                            });
                        }
                        (WrapperKind::PinnedFuture, _) => {
                            let kp_pinned_fn = format_ident!("{}_pinned", field_ident);
                            let kp_await_fn = format_ident!("{}_await", field_ident);
                            let kp_pin_future_fn = format_ident!("{}_pin_future_kp", field_ident);
                            let output_ty = quote! { <#ty as std::future::Future>::Output };
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                /// Pinned projection for #[pin] Future field. Requires #[pin_project] on struct.
                                #[inline(always)]
                                pub fn #kp_pinned_fn(this: std::pin::Pin<&mut #name>) -> std::pin::Pin<&mut #ty> {
                                    this.project().#field_ident
                                }
                                /// Poll the pinned future. Requires #[pin_project] on struct.
                                pub async fn #kp_await_fn(this: std::pin::Pin<&mut #name>) -> Option<#output_ty>
                                where #ty: std::future::Future
                                {
                                    use std::future::Future;
                                    Some(this.project().#field_ident.await)
                                }
                                /// Keypath for [rust_key_paths::Kp::then_pin_future]. Composable pin future await.
                                #[inline(always)]
                                pub fn #kp_pin_future_fn() -> impl rust_key_paths::pin::PinFutureAwaitLike<#name, #output_ty> {
                                    rust_key_paths::pin_future_await_kp!(#name, #kp_await_fn -> #output_ty)
                                }
                            });
                        }
                        (WrapperKind::PinnedBoxFuture, Some(output_ty)) => {
                            let kp_pinned_fn = format_ident!("{}_pinned", field_ident);
                            let kp_await_fn = format_ident!("{}_await", field_ident);
                            let kp_pin_future_fn = format_ident!("{}_pin_future_kp", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                /// Pinned projection for #[pin] Box<dyn Future> field. Requires #[pin_project] on struct.
                                #[inline(always)]
                                pub fn #kp_pinned_fn(this: std::pin::Pin<&mut #name>) -> std::pin::Pin<&mut #ty> {
                                    this.project().#field_ident
                                }
                                /// Poll the pinned boxed future. Requires #[pin_project] on struct.
                                pub async fn #kp_await_fn(this: std::pin::Pin<&mut #name>) -> Option<#output_ty> {
                                    Some(this.project().#field_ident.await)
                                }
                                /// Keypath for [rust_key_paths::Kp::then_pin_future]. Composable pin future await.
                                #[inline(always)]
                                pub fn #kp_pin_future_fn() -> impl rust_key_paths::pin::PinFutureAwaitLike<#name, #output_ty> {
                                    rust_key_paths::pin_future_await_kp!(#name, #kp_await_fn -> #output_ty)
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) => {
                            // For Rc<T>, deref to inner type (returns &T; get_mut when uniquely owned)
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(root.#field_ident.as_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| std::rc::Rc::get_mut(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Arc, Some(inner_ty)) => {
                            // For Arc<T>, deref to inner type (returns &T; get_mut when uniquely owned)
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(root.#field_ident.as_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| std::sync::Arc::get_mut(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Cow, Some(inner_ty)) => {
                            // For Cow<'_, B>, deref to inner type (as_ref/to_mut)
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(root.#field_ident.as_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(root.#field_ident.to_mut())),
                                    )
                                }
                            });
                        }

                        (WrapperKind::OptionCow, Some(inner_ty)) => {
                            // For Option<Cow<'_, B>>
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref().map(|c| c.as_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut().map(|c| c.to_mut())),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionTagged, Some(inner_ty)) => {
                            // For Option<Tagged<Tag, T>> - Tagged implements Deref/DerefMut
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref().map(|t| std::ops::Deref::deref(t))),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut().map(|t| std::ops::DerefMut::deref_mut(t))),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionReference, Some(inner_ty)) => {
                            // For Option<&T>, Option<&str>, Option<&[T]> - read-only, setter returns None
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty))
                        | (WrapperKind::HashSetOption, Some(inner_ty)) => {
                            let kp_at_fn = format_ident!("{}_at", field_ident);

                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }

                                /// _at: check if element exists and get reference.
                                /// HashSet does not allow mutable element access (would break hash invariant).
                                #[inline(always)]
                                pub fn #kp_at_fn(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                where
                                    #inner_ty: Clone + std::hash::Hash + Eq + 'static,
                                {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#field_ident.get(&key)),
                                        Box::new(move |_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty))
                        | (WrapperKind::BTreeSetOption, Some(inner_ty)) => {
                            let kp_at_fn = format_ident!("{}_at", field_ident);

                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }

                                /// _at: check if element exists and get reference.
                                /// BTreeSet does not allow mutable element access (would break ordering invariant).
                                #[inline(always)]
                                pub fn #kp_at_fn(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                where
                                    #inner_ty: Clone + Ord + 'static,
                                {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#field_ident.get(&key)),
                                        Box::new(move |_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty))
                        | (WrapperKind::VecDequeOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_at_fn(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#field_ident.get(index)),
                                        Box::new(move |root: &mut #name| root.#field_ident.get_mut(index)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(_inner_ty))
                        | (WrapperKind::LinkedListOption, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(_inner_ty))
                        | (WrapperKind::BinaryHeapOption, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            // For Result<T, E>, access Ok value
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref().ok()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut().ok()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::StdArcMutex, Some(inner_ty)) => {
                            // For Arc<std::sync::Mutex<T>>
                            let lock_kp_return_ty = kp_lock_ty_arc_mutex(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ArcMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::StdArcRwLock, Some(inner_ty)) => {
                            // For Arc<std::sync::RwLock<T>>
                            let lock_kp_return_ty = kp_lock_ty_arc_rw_lock(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ArcRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::StdArcMutexOption, Some(inner_ty)) => {
                            // For Arc<std::sync::Mutex<Option<T>>> — SyncKp value T (extract from Option); guard gives &Option<T>
                            let lock_kp_return_ty =
                                kp_lock_ty_arc_mutex_option(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ArcMutexAccess::<Option<#inner_ty>>::new(),
                                        rust_key_paths::Kp::new(
                                            Option::<#inner_ty>::as_ref,
                                            Option::<#inner_ty>::as_mut,
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::StdArcRwLockOption, Some(inner_ty)) => {
                            // For Arc<std::sync::RwLock<Option<T>>> — SyncKp value T (extract from Option); guard gives &Option<T>
                            let lock_kp_return_ty =
                                kp_lock_ty_arc_rw_lock_option(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ArcRwLockAccess::<Option<#inner_ty>>::new(),
                                        rust_key_paths::Kp::new(
                                            Option::<#inner_ty>::as_ref,
                                            Option::<#inner_ty>::as_mut,
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BareArcSwap, Some(inner_ty)) => {
                            // `arc_swap::ArcSwap<T>` owned on the struct — `field()` is `SyncKp` only (no `field_kp()`).
                            let lock_kp_return_ty = kp_lock_ty_arc_arc_swap(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ArcArcSwapAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BareArcSwapOption, Some(inner_ty)) => {
                            let lock_kp_return_ty =
                                kp_lock_ty_arc_arc_swap_option(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ArcArcSwapOptionAccess::<#inner_ty>::new(),
                                        rust_key_paths::Kp::new(
                                            |o: &::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                o.as_ref().map(|a| a.as_ref())
                                            },
                                            |o: &mut ::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                o.as_mut().and_then(::std::sync::Arc::get_mut)
                                            },
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::ArcArcSwap, Some(inner_ty)) => {
                            let lock_kp_return_ty = kp_lock_ty_arc_arc_swap(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ArcArcSwapAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::ArcArcSwapOption, Some(inner_ty)) => {
                            let lock_kp_return_ty =
                                kp_lock_ty_arc_arc_swap_option(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ArcArcSwapOptionAccess::<#inner_ty>::new(),
                                        rust_key_paths::Kp::new(
                                            |o: &::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                o.as_ref().map(|a| a.as_ref())
                                            },
                                            |o: &mut ::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                o.as_mut().and_then(::std::sync::Arc::get_mut)
                                            },
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::ArcRwLock, Some(inner_ty)) => {
                            // For Arc<parking_lot::RwLock<T>> (requires rust-key-paths "parking_lot" feature)
                            let lock_kp_return_ty =
                                kp_lock_ty_parking_lot_rw_lock(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ParkingLotRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::ArcMutex, Some(inner_ty)) => {
                            // For Arc<parking_lot::Mutex<T>> (requires rust-key-paths "parking_lot" feature)
                            let lock_kp_return_ty =
                                kp_lock_ty_parking_lot_mutex(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ParkingLotMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::ArcMutexOption, Some(inner_ty)) => {
                            // For Arc<parking_lot::Mutex<Option<T>>> — SyncKp value T (extract from Option); guard gives &Option<T>
                            let lock_kp_return_ty =
                                kp_lock_ty_parking_lot_mutex_option(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ParkingLotMutexAccess::<Option<#inner_ty>>::new(),
                                        rust_key_paths::Kp::new(
                                            Option::<#inner_ty>::as_ref,
                                            Option::<#inner_ty>::as_mut,
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::ArcRwLockOption, Some(inner_ty)) => {
                            // For Arc<parking_lot::RwLock<Option<T>>> — SyncKp value T (extract from Option); guard gives &Option<T>
                            let lock_kp_return_ty =
                                kp_lock_ty_parking_lot_rw_lock_option(name, ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::sync_kp::ParkingLotRwLockAccess::<Option<#inner_ty>>::new(),
                                        rust_key_paths::Kp::new(
                                            Option::<#inner_ty>::as_ref,
                                            Option::<#inner_ty>::as_mut,
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(_inner_ty))
                        | (WrapperKind::StdMutex, Some(_inner_ty)) => {
                            // For Mutex<T>, return keypath to container
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::RwLock, Some(_inner_ty))
                        | (WrapperKind::StdRwLock, Some(_inner_ty)) => {
                            // For RwLock<T>, return keypath to container
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::TokioArcMutex, Some(inner_ty)) => {
                            let kp_async_fn = format_ident!("{}_kp", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_async_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                pub fn #kp_fn() -> rust_key_paths::async_lock::AsyncLockKpMutexFor<#name, #ty, #inner_ty> {
                                    rust_key_paths::async_lock::AsyncLockKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::async_lock::TokioMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::TokioArcRwLock, Some(inner_ty)) => {
                            let kp_async_fn = format_ident!("{}_kp", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_async_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                pub fn #kp_fn() -> rust_key_paths::async_lock::AsyncLockKpRwLockFor<#name, #ty, #inner_ty> {
                                    rust_key_paths::async_lock::AsyncLockKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                        ),
                                        rust_key_paths::async_lock::TokioRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionTokioArcMutex, Some(inner_ty)) => {
                            let kp_async_fn = format_ident!("{}_kp", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_async_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                pub fn #kp_fn() -> rust_key_paths::async_lock::AsyncLockKpMutexFor<#name, std::sync::Arc<tokio::sync::Mutex<#inner_ty>>, #inner_ty> {
                                    rust_key_paths::async_lock::AsyncLockKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                        ),
                                        rust_key_paths::async_lock::TokioMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionTokioArcRwLock, Some(inner_ty)) => {
                            let kp_async_fn = format_ident!("{}_kp", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_async_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                pub fn #kp_fn() -> rust_key_paths::async_lock::AsyncLockKpRwLockFor<#name, std::sync::Arc<tokio::sync::RwLock<#inner_ty>>, #inner_ty> {
                                    rust_key_paths::async_lock::AsyncLockKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                        ),
                                        rust_key_paths::async_lock::TokioRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionStdArcMutex, Some(inner_ty)) => {
                            // let kp_unlocked_fn = format_ident!("{}_unlocked", field_ident);
                            let lock_ty = quote! { std::sync::Arc<std::sync::Mutex<#inner_ty>> };
                            let lock_kp_return_ty = kp_lock_ty_arc_mutex(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                // pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<#name, std::sync::Arc<std::sync::Mutex<#inner_ty>>, &'static #name, &'static std::sync::Arc<std::sync::Mutex<#inner_ty>>, &'static mut #name, &'static mut std::sync::Arc<std::sync::Mutex<#inner_ty>>, impl Fn(&#name) -> Option<&std::sync::Arc<std::sync::Mutex<#inner_ty>>>, impl Fn(&mut #name) -> Option<&mut std::sync::Arc<std::sync::Mutex<#inner_ty>>>,> {
                                //     rust_key_paths::Kp::new(
                                //         |root: &#name| root.#field_ident.as_ref(),
                                //         |root: &mut #name| root.#field_ident.as_mut(),
                                //     )
                                // }
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ArcMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionArcMutex, Some(inner_ty)) => {
                            // let kp_unlocked_fn = format_ident!("{}_unlocked", field_ident);
                            let lock_ty = quote! { std::sync::Arc<parking_lot::Mutex<#inner_ty>> };
                            let lock_kp_return_ty =
                                kp_lock_ty_parking_lot_mutex(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                // pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<#name, std::sync::Arc<parking_lot::Mutex<#inner_ty>>, &'static #name, &'static std::sync::Arc<parking_lot::Mutex<#inner_ty>>, &'static mut #name, &'static mut std::sync::Arc<parking_lot::Mutex<#inner_ty>>, impl Fn(&#name) -> Option<&std::sync::Arc<parking_lot::Mutex<#inner_ty>>>, impl Fn(&mut #name) -> Option<&mut std::sync::Arc<parking_lot::Mutex<#inner_ty>>>,> {
                                //     rust_key_paths::Kp::new(
                                //         |root: &#name| root.#field_ident.as_ref(),
                                //         |root: &mut #name| root.#field_ident.as_mut(),
                                //     )
                                // }
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ParkingLotMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionStdArcRwLock, Some(inner_ty)) => {
                            // let kp_unlocked_fn = format_ident!("{}_unlocked", field_ident);
                            let lock_ty = quote! { std::sync::Arc<std::sync::RwLock<#inner_ty>> };
                            let lock_kp_return_ty =
                                kp_lock_ty_arc_rw_lock(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                // pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<#name, std::sync::Arc<std::sync::RwLock<#inner_ty>>, &'static #name, &'static std::sync::Arc<std::sync::RwLock<#inner_ty>>, &'static mut #name, &'static mut std::sync::Arc<std::sync::RwLock<#inner_ty>>, impl Fn(&#name) -> Option<&std::sync::Arc<std::sync::RwLock<#inner_ty>>>, impl Fn(&mut #name) -> Option<&mut std::sync::Arc<std::sync::RwLock<#inner_ty>>>,> {
                                //     rust_key_paths::Kp::new(
                                //         |root: &#name| root.#field_ident.as_ref(),
                                //         |root: &mut #name| root.#field_ident.as_mut(),
                                //     )
                                // }
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ArcRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionArcArcSwap, Some(inner_ty)) => {
                            let arc_mod = arc_swap_crate_ident_from_container_ty(ty);
                            let lock_ty =
                                quote! { ::std::sync::Arc<#arc_mod::ArcSwap<#inner_ty>> };
                            let lock_kp_return_ty = kp_lock_ty_arc_arc_swap(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ArcArcSwapAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionArcArcSwapOption, Some(inner_ty)) => {
                            let arc_mod = arc_swap_crate_ident_from_container_ty(ty);
                            let lock_ty =
                                quote! { ::std::sync::Arc<#arc_mod::ArcSwapOption<#inner_ty>> };
                            let lock_kp_return_ty =
                                kp_lock_ty_arc_arc_swap_option(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ArcArcSwapOptionAccess::<#inner_ty>::new(),
                                        rust_key_paths::Kp::new(
                                            |o: &::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                o.as_ref().map(|a| a.as_ref())
                                            },
                                            |o: &mut ::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                o.as_mut().and_then(::std::sync::Arc::get_mut)
                                            },
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionArcRwLock, Some(inner_ty)) => {
                            // let kp_unlocked_fn = format_ident!("{}_unlocked", field_ident);
                            let lock_ty = quote! { std::sync::Arc<parking_lot::RwLock<#inner_ty>> };
                            let lock_kp_return_ty =
                                kp_lock_ty_parking_lot_rw_lock(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                // pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<#name, std::sync::Arc<parking_lot::RwLock<#inner_ty>>, &'static #name, &'static std::sync::Arc<parking_lot::RwLock<#inner_ty>>, &'static mut #name, &'static mut std::sync::Arc<parking_lot::RwLock<#inner_ty>>, impl Fn(&#name) -> Option<&std::sync::Arc<parking_lot::RwLock<#inner_ty>>>, impl Fn(&mut #name) -> Option<&mut std::sync::Arc<parking_lot::RwLock<#inner_ty>>>,> {
                                //     rust_key_paths::Kp::new(
                                //         |root: &#name| root.#field_ident.as_ref(),
                                //         |root: &mut #name| root.#field_ident.as_mut(),
                                //     )
                                // }
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ParkingLotRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionStdMutex, Some(inner_ty)) => {
                            let kp_unlocked_fn = format_ident!("{}_unlocked", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<
                                    #name,
    std::sync::Mutex<#inner_ty>,
    &'static #name,
    &'static std::sync::Mutex<#inner_ty>,
    &'static mut #name,
    &'static mut std::sync::Mutex<#inner_ty>,
    impl Fn(&#name) -> Option<&std::sync::Mutex<#inner_ty>>,
    impl Fn(&mut #name) -> Option<&mut std::sync::Mutex<#inner_ty>>,
> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionMutex, Some(inner_ty)) => {
                            let kp_unlocked_fn = format_ident!("{}_unlocked", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<
                                    #name,
    parking_lot::Mutex<#inner_ty>,
    &'static #name,
    &'static parking_lot::Mutex<#inner_ty>,
    &'static mut #name,
    &'static mut parking_lot::Mutex<#inner_ty>,
    impl Fn(&#name) -> Option<&parking_lot::Mutex<#inner_ty>>,
    impl Fn(&mut #name) -> Option<&mut parking_lot::Mutex<#inner_ty>>,
> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionStdRwLock, Some(inner_ty)) => {
                            let kp_unlocked_fn = format_ident!("{}_unlocked", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<
                                    #name,
    std::sync::RwLock<#inner_ty>,
    &'static #name,
    &'static std::sync::RwLock<#inner_ty>,
    &'static mut #name,
    &'static mut std::sync::RwLock<#inner_ty>,
    impl Fn(&#name) -> Option<&std::sync::RwLock<#inner_ty>>,
    impl Fn(&mut #name) -> Option<&mut std::sync::RwLock<#inner_ty>>,
> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionRwLock, Some(inner_ty)) => {
                            let kp_unlocked_fn = format_ident!("{}_unlocked", field_ident);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                                pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<
                                    #name,
    parking_lot::RwLock<#inner_ty>,
    &'static #name,
    &'static parking_lot::RwLock<#inner_ty>,
    &'static mut #name,
    &'static mut parking_lot::RwLock<#inner_ty>,
    impl Fn(&#name) -> Option<&parking_lot::RwLock<#inner_ty>>,
    impl Fn(&mut #name) -> Option<&mut parking_lot::RwLock<#inner_ty>>,
> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Weak, Some(_inner_ty)) => {
                            // For Weak<T>, return keypath to container
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None), // Weak doesn't support mutable access
                                    )
                                }
                            });
                        }
                        (WrapperKind::Atomic, None | Some(_)) => {
                            // For atomic types: return keypath to the atomic (user calls .load()/.store())
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionAtomic, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::String, None) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionString, None) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<#name, std::string::String, &'static #name, &'static std::string::String, &'static mut #name, &'static mut std::string::String, impl Fn(&#name) -> Option<&std::string::String>, impl Fn(&mut #name) -> Option<&mut std::string::String>,>  {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Cell, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::RefCell, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OnceCell, Some(inner_ty)) => {
                            // OnceLock/OnceCell: keypath to inner value; get = .get() -> Option<&T>, set = None
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.get()),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Lazy, Some(inner_ty)) => {
                            // Lazy/LazyLock: keypath to inner value; get = .get() -> &T wrapped in Some, set = None
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(root.#field_ident.get())),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::PhantomData, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Range, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionCell, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionRefCell, Some(inner_ty)) => {
                            // Option<RefCell<T>>: keypath to T via borrow()/borrow_mut(); get returns Option<Ref<V>> so caller holds guard (deref for &V)
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::KpOptionRefCellType<'_, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref().map(|r| r.borrow())),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#field_ident.as_ref().map(|r| r.borrow_mut())),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionOnceCell, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref().and_then(|c| c.get())),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionLazy, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#field_ident.as_ref().map(|c| c.get())),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionPhantomData, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionRange, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Reference, Some(_inner_ty)) => {
                            // For reference types (&T, &str, &[T]): read-only, setter returns None
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None), // references: read-only
                                    )
                                }
                            });
                        }
                        (WrapperKind::None, None) => {
                            // For basic types, direct access
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                        _ => {
                            // For unknown/complex nested types, return keypath to field itself
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#field_ident)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#field_ident)),
                                    )
                                }
                            });
                        }
                    }
                }

                tokens
            }
            Fields::Unnamed(unnamed) => {
                let mut tokens = proc_macro2::TokenStream::new();

                // Generate identity methods for the tuple struct
                tokens.extend(quote! {
                    /// Returns a generic identity keypath for this type
                    #[inline(always)]
                    pub fn identity_typed<'a, Root, MutRoot>() -> rust_key_paths::Kp<
                        #name,
                        #name,
                        Root,
                        Root,
                        MutRoot,
                        MutRoot,
                        fn(Root) -> Option<Root>,
                        fn(MutRoot) -> Option<MutRoot>,
                    >
                    where
                        Root: std::borrow::Borrow<#name>,
                        MutRoot: std::borrow::BorrowMut<#name>,
                    {
                        rust_key_paths::Kp::new(
                            |r: Root| Some(r),
                            |r: MutRoot| Some(r)
                        )
                    }

                    /// Returns a simple identity keypath for this type
                    #[inline(always)]
                    pub fn identity() -> rust_key_paths::Kp<#name, #name, &'static #name, &'static #name, &'static mut #name, &'static mut #name, impl Fn(&#name) -> Option<&#name>, impl Fn(&mut #name) -> Option<&mut #name>,> {
                        rust_key_paths::Kp::new(
                            rust_key_paths::constrain_get(|r: &#name| Some(r)),
                            rust_key_paths::constrain_set(|r: &mut #name| Some(r))
                        )
                    }
                });

                for (idx, field) in unnamed.unnamed.iter().enumerate() {
                    let idx_lit = syn::Index::from(idx);
                    let ty = &field.ty;
                    // Centralized keypath method names for tuple fields – change here to adjust for all types
                    let kp_fn = format_ident!("f{}", idx);
                    let kp_at_fn = format_ident!("f{}_at", idx);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionBox, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_deref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_deref_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionRc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_deref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut().and_then(std::rc::Rc::get_mut)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionArc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_deref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut().and_then(std::sync::Arc::get_mut)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionHashMap, Some(inner_ty)) => {
                            if let Some((key_ty, _)) = extract_map_key_value_through_option(ty) {
                                let type_name = name.to_string();
                                let whole_fn = kp_fn.to_string();
                                let at_fn = kp_at_fn.to_string();
                                let whole_doc = format!(
                                    "Keypath to the whole optional `HashMap` (`{type_name}::{whole_fn}`). For a value at one key when the map is `Some`, use `{type_name}::{at_fn}(key)`."
                                );
                                let at_doc = format!(
                                    "Keyed access into the inner `HashMap` (`{type_name}::{at_fn}`). `get`/`get_mut` return `None` if the field is `None` or the key is missing. See also `{type_name}::{whole_fn}` for the full optional map."
                                );
                                tokens.extend(quote! {
                                    #[doc = #whole_doc]
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        )
                                    }
                                    #[doc = #at_doc]
                                    #[inline(always)]
                                    pub fn #kp_at_fn(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                    where
                                        #key_ty: Clone + std::hash::Hash + Eq + 'static,
                                    {
                                        let key2 = key.clone();
                                        rust_key_paths::Kp::new(
                                            Box::new(move |root: &#name| root.#idx_lit.as_ref().and_then(|m| m.get(&key))),
                                            Box::new(move |root: &mut #name| root.#idx_lit.as_mut().and_then(|m| m.get_mut(&key2))),
                                        )
                                    }
                                });
                            } else {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        )
                                    }
                                });
                            }
                        }
                        (WrapperKind::OptionBTreeMap, Some(inner_ty)) => {
                            if let Some((key_ty, _)) = extract_map_key_value_through_option(ty) {
                                let type_name = name.to_string();
                                let whole_fn = kp_fn.to_string();
                                let at_fn = kp_at_fn.to_string();
                                let whole_doc = format!(
                                    "Keypath to the whole optional `BTreeMap` (`{type_name}::{whole_fn}`). For a value at one key when the map is `Some`, use `{type_name}::{at_fn}(key)`."
                                );
                                let at_doc = format!(
                                    "Keyed access into the inner `BTreeMap` (`{type_name}::{at_fn}`). `get`/`get_mut` return `None` if the field is `None` or the key is missing. See also `{type_name}::{whole_fn}` for the full optional map."
                                );
                                tokens.extend(quote! {
                                    #[doc = #whole_doc]
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        )
                                    }
                                    #[doc = #at_doc]
                                    #[inline(always)]
                                    pub fn #kp_at_fn(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                    where
                                        #key_ty: Clone + Ord + 'static,
                                    {
                                        let key2 = key.clone();
                                        rust_key_paths::Kp::new(
                                            Box::new(move |root: &#name| root.#idx_lit.as_ref().and_then(|m| m.get(&key))),
                                            Box::new(move |root: &mut #name| root.#idx_lit.as_mut().and_then(|m| m.get_mut(&key2))),
                                        )
                                    }
                                });
                            } else {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        )
                                    }
                                });
                            }
                        }
                        (WrapperKind::OptionHashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }

                                /// _at: check if element exists and get reference.
                                /// HashSet does not allow mutable element access (would break hash invariant).
                                #[inline(always)]
                                pub fn #kp_at_fn(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                where
                                    #inner_ty: Clone + std::hash::Hash + Eq + 'static,
                                {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#idx_lit.as_ref().and_then(|s| s.get(&key))),
                                        Box::new(move |_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionBTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }

                                /// _at: check if element exists and get reference.
                                /// BTreeSet does not allow mutable element access (would break ordering invariant).
                                #[inline(always)]
                                pub fn #kp_at_fn(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                where
                                    #inner_ty: Clone + Ord + 'static,
                                {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#idx_lit.as_ref().and_then(|s| s.get(&key))),
                                        Box::new(move |_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionVec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_at_fn(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#idx_lit.as_ref().and_then(|v| v.get(index))),
                                        Box::new(move |root: &mut #name| root.#idx_lit.as_mut().and_then(|v| v.get_mut(index))),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionVecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_at_fn(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#idx_lit.as_ref().and_then(|v| v.get(index))),
                                        Box::new(move |root: &mut #name| root.#idx_lit.as_mut().and_then(|v| v.get_mut(index))),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionLinkedList, Some(_inner_ty))
                        | (WrapperKind::OptionBinaryHeap, Some(_inner_ty))
                        | (WrapperKind::OptionResult, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_at_fn(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#idx_lit.get(index)),
                                        Box::new(move |root: &mut #name| root.#idx_lit.get_mut(index)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            if let Some((key_ty, _)) = extract_map_key_value(ty) {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        )
                                    }
                                    #[inline(always)]
                                    pub fn #kp_at_fn(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                    where
                                        #key_ty: Clone + std::hash::Hash + Eq + 'static,
                                    {
                                        let key2 = key.clone();
                                        rust_key_paths::Kp::new(
                                            Box::new(move |root: &#name| root.#idx_lit.get(&key)),
                                            Box::new(move |root: &mut #name| root.#idx_lit.get_mut(&key2)),
                                        )
                                    }
                                });
                            } else {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        )
                                    }
                                });
                            }
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty))
                        | (WrapperKind::BTreeMapOption, Some(inner_ty)) => {
                            if let Some((key_ty, _)) = extract_map_key_value(ty) {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        )
                                    }
                                    #[inline(always)]
                                    pub fn #kp_at_fn(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                    where
                                        #key_ty: Clone + Ord + 'static,
                                    {
                                        let key2 = key.clone();
                                        rust_key_paths::Kp::new(
                                            Box::new(move |root: &#name| root.#idx_lit.get(&key)),
                                            Box::new(move |root: &mut #name| root.#idx_lit.get_mut(&key2)),
                                        )
                                    }
                                });
                            } else {
                                tokens.extend(quote! {
                                    #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        )
                                    }
                                });
                            }
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            // Box: deref to inner (returns &T / &mut T)
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&*root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut *root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BoxOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| (&*root.#idx_lit).as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| (&mut *root.#idx_lit).as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::RcOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| (&*root.#idx_lit).as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| std::rc::Rc::get_mut(&mut root.#idx_lit).and_then(std::option::Option::as_mut)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::ArcOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| (&*root.#idx_lit).as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| std::sync::Arc::get_mut(&mut root.#idx_lit).and_then(std::option::Option::as_mut)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Pin, Some(inner_ty)) => {
                            let kp_inner_fn = format_ident!("{}_inner", kp_fn);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_inner_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
>
>
                                where #inner_ty: std::marker::Unpin
                                {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(std::pin::Pin::as_ref(&root.#idx_lit).get_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(std::pin::Pin::as_mut(&mut root.#idx_lit).get_mut())),
                                    )
                                }
                            });
                        }
                        (WrapperKind::PinBox, Some(inner_ty)) => {
                            let kp_inner_fn = format_ident!("{}_inner", kp_fn);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_inner_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
>
>
                                where #inner_ty: std::marker::Unpin
                                {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(std::pin::Pin::as_ref(&root.#idx_lit).get_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(std::pin::Pin::as_mut(&mut root.#idx_lit).get_mut())),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(root.#idx_lit.as_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| std::rc::Rc::get_mut(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(root.#idx_lit.as_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| std::sync::Arc::get_mut(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }

                        (WrapperKind::Cow, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(root.#idx_lit.as_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(root.#idx_lit.to_mut())),
                                    )
                                }
                            });
                        }

                        (WrapperKind::OptionCow, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref().map(|c| c.as_ref())),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut().map(|c| c.to_mut())),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionTagged, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref().map(|t| std::ops::Deref::deref(t))),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut().map(|t| std::ops::DerefMut::deref_mut(t))),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionReference, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty))
                        | (WrapperKind::HashSetOption, Some(inner_ty)) => {
                            let kp_at_fn = format_ident!("f{}_at", idx);

                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }

                                /// _at: check if element exists and get reference.
                                /// HashSet does not allow mutable element access (would break hash invariant).
                                #[inline(always)]
                                pub fn #kp_at_fn(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                where
                                    #inner_ty: Clone + std::hash::Hash + Eq + 'static,
                                {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#idx_lit.get(&key)),
                                        Box::new(move |_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty))
                        | (WrapperKind::BTreeSetOption, Some(inner_ty)) => {
                            let kp_at_fn = format_ident!("f{}_at", idx);

                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }

                                /// _at: check if element exists and get reference.
                                /// BTreeSet does not allow mutable element access (would break ordering invariant).
                                #[inline(always)]
                                pub fn #kp_at_fn(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                where
                                    #inner_ty: Clone + Ord + 'static,
                                {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#idx_lit.get(&key)),
                                        Box::new(move |_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty))
                        | (WrapperKind::VecDequeOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                #[inline(always)]
                                pub fn #kp_at_fn(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        Box::new(move |root: &#name| root.#idx_lit.get(index)),
                                        Box::new(move |root: &mut #name| root.#idx_lit.get_mut(index)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(_inner_ty))
                        | (WrapperKind::LinkedListOption, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(_inner_ty))
                        | (WrapperKind::BinaryHeapOption, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref().ok()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut().ok()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(_inner_ty))
                        | (WrapperKind::StdMutex, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::RwLock, Some(_inner_ty))
                        | (WrapperKind::StdRwLock, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::TokioArcMutex, Some(inner_ty)) => {
                            let kp_async_fn = format_ident!("f{}_kp", idx);
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_async_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                pub fn #kp_fn() -> rust_key_paths::async_lock::AsyncLockKpMutexFor<#name, #ty, #inner_ty> {
                                    rust_key_paths::async_lock::AsyncLockKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        ),
                                        rust_key_paths::async_lock::TokioMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::TokioArcRwLock, Some(inner_ty)) => {
                            let kp_async_fn = format_ident!("f{}_kp", idx);
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_async_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                pub fn #kp_fn() -> rust_key_paths::async_lock::AsyncLockKpRwLockFor<#name, #ty, #inner_ty> {
                                    rust_key_paths::async_lock::AsyncLockKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                            rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                        ),
                                        rust_key_paths::async_lock::TokioRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionTokioArcMutex, Some(inner_ty)) => {
                            let kp_async_fn = format_ident!("f{}_kp", idx);
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_async_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                pub fn #kp_fn() -> rust_key_paths::async_lock::AsyncLockKpMutexFor<#name, std::sync::Arc<tokio::sync::Mutex<#inner_ty>>, #inner_ty> {
                                    rust_key_paths::async_lock::AsyncLockKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                        ),
                                        rust_key_paths::async_lock::TokioMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionTokioArcRwLock, Some(inner_ty)) => {
                            let kp_async_fn = format_ident!("f{}_kp", idx);
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_async_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                pub fn #kp_fn() -> rust_key_paths::async_lock::AsyncLockKpRwLockFor<#name, std::sync::Arc<tokio::sync::RwLock<#inner_ty>>, #inner_ty> {
                                    rust_key_paths::async_lock::AsyncLockKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                        ),
                                        rust_key_paths::async_lock::TokioRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionStdArcMutex, Some(inner_ty)) => {
                            let lock_ty = quote! { std::sync::Arc<std::sync::Mutex<#inner_ty>> };
                            let lock_kp_return_ty = kp_lock_ty_arc_mutex(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ArcMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionArcMutex, Some(inner_ty)) => {
                            let lock_ty = quote! { std::sync::Arc<parking_lot::Mutex<#inner_ty>> };
                            let lock_kp_return_ty =
                                kp_lock_ty_parking_lot_mutex(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ParkingLotMutexAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionStdArcRwLock, Some(inner_ty)) => {
                            let lock_ty = quote! { std::sync::Arc<std::sync::RwLock<#inner_ty>> };
                            let lock_kp_return_ty =
                                kp_lock_ty_arc_rw_lock(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ArcRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionArcRwLock, Some(inner_ty)) => {
                            let lock_ty = quote! { std::sync::Arc<parking_lot::RwLock<#inner_ty>> };
                            let lock_kp_return_ty =
                                kp_lock_ty_parking_lot_rw_lock(name, &lock_ty, &inner_ty);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn<'b>() -> #lock_kp_return_ty {
                                    rust_key_paths::sync_kp::SyncKp::new(
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                            rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                        ),
                                        rust_key_paths::sync_kp::ParkingLotRwLockAccess::new(),
                                        rust_key_paths::Kp::new(
                                            rust_key_paths::constrain_get(|v: &#inner_ty| Some(v)),
                                            rust_key_paths::constrain_set(|v: &mut #inner_ty| Some(v)),
                                        ),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionStdMutex, Some(inner_ty)) => {
                            let kp_unlocked_fn = format_ident!("f{}_unlocked", idx);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<
                                    #name,
    std::sync::Mutex<#inner_ty>,
    &'static #name,
    &'static std::sync::Mutex<#inner_ty>,
    &'static mut #name,
    &'static mut std::sync::Mutex<#inner_ty>,
    impl Fn(&#name) -> Option<&std::sync::Mutex<#inner_ty>>,
    impl Fn(&mut #name) -> Option<&mut std::sync::Mutex<#inner_ty>>,
> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionMutex, Some(inner_ty)) => {
                            let kp_unlocked_fn = format_ident!("f{}_unlocked", idx);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<
                                    #name,
    parking_lot::Mutex<#inner_ty>,
    &'static #name,
    &'static parking_lot::Mutex<#inner_ty>,
    &'static mut #name,
    &'static mut parking_lot::Mutex<#inner_ty>,
    impl Fn(&#name) -> Option<&parking_lot::Mutex<#inner_ty>>,
    impl Fn(&mut #name) -> Option<&mut parking_lot::Mutex<#inner_ty>>,
> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionStdRwLock, Some(inner_ty)) => {
                            let kp_unlocked_fn = format_ident!("f{}_unlocked", idx);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<
                                    #name,
    std::sync::RwLock<#inner_ty>,
    &'static #name,
    &'static std::sync::RwLock<#inner_ty>,
    &'static mut #name,
    &'static mut std::sync::RwLock<#inner_ty>,
    impl Fn(&#name) -> Option<&std::sync::RwLock<#inner_ty>>,
    impl Fn(&mut #name) -> Option<&mut std::sync::RwLock<#inner_ty>>,
> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionRwLock, Some(inner_ty)) => {
                            let kp_unlocked_fn = format_ident!("f{}_unlocked", idx);
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                                pub fn #kp_unlocked_fn() -> rust_key_paths::Kp<
                                    #name,
    parking_lot::RwLock<#inner_ty>,
    &'static #name,
    &'static parking_lot::RwLock<#inner_ty>,
    &'static mut #name,
    &'static mut parking_lot::RwLock<#inner_ty>,
    impl Fn(&#name) -> Option<&parking_lot::RwLock<#inner_ty>>,
    impl Fn(&mut #name) -> Option<&mut parking_lot::RwLock<#inner_ty>>,
> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Weak, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Atomic, None | Some(_)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionAtomic, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::String, None) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionString, None) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<#name, std::string::String, &'static #name, &'static std::string::String, &'static mut #name, &'static mut std::string::String, impl Fn(&#name) -> Option<&std::string::String>, impl Fn(&mut #name) -> Option<&mut std::string::String>,>  {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref()),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_mut()),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OnceCell, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.get()),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Lazy, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(root.#idx_lit.get())),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionOnceCell, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref().and_then(|c| c.get())),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionLazy, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #inner_ty,
                                    &'static #name,
                                    &'static #inner_ty,
                                    &'static mut #name,
                                    &'static mut #inner_ty,
                                    impl Fn(&#name) -> Option<&#inner_ty>,
                                    impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref().map(|c| c.get())),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Cell, Some(_inner_ty))
                        | (WrapperKind::RefCell, Some(_inner_ty))
                        | (WrapperKind::PhantomData, Some(_inner_ty))
                        | (WrapperKind::Range, Some(_inner_ty))
                        | (WrapperKind::OptionCell, Some(_inner_ty))
                        | (WrapperKind::OptionPhantomData, Some(_inner_ty))
                        | (WrapperKind::OptionRange, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        (WrapperKind::OptionRefCell, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::KpOptionRefCellType<'_, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| root.#idx_lit.as_ref().map(|r| r.borrow())),
                                        rust_key_paths::constrain_set(|root: &mut #name| root.#idx_lit.as_ref().map(|r| r.borrow_mut())),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Reference, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|_root: &mut #name| None),
                                    )
                                }
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #kp_fn() -> rust_key_paths::Kp<
                                    #name,
                                    #ty,
                                    &'static #name,
                                    &'static #ty,
                                    &'static mut #name,
                                    &'static mut #ty,
                                    impl Fn(&#name) -> Option<&#ty>,
                                    impl Fn(&mut #name) -> Option<&mut #ty>,
                                    > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                #[inline(always)]
                                    pub fn #kp_fn() -> rust_key_paths::Kp<
                                        #name,
                                        #ty,
                                        &'static #name,
                                        &'static #ty,
                                        &'static mut #name,
                                        &'static mut #ty,
                                        impl Fn(&#name) -> Option<&#ty>,
                                        impl Fn(&mut #name) -> Option<&mut #ty>,
                                        > {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| Some(&root.#idx_lit)),
                                        rust_key_paths::constrain_set(|root: &mut #name| Some(&mut root.#idx_lit)),
                                    )
                                }
                            });
                        }
                    }
                }

                tokens
            }
            Fields::Unit => {
                return syn::Error::new(input_span, "Kp derive does not support unit structs")
                    .to_compile_error()
                    .into();
            }
        },
        Data::Enum(data_enum) => {
            let mut tokens = proc_macro2::TokenStream::new();

            // Generate identity methods for the enum
            tokens.extend(quote! {
                /// Returns a generic identity keypath for this type
                #[inline(always)]
                pub fn identity_typed<'a, Root, MutRoot>() -> rust_key_paths::Kp<
                    #name,
                    #name,
                    Root,
                    Root,
                    MutRoot,
                    MutRoot,
                    fn(Root) -> Option<Root>,
                    fn(MutRoot) -> Option<MutRoot>,
                >
                where
                    Root: std::borrow::Borrow<#name>,
                    MutRoot: std::borrow::BorrowMut<#name>,
                {
                    rust_key_paths::Kp::new(
                        |r: Root| Some(r),
                        |r: MutRoot| Some(r)
                    )
                }

                /// Returns a simple identity keypath for this type
                #[inline(always)]
                pub fn identity() -> rust_key_paths::Kp<#name, #name, &'static #name, &'static #name, &'static mut #name, &'static mut #name, impl Fn(&#name) -> Option<&#name>, impl Fn(&mut #name) -> Option<&mut #name>,> {
                    rust_key_paths::Kp::new(
                        rust_key_paths::constrain_get(|r: &#name| Some(r)),
                        rust_key_paths::constrain_set(|r: &mut #name| Some(r)
                    ))
                }
            });

            for variant in data_enum.variants.iter() {
                let v_ident = &variant.ident;
                let snake = format_ident!("{}", to_snake_case(&v_ident.to_string()));

                match &variant.fields {
                    Fields::Unit => {
                        // Unit variant - return keypath that checks if enum matches variant
                        tokens.extend(quote! {
                            #[inline(always)]
                            pub fn #snake() -> rust_key_paths::Kp<#name, (), &'static #name, &'static (), &'static mut #name, &'static mut (), impl Fn(&#name) -> Option<&()>, impl Fn(&mut #name) -> Option<&mut ()>,>  {
                                rust_key_paths::Kp::new(
                                    rust_key_paths::constrain_get(|root: &#name| match root {
                                        #name::#v_ident => {
                                            static UNIT: () = ();
                                            Some(&UNIT)
                                        },
                                        _ => None,
                                    }),
                                    rust_key_paths::constrain_set(|_root: &mut #name| None), // Can't mutate unit variant
                                )
                            }
                        });
                    }
                    Fields::Unnamed(unnamed) => {
                        if unnamed.unnamed.len() == 1 {
                            // Single-field tuple variant
                            let field_ty = &unnamed.unnamed[0].ty;
                            let (kind, inner_ty) = extract_wrapper_inner_type(field_ty);

                            match (kind, inner_ty.clone()) {
                                (WrapperKind::Option, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut(),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionHashMap, Some(inner_ty)) => {
                                    let snake_at = format_ident!("{}_at", snake);
                                    if let Some((key_ty, _)) =
                                        extract_map_key_value_through_option(field_ty)
                                    {
                                        tokens.extend(quote! {
                                            #[inline(always)]
                                            pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                )
                                            }
                                            #[inline(always)]
                                            pub fn #snake_at(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                            where
                                                #key_ty: Clone + std::hash::Hash + Eq + 'static,
                                            {
                                                let key2 = key.clone();
                                                rust_key_paths::Kp::new(
                                                    Box::new(move |root: &#name| match root {
                                                        #name::#v_ident(inner) => inner.as_ref().and_then(|m| m.get(&key)),
                                                        _ => None,
                                                    }),
                                                    Box::new(move |root: &mut #name| match root {
                                                        #name::#v_ident(inner) => inner.as_mut().and_then(|m| m.get_mut(&key2)),
                                                        _ => None,
                                                    }),
                                                )
                                            }
                                        });
                                    } else {
                                        tokens.extend(quote! {
                                            #[inline(always)]
                                            pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                )
                                            }
                                        });
                                    }
                                }
                                (WrapperKind::OptionBTreeMap, Some(inner_ty)) => {
                                    let snake_at = format_ident!("{}_at", snake);
                                    if let Some((key_ty, _)) =
                                        extract_map_key_value_through_option(field_ty)
                                    {
                                        tokens.extend(quote! {
                                            #[inline(always)]
                                            pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                )
                                            }
                                            #[inline(always)]
                                            pub fn #snake_at(key: #key_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                            where
                                                #key_ty: Clone + Ord + 'static,
                                            {
                                                let key2 = key.clone();
                                                rust_key_paths::Kp::new(
                                                    Box::new(move |root: &#name| match root {
                                                        #name::#v_ident(inner) => inner.as_ref().and_then(|m| m.get(&key)),
                                                        _ => None,
                                                    }),
                                                    Box::new(move |root: &mut #name| match root {
                                                        #name::#v_ident(inner) => inner.as_mut().and_then(|m| m.get_mut(&key2)),
                                                        _ => None,
                                                    }),
                                                )
                                            }
                                        });
                                    } else {
                                        tokens.extend(quote! {
                                            #[inline(always)]
                                            pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                )
                                            }
                                        });
                                    }
                                }
                                (WrapperKind::OptionHashSet, Some(inner_ty)) => {
                                    let snake_at = format_ident!("{}_at", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }

                                        /// _at: check if element exists and get reference.
                                        /// HashSet does not allow mutable element access (would break hash invariant).
                                        #[inline(always)]
                                        pub fn #snake_at(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                        where
                                            #inner_ty: Clone + std::hash::Hash + Eq + 'static,
                                        {
                                            rust_key_paths::Kp::new(
                                                Box::new(move |root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().and_then(|s| s.get(&key)),
                                                    _ => None,
                                                }),
                                                Box::new(move |_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionBTreeSet, Some(inner_ty)) => {
                                    let snake_at = format_ident!("{}_at", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }

                                        /// _at: check if element exists and get reference.
                                        /// BTreeSet does not allow mutable element access (would break ordering invariant).
                                        #[inline(always)]
                                        pub fn #snake_at(key: #inner_ty) -> rust_key_paths::KpDynamic<#name, #inner_ty>
                                        where
                                            #inner_ty: Clone + Ord + 'static,
                                        {
                                            rust_key_paths::Kp::new(
                                                Box::new(move |root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().and_then(|s| s.get(&key)),
                                                    _ => None,
                                                }),
                                                Box::new(move |_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionVec, Some(inner_ty)) => {
                                    let snake_at = format_ident!("{}_at", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                        #[inline(always)]
                                        pub fn #snake_at(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                            rust_key_paths::Kp::new(
                                                Box::new(move |root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().and_then(|v| v.get(index)),
                                                    _ => None,
                                                }),
                                                Box::new(move |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut().and_then(|v| v.get_mut(index)),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionVecDeque, Some(inner_ty)) => {
                                    let snake_at = format_ident!("{}_at", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                        #[inline(always)]
                                        pub fn #snake_at(index: usize) -> rust_key_paths::KpDynamic<#name, #inner_ty> {
                                            rust_key_paths::Kp::new(
                                                Box::new(move |root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().and_then(|v| v.get(index)),
                                                    _ => None,
                                                }),
                                                Box::new(move |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut().and_then(|v| v.get_mut(index)),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionLinkedList, Some(_inner_ty))
                                | (WrapperKind::OptionBinaryHeap, Some(_inner_ty))
                                | (WrapperKind::OptionResult, Some(_inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Vec, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.first(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.first_mut(),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Box, Some(inner_ty)) => {
                                    // Box in enum: deref to inner (&T / &mut T)
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(&**inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(&mut **inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Pin, Some(inner_ty)) => {
                                    let snake_inner = format_ident!("{}_inner", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                        #[inline(always)]
                                        pub fn #snake_inner() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
>
>
                                        where #inner_ty: std::marker::Unpin
                                        {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(std::pin::Pin::as_ref(inner).get_ref()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(std::pin::Pin::as_mut(inner).get_mut()),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::PinBox, Some(inner_ty)) => {
                                    let snake_inner = format_ident!("{}_inner", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                        #[inline(always)]
                                        pub fn #snake_inner() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
>
>
                                        where #inner_ty: std::marker::Unpin
                                        {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(std::pin::Pin::as_ref(inner).get_ref()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(std::pin::Pin::as_mut(inner).get_mut()),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Rc, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner.as_ref()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => std::rc::Rc::get_mut(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Arc, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner.as_ref()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => std::sync::Arc::get_mut(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::StdArcRwLock, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_rw_lock(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcRwLockAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::StdArcMutex, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_mutex(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcMutexAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::ArcRwLock, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_parking_lot_rw_lock(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ParkingLotRwLockAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::ArcMutex, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_parking_lot_mutex(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ParkingLotMutexAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::StdArcMutexOption, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_mutex_option(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcMutexAccess::<Option<#inner_ty>>::new(),
                                                rust_key_paths::Kp::new(Option::<#inner_ty>::as_ref, Option::<#inner_ty>::as_mut),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::StdArcRwLockOption, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_rw_lock_option(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcRwLockAccess::<Option<#inner_ty>>::new(),
                                                rust_key_paths::Kp::new(Option::<#inner_ty>::as_ref, Option::<#inner_ty>::as_mut),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::BareArcSwap, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_arc_swap(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                ),
                                                rust_key_paths::sync_kp::ArcArcSwapAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::BareArcSwapOption, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_arc_swap_option(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcArcSwapOptionAccess::<#inner_ty>::new(),
                                                rust_key_paths::Kp::new(
                                                    |o: &::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                        o.as_ref().map(|a| a.as_ref())
                                                    },
                                                    |o: &mut ::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                        o.as_mut().and_then(::std::sync::Arc::get_mut)
                                                    },
                                                ),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::ArcArcSwap, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_arc_swap(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                        #name::#v_ident(inner) => Some(inner),
                                                        _ => None,
                                                    }),
                                                ),
                                                rust_key_paths::sync_kp::ArcArcSwapAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::ArcArcSwapOption, Some(inner_ty)) => {
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_arc_swap_option(name, field_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcArcSwapOptionAccess::<#inner_ty>::new(),
                                                rust_key_paths::Kp::new(
                                                    |o: &::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                        o.as_ref().map(|a| a.as_ref())
                                                    },
                                                    |o: &mut ::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                        o.as_mut().and_then(::std::sync::Arc::get_mut)
                                                    },
                                                ),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionArcArcSwap, Some(inner_ty)) => {
                                    let arc_mod = arc_swap_crate_ident_from_container_ty(field_ty);
                                    let lock_ty =
                                        quote! { ::std::sync::Arc<#arc_mod::ArcSwap<#inner_ty>> };
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_arc_swap(name, &lock_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcArcSwapAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionArcArcSwapOption, Some(inner_ty)) => {
                                    let arc_mod = arc_swap_crate_ident_from_container_ty(field_ty);
                                    let lock_ty =
                                        quote! { ::std::sync::Arc<#arc_mod::ArcSwapOption<#inner_ty>> };
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_arc_swap_option(name, &lock_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcArcSwapOptionAccess::<#inner_ty>::new(),
                                                rust_key_paths::Kp::new(
                                                    |o: &::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                        o.as_ref().map(|a| a.as_ref())
                                                    },
                                                    |o: &mut ::std::option::Option<::std::sync::Arc<#inner_ty>>| {
                                                        o.as_mut().and_then(::std::sync::Arc::get_mut)
                                                    },
                                                ),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::ArcMutexOption, Some(inner_ty)) => {
                                    let lock_kp_return_ty = kp_lock_ty_parking_lot_mutex_option(
                                        name, field_ty, &inner_ty,
                                    );
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ParkingLotMutexAccess::<Option<#inner_ty>>::new(),
                                                rust_key_paths::Kp::new(Option::<#inner_ty>::as_ref, Option::<#inner_ty>::as_mut),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::ArcRwLockOption, Some(inner_ty)) => {
                                    let lock_kp_return_ty = kp_lock_ty_parking_lot_rw_lock_option(
                                        name, field_ty, &inner_ty,
                                    );
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ParkingLotRwLockAccess::<Option<#inner_ty>>::new(),
                                                rust_key_paths::Kp::new(Option::<#inner_ty>::as_ref, Option::<#inner_ty>::as_mut),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::TokioArcMutex, Some(inner_ty)) => {
                                    let snake_async = format_ident!("{}_kp", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake_async() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                        pub fn #snake() -> rust_key_paths::async_lock::AsyncLockKpMutexFor<#name, #field_ty, #inner_ty> {
                                            rust_key_paths::async_lock::AsyncLockKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::async_lock::TokioMutexAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::TokioArcRwLock, Some(inner_ty)) => {
                                    let snake_async = format_ident!("{}_kp", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake_async() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                        pub fn #snake() -> rust_key_paths::async_lock::AsyncLockKpRwLockFor<#name, #field_ty, #inner_ty> {
                                            rust_key_paths::async_lock::AsyncLockKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => Some(inner), _ => None }),
                                                ),
                                                rust_key_paths::async_lock::TokioRwLockAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionTokioArcMutex, Some(inner_ty)) => {
                                    let snake_async = format_ident!("{}_kp", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake_async() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                        pub fn #snake() -> rust_key_paths::async_lock::AsyncLockKpMutexFor<#name, std::sync::Arc<tokio::sync::Mutex<#inner_ty>>, #inner_ty> {
                                            rust_key_paths::async_lock::AsyncLockKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                                ),
                                                rust_key_paths::async_lock::TokioMutexAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionTokioArcRwLock, Some(inner_ty)) => {
                                    let snake_async = format_ident!("{}_kp", snake);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake_async() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                        pub fn #snake() -> rust_key_paths::async_lock::AsyncLockKpRwLockFor<#name, std::sync::Arc<tokio::sync::RwLock<#inner_ty>>, #inner_ty> {
                                            rust_key_paths::async_lock::AsyncLockKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                                ),
                                                rust_key_paths::async_lock::TokioRwLockAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionStdArcMutex, Some(inner_ty)) => {
                                    let lock_ty =
                                        quote! { std::sync::Arc<std::sync::Mutex<#inner_ty>> };
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_mutex(name, &lock_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcMutexAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionArcMutex, Some(inner_ty)) => {
                                    let lock_ty =
                                        quote! { std::sync::Arc<parking_lot::Mutex<#inner_ty>> };
                                    let lock_kp_return_ty =
                                        kp_lock_ty_parking_lot_mutex(name, &lock_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ParkingLotMutexAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionStdArcRwLock, Some(inner_ty)) => {
                                    let lock_ty =
                                        quote! { std::sync::Arc<std::sync::RwLock<#inner_ty>> };
                                    let lock_kp_return_ty =
                                        kp_lock_ty_arc_rw_lock(name, &lock_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ArcRwLockAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionArcRwLock, Some(inner_ty)) => {
                                    let lock_ty =
                                        quote! { std::sync::Arc<parking_lot::RwLock<#inner_ty>> };
                                    let lock_kp_return_ty =
                                        kp_lock_ty_parking_lot_rw_lock(name, &lock_ty, &inner_ty);
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake<'b>() -> #lock_kp_return_ty {
                                            rust_key_paths::sync_kp::SyncKp::new(
                                                rust_key_paths::Kp::new(
                                                    rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                    rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                                ),
                                                rust_key_paths::sync_kp::ParkingLotRwLockAccess::new(),
                                                rust_key_paths::Kp::new(|v: &#inner_ty| Some(v), |v: &mut #inner_ty| Some(v)),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::StdMutex, Some(_inner_ty))
                                | (WrapperKind::Mutex, Some(_inner_ty))
                                | (WrapperKind::StdRwLock, Some(_inner_ty))
                                | (WrapperKind::RwLock, Some(_inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Tagged, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(std::ops::Deref::deref(inner)),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(std::ops::DerefMut::deref_mut(inner)),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Atomic, None | Some(_)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionAtomic, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root { #name::#v_ident(inner) => inner.as_ref(), _ => None }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root { #name::#v_ident(inner) => inner.as_mut(), _ => None }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Reference, Some(_inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Weak, Some(_inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Cow, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner.as_ref()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner.to_mut()),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionBox, Some(inner_ty)) => {
                                    // Option<Box<T>>: keypath to T via as_deref() / as_deref_mut()
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_deref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_deref_mut(),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::BoxOption, Some(inner_ty)) => {
                                    // Box<Option<T>>: keypath to T; inner is &Box<Option<T>>, deref then Option::as_ref/as_mut
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => (&*inner).as_ref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => (&mut *inner).as_mut(),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::RcOption, Some(inner_ty)) => {
                                    // Rc<Option<T>>: keypath to T; get = (&*inner).as_ref(), set = Rc::get_mut then as_mut
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => (&*inner).as_ref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => std::rc::Rc::get_mut(inner).and_then(std::option::Option::as_mut),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::ArcOption, Some(inner_ty)) => {
                                    // Arc<Option<T>>: keypath to T; get = (&*inner).as_ref(), set = Arc::get_mut then as_mut
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => (&*inner).as_ref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => std::sync::Arc::get_mut(inner).and_then(std::option::Option::as_mut),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionRc, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_deref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut().and_then(std::rc::Rc::get_mut),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionArc, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_deref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut().and_then(std::sync::Arc::get_mut),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionCow, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().map(|c| c.as_ref()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut().map(|c| c.to_mut()),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionTagged, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().map(|t| std::ops::Deref::deref(t)),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut().map(|t| std::ops::DerefMut::deref_mut(t)),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionReference, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::String, None) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionString, None) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, std::string::String, &'static #name, &'static std::string::String, &'static mut #name, &'static mut std::string::String, impl Fn(&#name) -> Option<&std::string::String>, impl Fn(&mut #name) -> Option<&mut std::string::String>,>  {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut(),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OnceCell, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.get(),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Lazy, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner.get()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionOnceCell, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().and_then(|c| c.get()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionLazy, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<
                                            #name,
                                            #inner_ty,
                                            &'static #name,
                                            &'static #inner_ty,
                                            &'static mut #name,
                                            &'static mut #inner_ty,
                                            impl Fn(&#name) -> Option<&#inner_ty>,
                                            impl Fn(&mut #name) -> Option<&mut #inner_ty>,
                                            > {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().map(|c| c.get()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|_root: &mut #name| None),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Cell, Some(_inner_ty))
                                | (WrapperKind::RefCell, Some(_inner_ty))
                                | (WrapperKind::PhantomData, Some(_inner_ty))
                                | (WrapperKind::Range, Some(_inner_ty))
                                | (WrapperKind::OptionCell, Some(_inner_ty))
                                | (WrapperKind::OptionPhantomData, Some(_inner_ty))
                                | (WrapperKind::OptionRange, Some(_inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::OptionRefCell, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::KpOptionRefCellType<'_, #name, #inner_ty> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().map(|r| r.borrow()),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref().map(|r| r.borrow_mut()),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::None, None) => {
                                    // Basic type
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                                _ => {
                                    // Other wrapper types - return keypath to field
                                    tokens.extend(quote! {
                                        #[inline(always)]
                                        pub fn #snake() -> rust_key_paths::Kp<#name, #field_ty, &'static #name, &'static #field_ty, &'static mut #name, &'static mut #field_ty, impl Fn(&#name) -> Option<&#field_ty>, impl Fn(&mut #name) -> Option<&mut #field_ty>,> {
                                            rust_key_paths::Kp::new(
                                                rust_key_paths::constrain_get(|root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                                rust_key_paths::constrain_set(|root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                }),
                                            )
                                        }
                                    });
                                }
                            }
                        } else {
                            // Multi-field tuple variant - return keypath to variant itself
                            tokens.extend(quote! {
                                #[inline(always)]
                                pub fn #snake() -> rust_key_paths::Kp<#name, #name, &'static #name, &'static #name, &'static mut #name, &'static mut #name, impl Fn(&#name) -> Option<&#name>, impl Fn(&mut #name) -> Option<&mut #name>,> {
                                    rust_key_paths::Kp::new(
                                        rust_key_paths::constrain_get(|root: &#name| match root {
                                            #name::#v_ident(..) => Some(root),
                                            _ => None,
                                        }),
                                        rust_key_paths::constrain_set(|root: &mut #name| match root {
                                            #name::#v_ident(..) => Some(root),
                                            _ => None,
                                        }),
                                    )
                                }
                            });
                        }
                    }
                    Fields::Named(_) => {
                        // Named field variant - return keypath to variant itself
                        tokens.extend(quote! {
                            pub fn #snake() -> rust_key_paths::Kp<#name, #name, &'static #name, &'static #name, &'static mut #name, &'static mut #name, impl Fn(&#name) -> Option<&#name>, impl Fn(&mut #name) -> Option<&mut #name>,> {
                                rust_key_paths::Kp::new(
                                    rust_key_paths::constrain_get(|root: &#name| match root {
                                        #name::#v_ident { .. } => Some(root),
                                        _ => None,
                                    }),
                                    rust_key_paths::constrain_set(|root: &mut #name| match root {
                                        #name::#v_ident { .. } => Some(root),
                                        _ => None,
                                    }),
                                )
                            }
                        });
                    }
                }
            }

            tokens
        }
        Data::Union(_) => {
            return syn::Error::new(input_span, "Kp derive does not support unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #name {
            #methods
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro that generates `partial_kps() -> Vec<PKp<Self>>` returning all field/variant keypaths.
/// **Requires `#[derive(Kp)]`** so the keypath accessor methods exist.
///
/// For structs: returns keypaths for each field. For enums: returns keypaths for each variant
/// (using the same methods Kp generates, e.g. `some_variant()`).
///
/// # Example
/// ```
/// use key_paths_derive::{Kp, Pkp};
/// use rust_key_paths::PKp;
///
/// #[derive(Kp, Pkp)]
/// struct Person {
///     name: String,
///     age: i32,
/// }
///
/// let kps = Person::partial_kps();
/// assert_eq!(kps.len(), 2);
/// ```
#[proc_macro_derive(Pkp)]
pub fn derive_partial_keypaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let kp_calls = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let calls: Vec<_> = fields_named
                    .named
                    .iter()
                    .filter_map(|f| f.ident.as_ref())
                    .map(|field_ident| {
                        quote! { rust_key_paths::PKp::new(Self::#field_ident()) }
                    })
                    .collect();
                quote! { #(#calls),* }
            }
            Fields::Unnamed(unnamed) => {
                let calls: Vec<_> = (0..unnamed.unnamed.len())
                    .map(|idx| {
                        let kp_fn = format_ident!("f{}", idx);
                        quote! { rust_key_paths::PKp::new(Self::#kp_fn()) }
                    })
                    .collect();
                quote! { #(#calls),* }
            }
            Fields::Unit => quote! {},
        },
        Data::Enum(data_enum) => {
            let calls: Vec<_> = data_enum
                .variants
                .iter()
                .map(|variant| {
                    let v_ident = &variant.ident;
                    let snake = format_ident!("{}", to_snake_case(&v_ident.to_string()));
                    quote! { rust_key_paths::PKp::new(Self::#snake()) }
                })
                .collect();
            quote! { #(#calls),* }
        }
        Data::Union(_) => {
            return syn::Error::new(input.ident.span(), "Pkp derive does not support unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #name {
            /// Returns a vec of all field keypaths as partial keypaths (type-erased).
            #[inline(always)]
            pub fn partial_kps() -> Vec<rust_key_paths::PKp<#name>> {
                vec![#kp_calls]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro that generates `any_kps() -> Vec<AKp>` returning all field/variant keypaths as any keypaths.
/// **Requires `#[derive(Kp)]`** so the keypath accessor methods exist.
/// AKp type-erases both Root and Value), enabling heterogeneous collections of keypaths.
///
/// For structs: returns keypaths for each field. For enums: returns keypaths for each variant
/// (using the same methods Kp generates, e.g. `some_variant()`).
///
/// # Example
/// ```
/// use key_paths_derive::{Kp, Akp};
/// use rust_key_paths::AKp;
///
/// #[derive(Kp, Akp)]
/// struct Person {
///     name: String,
///     age: i32,
/// }
///
/// let kps = Person::any_kps();
/// assert_eq!(kps.len(), 2);
/// let person = Person { name: "Akash".into(), age: 30 };
/// let name: Option<&String> = kps[0].get(&person as &dyn std::any::Any).and_then(|v| v.downcast_ref());
/// assert_eq!(name, Some(&"Akash".to_string()));
/// ```
#[proc_macro_derive(Akp)]
pub fn derive_any_keypaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let kp_calls = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let calls: Vec<_> = fields_named
                    .named
                    .iter()
                    .filter_map(|f| f.ident.as_ref())
                    .map(|field_ident| {
                        quote! { rust_key_paths::AKp::new(Self::#field_ident()) }
                    })
                    .collect();
                quote! { #(#calls),* }
            }
            Fields::Unnamed(unnamed) => {
                let calls: Vec<_> = (0..unnamed.unnamed.len())
                    .map(|idx| {
                        let kp_fn = format_ident!("f{}", idx);
                        quote! { rust_key_paths::AKp::new(Self::#kp_fn()) }
                    })
                    .collect();
                quote! { #(#calls),* }
            }
            Fields::Unit => quote! {},
        },
        Data::Enum(data_enum) => {
            let calls: Vec<_> = data_enum
                .variants
                .iter()
                .map(|variant| {
                    let v_ident = &variant.ident;
                    let snake = format_ident!("{}", to_snake_case(&v_ident.to_string()));
                    quote! { rust_key_paths::AKp::new(Self::#snake()) }
                })
                .collect();
            quote! { #(#calls),* }
        }
        Data::Union(_) => {
            return syn::Error::new(input.ident.span(), "Akp derive does not support unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #name {
            /// Returns a vec of all field keypaths as any keypaths (fully type-erased).
            #[inline(always)]
            pub fn any_kps() -> Vec<rust_key_paths::AKp> {
                vec![#kp_calls]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro that generates **casepath** (prism) accessors for enum variants.
///
/// Unlike [`Kp`], which produces extract-only keypaths (`variant()`), `Cp` produces
/// a full [`rust_key_paths::EnumKpType`] per variant via `variant_cp()` — combining
/// extraction *and* embedding (the variant constructor). This lets consumers such as
/// `rust-elm` wire scoped reducers/stores directly:
///
/// ```ignore
/// use key_paths_derive::Cp;
///
/// #[derive(Cp)]
/// enum Action {
///     Child(ChildAction),
///     Tick,
/// }
///
/// // Ready-to-use casepath — no `variant_of(..)` boilerplate:
/// let kp = Action::child_cp(); // EnumKpType<'static, Action, ChildAction>
/// let embedded = kp.embed(ChildAction::Inc);
/// let extracted = kp.get_ref(&embedded);
/// ```
///
/// Generated per variant:
/// - **Unit** `Tick` → `tick_cp() -> EnumKpType<'static, Self, ()>`.
/// - **Single-field tuple** `Child(T)` → `child_cp() -> EnumKpType<'static, Self, T>`.
/// - **Multi-field / named-field** variants are skipped (no single-argument constructor).
///
/// Method names use a `_cp` suffix so `#[derive(Kp, Cp)]` can coexist on the same enum.
#[proc_macro_derive(Cp)]
pub fn derive_casepaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let data_enum = match &input.data {
        Data::Enum(data_enum) => data_enum,
        _ => {
            return syn::Error::new(name.span(), "Cp derive only supports enums")
                .to_compile_error()
                .into();
        }
    };

    let mut methods = proc_macro2::TokenStream::new();

    for variant in data_enum.variants.iter() {
        let v_ident = &variant.ident;
        let cp_fn = format_ident!("{}_cp", to_snake_case(&v_ident.to_string()));

        match &variant.fields {
            Fields::Unit => {
                methods.extend(quote! {
                    /// Casepath (prism) for this unit variant: extract `()` / embed the variant.
                    #[inline(always)]
                    pub fn #cp_fn() -> rust_key_paths::EnumKpType<'static, #name, ()> {
                        rust_key_paths::variant_of(
                            |root: &#name| match root {
                                #name::#v_ident => {
                                    static UNIT: () = ();
                                    Some(&UNIT)
                                }
                                _ => None,
                            },
                            |_root: &mut #name| None,
                            |_value: ()| #name::#v_ident,
                        )
                    }
                });
            }
            Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
                let field_ty = &unnamed.unnamed[0].ty;
                methods.extend(quote! {
                    /// Casepath (prism) for this variant: extract + embed the payload.
                    #[inline(always)]
                    pub fn #cp_fn() -> rust_key_paths::EnumKpType<'static, #name, #field_ty> {
                        rust_key_paths::variant_of(
                            |root: &#name| match root {
                                #name::#v_ident(inner) => Some(inner),
                                _ => None,
                            },
                            |root: &mut #name| match root {
                                #name::#v_ident(inner) => Some(inner),
                                _ => None,
                            },
                            #name::#v_ident,
                        )
                    }
                });
            }
            // Multi-field tuple and named-field variants have no single-arg constructor.
            _ => {}
        }
    }

    let expanded = quote! {
        impl #name {
            #methods
        }
    };

    TokenStream::from(expanded)
}
