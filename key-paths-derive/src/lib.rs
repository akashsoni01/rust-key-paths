use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WrapperKind {
    None,
    Option,
    Box,
    Rc,
    Arc,
}

#[proc_macro_derive(Keypaths)]
pub fn derive_keypaths(input: TokenStream) -> TokenStream {
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
                    let w_fn = format_ident!("{}_w", field_ident);
                    let fr_fn = format_ident!("{}_fr", field_ident);
                    let fw_fn = format_ident!("{}_fw", field_ident);

                    let (kind, inner_ty) = extract_wrapper_inner_type(ty);

                    match (kind, inner_ty) {
                        (WrapperKind::Option, Some(inner_ty)) => {
                                                // Option<T>
                                                tokens.extend(quote! {
                                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                                    }
                                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                                    }
                                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#field_ident.as_ref())
                                                    }
                                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#field_ident.as_mut())
                                                    }
                                                });
                                            }
                        (WrapperKind::Box, Some(inner_ty)) => {
                                                // Box<T>
                                                tokens.extend(quote! {
                                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                                        key_paths_core::KeyPaths::readable(|s: &#name| &*s.#field_ident)
                                                    }
                                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut *s.#field_ident)
                                                    }
                                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#field_ident))
                                                    }
                                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut *s.#field_ident))
                                                    }
                                                });
                                            }
                        (WrapperKind::Rc, Some(inner_ty)) | (WrapperKind::Arc, Some(inner_ty)) => {
                                                // Rc<T> or Arc<T> → read-only only
                                                tokens.extend(quote! {
                                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                                        key_paths_core::KeyPaths::readable(|s: &#name| &*s.#field_ident)
                                                    }
                                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#field_ident))
                                                    }
                                                });
                                            }
                        (_, None) => {
                                                // Regular field
                                                tokens.extend(quote! {
                                                    pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                                        key_paths_core::KeyPaths::readable(|s: &#name| &s.#field_ident)
                                                    }
                                                    pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                                        key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#field_ident)
                                                    }
                                                    pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                                        key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&s.#field_ident))
                                                    }
                                                    pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                                        key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut s.#field_ident))
                                                    }
                                                });
                                            }
                    (WrapperKind::None, Some(_)) => todo!(),
                    }
                }
                tokens
            }
            
            _ => quote! {
                compile_error!("Keypaths derive supports only structs with named fields");
            },
        },
        _ => quote! {
            compile_error!("Keypaths derive supports only structs with named fields");
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
                for arg in ab.args.iter() {
                    if let GenericArgument::Type(inner) = arg {
                        return match ident_str.as_str() {
                            "Option" => (WrapperKind::Option, Some(inner.clone())),
                            "Box"    => (WrapperKind::Box, Some(inner.clone())),
                            "Rc"     => (WrapperKind::Rc, Some(inner.clone())),
                            "Arc"    => (WrapperKind::Arc, Some(inner.clone())),
                            _        => (WrapperKind::None, None),
                        };
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
                    _ => {
                        tokens.extend(quote! {
                            compile_error!("Casepaths derive supports only unit and single-field tuple variants");
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