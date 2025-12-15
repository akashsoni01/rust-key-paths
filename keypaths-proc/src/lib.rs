use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, Attribute, parse_macro_input, spanned::Spanned};

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
    // Synchronization primitives
    Mutex,
    RwLock,
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
    // Arc with synchronization primitives
    ArcMutex,
    ArcRwLock,
    // Tagged types
    Tagged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MethodScope {
    All,
    Readable,
    Writable,
    Owned,
}

impl MethodScope {
    fn includes_read(self) -> bool {
        matches!(self, MethodScope::All | MethodScope::Readable)
    }

    fn includes_write(self) -> bool {
        matches!(self, MethodScope::All | MethodScope::Writable)
    }

    fn includes_owned(self) -> bool {
        matches!(self, MethodScope::All | MethodScope::Owned)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MethodKind {
    Readable,
    Writable,
    Owned,
}

fn push_method(
    target: &mut proc_macro2::TokenStream,
    scope: MethodScope,
    kind: MethodKind,
    method_tokens: proc_macro2::TokenStream,
) {
    let include = match kind {
        MethodKind::Readable => scope.includes_read(),
        MethodKind::Writable => scope.includes_write(),
        MethodKind::Owned => scope.includes_owned(),
    };

    if include {
        target.extend(method_tokens);
    }
}

fn method_scope_from_attrs(attrs: &[Attribute]) -> syn::Result<Option<MethodScope>> {
    let mut scope: Option<MethodScope> = None;
    for attr in attrs {
        if attr.path().is_ident("Readable") {
            if scope.is_some() {
                return Err(syn::Error::new(attr.span(), "Only one of #[All], #[Readable], #[Writable], or #[Owned] may be used per field or variant"));
            }
            scope = Some(MethodScope::Readable);
        } else if attr.path().is_ident("Writable") {
            if scope.is_some() {
                return Err(syn::Error::new(attr.span(), "Only one of #[All], #[Readable], #[Writable], or #[Owned] may be used per field or variant"));
            }
            scope = Some(MethodScope::Writable);
        } else if attr.path().is_ident("Owned") {
            if scope.is_some() {
                return Err(syn::Error::new(attr.span(), "Only one of #[All], #[Readable], #[Writable], or #[Owned] may be used per field or variant"));
            }
            scope = Some(MethodScope::Owned);
        } else if attr.path().is_ident("All") {
            if scope.is_some() {
                return Err(syn::Error::new(attr.span(), "Only one of #[All], #[Readable], #[Writable], or #[Owned] may be used per field or variant"));
            }
            scope = Some(MethodScope::All);
        }
    }
    Ok(scope)
}

/// Derives keypath methods for struct fields.
///
/// This macro generates methods to create keypaths for accessing struct fields.
/// By default, it generates readable keypaths, but you can control which methods
/// are generated using attributes.
///
/// # Generated Methods
///
/// For each field `field_name`, the following methods are generated (depending on attributes):
///
/// - `field_name_r()` - Returns a `KeyPath<Struct, FieldType>` for non-optional fields
/// - `field_name_w()` - Returns a `WritableKeyPath<Struct, FieldType>` for non-optional fields
/// - `field_name_fr()` - Returns an `OptionalKeyPath<Struct, InnerType>` for optional/container fields
/// - `field_name_fw()` - Returns a `WritableOptionalKeyPath<Struct, InnerType>` for optional/container fields
/// - `field_name_fr_at(index)` - Returns an `OptionalKeyPath` for indexed access (Vec, HashMap, etc.)
/// - `field_name_fw_at(index)` - Returns a `WritableOptionalKeyPath` for indexed mutable access
/// - `field_name_o()` - Returns a `KeyPath` for owned access (when `#[Owned]` is used)
/// - `field_name_fo()` - Returns an `OptionalKeyPath` for owned optional access
///
/// # Attributes
///
/// ## Struct-level attributes:
///
/// - `#[All]` - Generate all methods (readable, writable, and owned)
/// - `#[Readable]` - Generate only readable methods (default)
/// - `#[Writable]` - Generate only writable methods
/// - `#[Owned]` - Generate only owned methods
///
/// ## Field-level attributes:
///
/// - `#[Readable]` - Generate readable methods for this field only
/// - `#[Writable]` - Generate writable methods for this field only
/// - `#[Owned]` - Generate owned methods for this field only
/// - `#[All]` - Generate all methods for this field
///
/// # Supported Field Types
///
/// The macro automatically handles various container types:
///
/// - `Option<T>` - Generates failable keypaths
/// - `Vec<T>` - Generates keypaths with iteration support
/// - `Box<T>`, `Rc<T>`, `Arc<T>` - Generates keypaths that dereference
/// - `HashMap<K, V>`, `BTreeMap<K, V>` - Generates key-based access methods
/// - `Result<T, E>` - Generates failable keypaths for `Ok` variant
/// - Tuple structs - Generates `f0_r()`, `f1_r()`, etc. for each field
///
/// # Examples
///
/// ```rust,ignore
/// use keypaths_proc::Keypaths;
///
/// #[derive(Keypaths)]
/// #[All]  // Generate all methods
/// struct User {
///     name: String,
///     age: Option<u32>,
///     tags: Vec<String>,
/// }
///
/// // Usage:
/// let name_path = User::name_r();  // KeyPath<User, String>
/// let age_path = User::age_fr();   // OptionalKeyPath<User, u32>
/// let tags_path = User::tags_r();  // KeyPath<User, Vec<String>>
///
/// let user = User {
///     name: "Alice".to_string(),
///     age: Some(30),
///     tags: vec!["admin".to_string()],
/// };
///
/// // Read values
/// let name = name_path.get(&user);
/// let age = age_path.get(&user);  // Returns Option<&u32>
/// ```
///
/// # Field-level Control
///
/// ```rust,ignore
/// #[derive(Keypaths)]
/// struct Config {
///     #[Readable]  // Only readable methods for this field
///     api_key: String,
///
///     #[Writable]  // Only writable methods for this field
///     counter: u32,
///
///     #[All]  // All methods for this field
///     settings: Option<Settings>,
/// }
/// ```
#[proc_macro_derive(Keypaths, attributes(Readable, Writable, Owned, All))]
pub fn derive_keypaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let default_scope = match method_scope_from_attrs(&input.attrs) {
        Ok(Some(scope)) => scope,
        Ok(None) => MethodScope::Readable,
        Err(err) => return err.to_compile_error().into(),
    };

    let methods = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => {/**/
                let mut tokens = proc_macro2::TokenStream::new();
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;

                    let r_fn = format_ident!("{}_r", field_ident);
                    let w_fn = format_ident!("{}_w", field_ident);
                    let fr_fn = format_ident!("{}_fr", field_ident);
                    let fw_fn = format_ident!("{}_fw", field_ident);
                    let fr_at_fn = format_ident!("{}_fr_at", field_ident);
                    let fw_at_fn = format_ident!("{}_fw_at", field_ident);
                    // Owned keypath method names
                    let o_fn = format_ident!("{}_o", field_ident);
                    let fo_fn = format_ident!("{}_fo", field_ident);

                    let method_scope = match method_scope_from_attrs(&field.attrs) {
                        Ok(Some(scope)) => scope,
                        Ok(None) => default_scope,
                        Err(err) => return err.to_compile_error().into(),
                    };

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_read = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_read, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_read>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_write = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_write, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_write>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.as_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            // For Option fields, fo_fn() returns OptionalKeyPath that unwraps the Option
                            let inner_ty_owned = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_owned, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_owned>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref())
                                    }
                                },
                            );
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.first())
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.first_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.first())
                                    }
                                },
                            );
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.values().next())
                                    }
                                },
                            );
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &*s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&*s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut *s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut *s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| *s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(*s.#field_ident))
                                    }
                                },
                            );
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &*s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&*s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| (*s.#field_ident).clone())
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some((*s.#field_ident).clone()))
                                    }
                                },
                            );
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.into_values().next())
                                    }
                                },
                            );
                            // Note: Key-based access methods for BTreeMap require the exact key type
                            // For now, we'll skip generating these methods to avoid generic constraint issues
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.iter().next())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.into_iter().next())
                                    }
                                },
                            );
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.iter().next())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.into_iter().next())
                                    }
                                },
                            );
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.front())
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.front_mut())
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.into_iter().next())
                                    }
                                },
                            );
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.front())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.front_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.into_iter().next())
                                    }
                                },
                            );
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.into_iter().next())
                                    }
                                },
                            );
                            // Note: BinaryHeap peek() returns &T, but we need &inner_ty
                            // For now, we'll skip failable methods for BinaryHeap to avoid type issues
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref().ok())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            // Note: Result<T, E> doesn't support failable_writable for inner type
                            // Only providing container-level writable access
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.ok())
                                    }
                                },
                            );
                        }
                        (WrapperKind::Mutex, Some(_inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                            // Only providing container-level access
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                        }
                        (WrapperKind::RwLock, Some(_inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                            // Only providing container-level access
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                        }
                        (WrapperKind::ArcMutex, Some(_inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            // Note: Arc<Mutex<T>> doesn't support writable access (Arc is immutable)
                            // Note: Arc<Mutex<T>> doesn't support direct access to inner type due to lifetime constraints
                            // Only providing container-level access
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                        }
                        (WrapperKind::ArcRwLock, Some(_inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            // Note: Arc<RwLock<T>> doesn't support writable access (Arc is immutable)
                            // Note: Arc<RwLock<T>> doesn't support direct access to inner type due to lifetime constraints
                            // Only providing container-level access
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                        }
                        (WrapperKind::Weak, Some(_inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            // Note: Weak<T> doesn't support writable access (it's immutable)
                            // Note: Weak<T> doesn't support direct access to inner type due to lifetime constraints
                            // Only providing container-level access
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                        }
                        // Nested container combinations
                        (WrapperKind::OptionBox, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref().map(|b| &**b))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.as_mut().map(|b| &mut **b))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.map(|b| *b))
                                    }
                                },
                            );
                        }
                        (WrapperKind::OptionRc, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref().map(|r| &**r))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.map(|r| (*r).clone()))
                                    }
                                },
                            );
                        }
                        (WrapperKind::OptionArc, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref().map(|a| &**a))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.map(|a| (*a).clone()))
                                    }
                                },
                            );
                        }
                        (WrapperKind::BoxOption, Some(inner_ty)) => {
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| (*s.#field_ident).as_ref())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| (*s.#field_ident).as_mut())
                                    }
                                },
                            );
                        }
                        (WrapperKind::RcOption, Some(inner_ty)) => {
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| (*s.#field_ident).as_ref())
                                    }
                                },
                            );
                        }
                        (WrapperKind::ArcOption, Some(inner_ty)) => {
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| (*s.#field_ident).as_ref())
                                    }
                                },
                            );
                        }
                        (WrapperKind::VecOption, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.first().and_then(|opt| opt.as_ref()))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(index).and_then(|opt| opt.as_ref()))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.first_mut().and_then(|opt| opt.as_mut()))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(index).and_then(|opt| opt.as_mut()))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.into_iter().flatten().next())
                                    }
                                },
                            );
                        }
                        (WrapperKind::OptionVec, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref().and_then(|v| v.first()))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.as_ref().and_then(|v| v.get(index)))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.as_mut().and_then(|v| v.first_mut()))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.as_mut().and_then(|v| v.get_mut(index)))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.and_then(|v| v.into_iter().next()))
                                    }
                                },
                            );
                        }
                        (WrapperKind::HashMapOption, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key).and_then(|opt| opt.as_ref()))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key).and_then(|opt| opt.as_ref()))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key).and_then(|opt| opt.as_mut()))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key).and_then(|opt| opt.as_mut()))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.into_values().flatten().next())
                                    }
                                },
                            );
                        }
                        (WrapperKind::OptionHashMap, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.as_ref().and_then(|m| m.get(&key)))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.as_ref().and_then(|m| m.get(&key)))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.as_mut().and_then(|m| m.get_mut(&key)))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.as_mut().and_then(|m| m.get_mut(&key)))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.and_then(|m| m.into_values().next()))
                                    }
                                },
                            );
                        }
                        (WrapperKind::None, None) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #ty>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#field_ident))
                                    }
                                },
                            );
                        }
                        _ => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                        }
                    }
                }
                tokens
            }
            Fields::Unnamed(unnamed) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for (idx, field) in unnamed.unnamed.iter().enumerate() {
                    let idx_lit = syn::Index::from(idx);
                    let ty = &field.ty;

                    let r_fn = format_ident!("f{}_r", idx);
                    let w_fn = format_ident!("f{}_w", idx);
                    let fr_fn = format_ident!("f{}_fr", idx);
                    let fw_fn = format_ident!("f{}_fw", idx);
                    let fr_at_fn = format_ident!("f{}_fr_at", idx);
                    let fw_at_fn = format_ident!("f{}_fw_at", idx);
                    // Owned keypath method names
                    let o_fn = format_ident!("f{}_o", idx);
                    let fo_fn = format_ident!("f{}_fo", idx);

                    let method_scope = match method_scope_from_attrs(&field.attrs) {
                        Ok(Some(scope)) => scope,
                        Ok(None) => default_scope,
                        Err(err) => return err.to_compile_error().into(),
                    };

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.as_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref())
                                    }
                                },
                            );
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.first_mut())
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: &'static usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.get(*index))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: &'static usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.get_mut(*index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                    }
                                },
                            );
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#idx_lit.get(&key))
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr_at, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr_at>> {
                                        rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#idx_lit.get(&key))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw_at, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw_at>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.into_values().next())
                                    }
                                },
                            );
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            let inner_ty_read = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty_read, impl for<'r> Fn(&'r #name) -> &'r #inner_ty_read> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_write = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty_write, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty_write> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut *s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&*s.#idx_lit))
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut *s.#idx_lit))
                                    }
                                },
                            );
                            let inner_ty_owned = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #inner_ty_owned, impl for<'r> Fn(&'r #name) -> &'r #inner_ty_owned> {
                                        rust_keypaths::KeyPath::new(|s: &#name| *s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(*s.#idx_lit))
                                    }
                                },
                            );
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            let inner_ty_read = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty_read, impl for<'r> Fn(&'r #name) -> &'r #inner_ty_read> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&*s.#idx_lit))
                                    }
                                },
                            );
                            let inner_ty_owned = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #inner_ty_owned, impl for<'r> Fn(&'r #name) -> &'r #inner_ty_owned> {
                                        rust_keypaths::KeyPath::new(|s: &#name| (*s.#idx_lit).clone())
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some((*s.#idx_lit).clone()))
                                    }
                                },
                            );
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.into_values().next())
                                    }
                                    // Note: Key-based access methods for BTreeMap require the exact key type
                                    // For now, we'll skip generating these methods to avoid generic constraint issues
                                },
                            );
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.iter().next())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                    }
                                },
                            );
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.iter().next())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                    }
                                },
                            );
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.front())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.front_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                    }
                                },
                            );
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.front())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.front_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                    }
                                },
                            );
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.peek())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty_fw, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty_fw>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.peek_mut().map(|v| &mut **v))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                    }
                                },
                            );
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fr, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fr>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref().ok())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Note: Result<T, E> doesn't support failable_writable for inner type
                                    // Only providing container-level writable access
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty_fo, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty_fo>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.ok())
                                    }
                                },
                            );
                        }
                        (WrapperKind::Mutex, Some(_inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                    // Only providing container-level access
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                        }
                        (WrapperKind::RwLock, Some(_inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                                    // Only providing container-level access
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                        }
                        (WrapperKind::Weak, Some(_inner_ty)) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Note: Weak<T> doesn't support writable access (it's immutable)
                                    // Note: Weak<T> doesn't support direct access to inner type due to lifetime constraints
                                    // Only providing container-level access
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                        }
                        // Nested container combinations for tuple structs - COMMENTED OUT FOR NOW
                        /*
                        (WrapperKind::OptionBox, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref().map(|b| &**b))
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.as_mut().map(|b| &mut **b))
                                }
                            });
                        }
                        (WrapperKind::OptionRc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref().map(|r| &**r))
                                }
                            });
                        }
                        (WrapperKind::OptionArc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref().map(|a| &**a))
                                }
                            });
                        }
                        (WrapperKind::BoxOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut *s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| (*s.#idx_lit).as_ref())
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| (*s.#idx_lit).as_mut())
                                }
                            });
                        }
                        (WrapperKind::RcOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| (*s.#idx_lit).as_ref())
                                }
                            });
                        }
                        (WrapperKind::ArcOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| (*s.#idx_lit).as_ref())
                                }
                            });
                        }
                        (WrapperKind::VecOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first().and_then(|opt| opt.as_ref()))
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.first_mut().and_then(|opt| opt.as_mut()))
                                }
                            });
                        }
                        (WrapperKind::OptionVec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref().and_then(|v| v.first()))
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.as_mut().and_then(|v| v.first_mut()))
                                }
                            });
                        }
                        (WrapperKind::HashMapOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#idx_lit.get(&key).and_then(|opt| opt.as_ref()))
                                }
                                pub fn #fw_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#idx_lit.get_mut(&key).and_then(|opt| opt.as_mut()))
                                }
                            });
                        }
                        (WrapperKind::OptionHashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#idx_lit.as_ref().and_then(|m| m.get(&key)))
                                }
                                pub fn #fw_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#idx_lit.as_mut().and_then(|m| m.get_mut(&key)))
                                }
                            });
                        }
                        */
                        (WrapperKind::None, None) => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                        rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#idx_lit))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #ty>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut s.#idx_lit))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                        rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#idx_lit))
                                    }
                                },
                            );
                        }
                        _ => {
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                        rust_keypaths::KeyPath::new(|s: &#name| s.#idx_lit)
                                    }
                                },
                            );
                        }
                    }
                }
                tokens
            }
            _ => quote! {
                compile_error!("Keypaths derive supports only structs with named or unnamed fields");
            },
        },
        Data::Enum(data_enum) => {
            let mut tokens = proc_macro2::TokenStream::new();
            for variant in data_enum.variants.iter() {
                let v_ident = &variant.ident;
                let snake = format_ident!("{}", to_snake_case(&v_ident.to_string()));
                let r_fn = format_ident!("{}_case_r", snake);
                let w_fn = format_ident!("{}_case_w", snake);
                let _fr_fn = format_ident!("{}_case_fr", snake);
                let _fw_fn = format_ident!("{}_case_fw", snake);
                let fr_at_fn = format_ident!("{}_case_fr_at", snake);
                let fw_at_fn = format_ident!("{}_case_fw_at", snake);

                match &variant.fields {
                    Fields::Unit => {
                        tokens.extend(quote! {
                            pub fn #r_fn() -> rust_keypaths::EnumKeyPath<#name, (), impl for<'r> Fn(&'r #name) -> Option<&'r ()>, impl Fn(()) -> #name> {
                                static UNIT: () = ();
                                rust_keypaths::EnumKeyPath::readable_enum(
                                    |_| #name::#v_ident,
                                    |e: &#name| match e { #name::#v_ident => Some(&UNIT), _ => None }
                                )
                            }
                        });
                    }
                    Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
                        let field_ty = &unnamed.unnamed.first().unwrap().ty;
                        let (kind, inner_ty_opt) = extract_wrapper_inner_type(field_ty);

                        match (kind, inner_ty_opt) {
                            (WrapperKind::Option, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.as_mut(), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::Vec, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut(), _ => None },
                                        )
                                    }
                                    pub fn #fr_at_fn(index: &'static usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.get(*index), _ => None }
                                        )
                                    }
                                    pub fn #fw_at_fn(index: &'static usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.get(*index), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.get_mut(*index), _ => None },
                                        )
                                    }
                                });
                            }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().map(|(_, v)| v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().map(|(_, v)| v), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut().map(|(_, v)| v), _ => None },
                                        )
                                    }
                                    pub fn #fr_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: &'static K) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.get(key), _ => None }
                                        )
                                    }
                                    pub fn #fw_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: &'static K) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.get(key), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.get_mut(key), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::Box, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => Some(&mut *v), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::Rc, Some(inner_ty))
                            | (WrapperKind::Arc, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::BTreeMap, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().map(|(_, v)| v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().map(|(_, v)| v), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut().map(|(_, v)| v), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::HashSet, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.iter().next(), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::BTreeSet, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.iter().next(), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::VecDeque, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.front(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.front(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.front_mut(), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::LinkedList, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.front(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.front(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.front_mut(), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.peek(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.peek(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.peek_mut().map(|v| &mut **v), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::Result, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().ok(), _ => None }
                                        )
                                    }
                                    // Note: Result<T, E> doesn't support writable access for inner type
                                    // Only providing readable access
                                });
                            }
                            (WrapperKind::Mutex, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> &'r #field_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None }
                                        )
                                    }
                                    // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                    // Only providing container-level access
                                });
                            }
                            (WrapperKind::RwLock, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> &'r #field_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None }
                                        )
                                    }
                                    // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                                    // Only providing container-level access
                                });
                            }
                            (WrapperKind::Weak, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> &'r #field_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None }
                                        )
                                    }
                                    // Note: Weak<T> doesn't support writable access (it's immutable)
                                    // Note: Weak<T> doesn't support direct access to inner type due to lifetime constraints
                                    // Only providing container-level access
                                });
                            }
                            // Nested container combinations for enums - COMMENTED OUT FOR NOW
                            /*
                            (WrapperKind::OptionBox, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().map(|b| &**b), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().map(|b| &**b), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.as_mut().map(|b| &mut **b), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::OptionRc, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().map(|r| &**r), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::OptionArc, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().map(|a| &**a), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::BoxOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> &'r #field_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #field_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #field_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => Some(&mut *v), _ => None },
                                        )
                                    }
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => (*v).as_ref(), _ => None }
                                        )
                                    }
                                    pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => (*v).as_ref(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => (*v).as_mut(), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::RcOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> &'r #field_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => (*v).as_ref(), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::ArcOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> &'r #field_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                    pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => (*v).as_ref(), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::VecOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().and_then(|opt| opt.as_ref()), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().and_then(|opt| opt.as_ref()), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut().and_then(|opt| opt.as_mut()), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::OptionVec, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().and_then(|vec| vec.first()), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().and_then(|vec| vec.first()), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.as_mut().and_then(|vec| vec.first_mut()), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::HashMapOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().and_then(|(_, opt)| opt.as_ref()), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().and_then(|(_, opt)| opt.as_ref()), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut().and_then(|(_, opt)| opt.as_mut()), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::OptionHashMap, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().and_then(|map| map.first().map(|(_, v)| v)), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                        rust_keypaths::EnumKeyPath::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().and_then(|map| map.first().map(|(_, v)| v)), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.as_mut().and_then(|map| map.first_mut().map(|(_, v)| v)), _ => None },
                                        )
                                    }
                                });
                            }
                            */
                            (WrapperKind::None, None) => {
                                let inner_ty = field_ty;
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::EnumKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>, impl Fn(#inner_ty) -> #name> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                        rust_keypaths::WritableOptionalKeyPath::new(
                                            |e: &mut #name| match e { #name::#v_ident(v) => Some(v), _ => None }
                                        )
                                    }
                                });
                            }
                            _ => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> &'r #field_ty> {
                                        rust_keypaths::EnumKeyPath::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&v), _ => None }
                                        )
                                    }
                                });
                            }
                        }
                    }
                    Fields::Unnamed(unnamed) if unnamed.unnamed.len() > 1 => {
                        // Multi-field tuple variants - generate methods for each field
                        for (index, field) in unnamed.unnamed.iter().enumerate() {
                            let field_ty = &field.ty;
                            let field_fn = format_ident!("f{}", index);
                            let r_fn = format_ident!("{}_{}_r", snake, field_fn);
                            let w_fn = format_ident!("{}_{}_w", snake, field_fn);
                            
                            // Generate pattern matching for this specific field
                            let mut pattern_parts = Vec::new();
                            
                            for i in 0..unnamed.unnamed.len() {
                                if i == index {
                                    pattern_parts.push(quote! { v });
                                } else {
                                    pattern_parts.push(quote! { _ });
                                }
                            }
                            
                            let pattern = quote! { #name::#v_ident(#(#pattern_parts),*) };
                            let match_expr = quote! { match e { #pattern => Some(v), _ => None } };
                            let match_mut_expr = quote! { match e { #pattern => Some(v), _ => None } };
                            
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|e: &#name| #match_expr)
                                }
                                pub fn #w_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #field_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|e: &mut #name| #match_mut_expr)
                                }
                            });
                        }
                    }
                    Fields::Named(named) => {
                        // Labeled enum variants - generate methods for each field
                        for field in named.named.iter() {
                            let field_ident = field.ident.as_ref().unwrap();
                            let field_ty = &field.ty;
                            let r_fn = format_ident!("{}_{}_r", snake, field_ident);
                            let w_fn = format_ident!("{}_{}_w", snake, field_ident);
                            
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|e: &#name| match e { #name::#v_ident { #field_ident: v, .. } => Some(v), _ => None })
                                }
                                pub fn #w_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #field_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|e: &mut #name| match e { #name::#v_ident { #field_ident: v, .. } => Some(v), _ => None })
                                }
                            });
                        }
                    }
                    _ => {
                        tokens.extend(quote! {
                            compile_error!("Keypaths derive supports only unit, single-field, multi-field tuple, and labeled variants");
                        });
                    }
                }
            }
            tokens
        }
        _ => quote! {
            compile_error!("Keypaths derive supports only structs and enums");
        },
    };

    let expanded = quote! {
        impl #name {
            #methods
        }
    };

    TokenStream::from(expanded)
}

