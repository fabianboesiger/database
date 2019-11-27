extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Data;
use syn::Fields;
use syn::Type;
use std::fmt;

#[proc_macro_derive(Storable, attributes(id))]
pub fn storable_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_storable(&ast)
}

fn type_to_bytes(type: syn::Type) -> TokenStream {
    if field_type.is_ident("str") {
        quote! {
            as_bytes()
        }
    } else {
        quote! {
            to_le_bytes()
        }
    }
}

fn type_from_bytes(type: syn::Type) -> TokenStream {
    if field_type.is_ident("str") {
        quote! {
            as_bytes()
        }
    } else {
        quote! {
            to_le_bytes()
        }
    }
}

fn impl_storable(ast: &syn::DeriveInput) -> TokenStream {
    // find struct name
    let struct_name = &ast.ident;
    // find id name
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let (id_name, id_type) = match *&ast.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let mut id_name = Option::None;
                    let mut id_type = Option::None;
                    for field in &fields.named {
                        // check if id attribute is set
                        let mut is_id = false;
                        for attribute in &field.attrs {
                            if attribute.path.is_ident("id") {
                                if is_id {
                                    panic!("The id attribute can only be defined once");
                                }
                                is_id = true;
                                break;
                            }
                        };

                        if let Some(field_name) = &field.ident {
                            let mut current_field = &field.ty;
                            while let Type::Reference(tp) = current_field {
                                current_field = &tp.elem;
                            }
                            if let Type::Path(tp) = current_field {
                                field_names.push(field_name);
                                field_types.push(&tp.path);                      // check if id
                                if is_id {
                                    id_name = Some(field_name);
                                    id_type = Some(&tp.path);
                                }
                            }
                        }
                    }
                    (id_name, id_type)
                },
                Fields::Unnamed(_) => unimplemented!(),
                Fields::Unit => unimplemented!()
            }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };

    // currently, an id attribute has to exist
    if id_name == Option::None {
        panic!("Storable structs without id are not allowed");
    }

    let mut to_bytes = Vec::new();
    let mut from_bytes = Vec::new();
    for field_type in &field_types {
        to_bytes.push(type_to_bytes(field_type));
        from_bytes.push(type_from_bytes(field_type));
        /*
        from_binary.push(
            if field_type.is_ident("str") {
                quote! {
                    for _ in std::mem::size_of(#field_types) {

                    }
                    String::from_utf8(value).as_str()
                }
            } else {
                quote! {
                    #field_types::from_le_bytes(value)
                }
            }
        );
        */
    }

    let to_bytes_id = if let Some(id_type) = id_type {
        type_to_bytes(id_type);
    } else {
        unreachable!()
    };

    // generate implementation
    let gen = quote! {
        impl Storable for #struct_name {
            fn name() -> String {
                format!("{}", stringify!(#struct_name).to_lowercase())
            }
    
            fn id(&self) -> String {
                let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
                let mut output = String::new();
                let mut sextets = Vec::new();
                let bytes = self.#id_name.#to_bytes_id;
                for (i, byte) in bytes.iter().enumerate() {
                    match i % 3 {
                        0 => {
                            sextets.push(byte & 0b00000011);
                            sextets.push((byte & 0b11111100) >> 6);
                        },
                        1 => {
                            let last = sextets.pop().unwrap();
                            sextets.push(last & ((byte & 0b00001111) << 2));
                            sextets.push((byte & 0b11110000) >> 4);
                        },
                        2 => {
                            let last = sextets.pop().unwrap();
                            sextets.push(last & ((byte & 0b00111111) << 4));
                            sextets.push((byte & 0b11000000) >> 2);
                        },
                        _ => unreachable!()
                    }
                };
                if sextets.len() > 128 {
                    panic!("File name too large");
                }
                for sextet in sextets {
                    output.push(alphabet.chars().skip(sextet as usize).next().expect("Alphabet out of range"));
                }
                output
            }

            fn key(&self) -> String {
                format!("{}/{}", #struct_name::name(), self.id())
            }

            fn from_bin(&self, bin: &[u8]) -> Result<(), ()> {
                /*
                input.reverse();
                #(
                    let value = 
                    self.#field_names = #from_binary;
                )*
                */
                Ok(())
            }

            fn to_bin(&self) -> Vec<u8> {
                let mut bin = Vec::new();
                #(
                    bin.extend_from_slice(format!("{}\0", stringify!(#field_names)).as_bytes());
                    bin.extend_from_slice(format!("{}\0", stringify!(#tield_types)).as_bytes());
                    bin.extend_from_slice(self.#field_names.#to_bytes);
                )*
                bin
            }

            fn from_string(&self, input: String) {

            }
        }
        
        impl std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #(write!(f, "{}:{}={}\n", stringify!(#field_names), stringify!(#field_types), self.#field_names)?;)*
                Ok(())
            }
        }
    };
    gen.into()
}