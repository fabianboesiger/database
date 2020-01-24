extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::{Data, Fields, Type, Meta, NestedMeta};

#[proc_macro_derive(Store, attributes(id, rename))]
pub fn store_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_store(&ast)
}

/*
fn contains_name(nested: &Punctuated<NestedMeta, Comma>, name: &str) -> bool {
    for nested_meta in nested.clone() {
        if let NestedMeta::Meta(meta) = nested_meta {
            if let Meta::Path(path) = meta {
                if path.is_ident(name) {
                    return true;
                }
            } 
        }
    }
    false
}
*/

fn impl_store(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;

    let nested = 
        if let Some(Meta::List(meta_list)) = 
            if let Some(some) = &ast.attrs
                .clone()
                .into_iter()
                .filter(|attr| attr.path.is_ident("rename"))
                .nth(0)
            {
                Some(some
                    .parse_meta()
                    .unwrap())
            } else {
                None
            }
        { Some(meta_list.nested) } else { None };

    /*
    let from = Vec::new();

    for nested_meta in nested {
        if let NestedMeta::Meta(meta) = nested_meta {
            if let Meta::Path(path) = meta {
                from.push(path);
            } 
        }
    }
    */

    let new_name = if let Some(nested) = nested {
        let mut from = Vec::new();

        for nested_meta in nested {
            if let NestedMeta::Meta(meta) = nested_meta {
                if let Meta::Path(path) = meta {
                    from.push(path);
                } 
            }
        }

        if let Some(new_name) = from.into_iter().nth(0) {
            Some(new_name.get_ident().unwrap().to_string())
        } else {
            None
        }
    } else {
        None
    };
    

    let (id_name, id_type) = match *&ast.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let mut id_name = Option::None;
                    let mut id_type = Option::None;
                    for field in &fields.named {
                        if let Some(_) = field.attrs
                            .clone()
                            .into_iter()
                            .filter(|attr| attr.path.is_ident("id"))
                            .nth(0)
                        {
                            if let Some(field_name) = &field.ident {
                                if let Type::Path(tp) =  &field.ty {
                                    id_name = Some(field_name);
                                    id_type = Some(&tp.path);
                                }
                            }
                        }
                        /*
                        let nested = 
                            if let Meta::List(meta_list) = 
                                if let Some(some) = field.attrs
                                    .clone()
                                    .into_iter()
                                    .filter(|attr| attr.path.is_ident("store"))
                                    .nth(0)
                                {
                                    some
                                        .parse_meta()
                                        .unwrap()
                                } else {
                                    continue;
                                }
                            { meta_list.nested } else { panic!() };

                        if let Some(field_name) = &field.ident {
                            if let Type::Path(tp) =  &field.ty {
                                if contains_name(&nested, "id") {
                                    id_name = Some(field_name);
                                    id_type = Some(&tp.path);
                                }
                            }
                        }
                        */
                    }
                    (id_name, id_type)
                },
                Fields::Unnamed(_) => unimplemented!(),
                Fields::Unit => unimplemented!()
            }
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!()
    };

    if id_name == Option::None {
        panic!("Storable structs without id are not allowed");
    }

    let name = format!("{}", struct_name);
    if name.len() > 128 {
        panic!("Name exceeds maximum length.");
    }
    let name = format!("{}s", 
        if let Some(new_name) = new_name { new_name } else { name }
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

    let gen = quote! {
        impl Store for #struct_name {
            type Id = #id_type;

            const NAME: &'static str = #name;
            
            fn id(&self) -> &#id_type {
                &self.#id_name
            }
        }
    };
    gen.into()
}
