extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::{Data, Fields, Type};

#[proc_macro_derive(SerializeBinary)]
pub fn serialize_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_serialize(&ast)
}

fn impl_serialize(ast: &syn::DeriveInput) -> TokenStream {

    let struct_name = &ast.ident;
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    match *&ast.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    for field in &fields.named {
                        if let Some(field_name) = &field.ident {
                            if let Type::Path(tp) = &field.ty {
                                field_names.push(field_name); 
                                field_types.push(&tp.path);
                            }
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
        impl SerializeBinary for #struct_name {
            fn serialize(&self) -> Vec<u8> {
                let mut bytes = Vec::new();
                #(
                    bytes.append(&mut self.#field_names.serialize());
                )*
                bytes
            }

            fn deserialize(mut bytes: &mut Vec<u8>) -> #struct_name {
                bytes.reverse();
                #struct_name {
                    #(
                        #field_names: #field_types::deserialize(&mut bytes),
                    )*
                }
            }
        }
    };
    gen.into()
}