fn extract_wrapper_inner_type(ty: &Type) -> (WrapperKind, Option<Type>) {
    use syn::{GenericArgument, PathArguments};
    
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            let ident_str = seg.ident.to_string();
            
            if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                let args: Vec<_> = ab.args.iter().collect();
                
                // Handle map types (HashMap, BTreeMap) - they have K, V parameters
                if ident_str == "HashMap" || ident_str == "BTreeMap" {
                    if let (Some(_key_arg), Some(value_arg)) = (args.get(0), args.get(1)) {
                        if let GenericArgument::Type(inner) = value_arg {
                            eprintln!("Detected {} type, extracting value type", ident_str);
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
                            ("Arc", WrapperKind::Mutex) => {
                                return (WrapperKind::ArcMutex, inner_inner);
                            }
                            ("Arc", WrapperKind::RwLock) => {
                                return (WrapperKind::ArcRwLock, inner_inner);
                            }
                            _ => {
                                // Handle single-level containers
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

/// Derives only writable keypath methods for struct fields.
///
/// This macro is a convenience wrapper that generates only writable keypaths,
/// equivalent to using `#[derive(Keypaths)]` with `#[Writable]` on the struct.
///
/// # Generated Methods
///
/// For each field `field_name`, generates:
///
/// - `field_name_w()` - Returns a `WritableKeyPath<Struct, FieldType>` for non-optional fields
/// - `field_name_fw()` - Returns a `WritableOptionalKeyPath<Struct, InnerType>` for optional/container fields
/// - `field_name_fw_at(index)` - Returns a `WritableOptionalKeyPath` for indexed mutable access
///
/// # Examples
///
/// ```rust,ignore
/// use keypaths_proc::WritableKeypaths;
///
/// #[derive(WritableKeypaths)]
/// struct Counter {
///     value: u32,
///     history: Vec<u32>,
/// }
///
/// // Usage:
/// let mut counter = Counter { value: 0, history: vec![] };
/// let value_path = Counter::value_w();
/// *value_path.get_mut(&mut counter) += 1;
/// ```
#[proc_macro_derive(WritableKeypaths)]
pub fn derive_writable_keypaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let methods = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;

                    let w_fn = format_ident!("{}_w", field_ident);
                    let fw_fn = format_ident!("{}_fw", field_ident);
                    let fw_at_fn = format_ident!("{}_fw_at", field_ident);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.as_mut())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.first_mut())
                                }
                                pub fn #fw_at_fn(index: usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(index))
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                }
                                pub fn #fw_at_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut *s.#field_ident)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut *s.#field_ident))
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                // Note: Rc/Arc are not writable due to shared ownership
                                // Only providing readable methods for these types
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                }
                                pub fn #fw_at_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: HashSet doesn't have direct mutable access to elements
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: BTreeSet doesn't have direct mutable access to elements
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.front_mut())
                                }
                                pub fn #fw_at_fn(index: usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(index))
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.front_mut())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: BinaryHeap peek_mut() returns PeekMut wrapper that doesn't allow direct mutable access
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: Result<T, E> doesn't support failable_writable for inner type
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                // Note: Weak<T> doesn't support writable access (it's immutable)
                                // No methods generated for Weak<T>
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut s.#field_ident))
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident)
                                }
                            });
                        }
                    }
                }
                tokens
            }
            Fields::Unnamed(unnamed) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for (idx, field) in unnamed.unnamed.iter().enumerate() {
                    let idx_lit = syn::Index::from(idx);
                    let ty = &field.ty;

                    let w_fn = format_ident!("f{}_w", idx);
                    let fw_fn = format_ident!("f{}_fw", idx);
                    let fw_at_fn = format_ident!("f{}_fw_at", idx);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.as_mut())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.first_mut())
                                }
                                pub fn #fw_at_fn(index: &'static usize) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.get_mut(*index))
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                }
                                pub fn #fw_at_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #inner_ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut *s.#idx_lit)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut *s.#idx_lit))
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                // Note: Rc/Arc are not writable due to shared ownership
                                // Only providing readable methods for these types
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                }
                                pub fn #fw_at_fn(key: String) -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: HashSet doesn't have direct mutable access to elements
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: BTreeSet doesn't have direct mutable access to elements
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.front_mut())
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#idx_lit.front_mut())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: BinaryHeap peek_mut() returns PeekMut wrapper that doesn't allow direct mutable access
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: Result<T, E> doesn't support failable_writable for inner type
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                // Note: Weak<T> doesn't support writable access (it's immutable)
                                // No methods generated for Weak<T>
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut s.#idx_lit))
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> rust_keypaths::WritableKeyPath<#name, #ty, impl for<'r> Fn(&'r mut #name) -> &'r mut #ty> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#idx_lit)
                                }
                            });
                        }
                    }
                }
                tokens
            }
            _ => quote! {
                compile_error!("WritableKeypaths derive supports only structs with named or unnamed fields");
            },
        },
        _ => quote! {
            compile_error!("WritableKeypaths derive supports only structs");
        },
    };

    let expanded = quote! {
        impl #name {
            #methods
        }
    };

    TokenStream::from(expanded)
}

