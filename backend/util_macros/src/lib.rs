use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(ErrorPayloadMacro)]
pub fn error_payload_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_error_payload_macro(&ast)
}

fn impl_error_payload_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl From<#name> for ErrorPayload {
            fn from(value: #name) -> Self {
                    ErrorPayload::from_error(value)
            }
        }
    };
    gen.into()
}

