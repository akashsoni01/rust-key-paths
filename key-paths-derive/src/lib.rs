use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

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
                    let field_name = field_ident.to_string();
                    let ty = &field.ty;

                    let r_fn = format_ident!("{}_r", field_ident);
                    let w_fn = format_ident!("{}_w", field_ident);
                    let fr_fn = format_ident!("{}_fr", field_ident);
                    let fw_fn = format_ident!("{}_fw", field_ident);

                    // Helper: detect Option<T> or Box<T>
                    let (is_option, inner_ty) = extract_wrapper_inner_type(ty);

                    match (is_option, inner_ty) {
                        (true, Some(inner_ty)) => {
                            // Option<T> case
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
                        (false, Some(inner_ty)) => {
                            // Box<T> case - treat like a normal field but with dereferencing
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
                        (_, None) => {
                            // Regular field case
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

                    let (is_option, inner_ty) = extract_wrapper_inner_type(ty);
                    
                    match (is_option, inner_ty) {
                        (true, Some(inner_ty)) => {
                            // Option<T> case
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| s.#idx_lit.as_ref())
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| s.#idx_lit.as_mut())
                                }
                            });
                        }
                        (false, Some(inner_ty)) => {
                            // Box<T> case
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &*s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut *s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&*s.#idx_lit))
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #inner_ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut *s.#idx_lit))
                                }
                            });
                        }
                        (_, None) => {
                            // Regular field case
                            tokens.extend(quote! {
                                pub fn #r_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::readable(|s: &#name| &s.#idx_lit)
                                }
                                pub fn #w_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::writable(|s: &mut #name| &mut s.#idx_lit)
                                }
                                pub fn #fr_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::failable_readable(|s: &#name| Some(&s.#idx_lit))
                                }
                                pub fn #fw_fn() -> key_paths_core::KeyPaths<#name, #ty> {
                                    key_paths_core::KeyPaths::failable_writable(|s: &mut #name| Some(&mut s.#idx_lit))
                                }
                            });
                        }
                    }
                }
                tokens
            }
            _ => quote! {
                compile_error!("Keypaths derive supports only structs with named fields");
            },
        },
        Data::Enum(data_enum) => {
            let mut tokens = proc_macro2::TokenStream::new();
            for variant in data_enum.variants.iter() {
                let v_ident = &variant.ident;
                let r_fn = format_ident!("{}_r", to_snake_case(&v_ident.to_string()));
                let w_fn = format_ident!("{}_w", to_snake_case(&v_ident.to_string()));

                match &variant.fields {
                    Fields::Unit => {
                        // Only readable for unit variants. Writable would require returning &mut () to a non-existent payload.
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
                            compile_error!("Keypaths derive currently supports enum unit and single-field tuple variants only");
                        });
                    }
                }
            }
            tokens
        }
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

fn extract_wrapper_inner_type(ty: &Type) -> (bool, Option<Type>) {
    use syn::{GenericArgument, PathArguments};
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            let ident_str = seg.ident.to_string();
            
            if ident_str == "Option" {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for arg in ab.args.iter() {
                        if let GenericArgument::Type(inner) = arg {
                            return (true, Some(inner.clone()));
                        }
                    }
                }
            } else if ident_str == "Box" {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for arg in ab.args.iter() {
                        if let GenericArgument::Type(inner) = arg {
                            return (false, Some(inner.clone()));
                        }
                    }
                }
            }
        }
    }
    (false, None)
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