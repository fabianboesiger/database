extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::Data;
use syn::Fields;
use syn::Type;

#[proc_macro_derive(Store, attributes(id))]
pub fn store_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_store(&ast)
}

fn impl_store(ast: &syn::DeriveInput) -> TokenStream {
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

    let name = format!("{}", struct_name);
    if name.len() > 128 {
        panic!("Name exceeds maximum length.");
    }
    let name = format!("{}s", 
        name
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let mut output = Vec::new();
                if i > 0 && c.is_ascii_uppercase() {
                    output.push('-');
                }
                output.push(c.to_ascii_lowercase());
                output
            })
            .flatten()
            .collect::<String>()
    );

    // generate implementation
    let gen = quote! {
        /*
        impl #struct_name {
            fn create<'a>(&self, database: &'a Database) -> Result<(), Box<dyn std::error::Error>> {
                database.create(self)?;
                Ok(())
            }

            fn update<'a>(&self, database: &'a Database) -> Result<(), Box<dyn std::error::Error>> {
                database.update(self)?;
                Ok(())
            }

            fn read<'a>(id: #id_type, database: &'a Database) -> Result<#struct_name, Box<dyn std::error::Error>> {
                let mut output = Self::default();
                output.#id_name = id;
                database.read(&mut output)?;
                Ok(output)
            }

            fn delete<'a>(id: #id_type, database: &'a Database) -> Result<#struct_name, Box<dyn std::error::Error>> {
                let mut output = Self::default();
                output.#id_name = id;
                database.delete(&output)?;
                Ok(output)
            }
        }
        */
        
        impl Store for #struct_name {
            type ID = #id_type;
            /*
            fn with(#id_name: #id_type) -> #struct_name {
                #struct_name {
                    #id_name,
                    ..
                    Default::default()
                }
            }
            */
            fn name() -> &'static str {
                #name
            }
            
            fn id(&self) -> #id_type {
                self.#id_name.clone()
            }
            
        }
    };
    gen.into()
}
