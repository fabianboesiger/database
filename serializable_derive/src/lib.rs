extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Data;
use syn::Fields;

#[proc_macro_derive(Serializable)]
pub fn serializable_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_serializable(&ast)
}

fn impl_serializable(ast: &syn::DeriveInput) -> TokenStream {

    let struct_name = &ast.ident;
    let mut field_names = Vec::new();
    match *&ast.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    for field in &fields.named {
                        if let Some(field_name) = &field.ident {
                            field_names.push(field_name); 
                        }
                    }
                },
                Fields::Unnamed(_) => unimplemented!(),
                Fields::Unit => unimplemented!()
            }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };

    let gen = quote! {
        impl Serializable for #struct_name {
            fn serialize(&self) -> Vec<u8> {
                let mut bytes = Vec::new();
                #(
                    bytes.append(&mut self.#field_names.serialize());
                )*
                bytes
            }

            fn deserialize(&mut self, mut bytes: &mut Vec<u8>) {
                bytes.reverse();
                #(
                    self.#field_names.deserialize(&mut bytes);
                )*
            }
        }
    };
    gen.into()
}