use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{Data, DeriveInput, Fields, parse_macro_input, spanned::Spanned, Type};

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
    // String types (currently unused)
    // String,
    // OsString,
    // PathBuf,
    // Nested container support
    OptionBox,
    OptionRc,
    OptionArc,
    BoxOption,
    RcOption,
    ArcOption,
    VecOption,
    OptionVec,
    HashMapOption,
    OptionHashMap,
    // Arc with synchronization primitives (default)
    StdArcMutex,
    StdArcRwLock,
    OptionStdArcMutex,
    OptionStdArcRwLock,
    // Synchronization primitives default
    StdMutex,
    StdRwLock,
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
    OptionArcMutex,
    OptionArcRwLock,
    // Arc with synchronization primitives (tokio::sync - requires tokio feature)
    TokioArcMutex,
    TokioArcRwLock,
    OptionTokioArcMutex,
    OptionTokioArcRwLock,
    // Tagged types
    Tagged,
}

/// Helper function to check if a type path includes std::sync module
fn is_std_sync_type(path: &syn::Path) -> bool {
    // Check for paths like std::sync::Mutex, std::sync::RwLock
    let segments: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments.len() >= 2
        && segments.contains(&"std".to_string())
        && segments.contains(&"sync".to_string())
}

/// Helper function to check if a type path includes tokio::sync module
fn is_tokio_sync_type(path: &syn::Path) -> bool {
    // Check for paths like tokio::sync::Mutex, tokio::sync::RwLock
    let segments: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments.len() >= 2
        && segments.contains(&"tokio".to_string())
        && segments.contains(&"sync".to_string())
}