/// Derives a single keypath method for each struct field.
///
/// This macro generates a simplified set of keypath methods, creating only
/// the most commonly used readable keypaths. It's a lighter-weight alternative
/// to `Keypaths` when you only need basic field access.
///
/// # Generated Methods
///
/// For each field `field_name`, generates:
///
/// - `field_name_r()` - Returns a `KeyPath<Struct, FieldType>` for direct field access
///
/// # Examples
///
/// ```rust,ignore
/// use keypaths_proc::Keypath;
///
/// #[derive(Keypath)]
/// struct Point {
///     x: f64,
///     y: f64,
/// }
///
/// // Usage:
/// let point = Point { x: 1.0, y: 2.0 };
/// let x_path = Point::x_r();
/// let x_value = x_path.get(&point);  // &f64
/// ```
#[proc_macro_derive(Keypath)]
pub fn derive_keypath(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let methods = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            // For Option<T>, return failable readable keypath to inner type
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            // For Vec<T>, return failable readable keypath to first element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.first())
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            // For HashMap<K,V>, return readable keypath to the container
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            // For BTreeMap<K,V>, return readable keypath to the container
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            // For Box<T>, return readable keypath to inner type
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            // For Rc<T>/Arc<T>, return readable keypath to inner type
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            // For HashSet<T>, return failable readable keypath to any element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.iter().next())
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            // For BTreeSet<T>, return failable readable keypath to any element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.iter().next())
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            // For VecDeque<T>, return failable readable keypath to front element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.front())
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            // For LinkedList<T>, return failable readable keypath to front element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.front())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            // For BinaryHeap<T>, return failable readable keypath to peek element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.peek())
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            // For Result<T, E>, return failable readable keypath to Ok value
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref().ok())
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            // For Mutex<T>, return readable keypath to the container (not inner type due to lifetime issues)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            // For RwLock<T>, return readable keypath to the container (not inner type due to lifetime issues)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            // For Weak<T>, return readable keypath to the container (not inner type due to lifetime issues)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::None, None) => {
                            // For basic types, return readable keypath
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        _ => {
                            // For unknown types, return readable keypath
                            tokens.extend(quote! {
                                pub fn #field_ident() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                    }
                }
                tokens
            }
            Fields::Unnamed(unnamed) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for (idx, field) in unnamed.unnamed.iter().enumerate() {
                    let idx_lit = syn::Index::from(idx);
                    let ty = &field.ty;
                    let field_name = format_ident!("f{}", idx);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.iter().next())
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.iter().next())
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.front())
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.front())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.peek())
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref().ok())
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                    }
                }
                tokens
            }
            _ => quote! {
                compile_error!("Keypath derive supports only structs with named or unnamed fields");
            },
        },
        Data::Enum(data_enum) => {
            let mut tokens = proc_macro2::TokenStream::new();
            for variant in data_enum.variants.iter() {
                let v_ident = &variant.ident;
                let snake = format_ident!("{}", to_snake_case(&v_ident.to_string()));

                match &variant.fields {
                    Fields::Unit => {
                        // Unit variant - return failable readable keypath to the variant itself
                        tokens.extend(quote! {
                            pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, (), impl for<'r> Fn(&'r #name) -> Option<&'r ()>> {
                                rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                    #name::#v_ident => {
                                        static UNIT: () = ();
                                        Some(&UNIT)
                                    },
                                    _ => None,
                                })
                            }
                        });
                    }
                    Fields::Unnamed(unnamed) => {
                        if unnamed.unnamed.len() == 1 {
                            // Single-field tuple variant - smart keypath selection
                            let field_ty = &unnamed.unnamed[0].ty;
                    let (kind, inner_ty) = extract_wrapper_inner_type(field_ty);

                    match (kind, inner_ty.clone()) {
                                (WrapperKind::Option, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.as_ref(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Vec, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.first(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::HashMap, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::BTreeMap, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Box, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(&**inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(&**inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::HashSet, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.iter().next(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::BTreeSet, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.iter().next(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::VecDeque, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.front(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::LinkedList, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.front(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.peek(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Result, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.as_ref().ok(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Mutex, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::RwLock, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Weak, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::None, None) => {
                                    // Basic type - return failable readable keypath
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                _ => {
                                    // Unknown type - return failable readable keypath
                                    tokens.extend(quote! {
                                        pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #field_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #field_ty>> {
                                            rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                            }
                        } else {
                            // Multi-field tuple variant - return failable readable keypath to the variant
                            tokens.extend(quote! {
                                pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #name, impl for<'r> Fn(&'r #name) -> Option<&'r #name>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                        #name::#v_ident(..) => Some(s),
                                        _ => None,
                                    })
                                }
                            });
                        }
                    }
                    Fields::Named(_named) => {
                        // Named field variant - return failable readable keypath to the variant
                        tokens.extend(quote! {
                            pub fn #snake() -> rust_keypaths::OptionalKeyPath<#name, #name, impl for<'r> Fn(&'r #name) -> Option<&'r #name>> {
                                rust_keypaths::OptionalKeyPath::new(|s: &#name| match s {
                                    #name::#v_ident { .. } => Some(s),
                                    _ => None,
                                })
                            }
                        });
                    }
                }
            }
            tokens
        }
        _ => quote! {
            compile_error!("Keypath derive supports only structs and enums");
        },
    };

    let expanded = quote! {
        impl #name {
            #methods
        }
    };

    TokenStream::from(expanded)
}

/// Derives only readable keypath methods for struct fields.
///
/// This macro is a convenience wrapper that generates only readable keypaths,
/// equivalent to using `#[derive(Keypaths)]` with `#[Readable]` on the struct.
///
/// # Generated Methods
///
/// For each field `field_name`, generates:
///
/// - `field_name_r()` - Returns a `KeyPath<Struct, FieldType>` for non-optional fields
/// - `field_name_fr()` - Returns an `OptionalKeyPath<Struct, InnerType>` for optional/container fields
/// - `field_name_fr_at(index)` - Returns an `OptionalKeyPath` for indexed access
///
/// # Examples
///
/// ```rust,ignore
/// use keypaths_proc::ReadableKeypaths;
///
/// #[derive(ReadableKeypaths)]
/// struct User {
///     name: String,
///     email: Option<String>,
///     tags: Vec<String>,
/// }
///
/// // Usage:
/// let user = User {
///     name: "Alice".to_string(),
///     email: Some("alice@example.com".to_string()),
///     tags: vec!["admin".to_string()],
/// };
///
/// let name_path = User::name_r();
/// let email_path = User::email_fr();
/// let name = name_path.get(&user);  // &String
/// let email = email_path.get(&user);  // Option<&String>
/// ```
#[proc_macro_derive(ReadableKeypaths)]
pub fn derive_readable_keypaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let methods = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;

                    let r_fn = format_ident!("{}_r", field_ident);
                    let fr_fn = format_ident!("{}_fr", field_ident);
                    let fr_at_fn = format_ident!("{}_fr_at", field_ident);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.first())
                                }
                                pub fn #fr_at_fn(index: usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(index))
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key))
                                }
                                pub fn #fr_at_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key))
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&*s.#field_ident))
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&*s.#field_ident))
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key))
                                }
                                pub fn #fr_at_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key))
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.iter().next())
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.iter().next())
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.front())
                                }
                                pub fn #fr_at_fn(index: usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(index))
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.front())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.peek())
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref().ok())
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                // Note: Weak<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#field_ident))
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                    }
                }
                tokens
            }
            Fields::Unnamed(unnamed) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for (idx, field) in unnamed.unnamed.iter().enumerate() {
                    let idx_lit = syn::Index::from(idx);
                    let ty = &field.ty;

                    let r_fn = format_ident!("f{}_r", idx);
                    let fr_fn = format_ident!("f{}_fr", idx);
                    let fr_at_fn = format_ident!("f{}_fr_at", idx);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.first())
                                }
                                pub fn #fr_at_fn(index: &'static usize) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.get(*index))
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#idx_lit.get(&key))
                                }
                                pub fn #fr_at_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#idx_lit.get(&key))
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&*s.#idx_lit))
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> &'r #inner_ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&*s.#idx_lit))
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#idx_lit.get(&key))
                                }
                                pub fn #fr_at_fn(key: String) -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#idx_lit.get(&key))
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.iter().next())
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.iter().next())
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.front())
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.front())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.peek())
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#idx_lit.as_ref().ok())
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                // Note: Weak<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> Option<&'r #ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#idx_lit))
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::KeyPath<#name, #ty, impl for<'r> Fn(&'r #name) -> &'r #ty> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                    }
                }
                tokens
            }
            _ => quote! {
                compile_error!("ReadableKeypaths derive supports only structs with named or unnamed fields");
            },
        },
        _ => quote! {
            compile_error!("ReadableKeypaths derive supports only structs");
        },
    };

    let expanded = quote! {
        impl #name {
            #methods
        }
    };

    TokenStream::from(expanded)
}

