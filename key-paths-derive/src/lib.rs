use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

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

                    // Helper: detect Option<T>
                    let option_inner: Option<Type> = extract_option_inner_type(ty);

                    match option_inner {
                        Some(inner_ty) => {
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
                        None => {
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

fn extract_option_inner_type(ty: &Type) -> Option<Type> {
    use syn::{PathArguments, GenericArgument};
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Option" {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for arg in ab.args.iter() {
                        if let GenericArgument::Type(inner) = arg {
                            return Some(inner.clone());
                        }
                    }
                }
            }
        }
    }
    None
}


