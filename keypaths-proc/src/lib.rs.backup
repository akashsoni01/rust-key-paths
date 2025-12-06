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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_read = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_read> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_write = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_write> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.as_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_owned = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_owned> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident)
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
                                    pub fn #fr_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.first())
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.first_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_values().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &*s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut *s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut *s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| *s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| Some(*s.#field_ident))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &*s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| (*s.#field_ident).clone())
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| Some((*s.#field_ident).clone()))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_values().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.iter().next())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.iter().next())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.front())
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.front_mut())
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.front())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.front_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref().ok())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.ok())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref().map(|b| &**b))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.as_mut().map(|b| &mut **b))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.map(|b| *b))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref().map(|r| &**r))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.map(|r| (*r).clone()))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref().map(|a| &**a))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.map(|a| (*a).clone()))
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
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| (*s.#field_ident).as_ref())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| (*s.#field_ident).as_mut())
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
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| (*s.#field_ident).as_ref())
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
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| (*s.#field_ident).as_ref())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.first().and_then(|opt| opt.as_ref()))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(index).and_then(|opt| opt.as_ref()))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.first_mut().and_then(|opt| opt.as_mut()))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(index).and_then(|opt| opt.as_mut()))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().flatten().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref().and_then(|v| v.first()))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.as_ref().and_then(|v| v.get(index)))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.as_mut().and_then(|v| v.first_mut()))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.as_mut().and_then(|v| v.get_mut(index)))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.and_then(|v| v.into_iter().next()))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key).and_then(|opt| opt.as_ref()))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key).and_then(|opt| opt.as_ref()))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key).and_then(|opt| opt.as_mut()))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key).and_then(|opt| opt.as_mut()))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_values().flatten().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.as_ref().and_then(|m| m.get(&key)))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.as_ref().and_then(|m| m.get(&key)))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.as_mut().and_then(|m| m.get_mut(&key)))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.as_mut().and_then(|m| m.get_mut(&key)))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.and_then(|m| m.into_values().next()))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut s.#field_ident))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| Some(s.#field_ident))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.as_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.first())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.first_mut())
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(index: &'static usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.get(*index))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(index: &'static usize) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.get_mut(*index))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#idx_lit.get(&key))
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                    }
                                },
                            );
                            let inner_ty_fr_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty_fr_at> {
                                        key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#idx_lit.get(&key))
                                    }
                                },
                            );
                            let inner_ty_fw_at = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty_fw_at> {
                                        key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.into_values().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_read> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_write = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_write> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut *s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#idx_lit))
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut *s.#idx_lit))
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_owned> {
                                        key_paths_core::KeyPaths::owned(|s: #name| *s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| Some(*s.#idx_lit))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_read> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#idx_lit))
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_owned> {
                                        key_paths_core::KeyPaths::owned(|s: #name| (*s.#idx_lit).clone())
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| Some((*s.#idx_lit).clone()))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.into_values().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.iter().next())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.iter().next())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.front())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.front_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.front())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.front_mut())
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.peek())
                                    }
                                },
                            );
                            let inner_ty_fw = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fw> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.peek_mut().map(|v| &mut **v))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.into_iter().next())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fr = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fr> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref().ok())
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            let inner_ty_fo = inner_ty.clone();
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #inner_ty_fo> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| s.#idx_lit.ok())
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
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
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                        }
                        // Nested container combinations for tuple structs - COMMENTED OUT FOR NOW
                        /*
                        (WrapperKind::OptionBox, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref().map(|b| &**b))
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.as_mut().map(|b| &mut **b))
                                }
                            });
                        }
                        (WrapperKind::OptionRc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref().map(|r| &**r))
                                }
                            });
                        }
                        (WrapperKind::OptionArc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref().map(|a| &**a))
                                }
                            });
                        }
                        (WrapperKind::BoxOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut *s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| (*s.#idx_lit).as_ref())
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| (*s.#idx_lit).as_mut())
                                }
                            });
                        }
                        (WrapperKind::RcOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| (*s.#idx_lit).as_ref())
                                }
                            });
                        }
                        (WrapperKind::ArcOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| (*s.#idx_lit).as_ref())
                                }
                            });
                        }
                        (WrapperKind::VecOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.first().and_then(|opt| opt.as_ref()))
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.first_mut().and_then(|opt| opt.as_mut()))
                                }
                            });
                        }
                        (WrapperKind::OptionVec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref().and_then(|v| v.first()))
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.as_mut().and_then(|v| v.first_mut()))
                                }
                            });
                        }
                        (WrapperKind::HashMapOption, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#idx_lit.get(&key).and_then(|opt| opt.as_ref()))
                                }
                                pub fn #fw_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#idx_lit.get_mut(&key).and_then(|opt| opt.as_mut()))
                                }
                            });
                        }
                        (WrapperKind::OptionHashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#idx_lit.as_ref().and_then(|m| m.get(&key)))
                                }
                                pub fn #fw_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: K) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#idx_lit.as_mut().and_then(|m| m.get_mut(&key)))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Readable,
                                quote! {
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&s.#idx_lit))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Writable,
                                quote! {
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut s.#idx_lit))
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    pub fn #fo_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::failable_owned(|s: #name| Some(s.#idx_lit))
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                    }
                                },
                            );
                            push_method(
                                &mut tokens,
                                method_scope,
                                MethodKind::Owned,
                                quote! {
                                    // Owned keypath methods
                                    pub fn #o_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                        key_paths_core::KeyPaths::owned(|s: #name| s.#idx_lit)
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
                            pub fn #r_fn() -> key_paths_core::KeyPaths<#name, ()> {
                                static UNIT: () = ();
                                key_paths_core::KeyPaths::readable_enum(
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.as_mut(), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::Vec, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut(), _ => None },
                                        )
                                    }
                                    pub fn #fr_at_fn(index: &'static usize) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.get(*index), _ => None }
                                        )
                                    }
                                    pub fn #fw_at_fn(index: &'static usize) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.get(*index), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.get_mut(*index), _ => None },
                                        )
                                    }
                                });
                            }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().map(|(_, v)| v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().map(|(_, v)| v), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut().map(|(_, v)| v), _ => None },
                                        )
                                    }
                                    pub fn #fr_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: &'static K) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.get(key), _ => None }
                                        )
                                    }
                                    pub fn #fw_at_fn<K: ::std::hash::Hash + ::std::cmp::Eq + 'static>(key: &'static K) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.get(key), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.get_mut(key), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::Box, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::BTreeMap, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().map(|(_, v)| v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().map(|(_, v)| v), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut().map(|(_, v)| v), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::HashSet, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.iter().next(), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::BTreeSet, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.iter().next(), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::VecDeque, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.front(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.front(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.front_mut(), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::LinkedList, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.front(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.front(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.front_mut(), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.peek(), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.peek(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.peek_mut().map(|v| &mut **v), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::Result, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().map(|b| &**b), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().map(|b| &**b), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.as_mut().map(|b| &mut **b), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::OptionRc, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().map(|r| &**r), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::OptionArc, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().map(|a| &**a), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::BoxOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => Some(&mut *v), _ => None },
                                        )
                                    }
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => (*v).as_ref(), _ => None }
                                        )
                                    }
                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => (*v).as_ref(), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => (*v).as_mut(), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::RcOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => (*v).as_ref(), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::ArcOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(&*v), _ => None }
                                        )
                                    }
                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => (*v).as_ref(), _ => None }
                                        )
                                    }
                                });
                            }
                            (WrapperKind::VecOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().and_then(|opt| opt.as_ref()), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().and_then(|opt| opt.as_ref()), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut().and_then(|opt| opt.as_mut()), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::OptionVec, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().and_then(|vec| vec.first()), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().and_then(|vec| vec.first()), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.as_mut().and_then(|vec| vec.first_mut()), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::HashMapOption, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().and_then(|(_, opt)| opt.as_ref()), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.first().and_then(|(_, opt)| opt.as_ref()), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => v.first_mut().and_then(|(_, opt)| opt.as_mut()), _ => None },
                                        )
                                    }
                                });
                            }
                            (WrapperKind::OptionHashMap, Some(inner_ty)) => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => v.as_ref().and_then(|map| map.first().map(|(_, v)| v)), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
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
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None }
                                        )
                                    }
                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                        key_paths_core::KeyPaths::writable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None },
                                            |e: &mut #name| match e { #name::#v_ident(v) => Some(v), _ => None },
                                        )
                                    }
                                });
                            }
                            _ => {
                                tokens.extend(quote! {
                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                        key_paths_core::KeyPaths::readable_enum(
                                            #name::#v_ident,
                                            |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None }
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
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|e: &#name| #match_expr)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|e: &mut #name| #match_mut_expr)
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
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|e: &#name| match e { #name::#v_ident { #field_ident: v, .. } => Some(v), _ => None })
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|e: &mut #name| match e { #name::#v_ident { #field_ident: v, .. } => Some(v), _ => None })
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
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.as_mut())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.first_mut())
                                }
                                pub fn #fw_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(index))
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                }
                                pub fn #fw_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut *s.#field_ident)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut *s.#field_ident))
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
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                }
                                pub fn #fw_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key))
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: HashSet doesn't have direct mutable access to elements
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: BTreeSet doesn't have direct mutable access to elements
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.front_mut())
                                }
                                pub fn #fw_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(index))
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.front_mut())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: BinaryHeap peek_mut() returns PeekMut wrapper that doesn't allow direct mutable access
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: Result<T, E> doesn't support failable_writable for inner type
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
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
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut s.#field_ident))
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
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
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.as_mut())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.first_mut())
                                }
                                pub fn #fw_at_fn(index: &'static usize) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.get_mut(*index))
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                }
                                pub fn #fw_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut *s.#idx_lit)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut *s.#idx_lit))
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
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                }
                                pub fn #fw_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#idx_lit.get_mut(&key))
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: HashSet doesn't have direct mutable access to elements
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: BTreeSet doesn't have direct mutable access to elements
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.front_mut())
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.front_mut())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: BinaryHeap peek_mut() returns PeekMut wrapper that doesn't allow direct mutable access
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: Result<T, E> doesn't support failable_writable for inner type
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level writable access
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
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
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut s.#idx_lit))
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
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
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            // For Vec<T>, return failable readable keypath to first element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.first())
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            // For HashMap<K,V>, return readable keypath to the container
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            // For BTreeMap<K,V>, return readable keypath to the container
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            // For Box<T>, return readable keypath to inner type
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            // For Rc<T>/Arc<T>, return readable keypath to inner type
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            // For HashSet<T>, return failable readable keypath to any element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.iter().next())
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            // For BTreeSet<T>, return failable readable keypath to any element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.iter().next())
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            // For VecDeque<T>, return failable readable keypath to front element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.front())
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            // For LinkedList<T>, return failable readable keypath to front element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.front())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            // For BinaryHeap<T>, return failable readable keypath to peek element
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.peek())
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            // For Result<T, E>, return failable readable keypath to Ok value
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref().ok())
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            // For Mutex<T>, return readable keypath to the container (not inner type due to lifetime issues)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            // For RwLock<T>, return readable keypath to the container (not inner type due to lifetime issues)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            // For Weak<T>, return readable keypath to the container (not inner type due to lifetime issues)
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        (WrapperKind::None, None) => {
                            // For basic types, return readable keypath
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                            });
                        }
                        _ => {
                            // For unknown types, return readable keypath
                            tokens.extend(quote! {
                                pub fn #field_ident() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
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
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.first())
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.iter().next())
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.iter().next())
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.front())
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.front())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.peek())
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref().ok())
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #field_name() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
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
                        // Unit variant - return readable keypath to the variant itself
                        tokens.extend(quote! {
                            pub fn #snake() -> key_paths_core::KeyPaths<#name, #name> {
                                key_paths_core::KeyPaths::readable(|s: &#name| s)
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
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.as_ref(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Vec, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.first(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::HashMap, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::BTreeMap, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Box, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(&**inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(&**inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::HashSet, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.iter().next(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::BTreeSet, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.iter().next(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::VecDeque, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.front(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::LinkedList, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.front(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.peek(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Result, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => inner.as_ref().ok(),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Mutex, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::RwLock, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::Weak, Some(inner_ty)) => {
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                (WrapperKind::None, None) => {
                                    // Basic type - return failable readable keypath
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
                                                #name::#v_ident(inner) => Some(inner),
                                                _ => None,
                                            })
                                        }
                                    });
                                }
                                _ => {
                                    // Unknown type - return failable readable keypath
                                    tokens.extend(quote! {
                                        pub fn #snake() -> key_paths_core::KeyPaths<#name, #field_ty> {
                                            key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
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
                                pub fn #snake() -> key_paths_core::KeyPaths<#name, #name> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
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
                            pub fn #snake() -> key_paths_core::KeyPaths<#name, #name> {
                                key_paths_core::KeyPaths::failable_readable(|s: &#name| match s {
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
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.first())
                                }
                                pub fn #fr_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(index))
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key))
                                }
                                pub fn #fr_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key))
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#field_ident))
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#field_ident))
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key))
                                }
                                pub fn #fr_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key))
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.iter().next())
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.iter().next())
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.front())
                                }
                                pub fn #fr_at_fn(index: usize) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(index))
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.front())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.peek())
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref().ok())
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                // Note: Weak<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&s.#field_ident))
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
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
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref())
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.first())
                                }
                                pub fn #fr_at_fn(index: &'static usize) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.get(*index))
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#idx_lit.get(&key))
                                }
                                pub fn #fr_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#idx_lit.get(&key))
                                }
                            });
                        }
                        (WrapperKind::Box, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#idx_lit))
                                }
                            });
                        }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#idx_lit))
                                }
                            });
                        }
                        (WrapperKind::BTreeMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#idx_lit.get(&key))
                                }
                                pub fn #fr_at_fn(key: String) -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#idx_lit.get(&key))
                                }
                            });
                        }
                        (WrapperKind::HashSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.iter().next())
                                }
                            });
                        }
                        (WrapperKind::BTreeSet, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.iter().next())
                                }
                            });
                        }
                        (WrapperKind::VecDeque, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.front())
                                }
                            });
                        }
                        (WrapperKind::LinkedList, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.front())
                                }
                            });
                        }
                        (WrapperKind::BinaryHeap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.peek())
                                }
                            });
                        }
                        (WrapperKind::Result, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref().ok())
                                }
                            });
                        }
                        (WrapperKind::Mutex, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                // Note: Mutex<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::RwLock, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                // Note: RwLock<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::Weak, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                // Note: Weak<T> doesn't support direct access to inner type due to lifetime constraints
                                // Only providing container-level access
                            });
                        }
                        (WrapperKind::None, None) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&s.#idx_lit))
                                }
                            });
                        }
                        _ => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
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

#[proc_macro_derive(Casepaths)]
pub fn derive_casepaths(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let tokens = match input.data {
        Data::Enum(data_enum) => {
            let mut tokens = proc_macro2::TokenStream::new();
            for variant in data_enum.variants.iter() {
                let v_ident = &variant.ident;
                let snake = format_ident!("{}", to_snake_case(&v_ident.to_string()));
                let r_fn = format_ident!("{}_case_r", snake);
                let w_fn = format_ident!("{}_case_w", snake);

                match &variant.fields {
                    Fields::Unit => {
                        tokens.extend(quote! {
                            pub fn #r_fn() -> key_paths_core::KeyPaths<#name, ()> {
                                static UNIT: () = ();
                                key_paths_core::KeyPaths::readable_enum(
                                    |_| #name::#v_ident,
                                    |e: &#name| match e { #name::#v_ident => Some(&UNIT), _ => None }
                                )
                            }
                        });
                    }
                    Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
                        let inner_ty = &unnamed.unnamed.first().unwrap().ty;
                        tokens.extend(quote! {
                            pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                key_paths_core::KeyPaths::readable_enum(
                                    #name::#v_ident,
                                    |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None }
                                )
                            }
                            pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                key_paths_core::KeyPaths::writable_enum(
                                    #name::#v_ident,
                                    |e: &#name| match e { #name::#v_ident(v) => Some(v), _ => None },
                                    |e: &mut #name| match e { #name::#v_ident(v) => Some(v), _ => None },
                                )
                            }
                        });
                    }
                    // Multi-field tuple variant: Enum::Variant(T1, T2, ...)
                    Fields::Unnamed(unnamed) => {
                        let field_types: Vec<_> = unnamed.unnamed.iter().map(|f| &f.ty).collect();
                        let tuple_ty = quote! { (#(#field_types),*) };
                        
                        // Generate pattern matching for tuple fields
                        let field_patterns: Vec<_> = (0..unnamed.unnamed.len())
                            .map(|i| format_ident!("f{}", i))
                            .collect();
                        
                        tokens.extend(quote! {
                            pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #tuple_ty> {
                                key_paths_core::KeyPaths::failable_owned(
                                    |e: #name| match e { #name::#v_ident(#(#field_patterns),*) => Some((#(#field_patterns),*)), _ => None }
                                )
                            }
                            pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #tuple_ty> {
                                key_paths_core::KeyPaths::failable_owned(
                                    |e: #name| match e { #name::#v_ident(#(#field_patterns),*) => Some((#(#field_patterns),*)), _ => None }
                                )
                            }
                        });
                    }
                    
                    // Labeled variant: Enum::Variant { field1: T1, field2: T2, ... }
                    Fields::Named(named) => {
                        let field_names: Vec<_> = named.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                        let field_types: Vec<_> = named.named.iter().map(|f| &f.ty).collect();
                        let tuple_ty = quote! { (#(#field_types),*) };
                        
                        tokens.extend(quote! {
                            pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #tuple_ty> {
                                key_paths_core::KeyPaths::failable_owned(
                                    |e: #name| match e { #name::#v_ident { #(#field_names),* } => Some((#(#field_names),*)), _ => None }
                                )
                            }
                            pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #tuple_ty> {
                                key_paths_core::KeyPaths::failable_owned(
                                    |e: #name| match e { #name::#v_ident { #(#field_names),* } => Some((#(#field_names),*)), _ => None }
                                )
                            }
                        });
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
                                pub fn #r_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident).to_partial()
                                }
                                pub fn #w_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident).to_partial()
                                }
                                pub fn #fr_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref()).to_partial()
                                }
                                pub fn #fw_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.as_mut()).to_partial()
                                }
                                // Owned keypath methods
                                pub fn #o_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident).to_partial()
                                }
                                pub fn #fo_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident).to_partial()
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #fr_at_fn(index: usize) -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(index)).to_partial()
                                }
                                pub fn #fw_at_fn(index: usize) -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(index)).to_partial()
                                }
                                pub fn #r_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident).to_partial()
                                }
                                pub fn #w_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident).to_partial()
                                }
                                pub fn #fr_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.first()).to_partial()
                                }
                                pub fn #fw_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.first_mut()).to_partial()
                                }
                                // Owned keypath methods
                                pub fn #o_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident).to_partial()
                                }
                                pub fn #fo_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next()).to_partial()
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident).to_partial()
                                }
                                pub fn #w_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident).to_partial()
                                }
                                pub fn #fr_fn(key: String) -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key)).to_partial()
                                }
                                pub fn #fw_fn(key: String) -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key)).to_partial()
                                }
                                pub fn #fr_at_fn(key: String) -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key)).to_partial()
                                }
                                pub fn #fw_at_fn(key: String) -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key)).to_partial()
                                }
                                // Owned keypath methods
                                pub fn #o_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident).to_partial()
                                }
                                pub fn #fo_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next().map(|(_, v)| v)).to_partial()
                                }
                            });
                        }
                        _ => {
                            // Default case for simple types
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident).to_partial()
                                }
                                pub fn #w_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident).to_partial()
                                }
                                // Owned keypath methods
                                pub fn #o_fn() -> key_paths_core::PartialKeyPath<#name> {
                                    key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident).to_partial()
                                }
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
                                pub fn #r_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident).to_any()
                                }
                                pub fn #w_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident).to_any()
                                }
                                pub fn #fr_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref()).to_any()
                                }
                                pub fn #fw_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.as_mut()).to_any()
                                }
                                // Owned keypath methods
                                pub fn #o_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident).to_any()
                                }
                                pub fn #fo_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident).to_any()
                                }
                            });
                        }
                        (WrapperKind::Vec, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #fr_at_fn(index: usize) -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(index)).to_any()
                                }
                                pub fn #fw_at_fn(index: usize) -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(index)).to_any()
                                }
                                pub fn #r_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident).to_any()
                                }
                                pub fn #w_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident).to_any()
                                }
                                pub fn #fr_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.first()).to_any()
                                }
                                pub fn #fw_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.first_mut()).to_any()
                                }
                                // Owned keypath methods
                                pub fn #o_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident).to_any()
                                }
                                pub fn #fo_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next()).to_any()
                                }
                            });
                        }
                        (WrapperKind::HashMap, Some(inner_ty)) => {
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident).to_any()
                                }
                                pub fn #w_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident).to_any()
                                }
                                pub fn #fr_fn(key: String) -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key)).to_any()
                                }
                                pub fn #fw_fn(key: String) -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key)).to_any()
                                }
                                pub fn #fr_at_fn(key: String) -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_readable(move |s: &#name| s.#field_ident.get(&key)).to_any()
                                }
                                pub fn #fw_at_fn(key: String) -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_writable(move |s: &mut #name| s.#field_ident.get_mut(&key)).to_any()
                                }
                                // Owned keypath methods
                                pub fn #o_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident).to_any()
                                }
                                pub fn #fo_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::failable_owned(|s: #name| s.#field_ident.into_iter().next().map(|(_, v)| v)).to_any()
                                }
                            });
                        }
                        _ => {
                            // Default case for simple types
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident).to_any()
                                }
                                pub fn #w_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident).to_any()
                                }
                                // Owned keypath methods
                                pub fn #o_fn() -> key_paths_core::AnyKeyPath {
                                    key_paths_core::KeyPaths::owned(|s: #name| s.#field_ident).to_any()
                                }
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
//         "💡 Suggestion: For Mutex<T> containers, use the .with_mutex() method from WithContainer trait (no cloning):\n   use key_paths_core::WithContainer;\n   your_keypath.with_mutex(&mutex, |value| { /* work with value */ });"
//     } else if input_str.contains("RwLock<") && input_str.contains("KeyPaths<") {
//         "💡 Suggestion: For RwLock<T> containers, use the .with_rwlock() method from WithContainer trait (no cloning):\n   use key_paths_core::WithContainer;\n   your_keypath.with_rwlock(&rwlock, |value| { /* work with value */ });"
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
//         "🔧 KeyPaths Help: Use adapter methods to work with different container types:\n   - .for_arc() for Arc<T> containers\n   - .for_box() for Box<T> containers\n   - .for_rc() for Rc<T> containers\n   - .for_option() for Option<T> containers\n   - .for_result() for Result<T, E> containers\n   - .with_mutex() for Mutex<T> containers (import WithContainer trait)\n   - .with_rwlock() for RwLock<T> containers (import WithContainer trait)\n   - .for_arc_mutex() for Arc<Mutex<T>> containers (with parking_lot feature)\n   - .for_arc_rwlock() for Arc<RwLock<T>> containers (with parking_lot feature)\n\nExample: let arc_keypath = my_keypath.for_arc();\nFor Mutex/RwLock: use key_paths_core::WithContainer; then my_keypath.with_mutex(&mutex, |value| { ... });\nFor Arc<Mutex>/Arc<RwLock>: let arc_mutex_keypath = my_keypath.for_arc_mutex();".to_string()
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