/// Derives case path methods for enum variants.
///
/// Case paths (also known as prisms) provide a way to access and manipulate
/// enum variants in a composable way. They allow you to extract values from
/// enum variants and embed values back into variants.
///
/// # Generated Methods
///
/// For each variant `VariantName` with a single field of type `T`:
///
/// - `variant_name_case_r()` - Returns an `OptionalKeyPath<Enum, T>` for reading
/// - `variant_name_case_w()` - Returns a `WritableOptionalKeyPath<Enum, T>` for writing
/// - `variant_name_case_fr()` - Alias for `variant_name_case_r()`
/// - `variant_name_case_fw()` - Alias for `variant_name_case_w()`
/// - `variant_name_case_embed(value)` - Returns `Enum` by embedding a value into the variant
/// - `variant_name_case_enum()` - Returns an `EnumKeyPath<Enum, T>` with both extraction and embedding
///
/// For unit variants (no fields):
///
/// - `variant_name_case_fr()` - Returns an `OptionalKeyPath<Enum, ()>` that checks if variant matches
///
/// For multi-field tuple variants:
///
/// - `variant_name_case_fr()` - Returns an `OptionalKeyPath<Enum, (T1, T2, ...)>` for the tuple
/// - `variant_name_case_fw()` - Returns a `WritableOptionalKeyPath<Enum, (T1, T2, ...)>` for the tuple
///
/// # Attributes
///
/// ## Enum-level attributes:
///
/// - `#[All]` - Generate all methods (readable and writable)
/// - `#[Readable]` - Generate only readable methods (default)
/// - `#[Writable]` - Generate only writable methods
///
/// ## Variant-level attributes:
///
/// - `#[Readable]` - Generate readable methods for this variant only
/// - `#[Writable]` - Generate writable methods for this variant only
/// - `#[All]` - Generate all methods for this variant
///
/// # Examples
///
/// ```rust,ignore
/// use keypaths_proc::Casepaths;
///
/// #[derive(Casepaths)]
/// #[All]
/// enum Status {
///     Active(String),
///     Inactive,
///     Pending(u32),
/// }
///
/// // Usage:
/// let mut status = Status::Active("online".to_string());
///
/// // Extract value from variant
/// let active_path = Status::active_case_r();
/// if let Some(value) = active_path.get(&status) {
///     println!("Status is: {}", value);
/// }
///
/// // Embed value into variant
/// let new_status = Status::active_case_embed("offline".to_string());
///
/// // Use EnumKeyPath for both extraction and embedding
/// let active_enum = Status::active_case_enum();
/// let extracted = active_enum.extract(&status);  // Option<&String>
/// let embedded = active_enum.embed("new".to_string());  // Status::Active("new")
/// ```
#[proc_macro_derive(Casepaths, attributes(Readable, Writable, All))]
pub fn derive_casepaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Get default scope from attributes
    let default_scope = match method_scope_from_attrs(&input.attrs) {
        Ok(Some(scope)) => scope,
        Ok(None) => MethodScope::Readable, // Default to readable
        Err(e) => return e.to_compile_error().into(),
    };

    let tokens = match input.data {
        Data::Enum(data_enum) => {
            let mut tokens = proc_macro2::TokenStream::new();
            for variant in data_enum.variants.iter() {
                let v_ident = &variant.ident;
                let snake = format_ident!("{}", to_snake_case(&v_ident.to_string()));
                
                // Get variant-specific scope
                let variant_scope = match method_scope_from_attrs(&variant.attrs) {
                    Ok(Some(scope)) => scope,
                    Ok(None) => default_scope.clone(),
                    Err(_) => default_scope.clone(),
                };
                
                let r_fn = format_ident!("{}_case_r", snake);
                let w_fn = format_ident!("{}_case_w", snake);
                let fr_fn = format_ident!("{}_case_fr", snake);
                let fw_fn = format_ident!("{}_case_fw", snake);

                match &variant.fields {
                    Fields::Unit => {
                        // Unit variants - return OptionalKeyPath that checks if variant matches
                        if variant_scope.includes_read() {
                            tokens.extend(quote! {
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, (), impl for<'r> Fn(&'r #name) -> Option<&'r ()>> {
                                    static UNIT: () = ();
                                    rust_keypaths::OptionalKeyPath::new(|e: &#name| match e { #name::#v_ident => Some(&UNIT), _ => None })
                                }
                            });
                        }
                    }
                    Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
                        let inner_ty = &unnamed.unnamed.first().unwrap().ty;
                        
                        // Single-field variant - extract the inner value
                        // Generate EnumKeyPath for single-field variants to support embedding
                        if variant_scope.includes_read() {
                            let embed_fn = format_ident!("{}_case_embed", snake);
                            let enum_kp_fn = format_ident!("{}_case_enum", snake);
                            tokens.extend(quote! {
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None })
                                }
                                // Alias for fr_fn - returns OptionalKeyPath (enum casepaths are always optional)
                                pub fn #r_fn() -> rust_keypaths::OptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None })
                                }
                                // EnumKeyPath version with embedding support
                                pub fn #enum_kp_fn() -> rust_keypaths::EnumKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #inner_ty> + 'static, impl Fn(#inner_ty) -> #name + 'static> {
                                    rust_keypaths::EnumKeyPath::readable_enum(
                                        |value: #inner_ty| #name::#v_ident(value),
                                        |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None }
                                    )
                                }
                                // Embed method - creates the enum variant from a value
                                pub fn #embed_fn(value: #inner_ty) -> #name {
                                    #name::#v_ident(value)
                                }
                            });
                        }
                        if variant_scope.includes_write() {
                            tokens.extend(quote! {
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|e: &mut #name| match e { #name::#v_ident(v) => Some(v), _ => None })
                                }
                                // Alias for fw_fn - returns WritableOptionalKeyPath (enum casepaths are always optional)
                                pub fn #w_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #inner_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #inner_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|e: &mut #name| match e { #name::#v_ident(v) => Some(v), _ => None })
                                }
                            });
                        }
                    }
                    // Multi-field tuple variant: Enum::Variant(T1, T2, ...)
                    Fields::Unnamed(unnamed) => {
                        let field_types: Vec<_> = unnamed.unnamed.iter().map(|f| &f.ty).collect();
                        let tuple_ty = quote! { (#(#field_types),*) };
                        
                        // Generate pattern matching for tuple fields
                        let field_patterns: Vec<_> = (0..unnamed.unnamed.len())
                            .map(|i| format_ident!("f{}", i))
                            .collect();
                        
                        if variant_scope.includes_read() {
                            tokens.extend(quote! {
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #tuple_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #tuple_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|e: &#name| match e { #name::#v_ident(#(#field_patterns),*) => Some(&(#(#field_patterns),*)), _ => None })
                                }
                            });
                        }
                        if variant_scope.includes_write() {
                            tokens.extend(quote! {
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #tuple_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #tuple_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|e: &mut #name| match e { #name::#v_ident(#(#field_patterns),*) => Some((#(#field_patterns),*)), _ => None })
                                }
                            });
                        }
                    }
                    
                    // Labeled variant: Enum::Variant { field1: T1, field2: T2, ... }
                    Fields::Named(named) => {
                        let field_names: Vec<_> = named.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                        let field_types: Vec<_> = named.named.iter().map(|f| &f.ty).collect();
                        let tuple_ty = quote! { (#(#field_types),*) };
                        
                        if variant_scope.includes_read() {
                            tokens.extend(quote! {
                                pub fn #fr_fn() -> rust_keypaths::OptionalKeyPath<#name, #tuple_ty, impl for<'r> Fn(&'r #name) -> Option<&'r #tuple_ty>> {
                                    rust_keypaths::OptionalKeyPath::new(|e: &#name| match e { #name::#v_ident { #(#field_names: ref #field_names),* } => Some(&(#(#field_names),*)), _ => None })
                                }
                            });
                        }
                        if variant_scope.includes_write() {
                            tokens.extend(quote! {
                                pub fn #fw_fn() -> rust_keypaths::WritableOptionalKeyPath<#name, #tuple_ty, impl for<'r> Fn(&'r mut #name) -> Option<&'r mut #tuple_ty>> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|e: &mut #name| match e { #name::#v_ident { #(#field_names: ref mut #field_names),* } => Some((#(#field_names),*)), _ => None })
                                }
                            });
                        }
                    }
                }
            }
            tokens
        }
        _ => quote! { compile_error!("Casepaths can only be derived for enums"); },
    };

    let expanded = quote! {
        impl #name {
            #tokens
        }
    };

    TokenStream::from(expanded)
}

