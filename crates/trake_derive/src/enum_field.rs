use convert_case::{Case, Casing};
use darling::{ast, export::syn, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct EnumStructOpts {
    ident: syn::Ident,
    vis: syn::Visibility,
    data: ast::Data<(), EnumFieldOpts>,
}

#[derive(FromField)]
struct EnumFieldOpts {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

impl EnumStructOpts {
    pub fn generate(self) -> TokenStream {
        let struct_name = self.ident;
        let vis = self.vis;

        let mut generated = TokenStream::new();

        let enum_name = syn::Ident::new(
            &format!("{}Field", struct_name),
            proc_macro2::Span::call_site(),
        );

        let fields = self.data.take_struct().unwrap().fields;

        let common_type = {
            let mut fields = fields.iter();
            if let Some(field) = fields.next() {
                let common_type = field.ty.clone();
                for field in fields {
                    if field.ty != common_type {
                        panic!("all fields must be of the same type");
                    }
                }
                common_type
            } else {
                syn::Type::Verbatim(quote! { ! })
            }
        };

        let fields: Vec<_> = fields
            .into_iter()
            .map(|field| {
                let field = field.ident.unwrap();
                (
                    field.clone(),
                    syn::Ident::new(
                        &format!("{}", field).to_case(Case::UpperCamel),
                        proc_macro2::Span::call_site(),
                    ),
                )
            })
            .collect();

        let gen_enum = {
            let fields = fields.iter().map(|(_, name)| {
                quote! { #name, }
            });

            quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                #vis enum #enum_name {
                    #(#fields)*
                }
            }
        };

        let impl_struct = {
            let match_fields = fields.iter().map(|(field_name, variant_name)| {
                quote! { #variant_name => self.#field_name, }
            });

            quote! {
                impl #struct_name {
                    #vis fn get_field(&self, field: #enum_name) -> #common_type {
                        match field {
                            #(#match_fields)*
                        }
                    }
                }
            }
        };

        generated.extend(gen_enum);
        generated.extend(impl_struct);

        generated
    }
}
