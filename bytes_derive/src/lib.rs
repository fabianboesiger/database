extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::{Data, Fields, Type, Meta, NestedMeta};

#[proc_macro_derive(Bytes, attributes(from))]
pub fn bytes_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_serialize(&ast)
}

fn impl_serialize(ast: &syn::DeriveInput) -> TokenStream {

    let nested = 
        if let Some(Meta::List(meta_list)) = 
            if let Some(some) = &ast.attrs
                .clone()
                .into_iter()
                .filter(|attr| attr.path.is_ident("from"))
                .nth(0)
            {
                Some(some
                    .parse_meta()
                    .unwrap())
            } else {
                None
            }
        { Some(meta_list.nested) } else { None };
    
    let from = if let Some(nested) = nested {
        let mut from = Vec::new();

        for nested_meta in nested {
            if let NestedMeta::Meta(meta) = nested_meta {
                if let Meta::Path(path) = meta {
                    from.push(path);
                } 
            }
        }
        from
    } else {
        Vec::new()
    };

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
        impl Bytes for #struct_name {
            fn serialize(&self) -> Vec<u8> {
                let mut bytes = Vec::new();

                bytes.append(&mut #struct_name::hash().serialize());

                #(
                    bytes.append(&mut self.#field_names.serialize());
                )*
                bytes
            }

            fn deserialize(mut bytes: &mut Vec<u8>) -> Result<#struct_name, crate::Error> {
                bytes.reverse();

                let data_hash = u64::deserialize(&mut bytes)?;
                let mut hash_bytes = data_hash.serialize();
                hash_bytes.reverse();
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hasher::write(&mut hasher, #struct_name::signature().as_bytes());

                if data_hash ==  #struct_name::hash() {
                    return Ok(#struct_name {
                        #(
                            #field_names: #field_types::deserialize(&mut bytes)?,
                        )*
                    });
                }
                #(  
                    bytes.append(&mut hash_bytes.clone());
                    bytes.reverse();
                    if let Ok(old) = #from::deserialize(&mut bytes) {
                        return Ok(#struct_name::from(old));
                    }
                )*
                Err(crate::Error::new(format!("Hash not matching.")))
            }

            fn signature() -> String {
                let mut output = String::new();
                #(
                    output.push_str(&#field_types::signature());
                )*
                output
            }
        }
    };
    gen.into()
}