/// Derives type-erased keypath methods with known root type.
///
/// `PartialKeyPath` is similar to Swift's `PartialKeyPath<Root>`. It hides
/// the `Value` type but keeps the `Root` type visible. This is useful for
/// storing collections of keypaths with the same root type but different value types.
///
/// # Generated Methods
///
/// For each field `field_name`, generates:
///
/// - `field_name_r()` - Returns a `PartialKeyPath<Struct>` for readable access
/// - `field_name_w()` - Returns a `PartialWritableKeyPath<Struct>` for writable access
/// - `field_name_fr()` - Returns a `PartialOptionalKeyPath<Struct>` for optional fields
/// - `field_name_fw()` - Returns a `PartialWritableOptionalKeyPath<Struct>` for optional writable fields
///
/// # Type Erasure
///
/// The `get()` method returns `&dyn Any`, requiring downcasting to access the actual value.
/// Use `get_as::<Root, Value>()` for type-safe access when you know the value type.
///
/// # Examples
///
/// ```rust,ignore
/// use keypaths_proc::PartialKeypaths;
/// use rust_keypaths::PartialKeyPath;
///
/// #[derive(PartialKeypaths)]
/// struct User {
///     name: String,
///     age: u32,
///     email: Option<String>,
/// }
///
/// // Usage:
/// let mut paths: Vec<PartialKeyPath<User>> = vec![
///     User::name_r(),
///     User::age_r(),
/// ];
///
/// let user = User {
///     name: "Alice".to_string(),
///     age: 30,
///     email: Some("alice@example.com".to_string()),
/// };
///
/// // Access values (requires type information)
/// if let Some(name) = paths[0].get_as::<User, String>(&user) {
///     println!("Name: {}", name);
/// }
/// ```
#[proc_macro_derive(PartialKeypaths)]
pub fn derive_partial_keypaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let methods = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;

                    let r_fn = format_ident!("{}_partial_r", field_ident);
                    let w_fn = format_ident!("{}_partial_w", field_ident);
                    let fr_fn = format_ident!("{}_partial_fr", field_ident);
                    let fw_fn = format_ident!("{}_partial_fw", field_ident);
                    let fr_at_fn = format_ident!("{}_partial_fr_at", field_ident);
                    let fw_at_fn = format_ident!("{}_partial_fw_at", field_ident);
                    // Owned keypath method names
                    let o_fn = format_ident!("{}_partial_o", field_ident);
                    let fo_fn = format_ident!("{}_partial_fo", field_ident);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::PartialOptionalKeyPath<#name> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref()).to_partial()
                                }
                                pub fn #w_fn() -> rust_keypaths::PartialWritableOptionalKeyPath<#name> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.as_mut()).to_partial()
                                }
                                pub fn #fr_fn() -> rust_keypaths::PartialOptionalKeyPath<#name> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref()).to_partial()
                                }
                                pub fn #fw_fn() -> rust_keypaths::PartialWritableOptionalKeyPath<#name> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.as_mut()).to_partial()
                                }
                                // Owned keypath methods - these don't make sense for Option types
                                // as we can't return owned values from references
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #fr_at_fn(index: usize) -> rust_keypaths::PartialOptionalKeyPath<#name> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(index)).to_partial()
                                }
                                pub fn #fw_at_fn(index: usize) -> rust_keypaths::PartialWritableOptionalKeyPath<#name> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(index)).to_partial()
                                }
                                pub fn #r_fn() -> rust_keypaths::PartialKeyPath<#name> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident).to_partial()
                                }
                                pub fn #w_fn() -> rust_keypaths::PartialWritableKeyPath<#name> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident).to_partial()
                                }
                                pub fn #fr_fn() -> rust_keypaths::PartialOptionalKeyPath<#name> {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.first()).to_partial()
                                }
                                pub fn #fw_fn() -> rust_keypaths::PartialWritableOptionalKeyPath<#name> {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.first_mut()).to_partial()
                                }
                                // Owned keypath methods - not supported for Vec as we need references
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::PartialKeyPath<#name> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident).to_partial()
                                }
                                pub fn #w_fn() -> rust_keypaths::PartialWritableKeyPath<#name> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident).to_partial()
                                }
                                pub fn #fr_fn(key: String) -> rust_keypaths::PartialOptionalKeyPath<#name> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key)).to_partial()
                                }
                                pub fn #fw_fn(key: String) -> rust_keypaths::PartialWritableOptionalKeyPath<#name> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key)).to_partial()
                                }
                                pub fn #fr_at_fn(key: String) -> rust_keypaths::PartialOptionalKeyPath<#name> {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key)).to_partial()
                                }
                                pub fn #fw_at_fn(key: String) -> rust_keypaths::PartialWritableOptionalKeyPath<#name> {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key)).to_partial()
                                }
                                // Owned keypath methods - not supported for HashMap as we need references
                            });
                        }
                        _ => {
                            // Default case for simple types
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::PartialKeyPath<#name> {
                                    rust_keypaths::KeyPath::new(|s: &#name| &s.#field_ident).to_partial()
                                }
                                pub fn #w_fn() -> rust_keypaths::PartialWritableKeyPath<#name> {
                                    rust_keypaths::WritableKeyPath::new(|s: &mut #name| &mut s.#field_ident).to_partial()
                                }
                                // Owned keypath methods - not supported as we need references
                            });
                        }
                    }
                }
                tokens
            }
            _ => quote! { compile_error!("PartialKeypaths can only be derived for structs with named fields"); },
        },
        _ => quote! { compile_error!("PartialKeypaths can only be derived for structs"); },
    };

    let expanded = quote! {
        impl #name {
            #methods
        }
    };

    TokenStream::from(expanded)
}

