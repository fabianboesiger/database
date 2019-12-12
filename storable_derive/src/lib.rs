extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Data;
use syn::Fields;
use syn::Type;

#[proc_macro_derive(Storable, attributes(id))]
pub fn storable_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_storable(&ast)
}

fn impl_storable(ast: &syn::DeriveInput) -> TokenStream {
    // find struct name
    let struct_name = &ast.ident;
    // find id name and type
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
                            if let Type::Path(tp) =  &field.ty {
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

    // generate implementation
    let gen = quote! {

        impl #struct_name {
            fn from(id: #id_type, database: &Database) -> Result<#struct_name, Box<dyn std::error::Error>> {
                let mut output = Self::default();
                output.#id_name = id;
                database.read(&mut output)?;
                Ok(output)
            }
        }

        impl Storable for #struct_name {

            fn name() -> Result<String, Box<dyn std::error::Error>> {
                let name = stringify!(#struct_name);
                if name.len() > 128 {
                    return Err(Box::new(Error::new()) as Box<dyn std::error::Error>);
                }
                Ok(format!("{}", name.to_lowercase()))
            }
    
            fn id(&self) -> Result<String, Box<dyn std::error::Error>> {
                let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
                let mut output = String::new();
                let mut sextets = Vec::<u8>::new();
                let bytes = self.#id_name.serialize();
                
                for (i, &byte) in bytes.iter().enumerate() {
                    match i % 3 {
                        0 => {
                            sextets.push(byte & 0b00111111);
                            sextets.push((byte & 0b11000000) >> 6);
                        },
                        1 => {
                            let last = sextets.pop().unwrap();
                            sextets.push(last | ((byte & 0b00001111) << 2));
                            sextets.push((byte & 0b11110000) >> 4);
                        },
                        2 => {
                            let last = sextets.pop().unwrap();
                            sextets.push(last | ((byte & 0b00000011) << 4));
                            sextets.push((byte & 0b11111100) >> 2);
                        },
                        _ => unreachable!()
                    }
                };
                if sextets.len() > 128 {
                    return Err(Box::new(Error::new()) as Box<dyn std::error::Error>);
                }
                for &sextet in &sextets {
                    output.push(alphabet.chars().skip(sextet as usize).next().expect("Alphabet out of range"));
                }

                Ok(output)
            }

            fn key(&self) -> Result<String, Box<dyn std::error::Error>> {
                Ok(format!("{}/{}", #struct_name::name()?, self.id()?))
            }

        }
    };
    gen.into()
}
