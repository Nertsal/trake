mod enum_field;
mod texture_atlas;

use darling::{export::syn, FromDeriveInput};

#[proc_macro]
pub fn texture_atlas(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as texture_atlas::AtlasOpts);
    input.generate().into()
}

#[proc_macro_derive(EnumField, attributes(enum_field))]
pub fn enum_field(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(tokens);
    match enum_field::EnumStructOpts::from_derive_input(&input) {
        Ok(input) => input.generate().into(),
        Err(e) => e.write_errors().into(),
    }
}