/// Derives fully type-erased keypath methods.
///
/// `AnyKeyPath` is similar to Swift's `AnyKeyPath`. It hides both the `Root`
/// and `Value` types, making it useful for storing keypaths from different
/// struct types in the same collection.
///
/// # Generated Methods
///
/// For each field `field_name`, generates:
///
/// - `field_name_r()` - Returns an `AnyKeyPath` for readable access
/// - `field_name_w()` - Returns an `AnyWritableKeyPath` for writable access
/// - `field_name_fr()` - Returns an `AnyKeyPath` for optional fields
/// - `field_name_fw()` - Returns an `AnyWritableKeyPath` for optional writable fields
///
/// # Type Erasure
///
/// The `get()` method returns `&dyn Any`, requiring downcasting to access the actual value.
/// Use `get_as::<Root, Value>()` for type-safe access when you know both root and value types.
///
/// # Examples
///
/// ```rust,ignore
/// use keypaths_proc::AnyKeypaths;
/// use rust_keypaths::AnyKeyPath;
///
/// #[derive(AnyKeypaths)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// #[derive(AnyKeypaths)]
/// struct Product {
///     price: f64,
/// }
///
/// // Usage:
/// let mut paths: Vec<AnyKeyPath> = vec![
///     User::name_r(),
///     Product::price_r(),
/// ];
///
/// let user = User {
///     name: "Alice".to_string(),
///     age: 30,
/// };
///
/// // Access values (requires both root and value type information)
/// if let Some(name) = paths[0].get_as::<User, String>(&user) {
///     println!("Name: {}", name);
/// }
/// ```
#[proc_macro_derive(AnyKeypaths)]
pub fn derive_any_keypaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let methods = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => {
                let mut tokens = proc_macro2::TokenStream::new();
                for field in fields_named.named.iter() {
                    let field_ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;

                    let r_fn = format_ident!("{}_any_r", field_ident);
                    let w_fn = format_ident!("{}_any_w", field_ident);
                    let fr_fn = format_ident!("{}_any_fr", field_ident);
                    let fw_fn = format_ident!("{}_any_fw", field_ident);
                    let fr_at_fn = format_ident!("{}_any_fr_at", field_ident);
                    let fw_at_fn = format_ident!("{}_any_fw_at", field_ident);
                    // Owned keypath method names
                    let o_fn = format_ident!("{}_any_o", field_ident);
                    let fo_fn = format_ident!("{}_any_fo", field_ident);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty.clone()) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref()).to_any()
                                }
                                pub fn #w_fn() -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.as_mut()).to_any()
                                }
                                pub fn #fr_fn() -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.as_ref()).to_any()
                                }
                                pub fn #fw_fn() -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.as_mut()).to_any()
                                }
                                // Owned keypath methods - not supported for Option types
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #fr_at_fn(index: usize) -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(index)).to_any()
                                }
                                pub fn #fw_at_fn(index: usize) -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(index)).to_any()
                                }
                                pub fn #r_fn() -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#field_ident)).to_any()
                                }
                                pub fn #w_fn() -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut s.#field_ident)).to_any()
                                }
                                pub fn #fr_fn() -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| s.#field_ident.first()).to_any()
                                }
                                pub fn #fw_fn() -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| s.#field_ident.first_mut()).to_any()
                                }
                                // Owned keypath methods - not supported for Vec
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#field_ident)).to_any()
                                }
                                pub fn #w_fn() -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut s.#field_ident)).to_any()
                                }
                                pub fn #fr_fn(key: String) -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key)).to_any()
                                }
                                pub fn #fw_fn(key: String) -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key)).to_any()
                                }
                                pub fn #fr_at_fn(key: String) -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(move |s: &#name| s.#field_ident.get(&key)).to_any()
                                }
                                pub fn #fw_at_fn(key: String) -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(move |s: &mut #name| s.#field_ident.get_mut(&key)).to_any()
                                }
                                // Owned keypath methods - not supported for HashMap
                            });
                        }
                        _ => {
                            // Default case for simple types
                            tokens.extend(quote! {
                                pub fn #r_fn() -> rust_keypaths::AnyKeyPath {
                                    rust_keypaths::OptionalKeyPath::new(|s: &#name| Some(&s.#field_ident)).to_any()
                                }
                                pub fn #w_fn() -> rust_keypaths::AnyWritableKeyPath {
                                    rust_keypaths::WritableOptionalKeyPath::new(|s: &mut #name| Some(&mut s.#field_ident)).to_any()
                                }
                                // Owned keypath methods - not supported as we need references
                            });
                        }
                    }
                }
                tokens
            }
            _ => quote! { compile_error!("AnyKeypaths can only be derived for structs with named fields"); },
        },
        _ => quote! { compile_error!("AnyKeypaths can only be derived for structs"); },
    };

    let expanded = quote! {
        impl #name {
            #methods
        }
    };

    TokenStream::from(expanded)
}

