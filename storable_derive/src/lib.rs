extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Data;
use syn::Fields;

#[proc_macro_derive(Storable, attributes(id))]
pub fn storable_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_storable(&ast)
}

fn impl_storable(ast: &syn::DeriveInput) -> TokenStream {
    // find struct name
    let struct_name = &ast.ident;
    // find id name
    let id_name = match *&ast.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let mut id_name = Option::None;
                    for field in &fields.named {
                        // check if id attribute is set
                        let mut is_id = false;
                        for attribute in &field.attrs {
                            if attribute.path.is_ident("id") {
                                is_id = true;
                                break;
                            }
                        };
                        // if id attribute was found, we can return
                        if is_id {
                            id_name = Some(&field.ident);
                            break;
                        }
                    }
                    id_name
                },
                Fields::Unnamed(ref fields) => {
                    Option::None
                },
                Fields::Unit => {
                    Option::None
                }
            }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };

    // currently, an id attribute has to exist
    if id_name == Option::None {
        panic!("Storable structs without id are not allowed");
    }

    // generate implementation
    let gen = quote! {
        impl Storable for #struct_name {
            fn name() -> String {
                format!("{}s", stringify!(#struct_name).to_lowercase())
            }
    
            fn id(&self) -> String {
                format!("{}", self.#id_name)
            }

            fn key(&self) -> String {
                format!("{}/{}", #struct_name::name(), self.id())
            }
        }
    };
    gen.into()
}