fn extract_wrapper_inner_type(ty: &Type) -> (WrapperKind, Option<Type>) {
    use syn::{GenericArgument, PathArguments};

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
                            ("Option", WrapperKind::Arc) => {
                                return (WrapperKind::OptionArc, inner_inner);
                            }
                            ("Option", WrapperKind::Vec) => {
                                return (WrapperKind::OptionVec, inner_inner);
                            }
                            ("Option", WrapperKind::HashMap) => {
                                return (WrapperKind::OptionHashMap, inner_inner);
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
                            ("HashMap", WrapperKind::Option) => {
                                return (WrapperKind::HashMapOption, inner_inner);
                            }
                            // std::sync variants (when inner is StdMutex/StdRwLock)
                            ("Arc", WrapperKind::StdMutex) => {
                                return (WrapperKind::StdArcMutex, inner_inner);
                            }
                            ("Arc", WrapperKind::StdRwLock) => {
                                return (WrapperKind::StdArcRwLock, inner_inner);
                            }
                            // parking_lot variants (default - when inner is Mutex/RwLock without std::sync prefix)
                            ("Arc", WrapperKind::Mutex) => {
                                return (WrapperKind::ArcMutex, inner_inner);
                            }
                            ("Arc", WrapperKind::RwLock) => {
                                return (WrapperKind::ArcRwLock, inner_inner);
                            }
                            // tokio::sync variants (when inner is TokioMutex/TokioRwLock)
                            ("Arc", WrapperKind::TokioMutex) => {
                                return (WrapperKind::TokioArcMutex, inner_inner);
                            }
                            ("Arc", WrapperKind::TokioRwLock) => {
                                return (WrapperKind::TokioArcRwLock, inner_inner);
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
                                    // For std::sync::Mutex and std::sync::RwLock, use Std variants
                                    "Mutex" if is_std_sync => {
                                        (WrapperKind::StdMutex, Some(inner.clone()))
                                    }
                                    "RwLock" if is_std_sync => {
                                        (WrapperKind::StdRwLock, Some(inner.clone()))
                                    }
                                    // For tokio::sync::Mutex and tokio::sync::RwLock, use Tokio variants
                                    "Mutex" if is_tokio_sync => {
                                        (WrapperKind::TokioMutex, Some(inner.clone()))
                                    }
                                    "RwLock" if is_tokio_sync => {
                                        (WrapperKind::TokioRwLock, Some(inner.clone()))
                                    }
                                    // Default: parking_lot (no std::sync or tokio::sync prefix)
                                    "Mutex" => (WrapperKind::Mutex, Some(inner.clone())),
                                    "RwLock" => (WrapperKind::RwLock, Some(inner.clone())),
                                    "Weak" => (WrapperKind::Weak, Some(inner.clone())),
                                    "Tagged" => (WrapperKind::Tagged, Some(inner.clone())),
                                    _ => (WrapperKind::None, None),
                                };
                            }
                        }
                    }
                }
            }
        }
    }
    (WrapperKind::None, None)
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
                    pub fn identity_typed<Root, MutRoot>() -> rust_key_paths::Kp<
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
                    pub fn identity() -> rust_key_paths::KpType<'static, #name, #name> {
                        rust_key_paths::Kp::new(
                            |r: &#name| Some(r),
                            |r: &mut #name| Some(r)
                        )
                    }
                });
                
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            // For Option<T>, unwrap and access inner type
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#field_ident.as_ref(),
                                        |root: &mut #name| root.#field_ident.as_mut(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            // For Vec<T>, access first element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#field_ident.first(),
                                        |root: &mut #name| root.#field_ident.first_mut(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            // For HashMap<K,V>, return keypath to container
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#field_ident),
                                        |root: &mut #name| Some(&mut root.#field_ident),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(_inner_ty)) => {
                            // For BTreeMap<K,V>, return keypath to container
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#field_ident),
                                        |root: &mut #name| Some(&mut root.#field_ident),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            // For Box<T>, deref to inner type (returns &T / &mut T, not &Box<T>)
                            // Matches reference: WritableKeyPath::new(|s: &mut #name| &mut *s.#field_ident)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&*root.#field_ident),
                                        |root: &mut #name| Some(&mut *root.#field_ident),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) => {
                            // For Rc<T>, deref to inner type (returns &T; get_mut when uniquely owned)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(root.#field_ident.as_ref()),
                                        |root: &mut #name| std::rc::Rc::get_mut(&mut root.#field_ident),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Arc, Some(inner_ty)) => {
                            // For Arc<T>, deref to inner type (returns &T; get_mut when uniquely owned)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(root.#field_ident.as_ref()),
                                        |root: &mut #name| std::sync::Arc::get_mut(&mut root.#field_ident),
                                    )
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            // For HashSet<T>, access any element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#field_ident.iter().next(),
                                        |_root: &mut #name| None, // HashSet doesn't support mutable iteration
                                    )
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            // For BTreeSet<T>, access any element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#field_ident.iter().next(),
                                        |_root: &mut #name| None, // BTreeSet doesn't support mutable iteration
                                    )
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            // For VecDeque<T>, access front element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#field_ident.front(),
                                        |root: &mut #name| root.#field_ident.front_mut(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            // For LinkedList<T>, access front element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#field_ident.front(),
                                        |root: &mut #name| root.#field_ident.front_mut(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            // For BinaryHeap<T>, peek at top element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#field_ident.peek(),
                                        |_root: &mut #name| None, // BinaryHeap doesn't support mutable peek
                                    )
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            // For Result<T, E>, access Ok value
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#field_ident.as_ref().ok(),
                                        |root: &mut #name| root.#field_ident.as_mut().ok(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(_inner_ty)) | 
                        (WrapperKind::StdMutex, Some(_inner_ty)) => {
                            // For Mutex<T>, return keypath to container
                            // Users can compose this with LockKp for lock access
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#field_ident),
                                        |root: &mut #name| Some(&mut root.#field_ident),
                                    )
                                }
                            });
                        }
                        (WrapperKind::RwLock, Some(_inner_ty)) | 
                        (WrapperKind::StdRwLock, Some(_inner_ty)) => {
                            // For RwLock<T>, return keypath to container
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#field_ident),
                                        |root: &mut #name| Some(&mut root.#field_ident),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Weak, Some(_inner_ty)) => {
                            // For Weak<T>, return keypath to container
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#field_ident),
                                        |_root: &mut #name| None, // Weak doesn't support mutable access
                                    )
                                }
                            });
                        }
                        (WrapperKind::None, None) => {
                            // For basic types, direct access
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#field_ident),
                                        |root: &mut #name| Some(&mut root.#field_ident),
                                    )
                                }
                            });
                        }
                        _ => {
                            // For unknown/complex nested types, return keypath to field itself
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#field_ident),
                                        |root: &mut #name| Some(&mut root.#field_ident),
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
                    pub fn identity_typed<Root, MutRoot>() -> rust_key_paths::Kp<
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
                    pub fn identity() -> rust_key_paths::KpType<'static, #name, #name> {
                        rust_key_paths::Kp::new(
                            |r: &#name| Some(r),
                            |r: &mut #name| Some(r)
                        )
                    }
                });
                
                for (idx, field) in unnamed.unnamed.iter().enumerate() {
                    let idx_lit = syn::Index::from(idx);
                    let ty = &field.ty;
                    let field_name = format_ident!("f{}", idx);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#idx_lit.as_ref(),
                                        |root: &mut #name| root.#idx_lit.as_mut(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#idx_lit.first(),
                                        |root: &mut #name| root.#idx_lit.first_mut(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#idx_lit),
                                        |root: &mut #name| Some(&mut root.#idx_lit),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#idx_lit),
                                        |root: &mut #name| Some(&mut root.#idx_lit),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            // Box: deref to inner (returns &T / &mut T)
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&*root.#idx_lit),
                                        |root: &mut #name| Some(&mut *root.#idx_lit),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(root.#idx_lit.as_ref()),
                                        |root: &mut #name| std::rc::Rc::get_mut(&mut root.#idx_lit),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(root.#idx_lit.as_ref()),
                                        |root: &mut #name| std::sync::Arc::get_mut(&mut root.#idx_lit),
                                    )
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#idx_lit.iter().next(),
                                        |_root: &mut #name| None,
                                    )
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#idx_lit.iter().next(),
                                        |_root: &mut #name| None,
                                    )
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#idx_lit.front(),
                                        |root: &mut #name| root.#idx_lit.front_mut(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#idx_lit.front(),
                                        |root: &mut #name| root.#idx_lit.front_mut(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#idx_lit.peek(),
                                        |_root: &mut #name| None,
                                    )
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| root.#idx_lit.as_ref().ok(),
                                        |root: &mut #name| root.#idx_lit.as_mut().ok(),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(_inner_ty)) | 
                        (WrapperKind::StdMutex, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#idx_lit),
                                        |root: &mut #name| Some(&mut root.#idx_lit),
                                    )
                                }
                            });
                        }
                        (WrapperKind::RwLock, Some(_inner_ty)) | 
                        (WrapperKind::StdRwLock, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#idx_lit),
                                        |root: &mut #name| Some(&mut root.#idx_lit),
                                    )
                                }
                            });
                        }
                        (WrapperKind::Weak, Some(_inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#idx_lit),
                                        |_root: &mut #name| None,
                                    )
                                }
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#idx_lit),
                                        |root: &mut #name| Some(&mut root.#idx_lit),
                                    )
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_key_paths::KpType<'static, #name, #ty> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| Some(&root.#idx_lit),
                                        |root: &mut #name| Some(&mut root.#idx_lit),
                                    )
                                }
                            });
                        }
                    }
                }
                
                tokens
            }
            Fields::Unit => {
                return syn::Error::new(
                    input_span,
                    "Kp derive does not support unit structs"
                )
                .to_compile_error()
                .into();
            }
        },
        Data::Enum(data_enum) => {
            let mut tokens = proc_macro2::TokenStream::new();
            
            // Generate identity methods for the enum
            tokens.extend(quote! {
                /// Returns a generic identity keypath for this type
                pub fn identity_typed<Root, MutRoot>() -> rust_key_paths::Kp<
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
                pub fn identity() -> rust_key_paths::KpType<'static, #name, #name> {
                    rust_key_paths::Kp::new(
                        |r: &#name| Some(r),
                        |r: &mut #name| Some(r)
                    )
                }
            });
            
            for variant in data_enum.variants.iter() {
                let v_ident = &variant.ident;
                let snake = format_ident!("{}", to_snake_case(&v_ident.to_string()));

                match &variant.fields {
                    Fields::Unit => {
                        // Unit variant - return keypath that checks if enum matches variant
                        tokens.extend(quote! {
                            pub fn #snake() -> rust_key_paths::KpType<'static, #name, ()> {
                                rust_key_paths::Kp::new(
                                    |root: &#name| match root {
                                        #name::#v_ident => {
                                            static UNIT: () = ();
                                            Some(&UNIT)
                                        },
                                        _ => None,
                                    },
                                    |_root: &mut #name| None, // Can't mutate unit variant
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
                                        pub fn #snake() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                            rust_key_paths::Kp::new(
                                                |root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.as_ref(),
                                                    _ => None,
                                                },
                                                |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.as_mut(),
                                                    _ => None,
                                                },
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Vec, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                            rust_key_paths::Kp::new(
                                                |root: &#name| match root {
                                                    #name::#v_ident(inner) => inner.first(),
                                                    _ => None,
                                                },
                                                |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => inner.first_mut(),
                                                    _ => None,
                                                },
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Box, Some(inner_ty)) => {
                                    // Box in enum: deref to inner (&T / &mut T)
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                            rust_key_paths::Kp::new(
                                                |root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(&**inner),
                                                    _ => None,
                                                },
                                                |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(&mut **inner),
                                                    _ => None,
                                                },
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Rc, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                            rust_key_paths::Kp::new(
                                                |root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner.as_ref()),
                                                    _ => None,
                                                },
                                                |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => std::rc::Rc::get_mut(inner),
                                                    _ => None,
                                                },
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::Arc, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_key_paths::KpType<'static, #name, #inner_ty> {
                                            rust_key_paths::Kp::new(
                                                |root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner.as_ref()),
                                                    _ => None,
                                                },
                                                |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => std::sync::Arc::get_mut(inner),
                                                    _ => None,
                                                },
                                            )
                                        }
                                    });
                                }
                                (WrapperKind::None, None) => {
                                    // Basic type
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_key_paths::KpType<'static, #name, #field_ty> {
                                            rust_key_paths::Kp::new(
                                                |root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                },
                                                |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                },
                                            )
                                        }
                                    });
                                }
                                _ => {
                                    // Other wrapper types - return keypath to field
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_key_paths::KpType<'static, #name, #field_ty> {
                                            rust_key_paths::Kp::new(
                                                |root: &#name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                },
                                                |root: &mut #name| match root {
                                                    #name::#v_ident(inner) => Some(inner),
                                                    _ => None,
                                                },
                                            )
                                        }
                                    });
                                }
                            }
                        } else {
                            // Multi-field tuple variant - return keypath to variant itself
                            tokens.extend(quote! {
                                pub fn #snake() -> rust_key_paths::KpType<'static, #name, #name> {
                                    rust_key_paths::Kp::new(
                                        |root: &#name| match root {
                                            #name::#v_ident(..) => Some(root),
                                            _ => None,
                                        },
                                        |root: &mut #name| match root {
                                            #name::#v_ident(..) => Some(root),
                                            _ => None,
                                        },
                                    )
                                }
                            });
                        }
                    }
                    Fields::Named(_) => {
                        // Named field variant - return keypath to variant itself
                        tokens.extend(quote! {
                            pub fn #snake() -> rust_key_paths::KpType<'static, #name, #name> {
                                rust_key_paths::Kp::new(
                                    |root: &#name| match root {
                                        #name::#v_ident { .. } => Some(root),
                                        _ => None,
                                    },
                                    |root: &mut #name| match root {
                                        #name::#v_ident { .. } => Some(root),
                                        _ => None,
                                    },
                                )
                            }
                        });
                    }
                }
            }
            
            tokens
        }
        Data::Union(_) => {
            return syn::Error::new(
                input_span,
                "Kp derive does not support unions"
            )
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