// /// A helper macro that provides suggestions when there are type mismatches with container types.
// /// This macro helps users understand when to use adapter methods like for_arc(), for_box(), etc.
// #[proc_macro]
// pub fn keypath_suggestion(input: TokenStream) -> TokenStream {
//     let input_str = input.to_string();
//     
//     // Parse the input to understand what the user is trying to do
//     let suggestion = if input_str.contains("Arc<") && input_str.contains("KeyPaths<") {
//         "💡 Suggestion: If you have a KeyPaths<SomeStruct, Value> but need KeyPaths<Arc<SomeStruct>, Value>, use the .for_arc() adapter method:\n   let arc_keypath = your_keypath.for_arc();"
//     } else if input_str.contains("Box<") && input_str.contains("KeyPaths<") {
//         "💡 Suggestion: If you have a KeyPaths<SomeStruct, Value> but need KeyPaths<Box<SomeStruct>, Value>, use the .for_box() adapter method:\n   let box_keypath = your_keypath.for_box();"
//     } else if input_str.contains("Rc<") && input_str.contains("KeyPaths<") {
//         "💡 Suggestion: If you have a KeyPaths<SomeStruct, Value> but need KeyPaths<Rc<SomeStruct>, Value>, use the .for_rc() adapter method:\n   let rc_keypath = your_keypath.for_rc();"
//     } else if input_str.contains("Option<") && input_str.contains("KeyPaths<") {
//         "💡 Suggestion: If you have a KeyPaths<SomeStruct, Value> but need KeyPaths<Option<SomeStruct>, Value>, use the .for_option() adapter method:\n   let option_keypath = your_keypath.for_option();"
//     } else if input_str.contains("Result<") && input_str.contains("KeyPaths<") {
//         "💡 Suggestion: If you have a KeyPaths<SomeStruct, Value> but need KeyPaths<Result<SomeStruct, E>, Value>, use the .for_result() adapter method:\n   let result_keypath = your_keypath.for_result();"
//     } else if input_str.contains("Mutex<") && input_str.contains("KeyPaths<") {
//         "💡 Suggestion: For Mutex<T> containers, use the .with_mutex() method from WithContainer trait (no cloning):\n   use rust_keypaths::WithContainer;\n   your_keypath.with_mutex(&mutex, |value| { /* work with value */ });"
//     } else if input_str.contains("RwLock<") && input_str.contains("KeyPaths<") {
//         "💡 Suggestion: For RwLock<T> containers, use the .with_rwlock() method from WithContainer trait (no cloning):\n   use rust_keypaths::WithContainer;\n   your_keypath.with_rwlock(&rwlock, |value| { /* work with value */ });"
//     } else {
//         "💡 Suggestion: Use adapter methods to work with different container types:\n   - .for_arc() for Arc<T>\n   - .for_box() for Box<T>\n   - .for_rc() for Rc<T>\n   - .for_option() for Option<T>\n   - .for_result() for Result<T, E>\n   - .with_mutex() for Mutex<T> (import WithContainer trait)\n   - .with_rwlock() for RwLock<T> (import WithContainer trait)\n   - .for_arc_mutex() for Arc<Mutex<T>> (with parking_lot feature)\n   - .for_arc_rwlock() for Arc<RwLock<T>> (with parking_lot feature)"
//     };
//     
//     let expanded = quote! {
//         compile_error!(#suggestion);
//     };
//     
//     TokenStream::from(expanded)
// }

