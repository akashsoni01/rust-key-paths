use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

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