// /// A helper macro that provides compile-time suggestions for common KeyPaths usage patterns.
// /// This macro can be used to get helpful error messages when there are type mismatches.
// #[proc_macro]
// pub fn keypath_help(input: TokenStream) -> TokenStream {
//     let input_str = input.to_string();
//     
//     let help_message = if input_str.is_empty() {
//         "🔧 KeyPaths Help: Use adapter methods to work with different container types:\n   - .for_arc() for Arc<T> containers\n   - .for_box() for Box<T> containers\n   - .for_rc() for Rc<T> containers\n   - .for_option() for Option<T> containers\n   - .for_result() for Result<T, E> containers\n   - .with_mutex() for Mutex<T> containers (import WithContainer trait)\n   - .with_rwlock() for RwLock<T> containers (import WithContainer trait)\n   - .for_arc_mutex() for Arc<Mutex<T>> containers (with parking_lot feature)\n   - .for_arc_rwlock() for Arc<RwLock<T>> containers (with parking_lot feature)\n\nExample: let arc_keypath = my_keypath.for_arc();\nFor Mutex/RwLock: use rust_keypaths::WithContainer; then my_keypath.with_mutex(&mutex, |value| { ... });\nFor Arc<Mutex>/Arc<RwLock>: let arc_mutex_keypath = my_keypath.for_arc_mutex();".to_string()
//     } else {
//         format!("🔧 KeyPaths Help for '{}': Use adapter methods to work with different container types. See documentation for more details.", input_str)
//     };
//     
//     let expanded = quote! {
//         compile_error!(#help_message);
//     };
//     
//     TokenStream::from(expanded)
// }
